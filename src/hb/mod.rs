// Match harfbuzz code style.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

mod algs;
#[macro_use]
pub mod buffer;
mod aat_layout;
mod aat_layout_kerx_table;
mod aat_layout_morx_table;
mod aat_layout_trak_table;
mod aat_map;
pub mod common;
pub mod face;
mod glyph_set;
mod kerning;
mod machine_cursor;
mod ot_layout;
mod ot_layout_common;
mod ot_layout_gpos_table;
mod ot_layout_gsub_table;
mod ot_layout_gsubgpos;
mod ot_map;
mod ot_shape;
mod ot_shape_complex;
mod ot_shape_complex_arabic;
mod ot_shape_complex_arabic_table;
mod ot_shape_complex_hangul;
mod ot_shape_complex_hebrew;
mod ot_shape_complex_indic;
mod ot_shape_complex_indic_machine;
mod ot_shape_complex_indic_table;
mod ot_shape_complex_khmer;
mod ot_shape_complex_khmer_machine;
mod ot_shape_complex_myanmar;
mod ot_shape_complex_myanmar_machine;
mod ot_shape_complex_syllabic;
mod ot_shape_complex_thai;
mod ot_shape_complex_use;
mod ot_shape_complex_use_machine;
mod ot_shape_complex_use_table;
mod ot_shape_complex_vowel_constraints;
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
