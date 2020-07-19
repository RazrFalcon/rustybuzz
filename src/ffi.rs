#![allow(non_camel_case_types)]

use std::os::raw::{c_char, c_void};

pub type hb_bool_t = i32;
pub type hb_codepoint_t = u32;
pub type hb_mask_t = u32;
pub type hb_position_t = i32;
pub type hb_script_t = u32;

pub const HB_DIRECTION_INVALID: hb_direction_t = 0;
pub const HB_DIRECTION_LTR: hb_direction_t = 4;
pub const HB_DIRECTION_RTL: hb_direction_t = 5;
pub const HB_DIRECTION_TTB: hb_direction_t = 6;
pub const HB_DIRECTION_BTT: hb_direction_t = 7;
pub type hb_direction_t = u32;

pub const HB_BUFFER_SERIALIZE_FLAG_NO_CLUSTERS: hb_buffer_serialize_flags_t = 1;
pub const HB_BUFFER_SERIALIZE_FLAG_NO_POSITIONS: hb_buffer_serialize_flags_t = 2;
pub const HB_BUFFER_SERIALIZE_FLAG_NO_GLYPH_NAMES: hb_buffer_serialize_flags_t = 4;
pub const HB_BUFFER_SERIALIZE_FLAG_GLYPH_EXTENTS: hb_buffer_serialize_flags_t = 8;
pub const HB_BUFFER_SERIALIZE_FLAG_GLYPH_FLAGS: hb_buffer_serialize_flags_t = 16;
pub const HB_BUFFER_SERIALIZE_FLAG_NO_ADVANCES: hb_buffer_serialize_flags_t = 32;
pub type hb_buffer_serialize_flags_t = u32;

pub const HB_BUFFER_SERIALIZE_FORMAT_TEXT: hb_buffer_serialize_format_t = 1413830740;
pub const HB_BUFFER_SERIALIZE_FORMAT_JSON: hb_buffer_serialize_format_t = 1246973774;
pub type hb_buffer_serialize_format_t = u32;

pub const HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES: hb_buffer_cluster_level_t = 0;
pub const HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS: hb_buffer_cluster_level_t = 1;
pub const HB_BUFFER_CLUSTER_LEVEL_CHARACTERS: hb_buffer_cluster_level_t = 2;
pub type hb_buffer_cluster_level_t = u32;

pub const HB_MEMORY_MODE_READONLY: hb_memory_mode_t = 1;
pub type hb_memory_mode_t = u32;

pub const HB_UNICODE_GENERAL_CATEGORY_CONTROL: i32                  = 0;
pub const HB_UNICODE_GENERAL_CATEGORY_FORMAT: i32                   = 1;
pub const HB_UNICODE_GENERAL_CATEGORY_UNASSIGNED: i32               = 2;
pub const HB_UNICODE_GENERAL_CATEGORY_PRIVATE_USE: i32              = 3;
pub const HB_UNICODE_GENERAL_CATEGORY_SURROGATE: i32                = 4;
pub const HB_UNICODE_GENERAL_CATEGORY_LOWERCASE_LETTER: i32         = 5;
pub const HB_UNICODE_GENERAL_CATEGORY_MODIFIER_LETTER: i32          = 6;
pub const HB_UNICODE_GENERAL_CATEGORY_OTHER_LETTER: i32             = 7;
pub const HB_UNICODE_GENERAL_CATEGORY_TITLECASE_LETTER: i32         = 8;
pub const HB_UNICODE_GENERAL_CATEGORY_UPPERCASE_LETTER: i32         = 9;
pub const HB_UNICODE_GENERAL_CATEGORY_SPACING_MARK: i32             = 10;
pub const HB_UNICODE_GENERAL_CATEGORY_ENCLOSING_MARK: i32           = 11;
pub const HB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK: i32         = 12;
pub const HB_UNICODE_GENERAL_CATEGORY_DECIMAL_NUMBER: i32           = 13;
pub const HB_UNICODE_GENERAL_CATEGORY_LETTER_NUMBER: i32            = 14;
pub const HB_UNICODE_GENERAL_CATEGORY_OTHER_NUMBER: i32             = 15;
pub const HB_UNICODE_GENERAL_CATEGORY_CONNECT_PUNCTUATION: i32      = 16;
pub const HB_UNICODE_GENERAL_CATEGORY_DASH_PUNCTUATION: i32         = 17;
pub const HB_UNICODE_GENERAL_CATEGORY_CLOSE_PUNCTUATION: i32        = 18;
pub const HB_UNICODE_GENERAL_CATEGORY_FINAL_PUNCTUATION: i32        = 19;
pub const HB_UNICODE_GENERAL_CATEGORY_INITIAL_PUNCTUATION: i32      = 20;
pub const HB_UNICODE_GENERAL_CATEGORY_OTHER_PUNCTUATION: i32        = 21;
pub const HB_UNICODE_GENERAL_CATEGORY_OPEN_PUNCTUATION: i32         = 22;
pub const HB_UNICODE_GENERAL_CATEGORY_CURRENCY_SYMBOL: i32          = 23;
pub const HB_UNICODE_GENERAL_CATEGORY_MODIFIER_SYMBOL: i32          = 24;
pub const HB_UNICODE_GENERAL_CATEGORY_MATH_SYMBOL: i32              = 25;
pub const HB_UNICODE_GENERAL_CATEGORY_OTHER_SYMBOL: i32             = 26;
pub const HB_UNICODE_GENERAL_CATEGORY_LINE_SEPARATOR: i32           = 27;
pub const HB_UNICODE_GENERAL_CATEGORY_PARAGRAPH_SEPARATOR: i32      = 28;
pub const HB_UNICODE_GENERAL_CATEGORY_SPACE_SEPARATOR: i32          = 29;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hb_glyph_info_t {
    pub codepoint: hb_codepoint_t,
    pub mask: hb_mask_t,
    pub cluster: u32,
    pub var1: hb_var_int_t,
    pub var2: hb_var_int_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hb_glyph_position_t {
    pub x_advance: hb_position_t,
    pub y_advance: hb_position_t,
    pub x_offset: hb_position_t,
    pub y_offset: hb_position_t,
    pub var: hb_var_int_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union _hb_var_int_t {
    pub u32: u32,
    pub i32: i32,
    pub u16: [u16; 2usize],
    pub i16: [i16; 2usize],
    pub u8: [u8; 4usize],
    pub i8: [i8; 4usize],
    _bindgen_union_align: u32,
}

impl std::fmt::Debug for _hb_var_int_t {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_hb_var_int_t {{ ... }}")
    }
}

pub type hb_var_int_t = _hb_var_int_t;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hb_glyph_extents_t {
    pub x_bearing: hb_position_t,
    pub y_bearing: hb_position_t,
    pub width: hb_position_t,
    pub height: hb_position_t,
}

pub type hb_language_t = *const c_char;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hb_buffer_t {
    _unused: [u8; 0],
}

pub type hb_destroy_func_t = Option<unsafe extern "C" fn(user_data: *mut c_void)>;


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hb_blob_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hb_face_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hb_font_t {
    _unused: [u8; 0],
}

extern "C" {
    pub fn hb_blob_create(
        data: *const c_char,
        length: u32,
        mode: hb_memory_mode_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    ) -> *mut hb_blob_t;

    pub fn hb_blob_destroy(blob: *mut hb_blob_t);

    pub fn hb_buffer_create() -> *mut hb_buffer_t;

    pub fn hb_buffer_destroy(buffer: *mut hb_buffer_t);

    pub fn hb_buffer_set_direction(buffer: *mut hb_buffer_t, direction: hb_direction_t);

    pub fn hb_buffer_get_direction(buffer: *mut hb_buffer_t) -> hb_direction_t;

    pub fn hb_buffer_set_script(buffer: *mut hb_buffer_t, script: hb_script_t);

    pub fn hb_buffer_get_script(buffer: *mut hb_buffer_t) -> hb_script_t;

    pub fn hb_buffer_set_language(buffer: *mut hb_buffer_t, language: hb_language_t);

    pub fn hb_buffer_get_language(buffer: *mut hb_buffer_t) -> hb_language_t;

    pub fn hb_buffer_guess_segment_properties(buffer: *mut hb_buffer_t);

    pub fn hb_buffer_set_cluster_level(
        buffer: *mut hb_buffer_t,
        cluster_level: hb_buffer_cluster_level_t,
    );

    pub fn hb_buffer_get_cluster_level(buffer: *mut hb_buffer_t) -> hb_buffer_cluster_level_t;

    pub fn hb_buffer_reset_clusters(buffer: *mut hb_buffer_t);

    pub fn hb_buffer_clear_contents(buffer: *mut hb_buffer_t);

    pub fn hb_buffer_add_utf8(
        buffer: *mut hb_buffer_t,
        text: *const c_char,
        text_length: i32,
        item_offset: u32,
        item_length: i32,
    );

    pub fn hb_buffer_get_length(buffer: *mut hb_buffer_t) -> u32;

    pub fn hb_buffer_get_glyph_infos(
        buffer: *mut hb_buffer_t,
        length: *mut u32,
    ) -> *mut hb_glyph_info_t;

    pub fn hb_buffer_get_glyph_positions(
        buffer: *mut hb_buffer_t,
        length: *mut u32,
    ) -> *mut hb_glyph_position_t;

    pub fn hb_buffer_serialize_glyphs(
        buffer: *mut hb_buffer_t,
        start: u32,
        end: u32,
        buf: *mut c_char,
        buf_size: u32,
        buf_consumed: *mut u32,
        font: *const hb_font_t,
        format: hb_buffer_serialize_format_t,
        flags: hb_buffer_serialize_flags_t,
    ) -> u32;

    pub fn hb_face_create(blob: *mut hb_blob_t, index: u32) -> *mut hb_face_t;

    pub fn hb_face_destroy(face: *mut hb_face_t);

    pub fn hb_shape(
        font: *const hb_font_t,
        buffer: *mut hb_buffer_t,
        features: *const crate::Feature,
        num_features: u32,
    ) -> hb_bool_t;
}
