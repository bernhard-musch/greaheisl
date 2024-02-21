use num::traits::{CheckedNeg, Zero};

/// four directions in two dimensions
#[derive(Clone, Copy, Debug)]
pub enum RectDirection {
    /// in direction increasing x coordinates
    PlusX = 0,
    /// in direction of increasing y coordinates 
    PlusY, 
    /// in direction of decreasing x coordinates
    MinusX, 
    /// in direction of decreasing y coordinates
    MinusY, 
}

/// the two axes in two dimensions 
#[derive(Clone, Copy, Debug)]
pub enum Axis2D {
    X,
    Y,
}

impl RectDirection {
    /// get just the axis (do not care about the direction)
    pub fn axis(self) -> Axis2D {
        use RectDirection::*;
        match self {
            PlusX => Axis2D::X,
            PlusY => Axis2D::Y,
            MinusX => Axis2D::X,
            MinusY => Axis2D::Y,
        }
    }
    /// pointing in a direction of increasing coordinate?
    pub fn is_positive(self) -> bool {
        use RectDirection::*;
        match self {
            PlusX => true,
            PlusY => true,
            MinusX => false,
            MinusY => false,
        }
    }
    /// reverses the direction
    pub fn opposite(self) -> RectDirection {
        use RectDirection::*;
        match self {
            PlusX => MinusX,
            PlusY => MinusY,
            MinusX => PlusX,
            MinusY => PlusY,
        }
    }
    /// rotates by 90 degrees
    pub fn rot90(self) -> RectDirection {
        use RectDirection::*;
        match self {
            PlusX => PlusY,
            PlusY => MinusX,
            MinusX => MinusY,
            MinusY => PlusX,
        }
    }
    /// representation as a 2D-vector (x,y) of given length 
    pub fn as_vector<T: Zero + CheckedNeg + Copy>(self, length: T) -> [T; 2] {
        use RectDirection::*;
        match self {
            PlusX => [length, T::zero()],
            PlusY => [T::zero(), length],
            MinusX => [length.checked_neg().unwrap(), T::zero()],
            MinusY => [T::zero(), length.checked_neg().unwrap()],
        }
    }
    /// rotates a vector backward 
    ///
    /// The implied angle of rotation is 
    /// * 0 degrees for `RectDirection::PlusX`,
    /// * 90 degrees for `RectDirection::PlusY`
    /// * etc.
    ///
    /// If you  compute `self.unrotate_vec(self.as_vector(L))`,
    /// you always get a vector in positive x direction,
    /// i.e. you always end up with `RectDirection::PlusX.as_vector(L)`.
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
    /// rotates a vector 
    ///
    /// The implied angle of rotation is 
    /// * 0 degrees for `RectDirection::PlusX`,
    /// * 90 degrees for `RectDirection::PlusY`
    /// * etc.
    ///
    /// If you  compute `self.rotate_vec(RectDirection::PlusX.as_vector(L))`,
    /// it's the same as simply doing `self.as_vector(L)`.
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
    /// represents the axis as a 2D vector of the given length 
    pub fn as_vector<T: Zero + CheckedNeg + Copy>(self, length: T) -> [T; 2] {
        use Axis2D::*;
        match self {
            X => [length, T::zero()],
            Y => [T::zero(), length],
        }
    }
}
