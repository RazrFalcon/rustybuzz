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

mod hb;

pub use ttf_parser;

pub use hb::buffer::hb_glyph_info_t as GlyphInfo;
pub use hb::buffer::{
    BufferClusterLevel, BufferFlags, GlyphBuffer, GlyphPosition, SerializeFlags, UnicodeBuffer,
};
pub use hb::common::{script, Direction, Feature, Language, Script, Variation};
pub use hb::face::hb_font_t as Face;
pub use hb::shape::{shape, shape_with_plan};
pub use hb::shape_plan::hb_ot_shape_plan_t as ShapePlan;
