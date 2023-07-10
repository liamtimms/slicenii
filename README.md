# SliceNii

SliceNii is a very fast Rust utility for slicing and recombining NIfTI format neuroimaging data. It provides a commandline utility to split 3D NIfTI files into 2D slices and a commandline utility to recombine them into a 3D volume. The intended usecase is to slice a NIfTI, perform some processing on the individual slices and then create a new volume in the same space as the original image.

## Structure

SliceNii outputs two binaries from the following:

1. `slicenii`: A command-line utility for slicing 3D NIfTI volumes into 2D images along a specified axis.
2. `combinenii`: A command-line utility for combining a series of 2D NIfTI slices back into a 3D volume.

## Installation

A precompiled linux version that links to 22.04 Ubuntu libraries should be uploaded in github releases. Simply download, unzip the release and place the binaries somewhere on your `$PATH` environmental variable.

### Building

If you want to build it yourself, you need to have the Rust tool chain installed on your system. If you haven't installed Rust yet, you can do so from [here](https://www.rust-lang.org/tools/install).

Once Rust is installed, you can clone this repository and build the project:

```bash
git clone https://github.com/username/slicenii.git
cd slicenii
cargo build -r
```

The `-r` flag will build optimized release binaries for your system under `slicenii/target/release`.

## Usage

### Slicing

The `slicenii.rs` script slices a 3D NIfTI file into 2D slices. The script accepts several command-line arguments, including the input file, output directory, the axis along which to slice, the padding size, and the slice thickness. You can run it as follows:

```bash
slicenii -i INPUT.nii -o OUTPUT_DIRECTORY -a AXIS -p PADDING -t THICKNESS
```

Pass the `-h` flag to the binary to see further information:

```bash
slicenii -h
```

### Combining

The `combinenii.rs` script combines a series of 2D NIfTI files into a single 3D volume. It takes several command-line arguments, including the input directory, the output file name, the reference NIfTI file, the axis along which the volume was originally sliced, and a starting string to match the NIfTI files in the input directory. You can run it as follows:

```bash
combinenii -- -i INPUT_DIRECTORY -o OUTPUT.nii -r REFERENCE.nii -a AXIS -s START_STRING
```

Pass the `-h` flag to the binary to see further information.
