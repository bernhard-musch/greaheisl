// no_std only when freature "std" is missing
#![cfg_attr(not(feature = "std"), no_std)]

use bitvec::prelude::*;

#[cfg(feature = "font")]
pub mod font;

pub trait BitVecImgView {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn as_region(&self) -> ImageRegion<'_>;
    fn row_bits(&self, y: u32) -> &BitSlice<u32, Msb0> {
        let reg = self.as_region();
        let offset = y as usize * reg.frame_width as usize;
        &reg.data[offset..(offset + self.width() as usize)] // panics if y is out of range
    }
    fn region(&self, x: u32, y: u32, width: u32, height: u32) -> ImageRegion<'_> {
        let reg = self.as_region();
        let range = region_data_range(x, y, width, height, reg.frame_width);
        ImageRegion::new(&reg.data[range], reg.frame_width, width, height)
    }
}

pub trait BitVecImgViewMut: BitVecImgView {
    fn as_region_mut(&mut self) -> ImageRegionMut<'_>;
    fn row_bits_mut(&mut self, y: u32) -> &mut BitSlice<u32, Msb0> {
        let width = self.width();
        let reg = self.as_region_mut();
        let offset = y as usize * reg.frame_width as usize;
        &mut reg.data[offset..(offset + width as usize)]
    }
    fn region_mut(&mut self, x: u32, y: u32, width: u32, height: u32) -> ImageRegionMut<'_> {
        let reg = self.as_region_mut();
        let range = region_data_range(x, y, width, height, reg.frame_width);
        ImageRegionMut::new(&mut reg.data[range], reg.frame_width, width, height)
    }
    fn copy_from(&mut self, other: &(impl BitVecImgView + ?Sized)) {
        if other.height() != self.height() {
            panic!("Cannot copy image. Source and destination dimensions must be the same!");
        }
        for y in 0..self.height() {
            self.row_bits_mut(y).copy_from_bitslice(other.row_bits(y));
        }
    }
    /// paste `other` image into this one
    ///
    /// Panics if the image does not fit.  No automatic clipping.
    fn paste(
        &mut self,
        other: &(impl BitVecImgView + ?Sized),
        x: u32,
        y: u32,
        operation: PasteOperation,
    ) {
        let mut dst = self.region_mut(x, y, other.width(), other.height());
        match operation {
            PasteOperation::Overwrite => dst.copy_from(other),
        }
    }
    /// paste `other` image into this one, with automatic clipping if needed
    ///
    ///
    fn paste_and_clip(
        &mut self,
        other: &(impl BitVecImgView + ?Sized),
        mut x: i32,
        mut y: i32,
        operation: PasteOperation,
    ) -> ClippingInfo {
        let mut width = other.width();
        let mut height = other.height();
        let src_x = clip_range(&mut x, &mut width, self.width());
        let src_y = clip_range(&mut y, &mut height, self.height());
        let paste_region = other.region(src_x, src_y, width, height);
        self.paste(&paste_region, x as u32, y as u32, operation);
        if (width == 0) | (height == 0) {
            return ClippingInfo::Hidden;
        }
        if (src_x != 0) | (src_y != 0) | (width < other.width()) | (height < other.height()) {
            return ClippingInfo::SomeClipping;
        }
        ClippingInfo::NoClipping
    }
}

pub enum ClippingInfo {
    NoClipping,
    SomeClipping,
    Hidden,
}

impl ClippingInfo {
    // get combined clipping information for two objects that have been drawn
    pub fn merge(&self, other: &Self) -> Self {
        match self {
            Self::NoClipping => match other {
                Self::NoClipping => Self::NoClipping,
                Self::SomeClipping | Self::Hidden => Self::SomeClipping,
            },
            Self::SomeClipping => Self::SomeClipping,
            Self::Hidden => match other {
                Self::NoClipping | Self::SomeClipping => Self::SomeClipping,
                Self::Hidden => Self::Hidden,
            },
        }
    }
}

impl<'a, T: BitVecImgView> From<&'a T> for ImageRegion<'a> {
    fn from(value: &'a T) -> Self {
        value.as_region()
    }
}

impl<'a, T: BitVecImgViewMut> From<&'a mut T> for ImageRegionMut<'a> {
    fn from(value: &'a mut T) -> Self {
        value.as_region_mut()
    }
}

/// the pixel-wise bit opertion between the pasted image and the existing image region
///
/// to do: support other operations, like `or` and `xor`
pub enum PasteOperation {
    /// overwrites the bits in the destination region
    Overwrite,
}

fn region_data_range(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    frame_width: u32,
) -> core::ops::Range<usize> {
    if x + width > frame_width {
        panic!("Extent of region in x exceeds data range!")
    }
    let offset = x as usize + y as usize * (frame_width as usize);
    let length = if height > 0 {
        (height - 1) as usize * frame_width as usize + width as usize
    } else {
        0
    };
    offset..(offset + length)
}

/// limitations of const generics do not allow us to compute S = W*H/32 at compile time
pub struct Image<const W: u32, const H: u32, const S: usize>(pub BitArray<[u32; S], Msb0>);

#[derive(Clone)]
pub struct ImageRegion<'a> {
    data: &'a BitSlice<u32, Msb0>,
    frame_width: u32,
    width: u32,
    height: u32,
}

pub struct ImageRegionMut<'a> {
    data: &'a mut BitSlice<u32, Msb0>,
    frame_width: u32,
    width: u32,
    height: u32,
}

impl<const W: u32, const H: u32, const S: usize> Image<W, H, S> {
    pub fn zero() -> Self {
        Image(BitArray::ZERO)
    }
}

impl<const W: u32, const H: u32, const S: usize> Default for Image<W, H, S> {
    fn default() -> Self {
        Self::zero()
    }
}

impl<const W: u32, const H: u32, const S: usize> BitVecImgView for Image<W, H, S> {
    fn width(&self) -> u32 {
        W
    }
    fn height(&self) -> u32 {
        H
    }
    fn as_region(&self) -> ImageRegion<'_> {
        let length = W as usize * H as usize;
        ImageRegion::new(&self.0[0..length], W, W, H)
    }
}

impl<const W: u32, const H: u32, const S: usize> BitVecImgViewMut for Image<W, H, S> {
    fn as_region_mut(&mut self) -> ImageRegionMut<'_> {
        let length = W as usize * H as usize;
        ImageRegionMut::new(&mut self.0[0..length], W, W, H)
    }
}

impl<'a> ImageRegion<'a> {
    pub fn new(data: &'a BitSlice<u32, Msb0>, frame_width: u32, width: u32, height: u32) -> Self {
        Self {
            data,
            frame_width,
            width,
            height,
        }
    }
}

impl<'a> BitVecImgView for ImageRegion<'a> {
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
    fn as_region(&self) -> ImageRegion<'_> {
        self.clone()
    }
}

impl<'a> ImageRegionMut<'a> {
    pub fn new(
        data: &'a mut BitSlice<u32, Msb0>,
        frame_width: u32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            data,
            frame_width,
            width,
            height,
        }
    }
}

impl<'a> BitVecImgView for ImageRegionMut<'a> {
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
    fn as_region(&self) -> ImageRegion<'_> {
        ImageRegion::new(self.data, self.frame_width, self.width, self.height)
    }
}

impl<'a> BitVecImgViewMut for ImageRegionMut<'a> {
    fn as_region_mut(&mut self) -> ImageRegionMut<'_> {
        ImageRegionMut::new(self.data, self.frame_width, self.width, self.height)
    }
}

pub fn clip_range(target_start: &mut i32, target_length: &mut u32, window_length: u32) -> u32 {
    let old_target_start = *target_start;
    *target_start = (*target_start).max(0);
    let src_start = (*target_start)
        .checked_sub(old_target_start)
        .unwrap()
        .min((*target_length).try_into().unwrap());
    let window_length_i: i32 = window_length.try_into().unwrap();
    let remaining_length: i32 = window_length_i.checked_sub(*target_start).unwrap().max(0);
    *target_length = (*target_length).min(remaining_length as u32);
    return src_start as u32;
}
