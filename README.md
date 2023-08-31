# SliceNii

SliceNii is a very fast Rust utility for slicing and recombining NIfTI format volumetric images. It provides a command-line utility to split **3D** volumes into **2D** slices (with or without padded copies to make the 2D slice volumes 3D) and a command-line utility to recombine the slices into a 3D volume. The intended use case is to split a NIfTI into individual slices, perform some processing on the slices and then create a new volume in the same space as the original image (passed as a reference).

## Structure

SliceNii compiles into two binaries:

1. `slicenii`: A command-line utility for slicing 3D NIfTI volumes into 2D images along a specified axis.
2. `combinenii`: A command-line utility for combining a series of 2D NIfTI slices back into a 3D volume.

## Installation

A precompiled Linux version that links to 22.04 Ubuntu libraries (specifically any recent GNU libc) should be uploaded in GitHub releases. Additionally, a version statically compiled with musl is which offers wider compatiblity at the cost of slightly less optimized performance is provided for environments confined to older versions of glibc. Simply download, unzip the release and add them to your `$PATH` environmental variable.

### Building

Alternatively, if you want to build it yourself, you need to have the Rust toolchain installed on your system. If you haven't installed Rust yet, you can do so from [here](https://www.rust-lang.org/tools/install).

Once Rust is installed, you can clone this repository and build the project:

```bash
git clone https://github.com/username/slicenii.git
cd slicenii
cargo build -r
```

The `-r` flag will build optimized release binaries for your system under `slicenii/target/release`.

## Usage

### Slicing

The `slicenii` binary slices a 3D NIfTI file into 2D slices. The tool accepts several command-line arguments, including the input file, output directory, the axis along which to slice and whether or not to pad the slices. Here is the `--help` information:

```
A command line tool for slicing nifti files

Usage: slicenii [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>    the input nifti file
  -o, --output <OUTPUT>  an output path where a NEW directory will be created to store the slices [default: ./]
  -a, --axis <AXIS>      Number for the axis you want to slice along: 0 -> X, 1 -> Y, 2 -> Z, or 3 -> slicenii will guess [default: 3]
  -p, --pad <PAD>        How copies of the slice pad each slice volume [default: 1]
  -h, --help             Print help
  -V, --version          Print version
```

In the case taht the image is 4D, `slicenii` will assume the 4th dimension is time and split along that first. Splitting along X, Y, or Z in 4D is not supported. NIfTI files with more than 4 dimensions (e.g. some higher dimensional warp field files output by SPM12) are not supported.

If using for TOPUP, a padding of 4 is recommended.

### Combining

The `combinenii.rs` script combines a series of 2D NIfTI files into a single 3D volume. It takes several command-line arguments, including the input directory, the output file name, the reference NIfTI file, the axis along which the volume was originally sliced, and a starting string to match the NIfTI files in the input directory. Here is the `--help` information:

```
A command line tool for slicing nifti files

Usage: combinenii [OPTIONS] --reference <REFERENCE>

Options:
  -i, --input-dir <INPUT_DIR>        the input directory containing the nifti files [default: ./]
  -o, --output <OUTPUT>              the name of the output nifti file [default: combined.nii]
  -r, --reference <REFERENCE>        the original nifti file (required for reference)
  -a, --axis <AXIS>                  the axis along which the volume was sliced (0, 1, or 2). If not specified, combinenii will guess [default: 3]
  -s, --start-string <START_STRING>  a string to select nifti files in the input directory based on the start of their file names [default: *]
  -h, --help                         Print help
  -V, --version                      Print version
```

## Known issues

Currently, NIfTI headers may not correctly locate the slices in 3D space when the acquistion was tilted at an angle relative to the scanner/real world coordinate system. The NIfTI standard defines multiple coordinate systems which can lead to lots of confusion. This can be safely ignored for the most part as long as a reference image can be used for `combinenii` as this will place combined slices back into their original coordinate system.
