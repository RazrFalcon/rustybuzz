use crate::ffi;

/// A type to represent 4-byte SFNT tags.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Tag(pub ffi::hb_tag_t);

impl Tag {
    /// Create a `Tag` from its four-char textual representation.
    ///
    /// All the arguments must be ASCII values.
    pub const fn new(a: char, b: char, c: char, d: char) -> Self {
        Tag(((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) | (d as u32))
    }

    fn tag_to_string(self) -> String {
        let mut buf: [u8; 4] = [0; 4];
        unsafe { ffi::hb_tag_to_string(self.0, buf.as_mut_ptr() as *mut _) };
        String::from_utf8_lossy(&buf).into()
    }

    /// Returns tag as 4-element byte array.
    pub const fn to_bytes(self) -> [u8; 4] {
        [
            (self.0 >> 24 & 0xff) as u8,
            (self.0 >> 16 & 0xff) as u8,
            (self.0 >> 8 & 0xff) as u8,
            (self.0 >> 0 & 0xff) as u8,
        ]
    }
}

impl std::fmt::Debug for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self.tag_to_string();
        let mut chars = string.chars().chain(std::iter::repeat('\u{FFFD}'));
        write!(
            f,
            "Tag({:?}, {:?}, {:?}, {:?})",
            chars.next().unwrap(),
            chars.next().unwrap(),
            chars.next().unwrap(),
            chars.next().unwrap()
        )
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tag_to_string())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// An Error generated when a `Tag` fails to parse from a `&str` with the
/// `from_str` function.
pub enum TagFromStrErr {
    /// The string contains non-ASCII characters.
    NonAscii,
    /// The string has length zero.
    ZeroLengthString,
}

impl std::str::FromStr for Tag {
    type Err = TagFromStrErr;
    /// Parses a `Tag` from a `&str` that contains four or less ASCII
    /// characters. When the string's length is smaller than 4 it is extended
    /// with `' '` (Space) characters. The remaining bytes of strings longer
    /// than 4 bytes are ignored.
    fn from_str(s: &str) -> Result<Tag, TagFromStrErr> {
        if !s.is_ascii() {
            return Err(TagFromStrErr::NonAscii);
        }

        if s.is_empty() {
            return Err(TagFromStrErr::ZeroLengthString);
        }

        let len = std::cmp::max(s.len(), 4) as i32;
        unsafe { Ok(Tag(ffi::hb_tag_from_string(s.as_ptr() as *mut _, len))) }
    }
}

/// Defines the direction in which text is to be read.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
            Direction::Invalid     => ffi::HB_DIRECTION_INVALID,
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


/// A text language.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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

/// A text script.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Script(pub ffi::hb_script_t);

impl Script {
    /// Creates a `Script` form ISO 15924 tag.
    pub fn from_iso15924_tag(tag: Tag) -> Self {
        Script(unsafe { ffi::hb_script_from_iso15924_tag(tag.0) })
    }

    /// Converts `Script` into ISO 15924 tag.
    pub fn to_iso15924_tag(self) -> Tag {
        Tag(unsafe { ffi::hb_script_to_iso15924_tag(self.0) })
    }
}

impl std::str::FromStr for Script {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tag = Tag::from_str(s).map_err(|_| "invalid script")?;
        Ok(Script::from_iso15924_tag(tag))
    }
}
