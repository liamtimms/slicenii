//! Quick commandline utility to split a nifti file into a series of 2D slices.

use clap::Parser;
use nifti::error::NiftiError;
// use nifti::volume::shape::Idx;
use ndarray::prelude::*;
use ndarray::{ArrayD, Dim, IxDynImpl};
use nifti::{InMemNiftiVolume, IntoNdArray, NiftiObject, NiftiVolume, ReaderOptions, Sliceable};

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
}

// set up enums and structs
enum PhaseEncoding {
    X,
    Y,
    Z,
}

impl PhaseEncoding {
    fn from_usize(val: usize) -> Self {
        match val {
            0 => PhaseEncoding::X,
            1 => PhaseEncoding::Y,
            2 => PhaseEncoding::Z,
            _ => unreachable!(),
        }
    }
}

struct Slice {
    slice: ndarray::Array3<f32>,
    index: usize,
}

impl Slice {
    fn new(slice: ndarray::Array3<f32>, index: usize) -> Self {
        Self { slice, index }
    }
}

// main function parses commandline arguments and runs the program
fn main() {
    let cli = Args::parse();
    let input = cli.input;
    let _output = cli.output;
    let phase_dir = match cli.phase_encoding {
        0 => PhaseEncoding::X,
        1 => PhaseEncoding::Y,
        2 => PhaseEncoding::Z,
        _ => unreachable!(),
    };
    // steps:
    // 1. load nifti
    let volume = load_nifti(&input).unwrap_or_else(|e| {
        eprintln!("Error! {}", e);
        std::process::exit(-2);
    });
    // 2. slice nifti
    // 3. pad slices
    // 3. save padded slices
    // 4. run topup in parallel on slices?
    // 5. load corrected slices
    // 6. unpad slices
    // 7. save corrected nifti
    let position: u16 = 0;
    let slice = get_nifti_slice(&volume, &phase_dir, position).unwrap_or_else(|e| {
        eprintln!("Error! {}", e);
        std::process::exit(-2);
    });
    let dims = slice.dim();
    println!("Slice: {:?}", dims);

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

fn get_nifti_slice(
    volume: &InMemNiftiVolume,
    phase_dir: &PhaseEncoding,
    position: u16,
) -> Result<Array<f32, ndarray::Dim<IxDynImpl>>, NiftiError> {
    // let dims = volume.dim();
    let slice = match phase_dir {
        PhaseEncoding::X => volume.get_slice(0, position),
        PhaseEncoding::Y => volume.get_slice(1, position),
        PhaseEncoding::Z => volume.get_slice(2, position),
    }
    .unwrap_or_else(|e| {
        eprintln!("Error! {}", e);
        std::process::exit(-2);
    });

    let slice_array = slice.into_ndarray::<f32>().unwrap_or_else(|e| {
        eprintln!("Error! {}", e);
        std::process::exit(-2);
    });

    // let mut position = volume.position();
    // let mut slice = volume.slice(phase_encoding)?;
    // let slice_dims = slice.dim();
    // println!("Slice: {:?}", dims);
    // manipulate slice here
    // save slice to file
    // let filename = format!("slices/slice_{}.nii", dims[2]);
    Ok(slice_array)
}

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
