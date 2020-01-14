use std::io::Read;

use bitflags::bitflags;

use crate::{Direction, Language, Script, Tag};
use crate::ffi;

/// `GlyphPosition` is the structure that holds the positions of the glyph in
/// both horizontal and vertical directions. All positions in `GlyphPosition`
/// are relative to the current point.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GlyphPosition {
    /// How much the line advances after drawing this glyph when setting text in
    /// horizontal direction.
    pub x_advance: i32,
    /// How much the line advances after drawing this glyph when setting text in
    /// vertical direction.
    pub y_advance: i32,
    /// How much the glyph moves on the X-axis before drawing it, this should not
    /// affect how much the line advances.
    pub x_offset: i32,
    /// How much the glyph moves on the Y-axis before drawing it, this should
    /// not affect how much the line advances.
    pub y_offset: i32,
    var: ffi::hb_var_int_t,
}

impl std::fmt::Debug for GlyphPosition {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("GlyphBuffer")
            .field("x_advance", &self.x_advance)
            .field("y_advance", &self.y_advance)
            .field("x_offset", &self.x_offset)
            .field("y_offset", &self.y_offset)
            .finish()
    }
}


/// A glyph info.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct GlyphInfo {
    /// Unicode codepoint.
    pub codepoint: u32,
    mask: ffi::hb_mask_t,
    /// Codepoint cluster.
    pub cluster: u32,
    var1: ffi::hb_var_int_t,
    var2: ffi::hb_var_int_t,
}

impl std::fmt::Debug for GlyphInfo {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("GlyphBuffer")
            .field("codepoint", &self.codepoint)
            .field("cluster", &self.cluster)
            .finish()
    }
}


/// The serialization format used in `BufferSerializer`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SerializeFormat {
    /// A human-readable, plain text format
    Text,
    /// A machine-readable JSON format.
    Json,
}

impl From<SerializeFormat> for ffi::hb_buffer_serialize_format_t {
    fn from(fmt: SerializeFormat) -> Self {
        match fmt {
            SerializeFormat::Text => ffi::HB_BUFFER_SERIALIZE_FORMAT_TEXT,
            SerializeFormat::Json => ffi::HB_BUFFER_SERIALIZE_FORMAT_JSON,
        }
    }
}

bitflags! {
    /// Flags used for serialization with a `BufferSerializer`.
    #[derive(Default)]
    pub struct SerializeFlags: u32 {
        /// Do not serialize glyph cluster.
        const NO_CLUSTERS = ffi::HB_BUFFER_SERIALIZE_FLAG_NO_CLUSTERS;
        /// Do not serialize glyph position information.
        const NO_POSITIONS = ffi::HB_BUFFER_SERIALIZE_FLAG_NO_POSITIONS;
        /// Do no serialize glyph name.
        const NO_GLYPH_NAMES = ffi::HB_BUFFER_SERIALIZE_FLAG_NO_GLYPH_NAMES;
        /// Serialize glyph extents.
        const GLYPH_EXTENTS = ffi::HB_BUFFER_SERIALIZE_FLAG_GLYPH_EXTENTS;
        /// Serialize glyph flags.
        const GLYPH_FLAGS = ffi::HB_BUFFER_SERIALIZE_FLAG_GLYPH_FLAGS;
        /// Do not serialize glyph advances, glyph offsets will reflect absolute
        /// glyph positions.
        const NO_ADVANCES = ffi::HB_BUFFER_SERIALIZE_FLAG_NO_ADVANCES;
    }
}

/// A type that can be used to serialize a `GlyphBuffer`.
#[derive(Debug)]
pub struct BufferSerializer<'a> {
    font: Option<&'a crate::Font<'a>>,
    buffer: &'a Buffer,
    start: usize,
    end: usize,
    format: SerializeFormat,
    flags: SerializeFlags,

    bytes: std::io::Cursor<Vec<u8>>,
}

impl<'a> Read for BufferSerializer<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.bytes.read(buf) {
            // if `bytes` is empty refill it
            Ok(0) => {
                if self.start > self.end.saturating_sub(1) {
                    return Ok(0);
                }
                let mut bytes_written = 0;
                let num_serialized_items = unsafe {
                    ffi::hb_buffer_serialize_glyphs(
                        self.buffer.as_ptr(),
                        self.start as u32,
                        self.end as u32,
                        self.bytes.get_mut().as_mut_ptr() as *mut _,
                        self.bytes.get_ref().capacity() as u32,
                        &mut bytes_written,
                        self.font
                            .map(|f| f.as_ptr())
                            .unwrap_or(std::ptr::null_mut()),
                        self.format.into(),
                        self.flags.bits(),
                    )
                };
                self.start += num_serialized_items as usize;
                self.bytes.set_position(0);
                unsafe { self.bytes.get_mut().set_len(bytes_written as usize) };

                self.read(buf)
            }
            Ok(size) => Ok(size),
            Err(err) => Err(err),
        }
    }
}

/// A `Buffer` can be filled with unicode text and corresponding cluster
/// indices.
///
/// The buffer manages an allocation for the unicode codepoints to be shaped.
/// This allocation is reused for storing the results of the shaping operation
/// in a `GlyphBuffer` object. The intended usage is to keep one (or e.g. one
/// per thread) `Buffer` around. When needed, you fill it with text that
/// should be shaped and pass it as an argument to the `shape` function. That
/// method returns a `GlyphBuffer` object containing the shaped glyph indices.
/// Once you got the needed information out of the `GlyphBuffer` you call its
/// `.clear()` method which in turn gives you a fresh `Buffer` (also
/// reusing the original allocation again). This buffer can then be used to
/// shape more text.
pub struct Buffer(*mut ffi::hb_buffer_t);

impl Buffer {
    /// Creates a new empty `Buffer`.
    pub fn new() -> Buffer {
        unsafe { Buffer(ffi::hb_buffer_create()) }
    }

    pub(crate) fn as_ptr(&self) -> *mut ffi::hb_buffer_t {
        self.0
    }

    /// Returns the length of the data of the buffer.
    ///
    /// This corresponds to the number of unicode codepoints contained in the
    /// buffer.
    pub fn len(&self) -> usize {
        unsafe { ffi::hb_buffer_get_length(self.as_ptr()) as usize }
    }

    /// Checks that buffer contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Add a string to the `Buffer`.
    pub fn add_str(&mut self, string: &str) {
        let utf8_ptr = string.as_ptr() as *const _;
        unsafe {
            ffi::hb_buffer_add_utf8(
                self.as_ptr(),
                utf8_ptr,
                string.len() as i32,
                0,
                string.len() as i32,
            );
        }
    }

    /// Returns an Iterator over the stored unicode codepoints.
    pub fn codepoints(&self) -> Codepoints<'_> {
        let infos = unsafe {
            let mut length: u32 = 0;
            let glyph_infos = ffi::hb_buffer_get_glyph_infos(self.as_ptr(), &mut length as *mut u32);
            std::slice::from_raw_parts(glyph_infos as *const _, length as usize)
        };

        Codepoints {
            slice_iter: infos.iter(),
        }
    }

    /// Set the text direction of the `Buffer`'s contents.
    pub fn set_direction(&mut self, direction: Direction) {
        unsafe { ffi::hb_buffer_set_direction(self.as_ptr(), direction.to_raw()); }
    }

    /// Returns the `Buffer`'s text direction.
    pub fn get_direction(&self) -> Direction {
        Direction::from_raw(unsafe { ffi::hb_buffer_get_direction(self.as_ptr()) })
    }

    /// Set script.
    pub fn set_script(&mut self, script: Script) {
        unsafe {
            ffi::hb_buffer_set_script(self.as_ptr(), script.tag().as_u32());
        }
    }

    /// Get buffer's script.
    pub fn get_script(&self) -> Script {
        unsafe {
            let tag = ffi::hb_buffer_get_script(self.as_ptr());
            Script(Tag(tag))
        }
    }

    /// Set the buffer language.
    pub fn set_language(&mut self, lang: Language) {
        unsafe { ffi::hb_buffer_set_language(self.as_ptr(), lang.0) }
    }

    /// Get the buffer language.
    pub fn get_language(&self) -> Option<Language> {
        let raw_lang = unsafe { ffi::hb_buffer_get_language(self.as_ptr()) };
        if raw_lang.is_null() {
            None
        } else {
            Some(Language(raw_lang))
        }
    }

    /// Guess the segment properties (direction, language, script) for the
    /// current buffer.
    pub fn guess_segment_properties(&mut self) {
        unsafe { ffi::hb_buffer_guess_segment_properties(self.as_ptr()); }
    }

    /// Pre-allocate the buffer to hold a string at least `size` codepoints.
    pub fn pre_allocate(&mut self, size: usize) {
        let size = size.min(std::os::raw::c_uint::max_value() as usize);
        unsafe { ffi::hb_buffer_pre_allocate(self.as_ptr(), size as _); }
    }

    /// Clear the contents of the buffer (i.e. the stored string of unicode
    /// characters).
    pub fn clear_contents(&mut self) {
        unsafe { ffi::hb_buffer_clear_contents(self.as_ptr()); }
    }

    /// Sets the codepoint that replaces invisible characters in
    /// the shaping result.
    ///
    /// If set to zero (default), the glyph for the U+0020 SPACE character is used.
    /// Otherwise, this value is used verbatim.
    pub fn set_invisible_glyph(&mut self, glyph: u32) {
        unsafe { ffi::hb_buffer_set_invisible_glyph(self.as_ptr(), glyph) }
    }

    /// Sets a cluster level.
    pub fn set_cluster_level(&mut self, level: u32) {
        assert!(level <= ffi::HB_BUFFER_CLUSTER_LEVEL_CHARACTERS);
        unsafe { ffi::hb_buffer_set_cluster_level(self.as_ptr(), level) }
    }

    /// Resets clusters.
    pub fn reset_clusters(&mut self) {
        unsafe { ffi::hb_buffer_reset_clusters(self.as_ptr()); }
    }
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Buffer")
            .field("direction", &self.get_direction())
            .field("language", &self.get_language())
            .field("script", &self.get_script())
            .finish()
    }
}

impl std::default::Default for Buffer {
    fn default() -> Buffer {
        Buffer::new()
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { ffi::hb_buffer_destroy(self.0); }
    }
}


/// An iterator over the codepoints stored in a `Buffer`.
///
/// You get an iterator of this type from the `.codepoints()` method on
/// `Buffer`. It yields `u32`s that should be interpreted as unicode
/// codepoints stored in the underlying buffer.
#[derive(Debug, Clone)]
pub struct Codepoints<'a> {
    slice_iter: std::slice::Iter<'a, GlyphInfo>,
}

impl<'a> Iterator for Codepoints<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        self.slice_iter.next().map(|info| info.codepoint)
    }
}

/// A `GlyphBuffer` contains the resulting output information of the shaping
/// process.
///
/// An object of this type is obtained through the `shape` function.
pub struct GlyphBuffer(pub(crate) Buffer);

impl GlyphBuffer {
    /// Returns the length of the data of the buffer.
    ///
    /// When called before shaping this is the number of unicode codepoints
    /// contained in the buffer. When called after shaping it returns the number
    /// of glyphs stored.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the buffer contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the glyph positions.
    pub fn get_glyph_positions(&self) -> &[GlyphPosition] {
        unsafe {
            let mut length: u32 = 0;
            let glyph_pos = ffi::hb_buffer_get_glyph_positions(self.0.as_ptr(), &mut length as *mut u32);
            std::slice::from_raw_parts(glyph_pos as *const _, length as usize)
        }
    }

    /// Get the glyph infos.
    pub fn get_glyph_infos(&self) -> &[GlyphInfo] {
        unsafe {
            let mut length: u32 = 0;
            let glyph_infos = ffi::hb_buffer_get_glyph_infos(self.0.as_ptr(), &mut length as *mut u32);
            std::slice::from_raw_parts(glyph_infos as *const _, length as usize)
        }
    }

    /// Clears the contents of the glyph buffer and returns an empty
    /// `Buffer` reusing the existing allocation.
    pub fn clear(self) -> Buffer {
        unsafe { ffi::hb_buffer_clear_contents(self.0.as_ptr()); }
        self.0
    }

    /// Returns a serializer that allows the contents of the buffer to be
    /// converted into a human or machine readable representation.
    pub fn serializer<'a>(
        &'a self,
        font: Option<&'a crate::Font<'a>>,
        format: SerializeFormat,
        flags: SerializeFlags,
    ) -> BufferSerializer<'a> {
        BufferSerializer {
            font,
            buffer: &self.0,
            start: 0,
            end: self.len(),
            format,
            flags,
            bytes: std::io::Cursor::new(Vec::with_capacity(128)),
        }
    }

    /// Reorders a glyph buffer to have canonical in-cluster glyph order/position.
    ///
    /// The resulting clusters should behave identical to pre-reordering clusters.
    ///
    /// Note: This has nothing to do with Unicode normalization.
    pub fn normalize_glyphs(&mut self) {
        unsafe { ffi::hb_buffer_normalize_glyphs(self.0.as_ptr()) }
    }
}

impl std::fmt::Debug for GlyphBuffer {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("GlyphBuffer")
            .field("glyph_positions", &self.get_glyph_positions())
            .field("glyph_infos", &self.get_glyph_infos())
            .finish()
    }
}

impl std::fmt::Display for GlyphBuffer {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut serializer = self.serializer(None, SerializeFormat::Text, SerializeFlags::default());
        let mut string = String::new();
        serializer.read_to_string(&mut string).unwrap();
        write!(fmt, "{}", string)?;
        Ok(())
    }
}
