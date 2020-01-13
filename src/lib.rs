/*!
`rustybuzz` is an attempt to incrementally port [harfbuzz](https://github.com/harfbuzz/harfbuzz) to Rust.

You can use it already, since we simply linking `hardbuzz` statically.
And we're testing `rustybuzz` against `harfbuzz` test suite.

Embedded `harfbuzz` version: 2.6.4
*/

#![doc(html_root_url = "https://docs.rs/rustybuzz/0.1.0")]

#![warn(missing_docs)]

use std::ops::{Bound, RangeBounds};
use std::os::raw::c_uint;

mod buffer;
mod common;
mod ffi;
mod font;
mod unicode;

pub use crate::buffer::*;
pub use crate::common::*;
pub use crate::font::{Face, Font, Variation};

#[doc(hidden)]
pub mod implementation {
    // We must export extern symbols so the linker would be able to find them.
    pub use crate::unicode::*;
    pub use crate::font::{rb_ot_get_nominal_glyph, rb_ot_get_variation_glyph};
}

type CodePoint = u32;


/// A feature tag with an accompanying range specifying on which subslice of
/// `shape`s input it should be applied.
///
/// You can pass a slice of `Feature`s to `shape` that will be activated for the
/// corresponding slices of input.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Feature(ffi::hb_feature_t);

impl Feature {
    /// Create a new `Feature` struct.
    pub fn new(tag: Tag, value: u32, range: impl RangeBounds<usize>) -> Feature {
        // We have to do careful bounds checking since c_uint may be of
        // different sizes on different platforms. We do assume that
        // sizeof(usize) >= sizeof(c_uint).
        const MAX_UINT: usize = c_uint::max_value() as usize;
        let start = match range.start_bound() {
            Bound::Included(&included) => included.min(MAX_UINT) as c_uint,
            Bound::Excluded(&excluded) => excluded.min(MAX_UINT - 1) as c_uint + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&included) => included.min(MAX_UINT) as c_uint,
            Bound::Excluded(&excluded) => excluded.saturating_sub(1).min(MAX_UINT) as c_uint,
            Bound::Unbounded => c_uint::max_value(),
        };

        Feature(ffi::hb_feature_t {
            tag: tag.0,
            value,
            start,
            end,
        })
    }

    /// Returns the feature tag.
    pub fn tag(&self) -> Tag {
        Tag(self.0.tag)
    }

    /// Returns the feature value.
    pub fn value(&self) -> u32 {
        self.0.value
    }

    /// Returns the feature start index.
    pub fn start(&self) -> usize {
        self.0.start as usize
    }

    /// Returns the feature end index.
    pub fn end(&self) -> usize {
        self.0.end as usize
    }
}

impl std::str::FromStr for Feature {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unsafe {
            let mut f: ffi::hb_feature_t = std::mem::MaybeUninit::zeroed().assume_init();
            let ok = ffi::hb_feature_from_string(s.as_ptr() as *const _, s.len() as i32, &mut f as *mut _);
            if ok == 1 {
                Ok(Feature(f))
            } else {
                Err("invalid feature")
            }
        }
    }
}

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

/// Returns a list of available shapers.
pub fn list_shapers() -> &'static [&'static str] {
    &["ot", "fallback"]
}
