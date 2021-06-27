/*!
A complete [harfbuzz](https://github.com/harfbuzz/harfbuzz) shaping algorithm port to Rust.
*/

#![no_std]
#![doc(html_root_url = "https://docs.rs/rustybuzz/0.4.0")]
#![warn(missing_docs)]

extern crate alloc;

#[macro_use]
mod buffer;
mod aat;
mod common;
mod fallback;
mod glyph_set;
mod normalize;
mod shape;
mod plan;
mod face;
mod tables;
mod tag;
mod tag_table;
mod text_parser;
mod unicode;
mod unicode_norm;
mod complex;
mod ot;

pub use ttf_parser::Tag;

pub use crate::buffer::{
    GlyphPosition, GlyphInfo, BufferClusterLevel,
    SerializeFlags, UnicodeBuffer, GlyphBuffer
};
pub use crate::common::{Direction, Script, Language, Feature, Variation, script};
pub use crate::face::Face;
pub use crate::shape::shape;

type Mask = u32;
