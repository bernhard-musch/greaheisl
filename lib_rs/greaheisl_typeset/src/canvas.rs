use super::GlyphMetrics;

#[cfg(feature = "std")]
use blanket::blanket;

/// a trait of a canvas that knows how to draw a glyph of some kind
#[cfg_attr(feature = "std", blanket(derive(Box)))]
pub trait DrawGlyph<G: GlyphMetrics> {
    /// draws a glyph a the given position.
    ///
    /// This function does not report an error.
    /// The error state may be transmitted in another way,
    /// using the (mutable) state of the object.
    fn draw_glyph(&mut self, glyph: &G, pos_xy: [G::Length; 2]);
}

impl<T: DrawGlyph<G>, G: GlyphMetrics> DrawGlyph<G> for &mut T {
    fn draw_glyph(&mut self, glyph: &G, pos_xy: [G::Length; 2]) {
        (**self).draw_glyph(glyph, pos_xy)
    }
}

/// a specialized canvas copying a selection of glyphs to two "sheets"
///
/// Suppose we are given two canvases, both exhibiting the [`DrawGlyph`] trait.
/// Let's call these canvases "sheets". The `CarbonCopyCanvas` allows
/// you to print letters on both sheets at the same time.
/// You are also able to specify a mask that determines which of
/// the letters are copied to both sheets, and which are only shown
/// on one of the sheets.
///
/// This mechanism can be used to prepare blinking text.
pub struct CarbonCopyCanvas<'a, I> {
    sheets: [I; 2],
    glyph_pos: usize,
    blink_mask: &'a [bool],
}

impl<'a, I> CarbonCopyCanvas<'a, I> {
    /// constructor
    ///
    /// Everything is printed on the `front_sheet`,
    /// but only letters in positions where the `blink_mask` is false
    /// end up on the `back_sheet`.
    pub fn new(blink_mask: &'a [bool], front_sheet: I, back_sheet: I) -> Self {
        CarbonCopyCanvas {
            sheets: [front_sheet, back_sheet],
            glyph_pos: 0,
            blink_mask,
        }
    }
}

impl<'a, I> From<CarbonCopyCanvas<'a, I>> for [I; 2] {
    fn from(value: CarbonCopyCanvas<'a, I>) -> Self {
        value.sheets
    }
}

impl<'a, G, I> DrawGlyph<G> for CarbonCopyCanvas<'a, I>
where
    I: DrawGlyph<G>,
    G: GlyphMetrics,
{
    fn draw_glyph(&mut self, glyph: &G, pos_xy: [G::Length; 2]) {
        let is_blinking = if self.glyph_pos >= self.blink_mask.len() {
            true
        } else {
            self.blink_mask[self.glyph_pos]
        };
        self.sheets[0].draw_glyph(glyph, pos_xy);
        if !is_blinking {
            self.sheets[1].draw_glyph(glyph, pos_xy);
        }
        self.glyph_pos += 1;
    }
}
