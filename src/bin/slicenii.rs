//! Quick commandline utility to split a nifti file into a series of 2D slices.

use clap::Parser;
use ndarray::prelude::*;
use ndarray::{Array3, Ix3};
use nifti::writer::WriterOptions;
use nifti::{IntoNdArray, NiftiObject, NiftiVolume, ReaderOptions};
use std::fs;
use std::path::Path;

use slicenii::common::{Direction, Slice3D};

// TODO: add argument to choose padded vs not
// TODO: clean up
// TODO: add support for 4D images
// TODO: decide on behavior if given a directory
// TODO: test with .gz
// TODO: place slices at right place in physical space
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

    /// whether to pad the slices
    #[arg(short, long, default_value = "false")]
    pad: bool,
}

// creates a vector of single slices from a 3D array along a given axis
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

// creates a vector of volumes holding copies of the each slice from a 3D array along a given axis
fn slice_array_pad(img: Array3<f64>, axis: &Direction) -> Vec<Slice3D> {
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
        let slice3d = ndarray::stack![Axis(axis.to_usize()), slice, slice, slice, slice];
        let slice3d = slice3d.into_dimensionality::<Ix3>().unwrap_or_else(|e| {
            eprintln!("Error! {}", e);
            std::process::exit(-2);
        });
        slices.push(Slice3D::new(slice3d, i));
    }
    slices
}

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
    for s in slices {
        let index = s.index;
        // save each slice as a nifti file
        let save_index = format!("{:03}", index + 1);
        let output_filename = format!("{basename}_axis-{a}_slice-{end_string}{save_index}.nii");
        let output_path = save_dir.join(output_filename);
        // ideally we want to caluculate the correct position of the slice in the original image
        // and then use that in the header somehow
        // but for now we will try using an empty header just to see if it works
        WriterOptions::new(&output_path)
            .reference_header(header)
            .write_nifti(&s.slice)
            .unwrap_or_else(|e| {
                eprintln!("Error! {}", e);
                std::process::exit(-2);
            });
    }
}

// main function parses commandline arguments and runs the program
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
    let (slices, end_string) = match cli.pad {
        true => {
            let slices = slice_array_pad(img_single, &axis);
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
