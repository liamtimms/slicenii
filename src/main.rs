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
use rayon::prelude::*;

//TODO: add argument to choose padded vs not
//TODO: clean up
//TODO: add support for 4D images
//TODO: decide on behavior if given a directory
//TODO: test with .gz
//TODO: place slices at right place in physical space

// use clap to create commandline interface
#[derive(Parser, Debug)]
#[command(author, about, version, long_about)]
struct Args {
    // the input nifti file
    #[arg(short, long, default_value = "test.nii")]
    input: String,

    // an output path, must be a directory which already exists, a new directory will be created
    // within this directory to store the slices.
    #[arg(short, long, default_value = "./")]
    output: String,

    // the axis we will slice along, 0, 1, or 2 for first, second, or third axis respectively
    #[arg(short, long, default_value_t = 0)]
    axis: usize,

    // whether to pad the slices
    #[arg(short, long, default_value = "false")]
    pad: bool,
}

// set up enums and structs
#[derive(Debug, Clone)]
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
    fn to_string(&self) -> String {
        match self {
            Direction::X => 0.to_string(),
            Direction::Y => 1.to_string(),
            Direction::Z => 2.to_string(),
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

#[derive(Debug)]
struct Slice {
    slice: ndarray::Array2<f64>,
    index: usize,
}

impl Slice {
    fn new(slice: ndarray::Array2<f64>, index: usize) -> Self {
        Self { slice, index }
    }
}

#[derive(Debug)]
struct Slice3D {
    slice: ndarray::Array3<f64>,
    index: usize,
}
impl Slice3D {
    fn new(slice: ndarray::Array3<f64>, index: usize) -> Self {
        Self { slice, index }
    }
}

// fn pad_slice(slice: &Slice, axis: &Direction) -> Slice {
//     let mut slice = slice.clone();
//     let mut slice = slice.slice;
//     let mut shape = slice.shape();
//     let mut pad_width = vec![(0, 0); 2];
//     let axis_index = axis.to_usize();
//     shape[axis_index] = 1;
//     pad_width[axis_index] = (slice.shape()[axis_index], slice.shape()[axis_index]);
//     let pad_width = pad_width.as_slice();
//     // slice = ndarray::pad(&slice, pad_width, &ndarray::Zip::from(|x, y| *x = *y));
//     let slice = slice.into_owned();
//     let slice = slice.into_dimensionality::<Ix3>().unwrap_or_else(|e| {
//         eprintln!("Error! {}", e);
//         std::process::exit(-2);
//     });
//     let slice = Slice::new(slice, slice.index());
//     slice
// }

// creates a vector of slices from a 3D array
fn slice_array(img: ndarray::Array3<f64>, axis: &Direction) -> Vec<Slice3D> {
    let shape = img.shape();
    let end_index = shape[axis.to_usize()];
    let mut slices = Vec::new();
    for i in 0..end_index {
        let slice = img.index_axis(Axis(axis.to_usize()), i);
        let slice = slice.into_dimensionality::<Ix2>().unwrap_or_else(|e| {
            eprintln!("Error! {}", e);
            std::process::exit(-2);
        });
        let slice3d = slice.insert_axis(Axis(axis.to_usize()));
        let slice3d = slice3d.into_dimensionality::<Ix3>().unwrap_or_else(|e| {
            eprintln!("Error! {}", e);
            std::process::exit(-2);
        });
        let slice = slice.into_owned();
        let slice3d = slice3d.into_owned();
        // slices.push(Slice::new(slice, i));
        slices.push(Slice3D::new(slice3d, i));
    }
    slices
}

fn slice_array_pad(img: ndarray::Array3<f64>, axis: &Direction) -> Vec<Slice3D> {
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

fn fill_planes_along_axis(img: &Array3<f32>, axis: Axis) -> Array3<f32> {
    let shape = img.dim();
    let mut filled_img = Array3::<f32>::zeros(shape);

    // fill an array with the same plane along the axis
    for (index, plane) in filled_img.axis_iter(axis).enumerate() {}

    filled_img
}

fn save_slices(
    slices: Vec<Slice3D>,
    axis: &Direction,
    header: &nifti::NiftiHeader,
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
        let output_filename = format!("{basename}_axis-{a}_slice-{save_index}{end_string}.nii");
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
    // for slice in slices {
    //     let mut save_path = save_dir.clone();
    //     let index = slice.index;
    //     let index = format!("{:03}", index);
    //     let filename = format!("{}_{}.png", basename, index);
    //     save_path.push(filename);
    //     let slice = slice.slice;
    //     let slice = slice.into_dimensionality::<Ix2>().unwrap_or_else(|e| {
    //         eprintln!("Error! {}", e);
    //         std::process::exit(-2);
    //     });
    //     let slice = slice.into_owned();
    //     let slice = slice.mapv(|x| x * 255.0);
    //     let slice = slice.mapv(|x| x as u8);
    //     let slice = image::DynamicImage::ImageLuma8(slice);
    //     slice.save(save_path).unwrap_or_else(|e| {
    //         eprintln!("Error! {}", e);
    //         std::process::exit(-2);
    //     });
    // }
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
    let axis_pixdim = pixdim[axis.to_usize() + 1];
    // get the volume
    let volume = obj.volume();
    let dims = volume.dim();
    // print out some info
    println!("Orignal image array dimensions: {:?}", dims);
    println!("Orignal image voxel dimensions: {:?}", pixdim);
    println!("Selected axis voxel size: {:?}", axis_pixdim);
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
    // let slices = slice_array_pad(img_single, &axis);
    // create save directory if it doesn't exist
    // let save_dir_name = format!("{basename}_slices");
    // let save_dir = Path::new(&save_dir_name);
    // if !save_dir.exists() {
    //     fs::create_dir(save_dir).unwrap_or_else(|e| {
    //         eprintln!("Error! {}", e);
    //         std::process::exit(-2);
    //     });
    // }
    match cli.pad {
        true => {
            let slices = slice_array_pad(img_single, &axis);
            let end_string = format!("-padded");
            save_slices(
                slices,
                &axis,
                &header,
                &output_basepath,
                &basename,
                &end_string,
            );
        }
        false => {
            let slices = slice_array(img_single, &axis);
            let end_string = format!("");
            save_slices(
                slices,
                &axis,
                &header,
                &output_basepath,
                &basename,
                &end_string,
            );
        }
    }
    // let mut slice_header = NiftiHeader::default();
    // for s in slices {
    //     let index = s.index;
    //     // save each slice as a nifti file
    //     let save_index = format!("{:03}", index + 1);
    //     let output_filename = format!("{basename}_axis-{a}_slice-{save_index}{end_string}.nii");
    //     let output_path = save_dir.join(output_filename);
    //     // ideally we want to caluculate the correct position of the slice in the original image
    //     // and then use that in the header somehow
    //     // but for now we will try using an empty header just to see if it works
    //     WriterOptions::new(&output_path)
    //         .reference_header(header)
    //         .write_nifti(&s.slice)
    //         .unwrap_or_else(|e| {
    //             eprintln!("Error! {}", e);
    //             std::process::exit(-2);
    //         });
    // }
    //
    // write in parallel
    // slices.par_iter().for_each(|s| {
    //     println!("Slice index: {}", s.index);
    // })
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
