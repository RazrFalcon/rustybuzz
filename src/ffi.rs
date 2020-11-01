#![allow(non_camel_case_types)]

use std::os::raw::{c_char, c_void};

use crate::Tag;

pub type rb_bool_t = i32;
pub type rb_codepoint_t = u32;
pub type rb_mask_t = u32;
pub type rb_position_t = i32;
pub type rb_script_t = u32;
pub type rb_ot_shape_normalization_mode_t = u32;

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
pub struct rb_segment_properties_t {
    pub direction: rb_direction_t,
    pub script: rb_script_t,
    pub language: *const c_char,
}

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
#[derive(Clone, Copy, Default)]
pub struct rb_glyph_extents_t {
    pub x_bearing: rb_position_t,
    pub y_bearing: rb_position_t,
    pub width: rb_position_t,
    pub height: rb_position_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_ot_map_lookup_map_t {
    pub index: u16,
    pub auto_zwnj: bool,
    pub auto_zwj: bool,
    pub random: bool,
    pub mask: rb_mask_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_buffer_t {
    _unused: [u8; 0],
}

pub type rb_destroy_func_t = Option<unsafe extern "C" fn(user_data: *mut c_void)>;
pub type rb_ot_map_feature_flags_t = u32;

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
pub struct rb_font_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_ot_map_t { _unused: [u8; 0] }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_ot_map_builder_t { _unused: [u8; 0] }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_ot_shape_plan_t { _unused: [u8; 0] }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_ot_shape_planner_t { _unused: [u8; 0] }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_ot_complex_shaper_t { _unused: [u8; 0] }

pub type rb_ot_reorder_marks_func_t = unsafe extern "C" fn(
    plan: *const rb_ot_shape_plan_t,
    buffer: *mut rb_buffer_t,
    start: u32,
    end: u32,
);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_ot_shape_normalize_context_t { _unused: [u8; 0] }

pub type rb_ot_decompose_func_t = unsafe extern "C" fn(
    ctx: *const rb_ot_shape_normalize_context_t,
    ab: rb_codepoint_t,
    a: *mut rb_codepoint_t,
    b: *mut rb_codepoint_t,
) -> rb_bool_t;

pub type rb_ot_compose_func_t = unsafe extern "C" fn(
    ctx: *const rb_ot_shape_normalize_context_t,
    a: rb_codepoint_t,
    b: rb_codepoint_t,
    ab: *mut rb_codepoint_t,
) -> rb_bool_t;

pub type rb_ot_pause_func_t = Option<
    unsafe extern "C" fn(
        plan: *const rb_ot_shape_plan_t,
        font: *mut rb_font_t,
        buffer: *mut rb_buffer_t,
    ),
>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_would_apply_context_t { _unused: [u8; 0] }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_ot_apply_context_t { _unused: [u8; 0] }

extern "C" {
    pub fn rb_blob_create(
        data: *const c_char,
        length: u32,
        user_data: *mut c_void,
        destroy: rb_destroy_func_t,
    ) -> *mut rb_blob_t;

    pub fn rb_blob_destroy(blob: *mut rb_blob_t);

    pub fn rb_face_create(blob: *mut rb_blob_t, index: u32) -> *mut rb_face_t;

    pub fn rb_face_destroy(face: *mut rb_face_t);

    pub fn rb_ot_map_get_1_mask(map: *const rb_ot_map_t, tag: Tag) -> rb_mask_t;

    pub fn rb_ot_map_global_mask(map: *const rb_ot_map_t) -> rb_mask_t;

    pub fn rb_ot_map_get_found_script(map: *const rb_ot_map_t, index: u32) -> bool;

    pub fn rb_ot_map_get_chosen_script(map: *const rb_ot_map_t, index: u32) -> Tag;

    pub fn rb_ot_map_get_feature_stage(map: *const rb_ot_map_t, table_index: u32, feature_tag: Tag) -> u32;

    pub fn rb_ot_map_get_stage_lookups(
        plan: *const rb_ot_map_t,
        table_index: u32,
        stage: u32,
        plookups: *mut *const rb_ot_map_lookup_map_t,
        lookup_count: *mut u32,
    );

    pub fn rb_ot_shape_plan_get_ot_complex_shaper(plan: *const rb_ot_shape_plan_t) -> *const rb_ot_complex_shaper_t;

    pub fn rb_ot_shape_plan_get_ot_map(plan: *const rb_ot_shape_plan_t) -> *const rb_ot_map_t;

    pub fn rb_ot_shape_plan_get_data(plan: *const rb_ot_shape_plan_t) -> *const c_void;

    pub fn rb_ot_shape_plan_get_script(plan: *const rb_ot_shape_plan_t) -> rb_script_t;

    pub fn rb_ot_shape_plan_get_direction(plan: *const rb_ot_shape_plan_t) -> rb_direction_t;

    pub fn rb_ot_shape_plan_has_gpos_mark(plan: *const rb_ot_shape_plan_t) -> bool;

    pub fn rb_ot_shape_planner_get_ot_map(
        planner: *mut rb_ot_shape_planner_t,
    ) -> *mut rb_ot_map_builder_t;

    pub fn rb_ot_shape_planner_get_script(
        planner: *const rb_ot_shape_planner_t,
    ) -> rb_script_t;

    pub fn rb_ot_complex_shaper_get_normalization_preference(
        shaper: *const rb_ot_complex_shaper_t,
    ) -> rb_ot_shape_normalization_mode_t;

    pub fn rb_ot_complex_shaper_get_decompose(
        shaper: *const rb_ot_complex_shaper_t,
    ) -> Option<rb_ot_decompose_func_t>;

    pub fn rb_ot_complex_shaper_get_compose(
        shaper: *const rb_ot_complex_shaper_t,
    ) -> Option<rb_ot_compose_func_t>;

    pub fn rb_ot_complex_shaper_get_reorder_marks(
        shaper: *const rb_ot_complex_shaper_t,
    ) -> Option<rb_ot_reorder_marks_func_t>;

    pub fn rb_ot_map_builder_add_feature(
        builder: *mut rb_ot_map_builder_t,
        tag: Tag,
        flags: rb_ot_map_feature_flags_t,
        value: u32,
    );

    pub fn rb_ot_map_builder_add_gsub_pause(
        builder: *mut rb_ot_map_builder_t,
        pause: rb_ot_pause_func_t,
    );

    pub fn rb_ot_layout_lookup_would_substitute(
        face: *mut rb_face_t,
        lookup_index: u32,
        glyphs: *const rb_codepoint_t,
        glyphs_length: u32,
        zero_context: rb_bool_t,
    ) -> rb_bool_t;

    pub fn rb_would_apply_context_get_len(ctx: *const rb_would_apply_context_t) -> u32;

    pub fn rb_would_apply_context_get_glyph(
        ctx: *const rb_would_apply_context_t,
        index: u32,
    ) -> rb_codepoint_t;

    pub fn rb_would_apply_context_get_zero_context(ctx: *const rb_would_apply_context_t) -> rb_bool_t;

    pub fn rb_ot_apply_context_get_buffer(ctx: *const rb_ot_apply_context_t) -> *mut rb_buffer_t;

    pub fn rb_ot_apply_context_get_direction(ctx: *const rb_ot_apply_context_t) -> rb_direction_t;

    pub fn rb_ot_apply_context_get_lookup_mask(ctx: *const rb_ot_apply_context_t) -> rb_mask_t;

    pub fn rb_ot_apply_context_get_table_index(ctx: *const rb_ot_apply_context_t) -> u32;

    pub fn rb_ot_apply_context_get_lookup_index(ctx: *const rb_ot_apply_context_t) -> u32;

    pub fn rb_ot_apply_context_get_lookup_props(ctx: *const rb_ot_apply_context_t) -> u32;

    pub fn rb_ot_apply_context_get_nesting_level_left(ctx: *const rb_ot_apply_context_t) -> u32;

    pub fn rb_ot_apply_context_get_has_glyph_classes(ctx: *const rb_ot_apply_context_t) -> rb_bool_t;

    pub fn rb_ot_apply_context_get_auto_zwnj(ctx: *const rb_ot_apply_context_t) -> rb_bool_t;

    pub fn rb_ot_apply_context_get_auto_zwj(ctx: *const rb_ot_apply_context_t) -> rb_bool_t;

    pub fn rb_ot_apply_context_get_random(ctx: *const rb_ot_apply_context_t) -> rb_bool_t;

    pub fn rb_ot_apply_context_gdef_mark_set_covers(
        ctx: *const rb_ot_apply_context_t,
        set_index: u32,
        glyph_id: rb_codepoint_t,
    ) -> rb_bool_t;

    pub fn rb_ot_apply_context_gdef_get_glyph_props(
        ctx: *const rb_ot_apply_context_t,
        glyph_id: rb_codepoint_t,
    ) -> u32;

    pub fn rb_ot_apply_context_random_number(ctx: *mut rb_ot_apply_context_t) -> u32;

    pub fn rb_ot_apply_context_recurse(
        ctx: *mut rb_ot_apply_context_t,
        sub_lookup_index: u32,
    ) -> rb_bool_t;

    pub fn rb_layout_clear_syllables(
        plan: *const rb_ot_shape_plan_t,
        font: *mut rb_font_t,
        buffer: *mut rb_buffer_t,
    );

    pub fn rb_clear_substitution_flags(
        plan: *const rb_ot_shape_plan_t,
        font: *mut rb_font_t,
        buffer: *mut rb_buffer_t,
    );

    pub fn rb_shape(
        font: *const rb_font_t,
        buffer: *mut rb_buffer_t,
        features: *const crate::Feature,
        num_features: u32,
    ) -> rb_bool_t;
}
