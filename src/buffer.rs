use std::fmt;
use std::io::Read;
use std::ptr::NonNull;

use ttf_parser::Tag;

use crate::common::{Direction, Language, Script};
use crate::ffi;

/// Holds the positions of the glyph in both horizontal and vertical directions.
///
/// All positions are relative to the current point.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GlyphPosition {
    /// How much the line advances after drawing this glyph when setting text in
    /// horizontal direction.
    pub x_advance: i32,
    /// How much the line advances after drawing this glyph when setting text in
    /// vertical direction.
    pub y_advance: i32,
    /// How much the glyph moves on the X-axis before drawing it, this should
    /// not affect how much the line advances.
    pub x_offset: i32,
    /// How much the glyph moves on the Y-axis before drawing it, this should
    /// not affect how much the line advances.
    pub y_offset: i32,
    var: ffi::hb_var_int_t,
}


/// A glyph info.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct GlyphInfo {
    /// A selected glyph.
    pub glyph: u32,
    mask: ffi::hb_mask_t,
    /// An original cluster index.
    pub cluster: u32,
    var1: ffi::hb_var_int_t,
    var2: ffi::hb_var_int_t,
}


/// A cluster level.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BufferClusterLevel {
    MonotoneGraphemes,
    MonotoneCharacters,
    Characters,
}

impl BufferClusterLevel {
    fn from_raw(raw: ffi::hb_buffer_cluster_level_t) -> Self {
        match raw {
            ffi::HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES => BufferClusterLevel::MonotoneGraphemes,
            ffi::HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS => BufferClusterLevel::MonotoneCharacters,
            ffi::HB_BUFFER_CLUSTER_LEVEL_CHARACTERS => BufferClusterLevel::Characters,
            _ => panic!("received unrecognized HB_BUFFER_CLUSTER_LEVEL"),
        }
    }

    fn into_raw(self) -> ffi::hb_buffer_cluster_level_t {
        match self {
            BufferClusterLevel::MonotoneGraphemes => ffi::HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES,
            BufferClusterLevel::MonotoneCharacters => ffi::HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS,
            BufferClusterLevel::Characters => ffi::HB_BUFFER_CLUSTER_LEVEL_CHARACTERS,
        }
    }
}

impl Default for BufferClusterLevel {
    fn default() -> Self {
        BufferClusterLevel::MonotoneGraphemes
    }
}


pub(crate) struct Buffer {
    ptr: NonNull<ffi::hb_buffer_t>,
}

impl Buffer {
    fn new() -> Buffer {
        let ptr = NonNull::new(unsafe { ffi::hb_buffer_create() }).unwrap(); // can't fail
        Buffer { ptr }
    }

    pub fn as_ptr(&self) -> *mut ffi::hb_buffer_t {
        self.ptr.as_ptr()
    }

    fn len(&self) -> usize {
        unsafe { ffi::hb_buffer_get_length(self.as_ptr()) as usize }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn clear(&mut self) {
        unsafe { ffi::hb_buffer_clear_contents(self.as_ptr()) };
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { ffi::hb_buffer_destroy(self.as_ptr()) }
    }
}


/// The serialization format used in `BufferSerializer`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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

bitflags::bitflags! {
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
///
/// A `BufferSerializer` is obtained by calling the `GlyphBuffer::serializer`
/// method and provides a `Read` implementation that allows you to read the
/// serialized buffer contents.
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
                            .unwrap_or(std::ptr::null()),
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


/// A buffer that contains an input string ready for shaping.
pub struct UnicodeBuffer(pub(crate) Buffer);

impl UnicodeBuffer {
    /// Create a new `UnicodeBuffer`.
    #[inline]
    pub fn new() -> UnicodeBuffer {
        UnicodeBuffer(Buffer::new())
    }

    /// Returns the length of the data of the buffer.
    ///
    /// This corresponds to the number of unicode codepoints contained in the
    /// buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the buffer contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Pushes a string to a buffer.
    pub fn push_str(&mut self, str: &str) {
        unsafe {
            ffi::hb_buffer_add_utf8(
                self.0.as_ptr(),
                str.as_ptr() as *const _,
                str.len() as i32,
                0,
                str.len() as i32,
            );
        }
    }

    /// Set the text direction of the `Buffer`'s contents.
    pub fn set_direction(&mut self, direction: Direction) {
        unsafe { ffi::hb_buffer_set_direction(self.0.as_ptr(), direction.to_raw()) };
    }

    /// Returns the `Buffer`'s text direction.
    pub fn direction(&self) -> Direction {
        Direction::from_raw(unsafe { ffi::hb_buffer_get_direction(self.0.as_ptr()) })
    }

    /// Set the script from an ISO15924 tag.
    pub fn set_script(&mut self, script: Script) {
        unsafe {
            ffi::hb_buffer_set_script(self.0.as_ptr(), script.0.as_u32())
        }
    }

    /// Get the ISO15924 script tag.
    pub fn script(&self) -> Script {
        unsafe {
            Script(Tag(ffi::hb_buffer_get_script(self.0.as_ptr())))
        }
    }

    /// Set the buffer language.
    pub fn set_language(&mut self, lang: Language) {
        unsafe { ffi::hb_buffer_set_language(self.0.as_ptr(), lang.0) }
    }

    /// Get the buffer language.
    pub fn language(&self) -> Option<Language> {
        let raw_lang = unsafe { ffi::hb_buffer_get_language(self.0.as_ptr()) };
        if raw_lang.is_null() {
            None
        } else {
            Some(Language(raw_lang))
        }
    }

    /// Guess the segment properties (direction, language, script) for the
    /// current buffer.
    pub fn guess_segment_properties(&mut self) {
        unsafe { ffi::hb_buffer_guess_segment_properties(self.0.as_ptr()) };
    }

    /// Set the cluster level of the buffer.
    pub fn set_cluster_level(&mut self, cluster_level: BufferClusterLevel) {
        unsafe { ffi::hb_buffer_set_cluster_level(self.0.as_ptr(), cluster_level.into_raw()) }
    }

    /// Retrieve the cluster level of the buffer.
    pub fn cluster_level(&self) -> BufferClusterLevel {
        BufferClusterLevel::from_raw(unsafe { ffi::hb_buffer_get_cluster_level(self.0.as_ptr()) })
    }

    /// Resets clusters.
    pub fn reset_clusters(&mut self) {
        unsafe { ffi::hb_buffer_reset_clusters(self.0.as_ptr()) }
    }

    /// Clear the contents of the buffer.
    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl std::fmt::Debug for UnicodeBuffer {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("UnicodeBuffer")
            .field("direction", &self.direction())
            .field("language", &self.language())
            .field("script", &self.script())
            .field("cluster_level", &self.cluster_level())
            .finish()
    }
}

impl Default for UnicodeBuffer {
    fn default() -> UnicodeBuffer {
        UnicodeBuffer::new()
    }
}


/// A buffer that contains the results of the shaping process.
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
    pub fn glyph_positions(&self) -> &[GlyphPosition] {
        unsafe {
            let mut length: u32 = 0;
            let glyph_pos =
                ffi::hb_buffer_get_glyph_positions(self.0.as_ptr(), &mut length as *mut u32);
            std::slice::from_raw_parts(glyph_pos as *const _, length as usize)
        }
    }

    /// Get the glyph infos.
    pub fn glyph_infos(&self) -> &[GlyphInfo] {
        unsafe {
            let mut length: u32 = 0;
            let glyph_infos = ffi::hb_buffer_get_glyph_infos(self.0.as_ptr(), &mut length as *mut u32);
            std::slice::from_raw_parts(glyph_infos as *const _, length as usize)
        }
    }

    /// Clears the content of the glyph buffer and returns an empty
    /// `UnicodeBuffer` reusing the existing allocation.
    pub fn clear(mut self) -> UnicodeBuffer {
        self.0.clear();
        UnicodeBuffer(self.0)
    }

    /// Returns a serializer that allows the contents of the buffer to be
    /// converted into a human or machine readable representation.
    ///
    /// # Arguments
    /// - `font`: Optionally a font can be provided for access to glyph names
    ///   and glyph extents. If `None` is passed an empty font is assumed.
    /// - `format`: The serialization format to use.
    /// - `flags`: Allows you to control which information will be contained in
    ///   the serialized output.
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
}

impl fmt::Debug for GlyphBuffer {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("GlyphBuffer")
            .field("glyph_positions", &self.glyph_positions())
            .field("glyph_infos", &self.glyph_infos())
            .finish()
    }
}

impl fmt::Display for GlyphBuffer {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut serializer =
            self.serializer(None, SerializeFormat::Text, SerializeFlags::default());
        let mut string = String::new();
        serializer.read_to_string(&mut string).unwrap();
        write!(fmt, "{}", string)?;
        Ok(())
    }
}
