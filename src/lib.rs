/*!
A complete [harfbuzz](https://github.com/harfbuzz/harfbuzz) shaping algorithm port to Rust.
*/

#![no_std]
#![warn(missing_docs)]

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
mod normalize;
mod ot;
mod plan;
mod shape;
mod tag;
mod tag_table;
mod text_parser;
mod unicode;
mod unicode_norm;

pub use ttf_parser;

pub use ttf_parser::Tag;

pub use crate::buffer::{
    BufferClusterLevel, GlyphBuffer, GlyphInfo, GlyphPosition, SerializeFlags, UnicodeBuffer,
};
pub use crate::common::{script, Direction, Feature, Language, Script, Variation};
pub use crate::face::Face;
pub use crate::shape::shape;

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
