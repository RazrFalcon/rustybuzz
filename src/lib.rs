/*!
`rustybuzz` is an attempt to incrementally port [harfbuzz](https://github.com/harfbuzz/harfbuzz) to Rust.
*/

#![doc(html_root_url = "https://docs.rs/rustybuzz/0.2.0")]
#![warn(missing_docs)]

#[macro_use]
mod buffer;
mod aat;
mod common;
mod ffi;
mod fallback;
mod feature;
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
