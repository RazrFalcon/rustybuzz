/*!
A complete [harfbuzz](https://github.com/harfbuzz/harfbuzz) shaping algorithm port to Rust.
*/

#![no_std]
#![warn(missing_docs)]
// Match harfbuzz code style.
#![allow(non_camel_case_types)]

#[cfg(not(any(feature = "std", feature = "libm")))]
compile_error!("You have to activate either the `std` or the `libm` feature.");

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

#[macro_use]
mod buffer;
mod aat;
mod common;
mod complex;
mod face;
mod glyph_set;
mod ot;
mod ot_layout;
mod ot_layout_gpos_table;
mod ot_map;
mod ot_shape_fallback;
mod ot_shape_normalize;
mod shape;
mod shape_plan;
mod tag;
mod tag_table;
mod text_parser;
mod unicode;
mod unicode_norm;
mod ot_layout_gsub_table;
mod ot_shape_complex_arabic;
mod ot_shape_complex_arabic_table;

pub use ttf_parser;

use ttf_parser::Tag as hb_tag_t;

pub use crate::buffer::hb_glyph_info_t as GlyphInfo;
pub use crate::buffer::{
    BufferClusterLevel, BufferFlags, GlyphBuffer, GlyphPosition, SerializeFlags, UnicodeBuffer,
};
pub use crate::common::{script, Direction, Feature, Language, Script, Variation};
pub use crate::face::hb_font_t as Face;
pub use crate::shape::{shape, shape_with_plan};
pub use crate::shape_plan::hb_ot_shape_plan_t as ShapePlan;

use crate::buffer::hb_glyph_info_t;
use crate::face::hb_font_t;

type hb_mask_t = u32;

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
