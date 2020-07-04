/*!
`rustybuzz` is an attempt to incrementally port [harfbuzz](https://github.com/harfbuzz/harfbuzz) to Rust.
*/

// The current API is a heavily modified version of https://github.com/manuel-rhdt/harfbuzz_rs

#![doc(html_root_url = "https://docs.rs/rustybuzz/0.1.1")]
#![warn(missing_docs)]

mod buffer;
mod common;
mod ffi;
mod font;

pub use crate::buffer::*;
pub use crate::common::*;
pub use crate::font::*;


/// Shapes the buffer content using provided font and features.
///
/// Consumes the buffer. You can then run `GlyphBuffer::clear` to get the `UnicodeBuffer` back
/// without allocating a new one.
pub fn shape(font: &Font<'_>, features: &[Feature], mut buffer: UnicodeBuffer) -> GlyphBuffer {
    buffer.guess_segment_properties();
    unsafe {
        ffi::hb_shape_full(
            font.as_ptr(),
            buffer.0.as_ptr(),
            features.as_ptr() as *mut _,
            features.len() as u32,
            std::ptr::null(),
        )
    };

    GlyphBuffer(buffer.0)
}
