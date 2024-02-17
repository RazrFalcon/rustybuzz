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
mod fallback;
mod glyph_set;
mod ot;
mod ot_shape_normalize;
mod plan;
mod shape;
mod tag;
mod tag_table;
mod text_parser;
mod unicode;
mod unicode_norm;

pub use ttf_parser;

pub use ttf_parser::Tag;

pub use crate::buffer::hb_glyph_info_t as GlyphInfo;
pub use crate::buffer::{
    BufferClusterLevel, BufferFlags, GlyphBuffer, GlyphPosition, SerializeFlags, UnicodeBuffer,
};
pub use crate::common::{script, Direction, Feature, Language, Script, Variation};
pub use crate::face::hb_font_t as Face;
pub use crate::plan::hb_ot_shape_plan_t as ShapePlan;
pub use crate::shape::{shape, shape_with_plan};

use crate::buffer::hb_glyph_info_t;
use crate::face::hb_font_t;

type Mask = u32;

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
