use std::convert::TryFrom;
use std::fmt;
use std::io::Read;
use std::ptr::NonNull;

use ttf_parser::Tag;

use crate::common::{Direction, Language, Script};
use crate::ffi;
use crate::unicode::{GeneralCategory, GeneralCategoryExt};

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
    var: u32,
}


/// A glyph info.
#[derive(Clone, Copy, Default, Debug)]
#[repr(C)]
pub struct GlyphInfo {
    /// A selected glyph.
    pub codepoint: u32,
    pub(crate) mask: ffi::hb_mask_t,
    /// An original cluster index.
    pub cluster: u32,
    pub(crate) var1: u32,
    pub(crate) var2: u32,
}

impl GlyphInfo {
    #[inline]
    pub(crate) fn as_char(&self) -> char {
        char::try_from(self.codepoint).unwrap()
    }

    #[inline]
    pub(crate) fn glyph_props(&self) -> u16 {
        unsafe {
            let v: ffi::hb_var_int_t = std::mem::transmute(self.var1);
            v.var_u16[0]
        }
    }

    #[inline]
    fn unicode_props(&self) -> u16 {
        unsafe {
            let v: ffi::hb_var_int_t = std::mem::transmute(self.var2);
            v.var_u16[0]
        }
    }

    #[inline]
    fn set_unicode_props(&mut self, n: u16) {
        unsafe {
            let v: &mut ffi::hb_var_int_t = std::mem::transmute(&mut self.var2);
            v.var_u16[0] = n;
        }
    }

    #[inline]
    fn lig_props(&self) -> u8 {
        unsafe {
            let v: ffi::hb_var_int_t = std::mem::transmute(self.var1);
            v.var_u8[2]
        }
    }

    #[inline]
    pub(crate) fn general_category(&self) -> GeneralCategory {
        let n = self.unicode_props() & UnicodeProps::GENERAL_CATEGORY.bits;
        GeneralCategory::from_hb(n as u32)
    }

    #[inline]
    pub(crate) fn is_default_ignorable(&self) -> bool {
        let n = self.unicode_props() & UnicodeProps::IGNORABLE.bits;
        n != 0 && !self.is_ligated()
    }

    #[inline]
    pub(crate) fn is_ligated(&self) -> bool {
        self.glyph_props() & GlyphPropsFlags::LIGATED.bits != 0
    }

    #[inline]
    pub(crate) fn is_unicode_mark(&self) -> bool {
        self.general_category().is_mark()
    }

    #[inline]
    pub(crate) fn modified_combining_class(&self) -> u8 {
        if self.is_unicode_mark() {
            (self.unicode_props() >> 8) as u8
        } else {
            0
        }
    }

    #[inline]
    pub(crate) fn set_modified_combining_class(&mut self, mcc: u8) {
        if !self.is_unicode_mark() {
            return;
        }

        let n = ((mcc as u16) << 8) | (self.unicode_props() & 0xFF);
        self.set_unicode_props(n);
    }

    #[inline]
    pub(crate) fn is_multiplied(&self) -> bool {
        self.glyph_props() & GlyphPropsFlags::MULTIPLIED.bits != 0
    }

    #[inline]
    pub(crate) fn is_ligated_internal(&self) -> bool {
        const IS_LIG_BASE: u8 = 0x10;
        self.lig_props() & IS_LIG_BASE != 0
    }

    #[inline]
    pub(crate) fn lig_comp(&self) -> u8 {
        if self.is_ligated_internal() {
            0
        } else {
            self.lig_props() & 0x0F
        }
    }
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
    language: Option<Language>,
    should_drop: bool,
}

impl Buffer {
    #[inline]
    fn new() -> Self {
        let ptr = NonNull::new(unsafe { ffi::hb_buffer_create() }).unwrap(); // can't fail
        Buffer {
            ptr,
            language: None,
            should_drop: true,
        }
    }

    #[inline]
    pub(crate) fn from_ptr_mut(ptr: *mut ffi::hb_buffer_t) -> Self {
        Buffer {
            ptr: NonNull::new(ptr).unwrap(),
            language: None,
            should_drop: false,
        }
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *mut ffi::hb_buffer_t {
        self.ptr.as_ptr()
    }

    #[inline]
    pub(crate) fn set_cluster_level(&mut self, cluster_level: BufferClusterLevel) {
        unsafe { ffi::hb_buffer_set_cluster_level(self.as_ptr(), cluster_level.into_raw()) }
    }

    #[inline]
    pub(crate) fn cluster_level(&self) -> BufferClusterLevel {
        BufferClusterLevel::from_raw(unsafe { ffi::hb_buffer_get_cluster_level(self.as_ptr()) })
    }

    // buffer.info 0..allocated slice.
    #[inline]
    pub(crate) fn info(&self) -> &[GlyphInfo] {
        unsafe {
            let ptr = ffi::hb_buffer_get_glyph_infos_ptr(self.as_ptr());
            std::slice::from_raw_parts(ptr as *const _, self.allocated())
        }
    }

    // buffer.info 0..allocated slice.
    #[inline]
    pub(crate) fn info_mut(&mut self) -> &mut [GlyphInfo] {
        unsafe {
            let ptr = ffi::hb_buffer_get_glyph_infos_ptr(self.as_ptr());
            std::slice::from_raw_parts_mut(ptr as _, self.allocated())
        }
    }

    // buffer.info 0..len slice.
    #[inline]
    pub(crate) fn info_slice_mut(&mut self) -> &mut [GlyphInfo] {
        unsafe {
            let ptr = ffi::hb_buffer_get_glyph_infos_ptr(self.as_ptr());
            std::slice::from_raw_parts_mut(ptr as _, self.len())
        }
    }

    #[inline]
    pub(crate) fn pos(&self) -> &[GlyphPosition] {
        unsafe {
            let ptr = ffi::hb_buffer_get_glyph_positions_ptr(self.as_ptr());
            std::slice::from_raw_parts(ptr as *const _, self.allocated())
        }
    }

    #[inline]
    pub(crate) fn pos_mut(&mut self) -> &mut [GlyphPosition] {
        unsafe {
            let ptr = ffi::hb_buffer_get_glyph_positions_ptr(self.as_ptr());
            std::slice::from_raw_parts_mut(ptr as _, self.allocated())
        }
    }

    // buffer.out_info 0..allocated slice.
    #[inline]
    pub(crate) fn out_info_mut(&mut self) -> &mut [GlyphInfo] {
        unsafe {
            let ptr = ffi::hb_buffer_get_out_glyph_infos_ptr(self.as_ptr());
            std::slice::from_raw_parts_mut(ptr as _, self.allocated())
        }
    }

    #[inline]
    pub(crate) fn cur(&self, i: usize) -> &GlyphInfo {
        &self.info()[self.idx() + i]
    }

    #[inline]
    pub(crate) fn cur_mut(&mut self, i: usize) -> &mut GlyphInfo {
        let offset = self.idx() + i;
        &mut self.info_mut()[offset]
    }

    #[inline]
    pub(crate) fn idx(&self) -> usize {
        unsafe { ffi::hb_buffer_get_index(self.as_ptr()) as usize }
    }

    #[inline]
    pub(crate) fn set_idx(&self, idx: usize) {
        unsafe { ffi::hb_buffer_set_index(self.as_ptr(), idx as u32) };
    }

    #[inline]
    pub(crate) fn allocated(&self) -> usize {
        unsafe { ffi::hb_buffer_get_allocated(self.as_ptr()) as usize }
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        unsafe { ffi::hb_buffer_get_length(self.as_ptr()) as usize }
    }

    #[inline]
    pub(crate) fn set_len(&self, len: usize) {
        unsafe { ffi::hb_buffer_set_length_force(self.as_ptr(), len as u32) };
    }

    #[inline]
    pub(crate) fn out_len(&self) -> usize {
        unsafe { ffi::hb_buffer_get_out_length(self.as_ptr()) as usize }
    }

    #[inline]
    pub(crate) fn context_len(&self, index: u32) -> u32 {
        unsafe { ffi::hb_buffer_context_len(self.as_ptr(), index) }
    }

    #[inline]
    pub(crate) fn context(&self, context_index: u32, index: u32) -> char {
        let c = unsafe { ffi::hb_buffer_context(self.as_ptr(), context_index, index) };
        char::try_from(c).unwrap()
    }

    #[inline]
    pub(crate) fn ensure(&self, len: usize) {
        unsafe { ffi::hb_buffer_ensure(self.as_ptr(), len as u32) };
    }

    #[inline]
    pub(crate) fn flags(&self) -> BufferFlags {
        unsafe {
            BufferFlags::from_bits_unchecked(ffi::hb_buffer_get_flags(self.as_ptr()))
        }
    }

    #[inline]
    pub(crate) fn scratch_flags(&self) -> BufferScratchFlags {
        unsafe {
            BufferScratchFlags::from_bits_unchecked(ffi::hb_buffer_get_scratch_flags(self.as_ptr()))
        }
    }

    #[inline]
    pub(crate) fn set_scratch_flags(&mut self, flags: BufferScratchFlags) {
        unsafe {
            ffi::hb_buffer_set_scratch_flags(self.as_ptr(), flags.bits)
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub(crate) fn next_glyph(&mut self) {
        unsafe { ffi::hb_buffer_next_glyph(self.as_ptr()) };
    }

    #[inline]
    pub(crate) fn replace_glyphs(&mut self, num_in: usize, num_out: usize, glyph_data: &[ffi::hb_codepoint_t]) {
        unsafe { ffi::hb_buffer_replace_glyphs(self.as_ptr(), num_in as u32, num_out as u32, glyph_data.as_ptr()) };
    }

    #[inline]
    pub(crate) fn merge_clusters(&mut self, start: usize, end: usize) {
        unsafe { ffi::hb_buffer_merge_clusters(self.as_ptr(), start as u32, end as u32) };
    }

    #[inline]
    pub(crate) fn merge_out_clusters(&mut self, start: usize, end: usize) {
        unsafe { ffi::hb_buffer_merge_out_clusters(self.as_ptr(), start as u32, end as u32) };
    }

    #[inline]
    pub(crate) fn unsafe_to_break(&mut self, start: usize, end: usize) {
        unsafe { ffi::hb_buffer_unsafe_to_break(self.as_ptr(), start as u32, end as u32) };
    }

    #[inline]
    pub(crate) fn unsafe_to_break_from_outbuffer(&mut self, start: usize, end: usize) {
        unsafe { ffi::hb_buffer_unsafe_to_break_from_outbuffer(self.as_ptr(), start as u32, end as u32) };
    }

    #[inline]
    pub(crate) fn swap_buffers(&mut self) {
        unsafe { ffi::hb_buffer_swap_buffers(self.as_ptr()) };
    }

    #[inline]
    fn clear(&mut self) {
        unsafe { ffi::hb_buffer_clear_contents(self.as_ptr()) };
    }

    #[inline]
    pub(crate) fn clear_output(&mut self) {
        unsafe { ffi::hb_buffer_clear_output(self.as_ptr()) };
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if self.should_drop {
            unsafe { ffi::hb_buffer_destroy(self.as_ptr()) }
        }
    }
}


bitflags::bitflags! {
    #[derive(Default)]
    pub struct UnicodeProps: u16 {
        const GENERAL_CATEGORY  = 0x001F;
        const IGNORABLE         = 0x0020;
        // MONGOLIAN FREE VARIATION SELECTOR 1..3, or TAG characters
        const HIDDEN            = 0x0040;
        const CONTINUATION      = 0x0080;

        // If GEN_CAT=FORMAT, top byte masks:
        const CF_ZWJ            = 0x0100;
        const CF_ZWNJ           = 0x0200;
    }
}


bitflags::bitflags! {
    #[derive(Default)]
    pub struct GlyphPropsFlags: u16 {
        // The following three match LookupFlags::Ignore* numbers.
        const BASE_GLYPH    = 0x02;
        const LIGATURE      = 0x04;
        const MARK          = 0x08;

        // The following are used internally; not derived from GDEF.
        const SUBSTITUTED   = 0x10;
        const LIGATED       = 0x20;
        const MULTIPLIED    = 0x40;

        const PRESERVE      = Self::SUBSTITUTED.bits | Self::LIGATED.bits | Self::MULTIPLIED.bits;
    }
}


bitflags::bitflags! {
    #[derive(Default)]
    pub struct BufferFlags: u32 {
        const BEGINNING_OF_TEXT             = 1 << 1;
        const END_OF_TEXT                   = 1 << 2;
        const PRESERVE_DEFAULT_IGNORABLES   = 1 << 3;
        const REMOVE_DEFAULT_IGNORABLES     = 1 << 4;
        const DO_NOT_INSERT_DOTTED_CIRCLE   = 1 << 5;
    }
}


bitflags::bitflags! {
    #[derive(Default)]
    pub struct BufferScratchFlags: u32 {
        const HAS_NON_ASCII             = 0x00000001;
        const HAS_DEFAULT_IGNORABLES    = 0x00000002;
        const HAS_SPACE_FALLBACK        = 0x00000004;
        const HAS_GPOS_ATTACHMENT       = 0x00000008;
        const HAS_UNSAFE_TO_BREAK       = 0x00000010;
        const HAS_CGJ                   = 0x00000020;

        // Reserved for complex shapers' internal use.
        const COMPLEX0                  = 0x01000000;
        const COMPLEX1                  = 0x02000000;
        const COMPLEX2                  = 0x04000000;
        const COMPLEX3                  = 0x08000000;
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
        let lang_ptr = lang.0.as_ptr();
        self.0.language = Some(lang); // Language must outlive Buffer.
        unsafe { ffi::hb_buffer_set_language(self.0.as_ptr(), lang_ptr) }
    }

    /// Get the buffer language.
    pub fn language(&self) -> Option<Language> {
        let raw_lang = unsafe { ffi::hb_buffer_get_language(self.0.as_ptr()) };
        if raw_lang.is_null() {
            None
        } else {
            unsafe { Some(Language(std::ffi::CStr::from_ptr(raw_lang).into())) }
        }
    }

    /// Guess the segment properties (direction, language, script) for the
    /// current buffer.
    pub fn guess_segment_properties(&mut self) {
        unsafe { ffi::hb_buffer_guess_segment_properties(self.0.as_ptr()) };
    }

    /// Set the cluster level of the buffer.
    pub fn set_cluster_level(&mut self, cluster_level: BufferClusterLevel) {
        self.0.set_cluster_level(cluster_level)
    }

    /// Retrieve the cluster level of the buffer.
    pub fn cluster_level(&self) -> BufferClusterLevel {
        self.0.cluster_level()
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
