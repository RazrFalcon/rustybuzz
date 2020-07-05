use std::ops::{Bound, RangeBounds};

use crate::ffi;
use crate::text_parser::TextParser;

/// A 4-byte tag.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag(pub u32);

impl Tag {
    /// Creates a `Tag` from bytes.
    #[inline]
    pub const fn from_bytes(bytes: &[u8; 4]) -> Self {
        Tag(((bytes[0] as u32) << 24) | ((bytes[1] as u32) << 16) |
            ((bytes[2] as u32) << 8) | (bytes[3] as u32))
    }

    /// Creates a `Tag` from bytes.
    ///
    /// In case of empty data will return `Tag` set to 0.
    ///
    /// When `bytes` are shorter than 4, will set missing bytes to ` `.
    ///
    /// Data after first 4 bytes is ignored.
    #[inline]
    pub fn from_bytes_lossy(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return Tag::from_bytes(&[0, 0, 0, 0]);
        }

        let mut iter = bytes.iter().cloned().chain(core::iter::repeat(b' '));
        Tag::from_bytes(&[
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
        ])
    }

    /// Returns tag as 4-element byte array.
    #[inline]
    pub const fn to_bytes(self) -> [u8; 4] {
        [
            (self.0 >> 24 & 0xff) as u8,
            (self.0 >> 16 & 0xff) as u8,
            (self.0 >> 8 & 0xff) as u8,
            (self.0 >> 0 & 0xff) as u8,
        ]
    }

    /// Returns tag as 4-element byte array.
    #[inline]
    pub const fn to_chars(self) -> [char; 4] {
        [
            (self.0 >> 24 & 0xff) as u8 as char,
            (self.0 >> 16 & 0xff) as u8 as char,
            (self.0 >> 8 & 0xff) as u8 as char,
            (self.0 >> 0 & 0xff) as u8 as char,
        ]
    }

    /// Checks if tag is null / `[0, 0, 0, 0]`.
    #[inline]
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    /// Returns tag value as `u32` number.
    #[inline]
    pub const fn as_u32(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Debug for Tag {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Tag({})", self)
    }
}

impl std::fmt::Display for Tag {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let b = self.to_chars();
        write!(
            f,
            "{}{}{}{}",
            b.get(0).unwrap_or(&' '),
            b.get(1).unwrap_or(&' '),
            b.get(2).unwrap_or(&' '),
            b.get(3).unwrap_or(&' ')
        )
    }
}

impl std::str::FromStr for Tag {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Tag::from_bytes_lossy(s.as_bytes()))
    }
}


/// Defines the direction in which text is to be read.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Direction {
    /// Initial, unset direction.
    Invalid,
    /// Text is set horizontally from left to right.
    LeftToRight,
    /// Text is set horizontally from right to left.
    RightToLeft,
    /// Text is set vertically from top to bottom.
    TopToBottom,
    /// Text is set vertically from bottom to top.
    BottomToTop,
}

impl Direction {
    pub(crate) fn to_raw(self) -> ffi::hb_direction_t {
        match self {
            Direction::Invalid => ffi::HB_DIRECTION_INVALID,
            Direction::LeftToRight => ffi::HB_DIRECTION_LTR,
            Direction::RightToLeft => ffi::HB_DIRECTION_RTL,
            Direction::TopToBottom => ffi::HB_DIRECTION_TTB,
            Direction::BottomToTop => ffi::HB_DIRECTION_BTT,
        }
    }

    pub(crate) fn from_raw(dir: ffi::hb_direction_t) -> Self {
        match dir {
            ffi::HB_DIRECTION_LTR => Direction::LeftToRight,
            ffi::HB_DIRECTION_RTL => Direction::RightToLeft,
            ffi::HB_DIRECTION_TTB => Direction::TopToBottom,
            ffi::HB_DIRECTION_BTT => Direction::BottomToTop,
            _ => Direction::Invalid,
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Invalid
    }
}

impl std::str::FromStr for Direction {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("invalid direction");
        }

        // harfbuzz also matches only the first letter.
        match s.as_bytes()[0].to_ascii_lowercase() {
            b'l' => Ok(Direction::LeftToRight),
            b'r' => Ok(Direction::RightToLeft),
            b't' => Ok(Direction::TopToBottom),
            b'b' => Ok(Direction::BottomToTop),
            _ => Err("invalid direction"),
        }
    }
}


/// A script language.
pub struct Language(pub ffi::hb_language_t);

impl Default for Language {
    fn default() -> Language {
        Language(unsafe { ffi::hb_language_get_default() })
    }
}

impl std::fmt::Debug for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Language(\"{}\")", self)
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = unsafe {
            let char_ptr = ffi::hb_language_to_string(self.0);
            if char_ptr.is_null() {
                return Err(std::fmt::Error);
            }
            std::ffi::CStr::from_ptr(char_ptr)
                .to_str()
                .expect("String representation of language is not valid utf8.")
        };
        write!(f, "{}", string)
    }
}

impl std::str::FromStr for Language {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let len = std::cmp::min(s.len(), std::i32::MAX as _) as i32;
        let lang = unsafe { ffi::hb_language_from_string(s.as_ptr() as *mut _, len) };
        if lang.is_null() {
            Err("invalid language")
        } else {
            Ok(Language(lang))
        }
    }
}


/// A Unicode Script.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Script(pub ffi::hb_script_t);

impl Script {
    /// Creates a script from a ISO 15924 tag.
    pub fn from_iso15924_tag(tag: Tag) -> Self {
        Script(unsafe { ffi::hb_script_from_iso15924_tag(tag.0) })
    }

    /// Converts a script into a ISO 15924 tag.
    pub fn to_iso15924_tag(self) -> Tag {
        Tag(unsafe { ffi::hb_script_to_iso15924_tag(self.0) })
    }

    /// Returns a horizontal direction of a script.
    pub fn horizontal_direction(self) -> Direction {
        Direction::from_raw(unsafe { ffi::hb_script_get_horizontal_direction(self.0) })
    }
}

impl std::str::FromStr for Script {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tag = Tag::from_bytes_lossy(s.as_bytes());
        Ok(Script::from_iso15924_tag(tag))
    }
}


/// A feature tag with an accompanying range specifying on which subslice of
/// `shape`s input it should be applied.
#[repr(C)]
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Feature {
    pub tag: Tag,
    pub value: u32,
    pub start: u32,
    pub end: u32,
}

impl Feature {
    /// Create a new `Feature` struct.
    pub fn new(tag: Tag, value: u32, range: impl RangeBounds<usize>) -> Feature {
        let max = std::u32::MAX as usize;
        let start = match range.start_bound() {
            Bound::Included(&included) => included.min(max) as u32,
            Bound::Excluded(&excluded) => excluded.min(max - 1) as u32 + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&included) => included.min(max) as u32,
            Bound::Excluded(&excluded) => excluded.saturating_sub(1).min(max) as u32,
            Bound::Unbounded => max as u32,
        };

        Feature {
            tag,
            value,
            start,
            end,
        }
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

            let tag = p.consume_tag()?;

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

            Some(Feature {
                tag,
                value,
                start,
                end,
            })
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


/// A font variation.
#[repr(C)]
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Variation {
    pub tag: Tag,
    pub value: f32,
}

impl std::str::FromStr for Variation {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse(s: &str) -> Option<Variation> {
            if s.is_empty() {
                return None;
            }

            let mut p = TextParser::new(s);

            // Parse tag.
            p.skip_spaces();
            let quote = p.consume_quote();

            let tag = p.consume_tag()?;

            // Force closing quote.
            if let Some(quote) = quote {
                p.consume_byte(quote)?;
            }

            let _ = p.consume_byte(b'=');
            let value = p.consume_f32()?;
            p.skip_spaces();

            if !p.at_end() {
                return None;
            }

            Some(Variation {
                tag,
                value,
            })
        }

        parse(s).ok_or("invalid variation")
    }
}
