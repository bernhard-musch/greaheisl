//! This library defines what a font is and helps with typesetting.
//! 
//! * designed to support different writing directions, but I do not know
//!   if this is sufficient to support all major written languages
//! * independent of how the font is rendered (pixel font / splines / ... )
//! * designed with no_std-compatibility in mind
//! * may lack some important features, since I have little knowledge of typography
//! * not well tested at all, especialy for other writing directions or coordinate conventions
//! * So far, typesetting is restricted to a single line of text.

// no_std only when freature "std" is missing
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use blanket::blanket;

use num::traits::{CheckedAdd, CheckedNeg, CheckedSub};
use num::{FromPrimitive, ToPrimitive};

pub mod directions;
use directions::{Axis2D, RectDirection};

pub mod canvas;
use canvas::DrawGlyph;

#[cfg_attr(feature = "std", blanket(derive(Rc, Arc, Box)))]
/// size, margin and placement of a glyph
pub trait GlyphMetrics {
    /// data type for graphics coordinates, displacements and lengths
    type Length: num::Num + CheckedNeg + CheckedAdd + CheckedSub + Ord + Copy;
    /// (x,y) coordinates of the base point,
    /// in the coordinate system of the graphics
    ///
    /// The "base point" generalizes the idea of a "base line".
    /// A "base line" is only one coordinate.
    /// This not enough information if the writing direction
    /// is not known a priori.
    ///
    /// For writing direction from left to right,
    /// the base point must be placed on the base line.
    /// It is sensible to put it in a horizontally centered
    /// position. Then, should the user choose a
    /// downward writign direction, you get nicely
    /// centered letters, as expected.
    fn base_point(&self) -> [Self::Length; 2];
    /// size of the graphics bounding box: width, height (positive numbers)
    fn size(&self) -> [Self::Length; 2];
    /// queries the margin around the graphics on one of the four sides of the bounding box
    ///
    /// must return a positive number
    fn margin(&self, side: RectDirection) -> Self::Length;
}

impl<T: GlyphMetrics> GlyphMetrics for &T {
    type Length = T::Length;
    fn base_point(&self) -> [Self::Length; 2] {
        (**self).base_point()
    }
    fn size(&self) -> [Self::Length; 2] {
        (**self).size()
    }
    fn margin(&self, side: RectDirection) -> Self::Length {
        (**self).margin(side)
    }
}

/// positioning of glyphs along a line in writing direction
///
/// After each typesetting operation, the internal position is shifted,
/// such that the next glyph is placed in an adjacent position.
/// The margins defined by the glyphs determine the free space
/// between the glyphs. For example, if the writing direction is
/// left to right, then the right margin of the previous glyph
/// or the left margin of the current glyph, whichever one is larger,
/// determines the space between the glyphs.
///
/// The line typesetter does not support placing several
/// glyphs on top of each other. This might be desirable to
/// form "super glyphs" made up of several graphical elements.
/// However, this can also be achieved by implementing
/// glyphs that can be composed of several sub-glyphs.
pub struct LineTypesetter<L> {
    pos_unrot: [L; 2],
    writing_direction: RectDirection,
    last_margin: Option<L>,
}

impl<L: num::Num + CheckedNeg + CheckedAdd + CheckedSub + Ord + Copy> LineTypesetter<L> {
    /// constructor
    ///
    /// `initial_pos_xy` is the initial writing position and sets the base line.
    pub fn new(initial_pos_xy: [L; 2], writing_direction: RectDirection) -> LineTypesetter<L> {
        LineTypesetter {
            pos_unrot: writing_direction.unrotate_vec(initial_pos_xy),
            writing_direction,
            last_margin: None,
        }
    }
    /// returns the x,y-coordinates where to place the glyph and advances the writing position
    ///
    /// The way typeset_glyph works, the glyphs you typeset will never overlap.
    /// 
    /// If you have a character that is made up of several overlapping glyphs,
    /// you need to construct a glyph type that is able to 
    /// combine several glyphs into one "super glyph".
    pub fn typeset_glyph(&mut self, glyph: &impl GlyphMetrics<Length = L>) -> [L; 2] {
        let margin = if let Some(last_margin) = self.last_margin {
            last_margin.max(glyph.margin(self.writing_direction.opposite()))
        } else {
            L::zero()
        };
        let base_point_unrot = self.writing_direction.unrotate_vec(glyph.base_point());
        let size_unrot = self.writing_direction.unrotate_vec(glyph.size());
        self.pos_unrot[0] = self.pos_unrot[0].checked_add(&margin).unwrap();
        let result_unrot = [
            self.pos_unrot[0],
            self.pos_unrot[1].checked_sub(&base_point_unrot[1]).unwrap(),
        ];
        self.pos_unrot[0] = self.pos_unrot[0].checked_add(&size_unrot[0]).unwrap();
        self.last_margin = Some(glyph.margin(self.writing_direction));
        self.writing_direction.rotate_vec(result_unrot)
    }
    /// returns the current writing position
    ///
    /// Each time a glyph is placed using [`LineTypesetter::typeset_glyph`],
    /// the current writing position advances in writng direction,
    /// along the base line.
    pub fn pos_xy(&self) -> [L; 2] {
        self.writing_direction.rotate_vec(self.pos_unrot)
    }
    pub fn writing_direction(&self) -> RectDirection {
        self.writing_direction
    }
    /// advances the writing position along the base line by the given length
    pub fn skip(&mut self, width: L) {
        self.pos_unrot[0] = self.pos_unrot[0].checked_add(&width).unwrap();
    }
}

/// geometric properties of a font (rather than the individual glyph)
pub struct FontMetrics<L> {
    /// determines where to place the first base line relative to the coordinate origin of the text box
    ///
    /// This is an x-coordinate if the line feed direction is along the x-axis, otherwise a y-coordinate.
    pub base_line_offset: L,
    /// distance between two base lines
    pub line_to_line_distance: L,
}

#[cfg_attr(feature = "std", blanket(derive(Rc, Arc, Box)))]
/// basic information and capabilities associated with a font
pub trait FontInfo {
    /// the type of a glyph
    type Glyph: GlyphMetrics;
    /// get FontMetrics information depending on line feed axis
    ///
    /// If the given `line_feed_axis` is not supported by the font,
    /// the function returns `None`.
    /// The function should not return `None` for 
    /// `default_line_feed_direction().axis()`.
    fn get_font_spec(
        &self,
        line_feed_axis: Axis2D,
    ) -> Option<FontMetrics<<Self::Glyph as GlyphMetrics>::Length>>;
    /// The default line feed direction of the font.
    fn default_line_feed_direction(&self) -> RectDirection;
    /// The default writing direction of the font.
    ///
    /// Must be orthogonal to the default line feed direction.
    fn default_writing_direction(&self) -> RectDirection;
}

impl<T: FontInfo> FontInfo for &T {
    type Glyph = T::Glyph;
    fn get_font_spec(
        &self,
        line_feed_axis: Axis2D,
    ) -> Option<FontMetrics<<Self::Glyph as GlyphMetrics>::Length>> {
        (**self).get_font_spec(line_feed_axis)
    }
    fn default_line_feed_direction(&self) -> RectDirection {
        (**self).default_line_feed_direction()
    }
    fn default_writing_direction(&self) -> RectDirection {
        (**self).default_writing_direction()
    }
}

#[cfg_attr(feature = "std", blanket(derive(Rc, Arc, Box)))]
/// abstract definition of a font
///
/// A font is characterized by its ability to translate a string
/// into a sequence of glyphs.
pub trait Font: FontInfo {
    type GlyphIterator<'a>: Iterator<Item = Result<Self::Glyph, char>>
    where
        Self: 'a;
    /// Represents a character as a glyph. This only works for simple characters / fonts.
    /// 
    /// If you apply this function to the
    /// sequence of characters in a string, you do not necessarily
    /// get the same result as you would get from applying
    /// `str_to_glyph` to that string as a whole. 
    /// One obvious reason for this is that `str_to_glyph` can detect where
    /// to place ligatures, whereas `char_to_glyph` obviously cannot.
    fn char_to_glyph(&self, ch: char) -> Result<Self::Glyph, char>;
    /// translates a UTF8 string into a sequence of non-overlapping glyphs to be
    /// typeset using, e.g. `LineTypesetter`.
    fn str_to_glyphs<'a, 'b: 'a>(&'b self, text: &'a str) -> Self::GlyphIterator<'a>
    where
        Self: 'a;
    /// suggested symbol to be shown when a character is not defined
    ///
    /// Typically, fonts implement only a subset of the UTF8 character set.
    /// If a character is encountered for which no glyph is provided,
    /// we can substitude this glyph to make the problem obvious to the reader.
    fn default_notdef_glyph(&self) -> Option<Self::Glyph>;
}

impl<T: Font> Font for &T {
    type GlyphIterator<'a>  = T::GlyphIterator<'a> where Self: 'a;
    /// only works for simple characters / fonts
    fn char_to_glyph(&self, ch: char) -> Result<Self::Glyph, char> {
        (**self).char_to_glyph(ch)
    }
    fn str_to_glyphs<'a, 'b: 'a>(&'b self, text: &'a str) -> Self::GlyphIterator<'a>
    where
        Self: 'a,
    {
        (**self).str_to_glyphs(text)
    }
    fn default_notdef_glyph(&self) -> Option<Self::Glyph> {
        (**self).default_notdef_glyph()
    }
}

/// pre-defined iterator to make it easy to implement simple fonts
///
/// What we mean by simple fonts in this context: A font that
/// associates exactly one glyph with each one of the characters it supports.
/// For example, a simple font does not recognize and substitute ligatures.
/// In this case, it does not matter whether you use `str_to_glyphs`
/// or `char_to_glyph` on the sequence of characters. You 
/// get the same result in both cases.
pub struct SimpleFontGlyphIterator<'a, F> {
    pub font: &'a F,
    pub text: core::str::Chars<'a>,
}

impl<'a, F: Font> Iterator for SimpleFontGlyphIterator<'a, F> {
    type Item = Result<F::Glyph, char>;

    fn next(&mut self) -> Option<Self::Item> {
        self.text.next().map(|ch| self.font.char_to_glyph(ch))
    }
}

#[derive(Debug)]
pub enum PrinterError {
    GlyphNotDefined(char),
    UnsupportedControlChar(char),
}

/*
pub trait GlyphPrinter {
    type Glyph;
    fn typesetter(&self) -> &LineTypesetter;
    fn typesetter_mut(&mut self) -> &mut LineTypesetter;
    fn print_glyph(&mut self, glyph: &Self::Glyph);
    fn print_space(&mut self, width: i16);
}
*/

//Blanket macro does not work due to print_uint generics.
//#[blanket(derive(Mut,Box))]
/// capabilities a "text printer" should have
pub trait TextPrinterTrait {
    /// print a single character
    fn print_char(&mut self, ch: char) -> Result<(), PrinterError>;
    /// print a string
    fn print_str(&mut self, s: &str) -> Result<(), PrinterError>;
    /// print a decimal digit
    fn print_digit(&mut self, digit: u8) -> Result<(), PrinterError> {
        let Some(ch) = char::from_digit(digit as u32, 10) else {
            panic!("`print_digit() expects a digit in the range 0..9.")
        };
        self.print_char(ch)
    }
    /// print an unsigned integer as a decimal number with a fixed amount of digits
    fn print_uint<
        T: num::Integer + Copy + num::ToPrimitive + num::FromPrimitive + num::traits::Unsigned,
        const N: usize,
    >(
        &mut self,
        num: T,
    ) -> Result<(), PrinterError> {
        let mut digs: [u8; N] = [0; N];
        int_to_digits(num, &mut digs, 10);
        digs.iter()
            .copied()
            .try_for_each(|ch| self.print_digit(ch))?;
        Ok(())
    }
}

// manually implement the blanket
impl<U: TextPrinterTrait> TextPrinterTrait for &mut U {
    fn print_char(&mut self, ch: char) -> Result<(), PrinterError> {
        (**self).print_char(ch)
    }

    fn print_str(&mut self, s: &str) -> Result<(), PrinterError> {
        (**self).print_str(s)
    }

    fn print_digit(&mut self, digit: u8) -> Result<(), PrinterError> {
        (**self).print_digit(digit)
    }

    fn print_uint<
        T: num::Integer + Copy + num::ToPrimitive + num::FromPrimitive + num::traits::Unsigned,
        const N: usize,
    >(
        &mut self,
        num: T,
    ) -> Result<(), PrinterError> {
        (**self).print_uint::<T, N>(num)
    }
}

// manually implement the blanket
#[cfg(feature = "std")]
impl<U: TextPrinterTrait> TextPrinterTrait for Box<U> {
    fn print_char(&mut self, ch: char) -> Result<(), PrinterError> {
        (**self).print_char(ch)
    }

    fn print_str(&mut self, s: &str) -> Result<(), PrinterError> {
        (**self).print_str(s)
    }

    fn print_digit(&mut self, digit: u8) -> Result<(), PrinterError> {
        (**self).print_digit(digit)
    }

    fn print_uint<
        T: num::Integer + Copy + num::ToPrimitive + num::FromPrimitive + num::traits::Unsigned,
        const N: usize,
    >(
        &mut self,
        num: T,
    ) -> Result<(), PrinterError> {
        (**self).print_uint::<T, N>(num)
    }
}

/// prints a single line of text; no support for line feed
pub struct TextLinePrinter<G, F>
where
    F: Font,
    G: DrawGlyph<F::Glyph>,
    F::Glyph: GlyphMetrics,
{
    pub font: F,
    pub canvas: G,
    /// what to do when asked to print characters for which the font does not define glyphs
    /// 
    /// If set to `None`, missing glyph definitions result in an error.
    pub notdef_glyph: Option<F::Glyph>,
    pub typesetter: LineTypesetter<<F::Glyph as GlyphMetrics>::Length>,
}

impl<G, F> TextLinePrinter<G, F>
where
    F: Font,
    G: DrawGlyph<F::Glyph>,
    F::Glyph: GlyphMetrics,
{
    /// create a `TextLinePrinter` with defaults
    ///
    /// The `canvas` is anything that supports drawing glyphs on it.
    ///
    /// * The writing direction is set to the default defined by the font.
    /// * `notdef_glyph` is set to the default defined by the font.
    /// *  The initial position of the `typesetter` is chosen such that
    ///    the origin of the text box is at (0,0).
    /// You can change these values afterwards.
    pub fn new(canvas: G, font: F) -> Self {
        let writing_dir = font.default_writing_direction();
        let line_feed_axis = font.default_line_feed_direction().axis();
        let base_line_offset = font.get_font_spec(line_feed_axis).unwrap().base_line_offset;
        let start_xy = line_feed_axis.as_vector(base_line_offset);
        let notdef_glyph = font.default_notdef_glyph();
        let typesetter = LineTypesetter::new(start_xy, writing_dir);
        Self {
            font,
            canvas,
            notdef_glyph,
            typesetter,
        }
    }
    /// allows you to print a glyph
    pub fn print_glyph(&mut self, glyph: &F::Glyph) {
        Self::print_glyph_helper(&mut self.typesetter, &mut self.canvas, glyph)
    }
    /// advances the writing position along the base line by a given amount
    pub fn skip(&mut self, width: <F::Glyph as GlyphMetrics>::Length) {
        self.typesetter.skip(width);
    }
    /// This associated function exists to circumvent borrow checker issues.
    fn print_glyph_helper(
        typesetter: &mut LineTypesetter<<F::Glyph as GlyphMetrics>::Length>,
        canvas: &mut G,
        glyph: &F::Glyph,
    ) {
        let pos_xy = typesetter.typeset_glyph(glyph);
        canvas.draw_glyph(glyph, pos_xy)
    }
    /// helper function, written as associated function to circumvent borrow checker issues
    fn handle_non_printable(
        typesetter: &mut LineTypesetter<<F::Glyph as GlyphMetrics>::Length>,
        canvas: &mut G,
        notdef_glyph: Option<&F::Glyph>,
        ch: char,
    ) -> Result<(), PrinterError> {
        if ch.is_control() {
            return Err(PrinterError::UnsupportedControlChar(ch));
        }
        let Some(glyph) = notdef_glyph else {
            return Err(PrinterError::GlyphNotDefined(ch));
        };
        Self::print_glyph_helper(typesetter, canvas, glyph);
        Ok(())
    }
}

impl<G, F> TextPrinterTrait for TextLinePrinter<G, F>
where
    F: Font,
    G: DrawGlyph<F::Glyph>,
    F::Glyph: GlyphMetrics,
{
    fn print_char(&mut self, ch: char) -> Result<(), PrinterError> {
        let maybe_glyph = self.font.char_to_glyph(ch);
        match maybe_glyph {
            Ok(glyph) => self.print_glyph(&glyph),
            Err(ch) => Self::handle_non_printable(
                &mut self.typesetter,
                &mut self.canvas,
                self.notdef_glyph.as_ref(),
                ch,
            )?,
        }
        Ok(())
    }
    fn print_str(&mut self, s: &str) -> Result<(), PrinterError> {
        let glyph_iter = self.font.str_to_glyphs(s);
        // We cannot borrow self mutably and non-mutably at the same time,
        // but we *can* borrow individual fields in various ways simultaneously.
        // So this is the way out to avoid conflicts with the borrow checker.
        let typesetter = &mut self.typesetter;
        let canvas = &mut self.canvas;
        let notdef_glyph = self.notdef_glyph.as_ref();
        for maybe_glyph in glyph_iter {
            match maybe_glyph {
                Ok(glyph) => Self::print_glyph_helper(typesetter, canvas, &glyph),
                Err(ch) => Self::handle_non_printable(typesetter, canvas, notdef_glyph, ch)?,
            }
        }
        Ok(())
    }
}

fn int_to_digits<T: num::Integer + Copy + ToPrimitive + FromPrimitive>(
    mut x: T,
    digits: &mut [u8],
    base: u8,
) {
    use num::NumCast;
    for dig in digits.iter_mut().rev() {
        let rem;
        (x, rem) = x.div_rem(&T::from_u8(base).unwrap());
        *dig = <u8 as NumCast>::from(rem).unwrap();
    }
}
