#![allow(non_camel_case_types)]

use std::os::raw::{c_char, c_void};

use crate::Tag;

pub type rb_bool_t = i32;
pub type rb_codepoint_t = u32;
pub type rb_mask_t = u32;
pub type rb_position_t = i32;

pub const RB_DIRECTION_INVALID: rb_direction_t = 0;
pub const RB_DIRECTION_LTR: rb_direction_t = 4;
pub const RB_DIRECTION_RTL: rb_direction_t = 5;
pub const RB_DIRECTION_TTB: rb_direction_t = 6;
pub const RB_DIRECTION_BTT: rb_direction_t = 7;
pub type rb_direction_t = u32;

pub const RB_UNICODE_GENERAL_CATEGORY_CONTROL: u32                  = 0;
pub const RB_UNICODE_GENERAL_CATEGORY_FORMAT: u32                   = 1;
pub const RB_UNICODE_GENERAL_CATEGORY_UNASSIGNED: u32               = 2;
pub const RB_UNICODE_GENERAL_CATEGORY_PRIVATE_USE: u32              = 3;
pub const RB_UNICODE_GENERAL_CATEGORY_SURROGATE: u32                = 4;
pub const RB_UNICODE_GENERAL_CATEGORY_LOWERCASE_LETTER: u32         = 5;
pub const RB_UNICODE_GENERAL_CATEGORY_MODIFIER_LETTER: u32          = 6;
pub const RB_UNICODE_GENERAL_CATEGORY_OTHER_LETTER: u32             = 7;
pub const RB_UNICODE_GENERAL_CATEGORY_TITLECASE_LETTER: u32         = 8;
pub const RB_UNICODE_GENERAL_CATEGORY_UPPERCASE_LETTER: u32         = 9;
pub const RB_UNICODE_GENERAL_CATEGORY_SPACING_MARK: u32             = 10;
pub const RB_UNICODE_GENERAL_CATEGORY_ENCLOSING_MARK: u32           = 11;
pub const RB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK: u32         = 12;
pub const RB_UNICODE_GENERAL_CATEGORY_DECIMAL_NUMBER: u32           = 13;
pub const RB_UNICODE_GENERAL_CATEGORY_LETTER_NUMBER: u32            = 14;
pub const RB_UNICODE_GENERAL_CATEGORY_OTHER_NUMBER: u32             = 15;
pub const RB_UNICODE_GENERAL_CATEGORY_CONNECT_PUNCTUATION: u32      = 16;
pub const RB_UNICODE_GENERAL_CATEGORY_DASH_PUNCTUATION: u32         = 17;
pub const RB_UNICODE_GENERAL_CATEGORY_CLOSE_PUNCTUATION: u32        = 18;
pub const RB_UNICODE_GENERAL_CATEGORY_FINAL_PUNCTUATION: u32        = 19;
pub const RB_UNICODE_GENERAL_CATEGORY_INITIAL_PUNCTUATION: u32      = 20;
pub const RB_UNICODE_GENERAL_CATEGORY_OTHER_PUNCTUATION: u32        = 21;
pub const RB_UNICODE_GENERAL_CATEGORY_OPEN_PUNCTUATION: u32         = 22;
pub const RB_UNICODE_GENERAL_CATEGORY_CURRENCY_SYMBOL: u32          = 23;
pub const RB_UNICODE_GENERAL_CATEGORY_MODIFIER_SYMBOL: u32          = 24;
pub const RB_UNICODE_GENERAL_CATEGORY_MATH_SYMBOL: u32              = 25;
pub const RB_UNICODE_GENERAL_CATEGORY_OTHER_SYMBOL: u32             = 26;
pub const RB_UNICODE_GENERAL_CATEGORY_LINE_SEPARATOR: u32           = 27;
pub const RB_UNICODE_GENERAL_CATEGORY_PARAGRAPH_SEPARATOR: u32      = 28;
pub const RB_UNICODE_GENERAL_CATEGORY_SPACE_SEPARATOR: u32          = 29;

#[repr(C)]
#[derive(Clone, Copy)]
pub union rb_var_int_t {
    pub var_u32: u32,
    pub var_i32: i32,
    pub var_u16: [u16; 2usize],
    pub var_i16: [i16; 2usize],
    pub var_u8: [u8; 4usize],
    pub var_i8: [i8; 4usize],
    _bindgen_union_align: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_buffer_t {
    _unused: [u8; 0],
}

pub type rb_destroy_func_t = Option<unsafe extern "C" fn(user_data: *mut c_void)>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_blob_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_face_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_shape_plan_t { _unused: [u8; 0] }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_aat_map_t {
    pub _chain_flags: rb_vector_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_aat_map_builder_t {
    pub _face: *const rb_face_t,
    pub _features: rb_vector_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_vector_t {
    pub _allocated: i32,
    pub _length: u32,
    pub _array_z: *mut c_void,
}

impl rb_vector_t {
    pub fn zero() -> Self {
        Self {
            _allocated: 0,
            _length: 0,
            _array_z: std::ptr::null_mut(),
        }
    }
}

extern "C" {
    pub fn rb_blob_create(
        data: *const c_char,
        length: u32,
        user_data: *mut c_void,
        destroy: rb_destroy_func_t,
    ) -> *mut rb_blob_t;

    pub fn rb_blob_destroy(blob: *mut rb_blob_t);

    pub fn rb_face_sanitize_table(blob: *mut rb_blob_t, tag: Tag, glyph_count: u32) -> *mut rb_blob_t;

    pub fn rb_aat_map_init(map: *mut rb_aat_map_t);

    pub fn rb_aat_map_fini(map: *mut rb_aat_map_t);

    pub fn rb_aat_map_builder_init(builder: *mut rb_aat_map_builder_t, face: *const rb_face_t);

    pub fn rb_aat_map_builder_fini(builder: *mut rb_aat_map_builder_t);

    pub fn rb_aat_map_builder_add_feature(builder: *mut rb_aat_map_builder_t, kind: i32, setting: i32, is_exclusive: bool);

    pub fn rb_aat_map_builder_compile(builder: *mut rb_aat_map_builder_t, map: *mut rb_aat_map_t);

    pub fn rb_aat_layout_has_substitution(face: *const rb_face_t) -> rb_bool_t;

    pub fn rb_aat_layout_substitute(
        plan: *const rb_shape_plan_t,
        face: *const rb_face_t,
        buffer: *mut rb_buffer_t,
    );

    pub fn rb_aat_layout_zero_width_deleted_glyphs(buffer: *mut rb_buffer_t);

    pub fn rb_aat_layout_remove_deleted_glyphs(buffer: *mut rb_buffer_t);
}
