# SliceNii

SliceNii is a very fast Rust utility for slicing and recombining NIfTI format neuroimaging data. It provides a command-line utility to split 3D NIfTI files into 2D slices and a command-line utility to recombine them into a 3D volume. The intended use case is to slice a NIfTI, perform some processing on the individual slices and then create a new volume in the same space as the original image.

## Structure

SliceNii compiles into two binaries:

1. `slicenii`: A command-line utility for slicing 3D NIfTI volumes into 2D images along a specified axis.
2. `combinenii`: A command-line utility for combining a series of 2D NIfTI slices back into a 3D volume.

## Installation

A precompiled Linux version that links to 22.04 Ubuntu libraries (specifically any recent GNU libc) should be uploaded in GitHub releases. Additionally, a version statically compiled with musl is provided that should be usable across all Linux environments (however, performance may be less optimized). Simply download, unzip the release and place the binaries somewhere on your `$PATH` environmental variable. 

### Building

If you want to build it yourself, you need to have the Rust toolchain installed on your system. If you haven't installed Rust yet, you can do so from [here](https://www.rust-lang.org/tools/install).

Once Rust is installed, you can clone this repository and build the project:

```bash
git clone https://github.com/username/slicenii.git
cd slicenii
cargo build -r
```

The `-r` flag will build optimized release binaries for your system under `slicenii/target/release`.

## Usage

### Slicing

The `slicenii.rs` script slices a 3D NIfTI file into 2D slices. The script accepts several command-line arguments, including the input file, output directory, the axis along which to slice and whether or not to pad the slices. Here is the `--help` information:

```bash
Usage: slicenii [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>    the input nifti file
  -o, --output <OUTPUT>  an output path, must be a directory which already exists, a new directory will be created within this directory to store the slices [default: ./]
  -a, --axis <AXIS>      the axis we will slice along, 0, 1, or 2 for first, second, or third axis respectively [default: 0]
  -p, --pad              whether to pad the slices (stacks 5 copies of the slice)
  -h, --help             Print help
  -V, --version          Print version
```

### Combining

The `combinenii.rs` script combines a series of 2D NIfTI files into a single 3D volume. It takes several command-line arguments, including the input directory, the output file name, the reference NIfTI file, the axis along which the volume was originally sliced, and a starting string to match the NIfTI files in the input directory. Here is the `--help` information: 

```bash
A command line tool for slicing nifti files

Usage: combinenii [OPTIONS] --reference <REFERENCE>

Options:
  -i, --input-dir <INPUT_DIR>        the input directory containing the nifti files [default: ./]
  -o, --output <OUTPUT>              an output nifti file name [default: combined.nii]
  -r, --reference <REFERENCE>        the original nifti file for reference
  -a, --axis <AXIS>                  the axis along which the volume was originally sliced by slicenii 0, 1, or 2 for first, second, or third axis respectively [default: 0]
  -s, --start-string <START_STRING>  a starting string to match the nifti files in the input directory files will be selected with this start string and then sorted [default: *]
  -h, --help                         Print help
  -V, --version                      Print version

```
