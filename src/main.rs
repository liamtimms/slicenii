//! Quick commandline utility to split a nifti file into a series of 2D slices.

use clap::Parser;
use nifti::error::NiftiError;
use std::path::{Path, PathBuf};
use std::{fs, io};
// use nifti::volume::shape::Idx;
use ndarray::prelude::*;
use ndarray::{Array3, ArrayD, Ix3, IxDynImpl};
use nifti::writer::WriterOptions;
use nifti::{
    header, volume, InMemNiftiVolume, IntoNdArray, NiftiObject, NiftiVolume, ReaderOptions,
    Sliceable,
};

// use clap to create commandline interface
#[derive(Parser, Debug)]
#[command(author, about, version)]
struct Args {
    // the input nifti file
    #[arg(short, long, default_value = "test.nii")]
    input: String,

    // an output name
    #[arg(short, long, default_value = "output.nii")]
    output: String,

    #[arg(short, long, default_value_t = 0)]
    phase_encoding: usize,

    #[arg(short, long, default_value_t = 1)]
    second_axis: usize,

    #[arg(short, long, default_value_t = 0)]
    axis: usize,
}

// set up enums and structs
#[derive(Debug)]
enum Direction {
    X,
    Y,
    Z,
}

impl Direction {
    fn to_usize(&self) -> usize {
        match self {
            Direction::X => 0,
            Direction::Y => 1,
            Direction::Z => 2,
        }
    }
    fn from_usize(val: usize) -> Self {
        match val {
            0 => Direction::X,
            1 => Direction::Y,
            2 => Direction::Z,
            _ => unreachable!(),
        }
    }
    fn from_string(val: &str) -> Self {
        match val {
            "x" => Direction::X,
            "y" => Direction::Y,
            "z" => Direction::Z,
            _ => unreachable!(),
        }
    }
    fn from_unit_string(val: &str) -> Self {
        match val {
            "i" => Direction::X,
            "j" => Direction::Y,
            "k" => Direction::Z,
            _ => unreachable!(),
        }
    }
}

struct Slice {
    slice: ndarray::Array2<f64>,
    index: usize,
}

impl Slice {
    fn new(slice: ndarray::Array2<f64>, index: usize) -> Self {
        Self { slice, index }
    }
}

// creates a vector of slices from a 3D array
fn slice_array(img: ndarray::Array3<f64>, axis: Direction) -> Vec<Slice> {
    let shape = img.shape();
    let end_index = shape[axis.to_usize()];
    let mut slices = Vec::new();
    for i in 0..end_index {
        let slice = img.index_axis(Axis(axis.to_usize()), i);
        println!("Slice mean: {}", slice.mean().unwrap());
        let slice = slice.into_dimensionality::<Ix2>().unwrap_or_else(|e| {
            eprintln!("Error! {}", e);
            std::process::exit(-2);
        });
        let slice = slice.into_owned();
        slices.push(Slice::new(slice, i));
    }
    slices
}

// main function parses commandline arguments and runs the program
fn main() {
    let cli = Args::parse();
    let input = cli.input;
    let input_path = Path::new(&input);

    let basename = match input_path.file_stem() {
        Some(name) => name.to_str().unwrap(),
        None => {
            eprintln!("Error! Could not parse input file name.");
            std::process::exit(-2);
        }
    };

    let _output = cli.output;
    let phase_enc = match cli.phase_encoding {
        0 => Direction::X,
        1 => Direction::Y,
        2 => Direction::Z,
        _ => unreachable!(),
    };
    let second_axis = match cli.second_axis {
        0 => Direction::X,
        1 => Direction::Y,
        2 => Direction::Z,
        _ => unreachable!(),
    };
    let axis = match cli.axis {
        0 => Direction::X,
        1 => Direction::Y,
        2 => Direction::Z,
        _ => unreachable!(),
    };
    // steps:
    let obj = ReaderOptions::new().read_file(&input).unwrap_or_else(|e| {
        eprintln!("Error! {}", e);
        std::process::exit(-2);
    });
    let header = obj.header();
    let volume = obj.volume();
    let dims = volume.dim();
    println!("Dims: {:?}", dims);
    let img = volume.into_ndarray::<f64>().unwrap_or_else(|e| {
        eprintln!("Error! {}", e);
        std::process::exit(-2);
    });
    if img.ndim() != 3 {
        eprintln!("Error! Input nifti file must be 3D. Tip: You can use fslsplit to split a 4D file into 3D files.");
        std::process::exit(-2);
    }
    // shave off dimension 4 for now
    let img_single = img.into_dimensionality::<Ix3>().unwrap_or_else(|e| {
        eprintln!("Error! {}", e);
        std::process::exit(-2);
    });
    let slices = slice_array(img_single, axis);
    // create save directory if it doesn't exist
    let save_dir = Path::new("slices");
    if !save_dir.exists() {
        fs::create_dir(save_dir).unwrap_or_else(|e| {
            eprintln!("Error! {}", e);
            std::process::exit(-2);
        });
    }
    for s in slices {
        let shape = s.slice.shape();
        let index = s.index;
        // save each slice as a nifti file
        let save_index = (index + 1).to_string();
        let output_filename = format!("{basename}_slice-{save_index}.nii");
        let output_path = save_dir.join(output_filename);
        println!("Output: {}", output_path.display());
        println!("Slice mean: {}", s.slice.mean().unwrap());
        // ideally we want to caluculate the correct position of the slice in the original image
        // and then use that in the header somehow
        // but for now we will try using an empty header just to see if it works
        WriterOptions::new(&output_path)
            // .reference_header(header)
            .write_nifti(&s.slice)
            .unwrap_or_else(|e| {
                eprintln!("Error! {}", e);
                std::process::exit(-2);
            });
    }
    // now lets also output the padded slices

    // let shape = img_single.shape();
    // // iterate over the phase encoding direction
    // let end_index = shape[axis.to_usize()];
    // println!("Slicing along axis: {:?}", axis);
    // // println!("Second axis: {:?}", second_axis);
    // println!("End index: {}", end_index);
    // for i in 0..end_index {
    //     let slice = img_single.index_axis(Axis(axis.to_usize()), i);
    //     let slice = slice.into_dimensionality::<Ix2>().unwrap_or_else(|e| {
    //         eprintln!("Error! {}", e);
    //         std::process::exit(-2);
    //     });
    //     // let slice = slice.into_owned();
    //     println!("Slice shape: {:?}", slice.shape());
    // }

    // // Find the index of the smallest dimension
    // let (second_axis, smallest_dimension) = shape
    //     .iter()
    //     .enumerate()
    //     .min_by_key(|&(_, dim)| dim)
    //     .unwrap();

    // println!(
    //     "The smallest dimension is at index {} with value {}",
    //     second_axis, smallest_dimension
    // );
    // println!("Phase encoding direction: {:?}", phase_enc);

    // println!("Dims: {:?}", img_3_d.dim());
    // WriterOptions::new("test_out.nii")
    //     .reference_header(&header)
    //     .write_nifti(&img_3_d)
    //     .unwrap_or_else(|e| {
    //         eprintln!("Error! {}", e);
    //         std::process::exit(-2);
    //     });
    // figure out how to slice nifti
    // want to do it along the plane defined by phase encoding direction and smallest matrix dimension
    //
    // 2. slice nifti
    // 3. pad slices
    // 3. save padded slices
    // 4. run topup in parallel on slices?
    // 5. load corrected slices
    // 6. unpad slices
    // 7. save corrected nifti
    //
    // for now lets try just saving the new img array as a nifti

    // let position: u16 = 0;
    // let slice = get_nifti_slice(&volume, &phase_dir, position).unwrap_or_else(|e| {
    //     eprintln!("Error! {}", e);
    //     std::process::exit(-2);
    // });
    // let dims = slice.dim();
    // println!("Slice: {:?}", dims);
    //
    // run().unwrap_or_else(|e| {
    //     eprintln!("Error! {}", e);
    //     std::process::exit(-2);
    // });
    // run_stream().unwrap_or_else(|e| {
    //     eprintln!("Error! {}", e);
    //     std::process::exit(-2);
    // });
    //
    // ideally we want to break into slices based on the phase encoding direction in some way
    // for now we want to implement that as just a commandline parameter
}

// fn get_nifti_slice(
//     volume: &InMemNiftiVolume,
//     phase_dir: &Direction,
//     position: u16,
// ) -> Result<Array<f32, ndarray::Dim<IxDynImpl>>, NiftiError> {
//     // let dims = volume.dim();
//
//     let slice = match phase_dir {
//         Direction::X => volume.get_slice(0, position),
//         Direction::Y => volume.get_slice(1, position),
//         Direction::Z => volume.get_slice(2, position),
//     }
//     .unwrap_or_else(|e| {
//         eprintln!("Error! {}", e);
//         std::process::exit(-2);
//     });
//
//     let slice_array = slice.into_ndarray::<f32>().unwrap_or_else(|e| {
//         eprintln!("Error! {}", e);
//         std::process::exit(-2);
//     });
//
//     // let mut position = volume.position();
//     // let mut slice = volume.slice(phase_encoding)?;
//     // let slice_dims = slice.dim();
//     // println!("Slice: {:?}", dims);
//     // manipulate slice here
//     // save slice to file
//     // let filename = format!("slices/slice_{}.nii", dims[2]);
//     Ok(slice_array)
// }
//
fn load_nifti(input: &str) -> Result<InMemNiftiVolume, NiftiError> {
    let obj = ReaderOptions::new().read_file(input)?;
    let volume = obj.into_volume();
    Ok(volume)
}

// fn run() -> Result<(), NiftiError> {
//     let obj = ReaderOptions::new().read_file("test.nii")?;
//     // use obj
//     let header = obj.header();
//     let volume = obj.volume();
//     let dims = volume.dim();
//     Ok(())
// }
//
// fn run_stream() -> Result<(), NiftiError> {
//     let obj = ReaderStreamedOptions::new().read_file("test.nii")?;
//
//     // // make a folder to store individual slices
//     // let mut folder = std::fs::create_dir("slices")?;
//     let mut volume = obj.into_volume();
//     for slice_pair in volume.indexed() {
//         let (idx, slice): (Idx, InMemNiftiVolume) = slice_pair?;
//         // use idx and slice
//         // let dims = slice.dim();
//         // println!("Slice: {:?}", dims);
//         println!("Index: {:?}", idx);
//     }
//
//     // for slice in volume {
//     //     let slice = slice?;
//     //     let dims = slice.dim();
//     //     let mut position = slice.position();
//     //     println!("Slice: {:?}", dims);
//     //     // manipulate slice here
//     //     // save slice to file
//     //     // let filename = format!("slices/slice_{}.nii", dims[2]);
//     // }
//     Ok(())
// }
