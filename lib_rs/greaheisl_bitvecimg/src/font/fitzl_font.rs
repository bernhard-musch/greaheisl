use super::typeset::{
    directions::Axis2D, directions::RectDirection, Font, FontInfo, FontMetrics,
    SimpleFontGlyphIterator,
};
use super::BitVecImgGlyph;
use crate::{BitVecImgView, ImageRegion};

/// A very small font, defined only for capital letters and digits 0..9.
/// The digits are extremely narrow.
/// 
/// Implements the [`Font`] trait.
pub struct FitzlFontNarrowNum {}

impl FontInfo for FitzlFontNarrowNum {
    type Glyph = BitVecImgGlyph<ImageRegion<'static>>;
    fn get_font_spec(&self, line_feed_axis: Axis2D) -> Option<FontMetrics<i32>> {
        match line_feed_axis {
            Axis2D::Y => Some(FontMetrics {
                base_line_offset: 4,
                line_to_line_distance: 6,
            }),
            _ => None,
        }
    }
    fn default_line_feed_direction(&self) -> RectDirection {
        RectDirection::PlusY
    }
    fn default_writing_direction(&self) -> RectDirection {
        RectDirection::PlusX
    }
}

impl Font for FitzlFontNarrowNum {
    fn char_to_glyph(&self, ch: char) -> Result<Self::Glyph, char> {
        match ch {
            x if x.is_ascii_digit() => Ok(BitVecImgGlyph {
                image: images::DIGITS[(x as usize) - 0x30].as_region(),
                base_point: [1, 4],
                margin: [1, 1, 1, 1],
            }),
            x if x.is_ascii_uppercase() => Ok(BitVecImgGlyph {
                image: images::LETTERS[(x as usize) - 0x41].as_region(),
                base_point: [1, 4],
                margin: [1, 1, 1, 1],
            }),
            ' ' => Ok(BitVecImgGlyph {
                image: images::SPACE.as_region(),
                base_point: [4, 1],
                margin: [1, 1, 1, 1],
            }),
            _ => Err(ch),
        }
    }

    type GlyphIterator<'a> = SimpleFontGlyphIterator<'a,Self>
        where Self: 'a;

    fn str_to_glyphs<'a, 'b: 'a>(&'b self, text: &'a str) -> Self::GlyphIterator<'a>
    where
        Self: 'a,
    {
        SimpleFontGlyphIterator {
            font: self,
            text: text.chars(),
        }
    }

    fn default_notdef_glyph(&self) -> Option<Self::Glyph> {
        Some(BitVecImgGlyph {
            image: images::NOTDEF_GLYPH.as_region(),
            base_point: [4, 1],
            margin: [1, 1, 1, 1],
        })
    }
}

#[rustfmt::skip]
pub mod images {
    //! the bare images used in the font

    use crate::Image;
    use bitvec::prelude::*;


    pub const DIGITS: [Image<2, 5, 1>; 10] = [
        Image(bitarr![const u32,Msb0;
            1, 1, 
            1, 1,
            1, 1,
            1, 1,
            1, 1 ]),
        Image(bitarr![const u32,Msb0;
            0, 1, 
            1, 1,
            0, 1,
            0, 1,
            0, 1 ]),
        Image(bitarr![const u32,Msb0; 
            1, 1, 
            0, 1,
            0, 1,
            1, 0,
            1, 1 ]),
        Image(bitarr![const u32,Msb0;
            1, 1, 
            0, 1,
            1, 1,
            0, 1,
            1, 1 ]),
        Image(bitarr![const u32,Msb0;
            1, 0, 
            1, 0,
            1, 1,
            0, 1,
            0, 1 ]),
        Image(bitarr![const u32,Msb0; 
            1, 1, 
            1, 0,
            1, 1,
            0, 1,
            1, 1 ]),
        Image(bitarr![const u32,Msb0;
            1, 1, 
            1, 0,
            1, 1,
            1, 1,
            1, 1 ]),
        Image(bitarr![const u32,Msb0;
            1, 1, 
            0, 1,
            0, 1,
            0, 1,
            0, 1 ]),
        Image(bitarr![const u32,Msb0;  
            1, 1, 
            1, 1,
            1, 0,
            1, 1,
            1, 1 ]),
        Image(bitarr![const u32,Msb0;
            1, 1, 
            1, 1,
            0, 1,
            0, 1,
            1, 1 ]),
    ];

    pub const LETTERS: [Image<3, 5, 1>; 26] = [
        Image(bitarr![const u32,Msb0;
            0, 1, 0,
            1, 0, 1,
            1, 1, 1,
            1, 0, 1,
            1, 0, 1, ]),
        Image(bitarr![const u32,Msb0;
            1, 1, 0,
            1, 0, 1,
            1, 1, 0,
            1, 0, 1,
            1, 1, 0, ]),
        Image(bitarr![const u32,Msb0;
            0, 1, 1,
            1, 0, 0,
            1, 0, 0,
            1, 0, 0,
            0, 1, 1, ]),
        Image(bitarr![const u32,Msb0;
            1, 1, 0,
            1, 0, 1,
            1, 0, 1,
            1, 0, 1,
            1, 1, 0, ]),
        Image(bitarr![const u32,Msb0;
            1, 1, 1,
            1, 0, 0,
            1, 1, 0,
            1, 0, 0,
            1, 1, 1, ]),
        Image(bitarr![const u32,Msb0;
            1, 1, 1,
            1, 0, 0,
            1, 1, 0,
            1, 0, 0,
            1, 0, 0, ]),
        Image(bitarr![const u32,Msb0;
            0, 1, 1,
            1, 0, 0,
            1, 0, 1,
            1, 0, 1,
            0, 1, 1, ]),
        Image(bitarr![const u32,Msb0;
            1, 0, 1,
            1, 0, 1,
            1, 1, 1,
            1, 0, 1,
            1, 0, 1, ]),
        Image(bitarr![const u32,Msb0;
            1, 1, 1,
            0, 1, 0,
            0, 1, 0,
            0, 1, 0,
            1, 1, 1, ]),
        Image(bitarr![const u32,Msb0;
            1, 1, 1,
            0, 0, 1,
            0, 0, 1,
            1, 0, 1,
            0, 1, 0, ]),
        Image(bitarr![const u32,Msb0;
            1, 0, 1,
            1, 0, 1,
            1, 1, 0,
            1, 0, 1,
            1, 0, 1, ]),
        Image(bitarr![const u32,Msb0;
            1, 0, 0,
            1, 0, 0,
            1, 0, 0,
            1, 0, 0,
            1, 1, 1, ]),
        Image(bitarr![const u32,Msb0;
            1, 0, 1,
            1, 1, 1,
            1, 0, 1,
            1, 0, 1,
            1, 0, 1, ]),
        Image(bitarr![const u32,Msb0;
            1, 0, 1,
            1, 0, 1,
            1, 1, 1,
            1, 1, 1,
            1, 0, 1, ]),
        Image(bitarr![const u32,Msb0;
            0, 1, 0,
            1, 0, 1,
            1, 0, 1,
            1, 0, 1,
            0, 1, 0, ]),
        Image(bitarr![const u32,Msb0;
            1, 1, 0,
            1, 0, 1,
            1, 1, 0,
            1, 0, 0,
            1, 0, 0, ]),
        Image(bitarr![const u32,Msb0;
            0, 1, 0,
            1, 0, 1,
            1, 0, 1,
            1, 0, 1,
            0, 1, 1, ]),
        Image(bitarr![const u32,Msb0;
            1, 1, 0,
            1, 0, 1,
            1, 1, 0,
            1, 0, 1,
            1, 0, 1, ]),
        Image(bitarr![const u32,Msb0;
            0, 1, 1,
            1, 0, 0,
            0, 1, 0,
            0, 0, 1,
            1, 1, 0, ]),
        Image(bitarr![const u32,Msb0;
            1, 1, 1,
            0, 1, 0,
            0, 1, 0,
            0, 1, 0,
            0, 1, 0, ]),
        Image(bitarr![const u32,Msb0;
            1, 0, 1,
            1, 0, 1,
            1, 0, 1,
            1, 0, 1,
            1, 1, 1, ]),
        Image(bitarr![const u32,Msb0;
            1, 0, 1,
            1, 0, 1,
            1, 0, 1,
            0, 1, 0,
            0, 1, 0, ]),
        Image(bitarr![const u32,Msb0;
            1, 0, 1,
            1, 0, 1,
            1, 0, 1,
            1, 1, 1,
            1, 0, 1, ]),
        Image(bitarr![const u32,Msb0;
            1, 0, 1,
            1, 0, 1,
            0, 1, 0,
            1, 0, 1,
            1, 0, 1, ]),
        Image(bitarr![const u32,Msb0;
            1, 0, 1,
            1, 0, 1,
            0, 1, 0,
            0, 1, 0,
            0, 1, 0, ]),
        Image(bitarr![const u32,Msb0;
            1, 1, 1,
            0, 0, 1,
            0, 1, 0,
            1, 0, 0,
            1, 1, 1, ]),
    ];

    pub const SPACE: Image<1, 5, 1> = 
        Image(bitarr![const u32,Msb0;
            0,
            0,
            0,
            0,
            0, ]);

    pub const NOTDEF_GLYPH: Image<3, 5, 1> = 
        Image(bitarr![const u32,Msb0;
            1, 1, 1,
            1, 1, 1,
            0, 1, 0,
            1, 1, 1,
            0, 1, 0, ]);


    

    #[allow(dead_code)]
    const TEMPLATE: Image<12, 8, 3> = 
    Image(bitarr![const u32,Msb0;
        0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,
    ]);

}
