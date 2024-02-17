use super::GlyphMetrics;

#[cfg(feature = "std")]
use blanket::blanket;

#[cfg_attr(feature = "std", blanket(derive(Box)))]
pub trait DrawGlyph<G: GlyphMetrics> {
    /// Draws a glyph a the given position.
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

pub struct CarbonCopyCanvas<'a, I> {
    sheets: [I; 2],
    glyph_pos: usize,
    copy_mask: &'a [bool],
}

impl<'a, I> CarbonCopyCanvas<'a, I> {
    pub fn new(blink_mask: &'a [bool], front_sheet: I, back_sheet: I) -> Self {
        CarbonCopyCanvas {
            sheets: [front_sheet, back_sheet],
            glyph_pos: 0,
            copy_mask: blink_mask,
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
        let is_blinking = if self.glyph_pos >= self.copy_mask.len() {
            true
        } else {
            self.copy_mask[self.glyph_pos]
        };
        self.sheets[0].draw_glyph(glyph, pos_xy);
        if !is_blinking {
            self.sheets[1].draw_glyph(glyph, pos_xy);
        }
        self.glyph_pos += 1;
    }
}
