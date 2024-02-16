

use num::traits::{CheckedNeg, Zero};

#[derive(Clone, Copy, Debug)]
pub enum RectDirection {
    PlusX = 0,
    PlusY,
    MinusX,
    MinusY,
}

#[derive(Clone, Copy, Debug)]
pub enum Axis2D {
    X,
    Y,
}

impl RectDirection {
    pub fn axis(self) -> Axis2D {
        use RectDirection::*;
        match self {
            PlusX => Axis2D::X,
            PlusY => Axis2D::Y,
            MinusX => Axis2D::X,
            MinusY => Axis2D::Y,
        }
    }
    pub fn is_positive(self) -> bool {
        use RectDirection::*;
        match self {
            PlusX => true,
            PlusY => true,
            MinusX => false,
            MinusY => false,
        }
    }
    pub fn opposite(self) -> RectDirection {
        use RectDirection::*;
        match self {
            PlusX => MinusX,
            PlusY => MinusY,
            MinusX => PlusX,
            MinusY => PlusY,
        }
    }
    pub fn rot90(self) -> RectDirection {
        use RectDirection::*;
        match self {
            PlusX => PlusY,
            PlusY => MinusX,
            MinusX => MinusY,
            MinusY => PlusX,
        }
    }
    pub fn as_vector<T: Zero + CheckedNeg + Copy>(self, length: T) -> [T; 2] {
        use RectDirection::*;
        match self {
            PlusX => [length, T::zero()],
            PlusY => [T::zero(), length],
            MinusX => [length.checked_neg().unwrap(), T::zero()],
            MinusY => [T::zero(), length.checked_neg().unwrap()],
        }
    }
    pub fn unrotate_vec<T: CheckedNeg + Copy>(self, vector: [T; 2]) -> [T; 2] {
        use RectDirection::*;
        let [x, y] = vector;
        match self {
            PlusX => [x, y],
            PlusY => [y, x.checked_neg().unwrap()],
            MinusX => [x.checked_neg().unwrap(), y.checked_neg().unwrap()],
            MinusY => [y.checked_neg().unwrap(), x],
        }
    }
    pub fn rotate_vec<T: CheckedNeg + Copy>(self, vector: [T; 2]) -> [T; 2] {
        use RectDirection::*;
        let [x, y] = vector;
        match self {
            PlusX => [x, y],
            PlusY => [y.checked_neg().unwrap(), x],
            MinusX => [x.checked_neg().unwrap(), y.checked_neg().unwrap()],
            MinusY => [y, x.checked_neg().unwrap()],
        }
    }
}

impl Axis2D {
    pub fn as_vector<T: Zero + CheckedNeg + Copy>(self, length: T) -> [T; 2] {
        use Axis2D::*;
        match self {
            X => [length, T::zero()],
            Y => [T::zero(), length],
        }
    }
}
