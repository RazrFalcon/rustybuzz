#![allow(non_camel_case_types)]

use std::os::raw::{c_char, c_void};

use crate::Tag;

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

pub const HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES: hb_buffer_cluster_level_t = 0;
pub const HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS: hb_buffer_cluster_level_t = 1;
pub const HB_BUFFER_CLUSTER_LEVEL_CHARACTERS: hb_buffer_cluster_level_t = 2;
pub type hb_buffer_cluster_level_t = u32;

pub const HB_MEMORY_MODE_READONLY: hb_memory_mode_t = 1;
pub type hb_memory_mode_t = u32;

pub const HB_UNICODE_GENERAL_CATEGORY_CONTROL: u32                  = 0;
pub const HB_UNICODE_GENERAL_CATEGORY_FORMAT: u32                   = 1;
pub const HB_UNICODE_GENERAL_CATEGORY_UNASSIGNED: u32               = 2;
pub const HB_UNICODE_GENERAL_CATEGORY_PRIVATE_USE: u32              = 3;
pub const HB_UNICODE_GENERAL_CATEGORY_SURROGATE: u32                = 4;
pub const HB_UNICODE_GENERAL_CATEGORY_LOWERCASE_LETTER: u32         = 5;
pub const HB_UNICODE_GENERAL_CATEGORY_MODIFIER_LETTER: u32          = 6;
pub const HB_UNICODE_GENERAL_CATEGORY_OTHER_LETTER: u32             = 7;
pub const HB_UNICODE_GENERAL_CATEGORY_TITLECASE_LETTER: u32         = 8;
pub const HB_UNICODE_GENERAL_CATEGORY_UPPERCASE_LETTER: u32         = 9;
pub const HB_UNICODE_GENERAL_CATEGORY_SPACING_MARK: u32             = 10;
pub const HB_UNICODE_GENERAL_CATEGORY_ENCLOSING_MARK: u32           = 11;
pub const HB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK: u32         = 12;
pub const HB_UNICODE_GENERAL_CATEGORY_DECIMAL_NUMBER: u32           = 13;
pub const HB_UNICODE_GENERAL_CATEGORY_LETTER_NUMBER: u32            = 14;
pub const HB_UNICODE_GENERAL_CATEGORY_OTHER_NUMBER: u32             = 15;
pub const HB_UNICODE_GENERAL_CATEGORY_CONNECT_PUNCTUATION: u32      = 16;
pub const HB_UNICODE_GENERAL_CATEGORY_DASH_PUNCTUATION: u32         = 17;
pub const HB_UNICODE_GENERAL_CATEGORY_CLOSE_PUNCTUATION: u32        = 18;
pub const HB_UNICODE_GENERAL_CATEGORY_FINAL_PUNCTUATION: u32        = 19;
pub const HB_UNICODE_GENERAL_CATEGORY_INITIAL_PUNCTUATION: u32      = 20;
pub const HB_UNICODE_GENERAL_CATEGORY_OTHER_PUNCTUATION: u32        = 21;
pub const HB_UNICODE_GENERAL_CATEGORY_OPEN_PUNCTUATION: u32         = 22;
pub const HB_UNICODE_GENERAL_CATEGORY_CURRENCY_SYMBOL: u32          = 23;
pub const HB_UNICODE_GENERAL_CATEGORY_MODIFIER_SYMBOL: u32          = 24;
pub const HB_UNICODE_GENERAL_CATEGORY_MATH_SYMBOL: u32              = 25;
pub const HB_UNICODE_GENERAL_CATEGORY_OTHER_SYMBOL: u32             = 26;
pub const HB_UNICODE_GENERAL_CATEGORY_LINE_SEPARATOR: u32           = 27;
pub const HB_UNICODE_GENERAL_CATEGORY_PARAGRAPH_SEPARATOR: u32      = 28;
pub const HB_UNICODE_GENERAL_CATEGORY_SPACE_SEPARATOR: u32          = 29;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_glyph_info_t {
    pub codepoint: hb_codepoint_t,
    pub mask: hb_mask_t,
    pub cluster: u32,
    pub var1: hb_var_int_t,
    pub var2: hb_var_int_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_glyph_position_t {
    pub x_advance: hb_position_t,
    pub y_advance: hb_position_t,
    pub x_offset: hb_position_t,
    pub y_offset: hb_position_t,
    pub var: hb_var_int_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union hb_var_int_t {
    pub var_u32: u32,
    pub var_i32: i32,
    pub var_u16: [u16; 2usize],
    pub var_i16: [i16; 2usize],
    pub var_u8: [u8; 4usize],
    pub var_i8: [i8; 4usize],
    _bindgen_union_align: u32,
}

impl std::fmt::Debug for hb_var_int_t {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_hb_var_int_t {{ ... }}")
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct hb_glyph_extents_t {
    pub x_bearing: hb_position_t,
    pub y_bearing: hb_position_t,
    pub width: hb_position_t,
    pub height: hb_position_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_ot_map_lookup_map_t {
    pub index: u16,
    pub auto_zwnj: bool,
    pub auto_zwj: bool,
    pub random: bool,
    pub mask: hb_mask_t,
}

pub type hb_language_t = *const c_char;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_buffer_t {
    _unused: [u8; 0],
}

pub type hb_destroy_func_t = Option<unsafe extern "C" fn(user_data: *mut c_void)>;
pub type hb_ot_map_feature_flags_t = u32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_blob_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_face_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_font_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_ot_map_t { _unused: [u8; 0] }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_ot_map_builder_t { _unused: [u8; 0] }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_ot_shape_plan_t { _unused: [u8; 0] }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_ot_shape_planner_t { _unused: [u8; 0] }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_ot_shape_normalize_context_t { _unused: [u8; 0] }

pub type hb_ot_pause_func_t = Option<
    unsafe extern "C" fn(
        plan: *const hb_ot_shape_plan_t,
        font: *mut hb_font_t,
        buffer: *mut hb_buffer_t,
    ),
>;

pub type hb_sort_funct_t =
    unsafe extern "C" fn(
        a: *const hb_glyph_info_t,
        b: *const hb_glyph_info_t,
    ) -> i32;

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

    pub fn hb_buffer_next_glyph(buffer: *mut hb_buffer_t);

    pub fn hb_buffer_replace_glyph(buffer: *mut hb_buffer_t, glyph_index: hb_codepoint_t);

    pub fn hb_buffer_replace_glyphs(
        buffer: *mut hb_buffer_t,
        num_in: u32,
        num_out: u32,
        glyph_data: *const hb_codepoint_t,
    );

    pub fn hb_buffer_output_glyph(buffer: *mut hb_buffer_t, glyph_index: hb_codepoint_t);

    pub fn hb_buffer_output_info(buffer: *mut hb_buffer_t, glyph_index: hb_glyph_info_t);

    pub fn hb_buffer_merge_clusters(buffer: *mut hb_buffer_t, start: u32, end: u32);

    pub fn hb_buffer_merge_out_clusters(buffer: *mut hb_buffer_t, start: u32, end: u32);

    pub fn hb_buffer_unsafe_to_break(buffer: *mut hb_buffer_t, start: u32, end: u32);

    pub fn hb_buffer_unsafe_to_break_from_outbuffer(buffer: *mut hb_buffer_t, start: u32, end: u32);

    pub fn hb_buffer_swap_buffers(buffer: *mut hb_buffer_t);

    pub fn hb_buffer_sort(buffer: *mut hb_buffer_t, start: u32, end: u32, p: hb_sort_funct_t);

    pub fn hb_buffer_clear_contents(buffer: *mut hb_buffer_t);

    pub fn hb_buffer_clear_output(buffer: *mut hb_buffer_t);

    pub fn hb_buffer_add_utf8(
        buffer: *mut hb_buffer_t,
        text: *const c_char,
        text_length: i32,
        item_offset: u32,
        item_length: i32,
    );

    pub fn hb_buffer_get_index(buffer: *mut hb_buffer_t) -> u32;

    pub fn hb_buffer_set_index(buffer: *mut hb_buffer_t, len: u32);

    pub fn hb_buffer_get_length(buffer: *mut hb_buffer_t) -> u32;

    pub fn hb_buffer_set_length_force(buffer: *mut hb_buffer_t, len: u32);

    pub fn hb_buffer_get_out_length(buffer: *mut hb_buffer_t) -> u32;

    pub fn hb_buffer_get_allocated(buffer: *mut hb_buffer_t) -> u32;

    pub fn hb_buffer_ensure(buffer: *mut hb_buffer_t, len: u32) -> hb_bool_t;

    pub fn hb_buffer_context_len(buffer: *mut hb_buffer_t, index: u32) -> u32;

    pub fn hb_buffer_context(buffer: *mut hb_buffer_t, context_index: u32, index: u32) -> hb_codepoint_t;

    pub fn hb_buffer_get_flags(buffer: *mut hb_buffer_t) -> u32;

    pub fn hb_buffer_get_scratch_flags(buffer: *mut hb_buffer_t) -> u32;
    pub fn hb_buffer_set_scratch_flags(buffer: *mut hb_buffer_t, flags: u32);

    pub fn hb_buffer_get_glyph_infos(
        buffer: *mut hb_buffer_t,
        length: *mut u32,
    ) -> *mut hb_glyph_info_t;

    pub fn hb_buffer_get_glyph_infos_ptr(buffer: *mut hb_buffer_t) -> *mut hb_glyph_info_t;

    pub fn hb_buffer_get_out_glyph_infos_ptr(buffer: *mut hb_buffer_t) -> *mut hb_glyph_info_t;

    pub fn hb_buffer_get_glyph_positions(
        buffer: *mut hb_buffer_t,
        length: *mut u32,
    ) -> *mut hb_glyph_position_t;

    pub fn hb_buffer_get_glyph_positions_ptr(buffer: *mut hb_buffer_t) -> *mut hb_glyph_position_t;

    pub fn hb_layout_next_syllable(buffer: *mut hb_buffer_t, start: u32) -> u32;

    pub fn hb_face_create(blob: *mut hb_blob_t, index: u32) -> *mut hb_face_t;

    pub fn hb_face_destroy(face: *mut hb_face_t);

    pub fn hb_ot_map_get_1_mask(map: *const hb_ot_map_t, tag: Tag) -> hb_mask_t;

    pub fn hb_ot_map_global_mask(map: *const hb_ot_map_t) -> hb_mask_t;

    pub fn hb_ot_map_get_found_script(map: *const hb_ot_map_t, index: u32) -> bool;

    pub fn hb_ot_map_get_chosen_script(map: *const hb_ot_map_t, index: u32) -> Tag;

    pub fn hb_ot_map_get_feature_stage(map: *const hb_ot_map_t, table_index: u32, feature_tag: Tag) -> u32;

    pub fn hb_ot_map_get_stage_lookups(
        plan: *const hb_ot_map_t,
        table_index: u32,
        stage: u32,
        plookups: *mut *const hb_ot_map_lookup_map_t,
        lookup_count: *mut u32,
    );

    pub fn hb_ot_shape_plan_get_ot_map(plan: *const hb_ot_shape_plan_t) -> *const hb_ot_map_t;

    pub fn hb_ot_shape_plan_get_data(plan: *mut hb_ot_shape_plan_t) -> *const c_void;

    pub fn hb_ot_shape_plan_get_script(plan: *mut hb_ot_shape_plan_t) -> hb_script_t;

    pub fn hb_ot_shape_plan_has_gpos_mark(plan: *mut hb_ot_shape_plan_t) -> bool;

    pub fn hb_ot_shape_planner_get_ot_map(
        planner: *mut hb_ot_shape_planner_t,
    ) -> *mut hb_ot_map_builder_t;

    pub fn hb_ot_shape_planner_get_script(
        planner: *mut hb_ot_shape_planner_t,
    ) -> hb_script_t;

    pub fn hb_ot_map_builder_add_feature(
        builder: *mut hb_ot_map_builder_t,
        tag: Tag,
        flags: hb_ot_map_feature_flags_t,
        value: u32,
    );

    pub fn hb_ot_map_builder_add_gsub_pause(
        builder: *mut hb_ot_map_builder_t,
        pause: hb_ot_pause_func_t,
    );

    pub fn hb_ot_shape_normalize_context_get_plan(
        ctx: *const hb_ot_shape_normalize_context_t,
    ) -> *const hb_ot_shape_plan_t;

    pub fn hb_ot_shape_normalize_context_get_font(
        ctx: *const hb_ot_shape_normalize_context_t,
    ) -> *const hb_font_t;

    pub fn hb_ot_layout_lookup_would_substitute(
        face: *mut hb_face_t,
        lookup_index: u32,
        glyphs: *const hb_codepoint_t,
        glyphs_length: u32,
        zero_context: hb_bool_t,
    ) -> hb_bool_t;

    pub fn hb_layout_clear_syllables(
        plan: *const hb_ot_shape_plan_t,
        font: *mut hb_font_t,
        buffer: *mut hb_buffer_t,
    );

    pub fn hb_clear_substitution_flags(
        plan: *const hb_ot_shape_plan_t,
        font: *mut hb_font_t,
        buffer: *mut hb_buffer_t,
    );

    pub fn hb_shape(
        font: *const hb_font_t,
        buffer: *mut hb_buffer_t,
        features: *const crate::Feature,
        num_features: u32,
    ) -> hb_bool_t;
}
