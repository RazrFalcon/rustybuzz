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

macro_rules! matches {
    ($expression:expr, $($pattern:tt)+) => {
        match $expression {
            $($pattern)+ => true,
            _ => false
        }
    }
}

mod buffer;
mod common;
mod ffi;
mod font;
mod unicode;
mod tag;
mod tag_table;
mod text_parser;

pub use crate::buffer::*;
pub use crate::common::*;
pub use crate::font::{Face, Font, Variation};

#[doc(hidden)]
pub mod implementation {
    // We must export extern symbols so the linker would be able to find them.
    pub use crate::unicode::*;
    pub use crate::font::{rb_ot_get_nominal_glyph, rb_ot_get_variation_glyph};
    pub use crate::tag::rb_ot_tags_from_script_and_language;
}

type CodePoint = u32;

use text_parser::TextParser;


/// A feature tag with an accompanying range specifying on which subslice of
/// `shape`s input it should be applied.
///
/// You can pass a slice of `Feature`s to `shape` that will be activated for the
/// corresponding slices of input.
#[derive(Clone, Copy, PartialEq)]
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

impl std::fmt::Debug for Feature {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Feature")
            .field("tag", &Tag(self.0.tag))
            .field("value", &self.0.value)
            .field("start", &self.0.start)
            .field("end", &self.0.end)
            .finish()
    }
}

impl std::str::FromStr for Feature {
    type Err = &'static str;

    /// Parses a `Feature` form a string.
    ///
    /// Possible values:
    ///
    /// - `kern` -> kern .. 1
    /// - `+kern` -> kern .. 1
    /// - `-kern` -> kern .. 0
    /// - `kern=0` -> kern .. 0
    /// - `kern=1` -> kern .. 1
    /// - `aalt=2` -> altr .. 2
    /// - `kern[]` -> kern .. 1
    /// - `kern[:]` -> kern .. 1
    /// - `kern[5:]` -> kern 5.. 1
    /// - `kern[:5]` -> kern ..=5 1
    /// - `kern[3:5]` -> kern 3..=5 1
    /// - `kern[3]` -> kern 3..=4 1
    /// - `aalt[3:5]=2` -> kern 3..=5 1
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use std::convert::TryInto;

        fn parse(s: &str) -> Option<Feature> {
            if s.is_empty() {
                return None;
            }

            let mut p = TextParser::new(s);

            // Parse prefix.
            let mut value = 1;
            match p.curr_byte()? {
                b'-' => { value = 0; p.advance(1); }
                b'+' => { value = 1; p.advance(1); }
                _ => {}
            }

            // Parse tag.
            p.skip_spaces();
            let quote = p.consume_quote();

            let tag = p.consume_bytes(|c| c.is_ascii_alphanumeric() || c == b'_');
            if tag.len() != 4 {
                return None;
            }
            let tag = Tag::from_bytes(tag.as_bytes().try_into().unwrap());

            // Force closing quote.
            if let Some(quote) = quote {
                p.consume_byte(quote)?;
            }

            // Parse indices.
            p.skip_spaces();

            let (start, end) = if p.consume_byte(b'[').is_some() {
                let start_opt = p.consume_i32();
                let start = start_opt.unwrap_or(0) as u32; // negative value overflow is ok

                let end = if matches!(p.curr_byte(), Some(b':') | Some(b';')) {
                    p.advance(1);
                    p.consume_i32().unwrap_or(-1) as u32 // negative value overflow is ok
                } else {
                    if start_opt.is_some() && start != std::u32::MAX {
                        start + 1
                    } else {
                        std::u32::MAX
                    }
                };

                p.consume_byte(b']')?;

                (start, end)
            } else {
                (0, std::u32::MAX)
            };

            // Parse postfix.
            let had_equal = p.consume_byte(b'=').is_some();
            let value1 = p.consume_i32().or(p.consume_bool().map(|b| b as i32));

            if had_equal && value1.is_none() {
                return None;
            };

            if let Some(value1) = value1 {
                value = value1 as u32; // negative value overflow is ok
            }

            p.skip_spaces();

            if !p.at_end() {
                return None;
            }

            Some(Feature(ffi::hb_feature_t {
                tag: tag.0,
                value,
                start,
                end,
            }))
        }

        parse(s).ok_or("invalid feature")
    }
}

#[cfg(test)]
mod tests_features {
    use super::*;
    use std::str::FromStr;

    macro_rules! test {
        ($name:ident, $text:expr, $tag:expr, $value:expr, $range:expr) => (
            #[test]
            fn $name() {
                assert_eq!(
                    Feature::from_str($text).unwrap(),
                    Feature::new(Tag::from_bytes($tag), $value, $range)
                );
            }
        )
    }

    test!(parse_01, "kern",         b"kern", 1, ..);
    test!(parse_02, "+kern",        b"kern", 1, ..);
    test!(parse_03, "-kern",        b"kern", 0, ..);
    test!(parse_04, "kern=0",       b"kern", 0, ..);
    test!(parse_05, "kern=1",       b"kern", 1, ..);
    test!(parse_06, "kern=2",       b"kern", 2, ..);
    test!(parse_07, "kern[]",       b"kern", 1, ..);
    test!(parse_08, "kern[:]",      b"kern", 1, ..);
    test!(parse_09, "kern[5:]",     b"kern", 1, 5..);
    test!(parse_10, "kern[:5]",     b"kern", 1, ..=5);
    test!(parse_11, "kern[3:5]",    b"kern", 1, 3..=5);
    test!(parse_12, "kern[3]",      b"kern", 1, 3..=4);
    test!(parse_13, "kern[3:5]=2",  b"kern", 2, 3..=5);
    test!(parse_14, "kern[3;5]=2",  b"kern", 2, 3..=5);
    test!(parse_15, "kern[:-1]",    b"kern", 1, ..);
    test!(parse_16, "kern[-1]",     b"kern", 1, std::u32::MAX as usize..);
    test!(parse_17, "kern=on",      b"kern", 1, ..);
    test!(parse_18, "kern=off",     b"kern", 0, ..);
    test!(parse_19, "kern=oN",      b"kern", 1, ..);
    test!(parse_20, "kern=oFf",     b"kern", 0, ..);
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
