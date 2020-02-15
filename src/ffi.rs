#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::os::raw::{c_void, c_char};

pub type hb_bool_t = i32;
pub type hb_codepoint_t = u32;
pub type hb_position_t = i32;
pub type hb_mask_t = u32;
pub type hb_direction_t = u32;
pub type hb_tag_t = u32;
pub type hb_script_t = u32;
pub type hb_memory_mode_t = u32;
pub type hb_buffer_flags_t = u32;
pub type hb_buffer_scratch_flags_t = u32;
pub type hb_destroy_func_t = std::option::Option<unsafe extern "C" fn(user_data: *mut c_void)>;

pub const HB_DIRECTION_INVALID: hb_direction_t = 0;
pub const HB_DIRECTION_LTR: hb_direction_t = 4;
pub const HB_DIRECTION_RTL: hb_direction_t = 5;
pub const HB_DIRECTION_TTB: hb_direction_t = 6;
pub const HB_DIRECTION_BTT: hb_direction_t = 7;

pub const HB_MEMORY_MODE_READONLY: hb_memory_mode_t = 1;

pub const HB_UNICODE_GENERAL_CATEGORY_CONTROL: u32 = 0;
pub const HB_UNICODE_GENERAL_CATEGORY_FORMAT: u32 = 1;
pub const HB_UNICODE_GENERAL_CATEGORY_UNASSIGNED: u32 = 2;
pub const HB_UNICODE_GENERAL_CATEGORY_PRIVATE_USE: u32 = 3;
pub const HB_UNICODE_GENERAL_CATEGORY_SURROGATE: u32 = 4;
pub const HB_UNICODE_GENERAL_CATEGORY_LOWERCASE_LETTER: u32 = 5;
pub const HB_UNICODE_GENERAL_CATEGORY_MODIFIER_LETTER: u32 = 6;
pub const HB_UNICODE_GENERAL_CATEGORY_OTHER_LETTER: u32 = 7;
pub const HB_UNICODE_GENERAL_CATEGORY_TITLECASE_LETTER: u32 = 8;
pub const HB_UNICODE_GENERAL_CATEGORY_UPPERCASE_LETTER: u32 = 9;
pub const HB_UNICODE_GENERAL_CATEGORY_SPACING_MARK: u32 = 10;
pub const HB_UNICODE_GENERAL_CATEGORY_ENCLOSING_MARK: u32 = 11;
pub const HB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK: u32 = 12;
pub const HB_UNICODE_GENERAL_CATEGORY_DECIMAL_NUMBER: u32 = 13;
pub const HB_UNICODE_GENERAL_CATEGORY_LETTER_NUMBER: u32 = 14;
pub const HB_UNICODE_GENERAL_CATEGORY_OTHER_NUMBER: u32 = 15;
pub const HB_UNICODE_GENERAL_CATEGORY_CONNECT_PUNCTUATION: u32 = 16;
pub const HB_UNICODE_GENERAL_CATEGORY_DASH_PUNCTUATION: u32 = 17;
pub const HB_UNICODE_GENERAL_CATEGORY_CLOSE_PUNCTUATION: u32 = 18;
pub const HB_UNICODE_GENERAL_CATEGORY_FINAL_PUNCTUATION: u32 = 19;
pub const HB_UNICODE_GENERAL_CATEGORY_INITIAL_PUNCTUATION: u32 = 20;
pub const HB_UNICODE_GENERAL_CATEGORY_OTHER_PUNCTUATION: u32 = 21;
pub const HB_UNICODE_GENERAL_CATEGORY_OPEN_PUNCTUATION: u32 = 22;
pub const HB_UNICODE_GENERAL_CATEGORY_CURRENCY_SYMBOL: u32 = 23;
pub const HB_UNICODE_GENERAL_CATEGORY_MODIFIER_SYMBOL: u32 = 24;
pub const HB_UNICODE_GENERAL_CATEGORY_MATH_SYMBOL: u32 = 25;
pub const HB_UNICODE_GENERAL_CATEGORY_OTHER_SYMBOL: u32 = 26;
pub const HB_UNICODE_GENERAL_CATEGORY_LINE_SEPARATOR: u32 = 27;
pub const HB_UNICODE_GENERAL_CATEGORY_PARAGRAPH_SEPARATOR: u32 = 28;
pub const HB_UNICODE_GENERAL_CATEGORY_SPACE_SEPARATOR: u32 = 29;


#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct hb_feature_t {
    pub tag: hb_tag_t,
    pub value: u32,
    pub start: u32,
    pub end: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hb_blob_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct rb_buffer_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hb_segment_properties_t {
    pub direction: hb_direction_t,
    pub script: hb_script_t,
    pub language: *const c_char,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct hb_glyph_position_t {
    pub x_advance: hb_position_t,
    pub y_advance: hb_position_t,
    pub x_offset: hb_position_t,
    pub y_offset: hb_position_t,
    pub var: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct hb_glyph_info_t {
    pub codepoint: hb_codepoint_t,
    pub mask: hb_mask_t,
    pub cluster: u32,
    pub var1: u32,
    pub var2: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hb_font_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hb_glyph_bbox_t {
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct hb_glyph_extents_t {
    pub x_bearing: hb_position_t,
    pub y_bearing: hb_position_t,
    pub width: hb_position_t,
    pub height: hb_position_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hb_face_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct rb_ot_map_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct rb_ot_map_builder_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hb_shape_plan_t {
    _unused: [u8; 0],
}

pub type pause_func_t = Option<
    unsafe extern "C" fn(
        plan: *const hb_shape_plan_t,
        font: *mut hb_font_t,
        buffer: *mut rb_buffer_t,
    ),
>;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct rb_ttf_parser_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct rb_complex_shaper_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hb_ot_shape_planner_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hb_ot_shape_normalize_context_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union hb_var_int_t {
    pub var_u32: u32,
    pub var_i32: i32,
    pub var_u16: [u16; 2usize],
    pub var_i16: [i16; 2usize],
    pub var_u8: [u8; 4usize],
    pub var_i8: [i8; 4usize],
    _bindgen_union_align: u32,
}

pub const HB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE: hb_ot_shape_zero_width_marks_type_t = 0;
pub const HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_EARLY: hb_ot_shape_zero_width_marks_type_t = 1;
pub const HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE: hb_ot_shape_zero_width_marks_type_t = 2;
pub type hb_ot_shape_zero_width_marks_type_t = u32;

//pub type hb_unicode_props_flags_t = u32;
pub const HB_OT_SHAPE_NORMALIZATION_MODE_NONE: hb_ot_shape_normalization_mode_t = 0;
//pub const HB_OT_SHAPE_NORMALIZATION_MODE_DECOMPOSED: hb_ot_shape_normalization_mode_t = 1;
//pub const HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS: hb_ot_shape_normalization_mode_t = 2;
pub const HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT : hb_ot_shape_normalization_mode_t = 3 ;
//pub const HB_OT_SHAPE_NORMALIZATION_MODE_AUTO: hb_ot_shape_normalization_mode_t = 4;
pub const HB_OT_SHAPE_NORMALIZATION_MODE_DEFAULT: hb_ot_shape_normalization_mode_t = 4;
pub type hb_ot_shape_normalization_mode_t = u32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct hb_ot_complex_shaper_t {
    pub collect_features: Option<unsafe extern "C" fn(plan: *mut hb_ot_shape_planner_t)>,
    pub override_features: Option<unsafe extern "C" fn(plan: *mut hb_ot_shape_planner_t)>,
    pub data_create: Option<unsafe extern "C" fn(plan: *const hb_shape_plan_t) -> *mut c_void>,
    pub data_destroy: Option<unsafe extern "C" fn(data: *mut c_void)>,
    pub preprocess_text: Option<
        unsafe extern "C" fn(
            plan: *const hb_shape_plan_t,
            buffer: *mut rb_buffer_t,
            font: *mut hb_font_t,
        ),
    >,
    pub postprocess_glyphs: Option<
        unsafe extern "C" fn(
            plan: *const hb_shape_plan_t,
            buffer: *mut rb_buffer_t,
            font: *mut hb_font_t,
        ),
    >,
    pub normalization_preference: hb_ot_shape_normalization_mode_t,
    pub decompose: Option<
        unsafe extern "C" fn(
            c: *const hb_ot_shape_normalize_context_t,
            ab: hb_codepoint_t,
            a: *mut hb_codepoint_t,
            b: *mut hb_codepoint_t,
        ) -> bool,
    >,
    pub compose: Option<
        unsafe extern "C" fn(
            c: *const hb_ot_shape_normalize_context_t,
            a: hb_codepoint_t,
            b: hb_codepoint_t,
            ab: *mut hb_codepoint_t,
        ) -> bool,
    >,
    pub setup_masks: Option<
        unsafe extern "C" fn(
            plan: *const hb_shape_plan_t,
            buffer: *mut rb_buffer_t,
            font: *mut hb_font_t,
        ),
    >,
    pub gpos_tag: hb_tag_t,
    pub reorder_marks: Option<
        unsafe extern "C" fn(
            plan: *const hb_shape_plan_t,
            buffer: *mut rb_buffer_t,
            start: u32,
            end: u32,
        ),
    >,
    pub zero_width_marks: hb_ot_shape_zero_width_marks_type_t,
    pub fallback_position: bool,
}

extern "C" {
    pub fn hb_blob_create(data: *const c_char, length: u32, mode: hb_memory_mode_t, user_data: *mut c_void,
                          destroy: hb_destroy_func_t) -> *mut hb_blob_t;
    pub fn hb_blob_destroy(blob: *mut hb_blob_t);
    pub fn hb_face_create(blob: *mut hb_blob_t, ttf_parser_data: *const rb_ttf_parser_t, index: u32) -> *mut hb_face_t;
    pub fn hb_face_destroy(face: *mut hb_face_t);
    pub fn hb_face_set_upem(face: *mut hb_face_t, upem: u32);
    pub fn hb_face_get_upem(face: *const hb_face_t) -> u32;
    pub fn hb_font_create(face: *mut hb_face_t, ttf_parser_data: *const rb_ttf_parser_t) -> *mut hb_font_t;
    pub fn hb_font_destroy(font: *mut hb_font_t);
    pub fn hb_font_face(font: *mut hb_font_t) -> *mut hb_face_t;
    pub fn hb_font_set_scale(font: *mut hb_font_t, x_scale: i32, y_scale: i32);
    pub fn hb_font_get_scale(font: *mut hb_font_t, x_scale: *mut i32, y_scale: *mut i32);
    pub fn hb_font_set_ppem(font: *mut hb_font_t, x_ppem: u32, y_ppem: u32);
    pub fn hb_font_get_ppem(font: *mut hb_font_t, x_ppem: *mut u32, y_ppem: *mut u32);
    pub fn hb_font_set_ptem(font: *mut hb_font_t, ptem: f32);
    pub fn hb_font_set_variations(font: *mut hb_font_t, coords: *const i32, coords_length: u32);
    pub fn hb_font_get_glyph_extents(font: *mut hb_font_t, glyph: hb_codepoint_t, extents: *mut hb_glyph_extents_t) -> bool;
    pub fn hb_font_get_glyph_h_advance_default(font: *mut hb_font_t, glyph: hb_codepoint_t) -> hb_position_t;
    pub fn hb_ot_glyf_get_side_bearing_var(font: *mut hb_font_t, glyph: u32, is_vertical: bool) -> i32;
    pub fn hb_ot_glyf_get_advance_var(font: *mut hb_font_t, glyph: u32, is_vertical: bool) -> u32;
    pub fn hb_shape(font: *mut hb_font_t, buffer: *mut rb_buffer_t, features: *const hb_feature_t, num_features: u32);
    pub fn hb_ot_layout_lookup_would_substitute(face: *const hb_face_t, lookup_index: u32, glyphs: *const hb_codepoint_t,
                                                glyphs_length: u32, zero_context: hb_bool_t) -> hb_bool_t;
    pub fn hb_shape_plan_map(plan: *const hb_shape_plan_t) -> *const rb_ot_map_t;
    pub fn hb_shape_plan_data(plan: *const hb_shape_plan_t) -> *const c_void;
    pub fn hb_shape_plan_script(plan: *const hb_shape_plan_t) -> hb_script_t;
    pub fn hb_shape_plan_ttf_parser(plan: *const hb_shape_plan_t) -> *const rb_ttf_parser_t;
    pub fn hb_ot_shape_planner_map(planner: *const hb_ot_shape_planner_t) -> *mut rb_ot_map_builder_t;
    pub fn hb_ot_shape_planner_script(plan: *const hb_ot_shape_planner_t) -> hb_script_t;
    pub fn hb_ot_shape_normalize_context_has_gpos_mark(c: *const hb_ot_shape_normalize_context_t) -> bool;
    pub fn hb_ot_shape_normalize_context_ttf_parser(c: *const hb_ot_shape_normalize_context_t) -> *const rb_ttf_parser_t;
    pub fn hb_ot_shape_normalize_context_plan_data(c: *const hb_ot_shape_normalize_context_t) -> *const c_void;
    pub fn hb_ot_shape_normalize_context_face(c: *const hb_ot_shape_normalize_context_t) -> *const hb_face_t;
}
