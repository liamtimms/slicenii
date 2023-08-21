//! This file provides common data structures and utilities used across the slicenii and combinenii utilities.
use ndarray::Array3;
use std::fmt;

/// The Direction enum represents the three spatial axes (X, Y, Z) in 3D space.
#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    X,
    Y,
    Z,
}

// Implement methods for the Direction enum
impl Direction {
    pub fn to_usize(&self) -> usize {
        match self {
            Direction::X => 0,
            Direction::Y => 1,
            Direction::Z => 2,
        }
    }
}
// Implement Display for Direction for printing and string conversion.
impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::X => write!(f, "0"),
            Direction::Y => write!(f, "1"),
            Direction::Z => write!(f, "2"),
        }
    }
}

// Slice3D represents a single slice of a 3D image.
#[derive(Debug)]
pub struct Slice3D {
    pub slice: Array3<f64>,
    pub index: usize,
}
// Implement methods for the Slice3D struct
impl Slice3D {
    /// Create a new Slice3D with the given slice and index.
    pub fn new(slice: Array3<f64>, index: usize) -> Self {
        Self { slice, index }
    }
}

// Vol3D represents a 3D volume from a 4D image.
#[derive(Debug)]
pub struct Vol3D {
    pub vol: Array3<f64>,
    pub index: usize,
}
// Implement methods for the Vol3D struct
impl Vol3D {
    /// Create a new Vol3D with the given volume and index.
    pub fn new(vol: Array3<f64>, index: usize) -> Self {
        Self { vol, index }
    }
}
