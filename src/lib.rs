/*!
`rustybuzz` is an attempt to incrementally port [harfbuzz](https://github.com/harfbuzz/harfbuzz) to Rust.

You can use it already, since we simply linking `hardbuzz` statically.
And we're testing `rustybuzz` against `harfbuzz` test suite.

Embedded `harfbuzz` version: 2.6.4
*/

#![doc(html_root_url = "https://docs.rs/rustybuzz/0.1.0")]

#![warn(missing_docs)]

macro_rules! matches {
    ($expression:expr, $($pattern:tt)+) => {
        match $expression {
            $($pattern)+ => true,
            _ => false
        }
    }
}

macro_rules! try_opt {
    ($task:expr) => {
        match $task {
            Some(v) => v,
            None => return,
        }
    };
}

macro_rules! try_opt_or {
    ($task:expr, $ret:expr) => {
        match $task {
            Some(v) => v,
            None => $ret,
        }
    };
}

macro_rules! try_or {
    ($task:expr, $ret:expr) => {
        match $task {
            Ok(v) => v,
            Err(_) => $ret,
        }
    };
}

mod buffer;
mod common;
mod ffi;
mod font;
mod unicode;
mod tag;
mod tag_table;
mod text_parser;

pub use ttf_parser::Tag;

pub use crate::buffer::*;
pub use crate::common::*;
pub use crate::font::{Face, Font};

type CodePoint = u32;

/// Shape the contents of the buffer using the provided font and activating all
/// OpenType features given in `features`.
///
/// This function consumes the `buffer` and returns a `GlyphBuffer` containing
/// the resulting glyph indices and the corresponding positioning information.
/// Once all the information from the `GlyphBuffer` has been processed as
/// necessary you can reuse the `GlyphBuffer` as an `Buffer` (using
/// `GlyphBuffer::clear`) and use that to call `shape` again with new
/// data.
pub fn shape(font: &Font<'_>, mut buffer: Buffer, features: &[Feature]) -> GlyphBuffer {
    buffer.guess_segment_properties();
    unsafe {
        ffi::hb_shape(
            font.as_ptr(),
            buffer.as_ptr(),
            features.as_ptr() as *mut _,
            features.len() as u32,
        )
    };
    GlyphBuffer(buffer)
}
