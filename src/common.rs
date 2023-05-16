use ndarray::Array3;
use std::fmt;

// set up enums and structs
#[derive(Debug, Clone)]
pub enum Direction {
    X,
    Y,
    Z,
}

impl Direction {
    pub fn to_usize(&self) -> usize {
        match self {
            Direction::X => 0,
            Direction::Y => 1,
            Direction::Z => 2,
        }
    }
    // pub fn to_string(&self) -> String {
    //     match self {
    //         Direction::X => 0.to_string(),
    //         Direction::Y => 1.to_string(),
    //         Direction::Z => 2.to_string(),
    //     }
    // }
    // fn from_usize(val: usize) -> Self {
    //     match val {
    //         0 => Direction::X,
    //         1 => Direction::Y,
    //         2 => Direction::Z,
    //         _ => unreachable!(),
    //     }
    // }
    // fn from_string(val: &str) -> Self {
    //     match val {
    //         "x" => Direction::X,
    //         "y" => Direction::Y,
    //         "z" => Direction::Z,
    //         _ => unreachable!(),
    //     }
    // }
    // fn from_unit_string(val: &str) -> Self {
    //     match val {
    //         "i" => Direction::X,
    //         "j" => Direction::Y,
    //         "k" => Direction::Z,
    //         _ => unreachable!(),
    //     }
    // }
}
impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::X => write!(f, "0"),
            Direction::Y => write!(f, "1"),
            Direction::Z => write!(f, "2"),
        }
    }
}

#[derive(Debug)]
pub struct Slice3D {
    pub slice: Array3<f64>,
    pub index: usize,
}
impl Slice3D {
    pub fn new(slice: Array3<f64>, index: usize) -> Self {
        Self { slice, index }
    }
}
