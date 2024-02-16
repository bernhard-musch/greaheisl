use self::typeset::{canvas::DrawGlyph, directions::RectDirection, GlyphMetrics};
pub use greaheisl_typeset as typeset;

use super::{BitVecImgView, BitVecImgViewMut, Image, ImageRegionMut, PasteOperation};

#[cfg(feature = "fitzl_font")]
pub mod fitzl_font;

pub struct BitVecImgGlyph<I: BitVecImgView> {
    pub image: I,
    pub base_point: [<Self as GlyphMetrics>::Length; 2],
    pub margin: [<Self as GlyphMetrics>::Length; 4],
}

impl<I: BitVecImgView> GlyphMetrics for BitVecImgGlyph<I> {
    type Length = i32;

    fn base_point(&self) -> [Self::Length; 2] {
        self.base_point
    }

    fn size(&self) -> [Self::Length; 2] {
        [
            self.image.width() as Self::Length,
            self.image.height() as Self::Length,
        ]
    }

    fn margin(&self, side: RectDirection) -> Self::Length {
        match side {
            RectDirection::PlusX => self.margin[0],
            RectDirection::PlusY => self.margin[1],
            RectDirection::MinusX => self.margin[2],
            RectDirection::MinusY => self.margin[3],
        }
    }
}

impl<const W: u32, const H: u32, const S: usize, I: BitVecImgView> DrawGlyph<BitVecImgGlyph<I>>
    for Image<W, H, S>
{
    fn draw_glyph(
        &mut self,
        glyph: &BitVecImgGlyph<I>,
        pos_xy: [<BitVecImgGlyph<I> as GlyphMetrics>::Length; 2],
    ) {
        self.paste_and_clip(
            &glyph.image,
            pos_xy[0],
            pos_xy[1],
            PasteOperation::Overwrite,
        );
    }
}

impl<'a, I: BitVecImgView> DrawGlyph<BitVecImgGlyph<I>> for ImageRegionMut<'a> {
    fn draw_glyph(
        &mut self,
        glyph: &BitVecImgGlyph<I>,
        pos_xy: [<BitVecImgGlyph<I> as GlyphMetrics>::Length; 2],
    ) {
        self.paste_and_clip(
            &glyph.image,
            pos_xy[0],
            pos_xy[1],
            PasteOperation::Overwrite,
        );
    }
}
