//! Quick commandline utility to split a nifti file into a series of 2D slices.
//!
//! This utility provides tools for manipulating NIfTI files, a common format
//! for storing neuroimaging data. It allows users to split a 3D NIfTI file into
//! a series of 2D slices, optionally padding the slices.

use clap::Parser;
use ndarray::prelude::*;
use ndarray::{Array3, Ix3};
use nifti::writer::WriterOptions;
use nifti::{IntoNdArray, NiftiObject, NiftiVolume, ReaderOptions};
use std::fs;
use std::path::Path;
extern crate nalgebra as na;
use na::Point4;

use slicenii::common::{Direction, Slice3D};

// TODO: add support for 4D images
// TODO: decide on behavior if given a directory
// TODO: test with .gz
// TODO: fix issue with filenames that have periods in them
// TODO: option to determine the amount of padding

// use clap to create commandline interface
#[derive(Parser, Debug)]
#[command(author, about, version, long_about)]
struct Args {
    /// the input nifti file
    #[arg(short, long)]
    input: String,

    /// an output path, must be a directory which already exists, a new directory will be created within this directory to store the slices.
    #[arg(short, long, default_value = "./")]
    output: String,

    /// the axis we will slice along, 0, 1, or 2 for first, second, or third axis respectively
    #[arg(short, long, default_value_t = 0)]
    axis: usize,

    /// whether to pad the slices (stacks 4 copies of the slice)
    #[arg(short, long, default_value = "false")]
    pad: bool,
}

/// Creates a vector of single slices from a 3D array along a given axis.
///
/// This function takes in a 3D array and a direction (axis) and returns a vector
/// of `Slice3D` objects. Each `Slice3D` object represents a 2D slice of the original
/// 3D array along the specified axis.
///
/// # Arguments
///
/// * `img` - A 3D array representing the NIfTI file.
/// * `axis` - The axis along which to slice the array.
///
/// # Returns
///
/// A `Vec<Slice3D>`, where each `Slice3D` is a 2D slice of the original 3D
fn slice_array(img: Array3<f64>, axis: &Direction) -> Vec<Slice3D> {
    let shape = img.shape();
    let end_index = shape[axis.to_usize()];
    let mut slices = Vec::new();
    for i in 0..end_index {
        let slice = img.index_axis(Axis(axis.to_usize()), i);
        // enforce 2D
        let slice = slice.into_dimensionality::<Ix2>().unwrap_or_else(|e| {
            eprintln!("Error! {}", e);
            std::process::exit(-2);
        });
        // then add back the missing axis
        let slice3d = slice.insert_axis(Axis(axis.to_usize()));
        // enforce 3D
        let slice3d = slice3d.into_dimensionality::<Ix3>().unwrap_or_else(|e| {
            eprintln!("Error! {}", e);
            std::process::exit(-2);
        });
        let slice3d = slice3d.into_owned();
        // add slice to vector
        slices.push(Slice3D::new(slice3d, i));
    }
    slices
}

/// Creates a vector of volumes holding copies of each slice from a 3D array along a given axis.
///
/// This function is similar to `slice_array`, but instead of returning a vector of single
/// slices, it returns a vector of volumes. Each volume consists of a number of identical
/// slices stacked along the specified axis. The number of slices in each volume should be
/// determined by the `_padding` argument, but this is currently ignored.
///
/// # Arguments
///
/// * `img` - A 3D array representing the NIfTI file.
/// * `axis` - The axis along which to slice and duplicate the array.
/// * `_padding` - In the future: the number of times to duplicate each slice.
///
/// # Returns
///
/// A `Vec<Slice3D>`, where each `Slice3D` is a volume consisting of identical slices
/// of the original 3D array.
fn slice_array_pad(img: Array3<f64>, axis: &Direction, _padding: usize) -> Vec<Slice3D> {
    // padding input is ignored for now
    let shape = img.shape();
    let end_index = shape[axis.to_usize()];
    let mut slices = Vec::new();
    for i in 0..end_index {
        let slice = img.index_axis(Axis(axis.to_usize()), i);
        let slice = slice.into_dimensionality::<Ix2>().unwrap_or_else(|e| {
            eprintln!("Error! {}", e);
            std::process::exit(-2);
        });
        let slice = slice.into_owned();

        // SMARTER WAY TO DO THIS
        // let mut final_shape = slice.raw_dim();
        // final_shape[axis.to_usize()] = padding;
        // println!("final_shape: {:?}", final_shape);
        // let slice3d = slice.broadcast(final_shape).unwrap();

        // DUMB WAY FOR NOW
        // Create a vector to hold the duplicated slices
        // let mut duplicate_slices = Vec::new();
        // for _ in 0..padding {
        //     duplicate_slices.push(slice.clone());
        // }

        // let slice3d = ndarray::stack![Axis(axis.to_usize()), duplicate_slices];

        // OLD HARD CODED WAY
        // let slice3d = ndarray::stack![Axis(axis.to_usize()), slice, slice, slice];
        // let slice3d = ndarray::stack![Axis(axis.to_usize()), slice, slice, slice, slice, slice];
        let slice3d = ndarray::stack![Axis(axis.to_usize()), slice, slice, slice, slice,];
        let slice3d = slice3d.into_dimensionality::<Ix3>().unwrap_or_else(|e| {
            eprintln!("Error! {}", e);
            std::process::exit(-2);
        });
        // slices.push(Slice3D::new(slice3d.into_owned(), i));
        slices.push(Slice3D::new(slice3d, i));
    }
    slices
}

/// Saves the slices from a 3D array as individual NIfTI files.
///
/// This function takes in a vector of `Slice3D` objects and saves each one as a separate
/// NIfTI file. The files are named according to the original NIfTI file, the axis along
/// which the slices were taken, and the index of the slice. They are saved in a directory
/// named after the original NIfTI file, within the directory specified by `output_basepath`.
///
/// # Arguments
///
/// * `slices` - A vector of `Slice3D` objects to be saved.
/// * `header` - The header from the original NIfTI file.
/// * `axis` - The axis along which the slices were taken.
/// * `output_basepath` - The directory in which to save the slice files.
/// * `basename` - The base name to use for the output files, typically derived from the original NIfTI file.
/// * `end_string` - A string to append to the end of each file name, indicating if the slice was padded.
fn save_slices(
    slices: Vec<Slice3D>,
    header: &nifti::NiftiHeader,
    axis: &Direction,
    output_basepath: &Path,
    basename: &str,
    end_string: &str,
) {
    let scan_save_dir_name = format!("{basename}_slices");
    let scan_save_dir = Path::new(&scan_save_dir_name);
    let a = axis.to_string();

    let save_dir = output_basepath.join(scan_save_dir);
    match fs::create_dir_all(&save_dir) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error! {}", e);
            std::process::exit(-2);
        }
    }
    let affine = header.affine::<f64>();
    println!("affine: {:?}", affine);
    let inv_affine = affine.try_inverse().unwrap();
    println!("inv_affine: {:?}", inv_affine);

    for s in slices {
        let index = s.index;
        let save_index = format!("{:03}", index + 1);
        let output_filename = format!("{basename}_axis-{a}_slice-{end_string}{save_index}.nii");
        let output_path = save_dir.join(output_filename);

        let mut slice_header = header.clone();

        // Compute the position of the slice in real-world coordinates
        let pos_real = s.index as f32 * header.pixdim[axis.to_usize() + 1];

        // Create a point in matrix-world coordinates at the position of the slice
        // using nalgebra
        let mut pos_point = Point4::new(0.0, 0.0, 0.0, 1.0);
        pos_point[axis.to_usize()] = pos_real as f64;
        // use the inverse of the affine to place the "real-worl" matrix point in voxel coordinates
        let pos_vox = inv_affine * pos_point;
        // create a new affine using this shifted voxel coordinate
        let mut slice_affine = affine;
        for i in 0..3 {
            slice_affine[(i, 3)] = pos_vox[i];
        }
        slice_header.set_affine(&slice_affine);

        // save each slice as a nifti file
        WriterOptions::new(&output_path)
            .reference_header(&slice_header)
            .write_nifti(&s.slice)
            .unwrap_or_else(|e| {
                eprintln!("Error! {}", e);
                std::process::exit(-2);
            });
    }
}

/// Main function that parses commandline arguments and runs the program.
///
/// This function handles the overall flow of the program. It parses the commandline arguments,
/// reads the input NIfTI file, slices it along the specified axis, and then saves the resulting
/// slices as separate NIfTI files. If the `pad` argument is true, then it pads each slice before
/// saving.
fn main() {
    let cli = Args::parse();
    let input = cli.input;
    let input_filepath = Path::new(&input);
    let output = cli.output;
    let output_basepath = Path::new(&output);

    let basename = match input_filepath.file_stem() {
        Some(name) => name.to_str().unwrap(),
        None => {
            eprintln!("Error! Could not parse input file name.");
            std::process::exit(-2);
        }
    };

    let axis = match cli.axis {
        0 => Direction::X,
        1 => Direction::Y,
        2 => Direction::Z,
        _ => {
            eprintln!("Error! Axis must be 0, 1, or 2. To indicate the 1st (x), 2nd (y), or 3rd axis (z), respectively.");
            std::process::exit(-2);
        }
    };
    // steps:
    let obj = ReaderOptions::new().read_file(&input).unwrap_or_else(|e| {
        eprintln!("Error! {}", e);
        std::process::exit(-2);
    });
    // gather header information
    let header = obj.header();

    // let affine = header.clone().affine();
    let pixdim = header.pixdim;
    let _axis_pixdim = pixdim[axis.to_usize() + 1];
    // get the volume
    let volume = obj.volume();
    let _dims = volume.dim();
    // convert volume to ndarray
    let img = volume.into_ndarray::<f64>().unwrap_or_else(|e| {
        eprintln!("Error! {}", e);
        std::process::exit(-2);
    });
    if img.ndim() != 3 {
        eprintln!("Error! Input nifti file must be 3D. Tip: You can use a utility like `fslsplit` to split a 4D file into 3D files.");
        std::process::exit(-2);
    }
    // shave off dimension 4 for now
    let img_single = img.into_dimensionality::<Ix3>().unwrap_or_else(|e| {
        eprintln!("Error! {}", e);
        std::process::exit(-2);
    });
    let padding = 4;
    let (slices, end_string) = match cli.pad {
        true => {
            let slices = slice_array_pad(img_single, &axis, padding);
            let end_string = "padded-".to_string();
            (slices, end_string)
        }
        false => {
            let slices = slice_array(img_single, &axis);
            let end_string = "".to_string();
            (slices, end_string)
        }
    };

    save_slices(
        slices,
        header,
        &axis,
        output_basepath,
        basename,
        &end_string,
    );
}
