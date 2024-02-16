//! # Instants and Durations
//!
//! Taylored and limited to our needs for the `greaheisl` project.
//! 32 bits wide and with milli-second precision.
//!
//! Instants do not necessarily refer to a well-defined reference.
//! Supported operations:
//!
//! *   Two instants can be subtracted, the result is a duration
//! *   A duration can be added to an instant.
//!
//! Durations can be positive or negative.
//! Due to the limitation to 32 bits, durations may not exceed
//! roughly 24 days.
//!

use core::ops::{Add, AddAssign, Sub};
//use serde::{Serialize, Deserialize};

/// An instant in time, with millisecond precision and 32 bit width.
///
/// See also the module level documentation.
///
#[derive(Clone, Copy, Debug)] // later also ,Serialize,Deserialize)]
pub struct InstantMillis(InstantMillisInner);

/// The underlying integer type representing an instant
pub type InstantMillisInner = u32;
/// The underlying integer type representing a duration
///
/// See also the module level documentation.
pub type DurationMillis = i32;

impl Sub for InstantMillis {
    type Output = DurationMillis;

    fn sub(self, rhs: Self) -> Self::Output {
        let diff = self.0.wrapping_sub(rhs.0);
        // because of the representation as a 2's complement,
        // the following line will automatically result in
        // negative numbers if `rhs` lies before `self` in time,
        // provided the time difference does not exceed
        // the range of i32
        diff as DurationMillis
    }
}

impl Add<DurationMillis> for InstantMillis {
    type Output = Self;

    fn add(self, rhs: DurationMillis) -> Self::Output {
        InstantMillis(self.0.wrapping_add_signed(rhs))
    }
}

impl AddAssign<DurationMillis> for InstantMillis {
    fn add_assign(&mut self, rhs: DurationMillis) {
        self.0 = self.0.wrapping_add_signed(rhs);
    }
}

impl InstantMillis {
    /// construct an instant from a given integer value
    ///
    /// The value must refer to the same reference
    /// as all the other instances that are constructed.
    pub fn from_absolute(millis: InstantMillisInner) -> Self {
        Self(millis)
    }
    /// convert to the underlying integer representation
    pub fn into_inner(self) -> InstantMillisInner {
        self.0
    }
}
