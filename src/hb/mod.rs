#[macro_use]
pub mod buffer;
mod aat;
pub mod common;
mod complex;
pub mod face;
mod glyph_set;
mod ot;
mod ot_layout;
mod ot_layout_gpos_table;
mod ot_layout_gsub_table;
mod ot_map;
mod ot_shape_complex_arabic;
mod ot_shape_complex_arabic_table;
mod ot_shape_fallback;
mod ot_shape_normalize;
pub mod shape;
pub mod shape_plan;
mod tag;
mod tag_table;
mod text_parser;
mod unicode;
mod unicode_norm;

use ttf_parser::Tag as hb_tag_t;

use self::buffer::hb_glyph_info_t;
use self::face::hb_font_t;

type hb_mask_t = u32;

use self::buffer::{GlyphBuffer, UnicodeBuffer};
use self::common::{script, Direction, Feature, Language, Script};

fn round(x: f32) -> f32 {
    #[cfg(feature = "std")]
    {
        x.round()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::roundf(x)
    }
}
