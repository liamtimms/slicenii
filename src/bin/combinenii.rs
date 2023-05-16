//! Quick commandline utility to combine a series of nifti files into a single 3D volume.

use clap::Parser;
use glob::glob;
use ndarray::prelude::*;
use ndarray::{Array3, Ix3};
use nifti::writer::WriterOptions;
use nifti::{IntoNdArray, NiftiObject, ReaderOptions};
use std::path::Path;

use slicenii::common::{Direction, Slice3D};

// use clap to create commandline interface
#[derive(Parser, Debug)]
#[command(author, about, version, long_about)]
struct Args {
    /// the input directory containing the nifti files
    #[arg(short, long, default_value = "./")]
    input_dir: String,

    /// an output nifti file name
    #[arg(short, long, default_value = "combined.nii")]
    output: String,

    /// the original nifti file for reference
    #[arg(short, long)]
    reference: String,

    /// the axis along which the volume was originally sliced by slicenii 0, 1, or 2 for first, second, or third axis respectively
    #[arg(short, long, default_value_t = 0)]
    axis: usize,

    /// a starting string to match the nifti files in the input directory files will be selected with this start string and then sorted
    #[arg(short, long, default_value = "*")]
    start_string: String,
}

fn load_slices_from_niftis(_input_dir: &Path, pattern: String) -> Vec<Slice3D> {
    let mut slices = Vec::new();
    // let mut index = 0;
    let mut paths: Vec<_> = glob(&pattern)
        .unwrap_or_else(|e| {
            eprintln!("Error! {}", e);
            std::process::exit(-2);
        })
        .filter_map(Result::ok)
        .collect();
    paths.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    for (index, path) in paths.into_iter().enumerate() {
        println!("Loading: {}", path.display());
        println!("To index: {}", index);
        let nifti = ReaderOptions::new().read_file(&path).unwrap_or_else(|e| {
            eprintln!("Error! {}", e);
            std::process::exit(-2);
        });
        let img = nifti.volume().into_ndarray::<f64>().unwrap_or_else(|e| {
            eprintln!("Error! {}", e);
            std::process::exit(-2);
        });
        let slice = img.into_dimensionality::<Ix3>().unwrap_or_else(|e| {
            eprintln!("Error! {}", e);
            std::process::exit(-2);
        });
        slices.push(Slice3D::new(slice, index));
        // index += 1;
    }

    slices
}

fn combine_slices(slices: Vec<Slice3D>, axis: Direction, ref_img: Array3<f64>) -> Array3<f64> {
    let shape = ref_img.shape();
    let fixed_shape = [shape[0], shape[1], shape[2]];
    let mut combined_img = Array::<f64, Ix3>::zeros(fixed_shape);
    let a = axis.to_usize();
    for slice in slices {
        // Calculate the middle index along the given axis
        let mid_index = slice.slice.shape()[a] / 2;

        // Slice the 3D array to get the 2D middle plane (assuming padded slices)
        let middle_plane = match axis {
            Direction::X => slice.slice.slice(s![mid_index, .., ..]).to_owned(),
            Direction::Y => slice.slice.slice(s![.., mid_index, ..]).to_owned(),
            Direction::Z => slice.slice.slice(s![.., .., mid_index]).to_owned(),
        };

        // Insert the 2D plane into the 3D array at the correct axis
        match axis {
            Direction::X => combined_img
                .slice_mut(s![slice.index, .., ..])
                .assign(&middle_plane),
            Direction::Y => combined_img
                .slice_mut(s![.., slice.index, ..])
                .assign(&middle_plane),
            Direction::Z => combined_img
                .slice_mut(s![.., .., slice.index])
                .assign(&middle_plane),
        };
    }
    combined_img
}

// main function parses commandline arguments and runs the program
fn main() {
    let cli = Args::parse();
    let input_dir = Path::new(&cli.input_dir);
    let output_filename = Path::new(&cli.output);
    let reference_filename = Path::new(&cli.reference);
    let axis = match cli.axis {
        0 => Direction::X,
        1 => Direction::Y,
        2 => Direction::Z,
        _ => {
            eprintln!("Error! Axis must be 0, 1, or 2. To indicate the 1st (x), 2nd (y), or 3rd axis (z), respectively.");
            std::process::exit(-2);
        }
    };

    // check that input directory exists and has nifti files
    if !input_dir.exists() {
        eprintln!("Error! Did not find input directory. Use -i to pass an existing directory.");
        std::process::exit(-2);
    } else if !input_dir.is_dir() {
        eprintln!("Error! Input is not a directory!");
        std::process::exit(-2);
    }

    let pattern = format!("{}/{}*.nii", input_dir.display(), cli.start_string);

    // read in reference nifti file
    if !reference_filename.exists() {
        eprintln!("Error! Did not find reference nifti file. Use -r to pass an existing file.");
        std::process::exit(-2);
    }
    let ref_obj = ReaderOptions::new()
        .read_file(reference_filename)
        .unwrap_or_else(|e| {
            eprintln!("Error! {}", e);
            std::process::exit(-2);
        });
    let ref_header = ref_obj.header();
    let ref_volume = ref_obj.volume();
    let ref_img = ref_volume.into_ndarray::<f64>().unwrap_or_else(|e| {
        eprintln!("Error! {}", e);
        std::process::exit(-2);
    });
    if ref_img.ndim() != 3 {
        eprintln!("Error! Reference nifti file must be 3D. Tip: You can use a utility like `fslsplit` to split a 4D file into 3D files.");
        std::process::exit(-2);
    }
    let ref_img = ref_img.into_dimensionality::<Ix3>().unwrap_or_else(|e| {
        eprintln!("Error! {}", e);
        std::process::exit(-2);
    });

    // load slices from nifti files
    let slices = load_slices_from_niftis(input_dir, pattern);
    if slices.len() != ref_img.shape()[axis.to_usize()] {
        eprintln!(
            "Error! Number of selected slices in input directory does not match reference image size along specified axis."
        );
        std::process::exit(-2);
    }

    let combined_img = combine_slices(slices, axis, ref_img);
    println!("Final shape: {:?}", combined_img.shape());
    // now save the combined image to a Nifti using the reference header
    WriterOptions::new(output_filename)
        .reference_header(ref_header)
        .write_nifti(&combined_img)
        .unwrap_or_else(|e| {
            eprintln!("Error! {}", e);
            std::process::exit(-2);
        });
}
