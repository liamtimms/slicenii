//! Quick commandline utility to combine a series of nifti files into a single 3D volume.
//!
//! This script is a utility for combining a series of Nifti files into a single 3D volume. It leverages several libraries, including `clap`, `glob`, `ndarray`, and `nifti`, to facilitate the handling of command-line arguments, file paths, multi-dimensional arrays, and Nifti-specific operations, respectively.
//!

use clap::Parser;
use glob::glob;
use nalgebra::QR;
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

    /// the name of the output nifti file
    #[arg(short, long, default_value = "combined.nii")]
    output: String,

    /// the original nifti file (required for reference)
    #[arg(short, long)]
    reference: String,

    /// the axis along which the volume was sliced (0 -> X, 1 -> Y, 2 -> Z, 3 -> time, 4-> guess).
    /// If not specified, combinenii will guess
    #[arg(short, long, default_value_t = 4)]
    axis: usize,

    /// a string to select nifti files in the input directory based on the start of
    /// their file names
    #[arg(short, long, default_value = "*")]
    start_string: String,
}

/// Load slices from Nifti files located in a specified directory and based on a provided file pattern.
///
/// The function iterates over the files in the directory, sorting them by filename,
/// and transforms each file into a 3D slice. Any errors encountered during file processing
/// result in termination of the program.
///
/// # Arguments
///
/// * `_input_dir` - A `&Path` reference representing the directory where the Nifti files are located.
/// * `pattern` - A `String` that specifies the file pattern to match.
///
/// # Returns
///
/// A `Vec<Slice3D>` - A vector of `Slice3D` objects representing the slices loaded from the Nifti files.
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
    println!("{:?}", paths);
    paths.sort_by_key(|path| extract_number_from_filename(path));
    // paths.sort_by_key(|path| path.path());
    // paths.sort_by(|a, b| a.to_str().unwrap().cmp(b.to_str().unwrap()));
    // paths.sort_by(|a, b| extract_number_from_filename(a).cmp(&extract_number_from_filename(b)));
    println!("{:?}", paths);
    // paths.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
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

fn extract_number_from_filename(path: &Path) -> u128 {
    let filename = path.file_name().unwrap().to_str().unwrap();
    let mut number_str = String::new();

    // Iterate through the characters of the filename, collecting digits
    for ch in filename.chars() {
        if ch.is_digit(10) {
            number_str.push(ch);
        }
    }
    println!("Extracted number: {}", number_str);

    // Parse the collected digits as a number
    number_str.parse::<u128>().unwrap_or(0)
}

// fn extract_number_from_filename(path: &Path) -> u64 {
//     let filename = path.file_name().unwrap().to_str().unwrap();
//     let re = Regex::new(r"\d+").unwrap();
//
//     // Find all matches of numbers and take the last one
//     let last_match = re.find_iter(filename).last();
//
//     match last_match {
//         Some(m) => {
//             let last_number_str = &filename[m.start()..m.end()];
//             last_number_str.parse::<u64>().unwrap_or(0)
//         }
//         None => 0,
//     }
// }

fn guess_dir(slice_dims: &[usize], ref_dims: &[usize]) -> Direction {
    // dimension that is smaller in the slice than the reference image should be the direction
    let mut scores = [0, 0, 0, 0];
    for i in 0..3 {
        if slice_dims[i] < ref_dims[i] {
            scores[i] += 1;
        }
    }

    match scores.iter().enumerate().max_by_key(|&(_, score)| score) {
        Some((2, _)) => Direction::Z,
        Some((1, _)) => Direction::Y,
        Some((0, _)) => Direction::X,
        _ => {
            eprintln!("Warning! Could not guess the direction of the slices. Guessing time.");
            Direction::T
        }
    }
}

/// Combine multiple slices into a single 3D array.
///
/// The function takes a vector of `Slice3D` objects, an axis of type `Direction`, and a reference 3D array.
/// Each slice is processed by extracting the middle plane along the specified axis and inserting it into the 3D array.
///
/// # Arguments
///
/// * `slices` - A `Vec<Slice3D>` that contains the slices to be combined.
/// * `axis` - A `Direction` value that specifies the axis along which to combine the slices.
/// * `ref_img` - An `Array3<f64>` that serves as a reference for the shape of the combined image.
///
/// # Returns
///
/// An `Array3<f64>` - The combined 3D image.
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
            Direction::T => {
                eprintln!("Error! Wrong function called internally for Time.");
                std::process::exit(-2);
            }
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
            Direction::T => {
                std::process::exit(-2);
            }
        };
    }
    // convert to 4D for compatibility with volume combinations
    // combined_img.insert_axis(Axis(3))
    combined_img
}

fn combine_volumes(slices: Vec<Slice3D>, ref_img: Array3<f64>) -> Array4<f64> {
    // combine volumes by stacking them along the 4th dimension
    let shape = ref_img.shape();
    let fixed_shape = [shape[0], shape[1], shape[2], slices.len()];
    let mut combined_img = Array::<f64, Ix4>::zeros(fixed_shape);
    for slice in slices {
        combined_img
            .slice_mut(s![.., .., .., slice.index])
            .assign(&slice.slice);
    }
    combined_img
}

// main function parses commandline arguments and runs the program
fn main() {
    let cli = Args::parse();
    let input_dir = Path::new(&cli.input_dir);
    let output_filename = Path::new(&cli.output);
    let reference_filename = Path::new(&cli.reference);

    // check that input directory exists and has nifti files
    if !input_dir.exists() {
        eprintln!("Error! Did not find input directory. Use -i to pass an existing directory.");
        std::process::exit(-2);
    } else if !input_dir.is_dir() {
        eprintln!("Error! Input is not a directory!");
        std::process::exit(-2);
    }
    if output_filename.exists() {
        eprintln!("Error! Output file already exists. Please specify a different output file or remove existing file.");
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
        eprintln!(
            "Error! Reference nifti file must be 3D. Tip: You can use slicenii to split a 4D file."
        );
        std::process::exit(-2);
    }
    let ref_img = ref_img.into_dimensionality::<Ix3>().unwrap_or_else(|e| {
        eprintln!("Error! {}", e);
        std::process::exit(-2);
    });

    // load slices from nifti files
    let slices = load_slices_from_niftis(input_dir, pattern);
    if slices.is_empty() {
        eprintln!("Error! Did not find any files matching the string in the input directory.");
        std::process::exit(-2);
    }
    // get first slice to check dimensions
    let first_slice = &slices[0];
    let slice_dims = first_slice.slice.shape();
    let ref_dims = ref_img.shape();

    let guessed_dir = guess_dir(slice_dims, ref_dims);
    let axis = match cli.axis {
        0 => Direction::X,
        1 => Direction::Y,
        2 => Direction::Z,
        3 => Direction::T,
        _ => {
            println!("Axis not specified. Guessing axis {:?}...", guessed_dir);
            guessed_dir.clone()
        }
    };
    if guessed_dir != axis {
        println!(
            "Warning! Guessed axis {:?} does not match specified axis {:?}.",
            guessed_dir, axis
        );
    }
    // let combined_img = {
    //     if axis == Direction::T {
    //         combine_volumes(slices, ref_img)
    //     } else if slices.len() == ref_img.shape()[axis.to_usize()] {
    //         combine_slices(slices, axis, ref_img);
    //     } else {
    //         std::process::exit(-2);
    //     }
    // };
    let combined_img = {
        if axis == Direction::T {
            // combine_volumes(slices, ref_img)
            eprintln!("Error! Combining volumes not yet implemented.");
            std::process::exit(-2);
        } else if slices.len() == ref_img.shape()[axis.to_usize()] {
            combine_slices(slices, axis, ref_img)
        } else {
            eprintln!("Error! Number of slices does not match reference image.");
            std::process::exit(-2);
        }
    };

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
