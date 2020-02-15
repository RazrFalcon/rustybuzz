use std::os::raw::c_void;

mod arabic;
mod arabic_table;
mod hangul;
mod hebrew;
mod indic;
mod indic_machine;
mod indic_table;
mod khmer;
mod khmer_machine;
mod myanmar;
mod myanmar_machine;
mod thai;
mod vowel_constraints;
mod universal;
mod universal_machine;
mod universal_table;

use crate::{Tag, CodePoint};
use crate::ffi;
use crate::map::{Map, MapLookup, get_stage_lookups};

#[inline]
pub const fn hb_flag(x: u32) -> u32 {
    1 << x
}

#[inline]
pub fn hb_flag_unsafe(x: u32) -> u32 {
    if x < 32 { 1 << x } else { 0 }
}

#[inline]
pub const fn hb_flag64(x: u32) -> u64 {
    1 << x as u64
}

#[inline]
pub fn hb_flag64_unsafe(x: u32) -> u64 {
    if x < 64 { 1 << (x as u64) } else { 0 }
}

#[inline]
pub fn hb_flag_range(x: u32, y: u32) -> u32 {
    (x < y) as u32 + hb_flag(y + 1) - hb_flag(x)
}

#[no_mangle]
pub extern "C" fn rb_preprocess_text_vowel_constraints(buffer: *mut ffi::rb_buffer_t) {
    let buffer = crate::Buffer::from_ptr_mut(buffer);
    vowel_constraints::preprocess_text_vowel_constraints(buffer);
}

extern "C" {
    pub fn hb_layout_clear_syllables(
        plan: *const ffi::hb_shape_plan_t,
        font: *mut ffi::hb_font_t,
        buffer: *mut ffi::rb_buffer_t,
    );
}

pub struct WouldSubstituteFeature {
    pub lookups: Vec<MapLookup>,
    pub zero_context: bool,
}

impl WouldSubstituteFeature {
    pub fn new(map: &Map, feature_tag: Tag, zero_context: bool) -> Self {
        WouldSubstituteFeature {
            lookups: get_stage_lookups(map, 0, map.feature_stage(0, feature_tag)),
            zero_context
        }
    }

    pub fn would_substitute(&self, glyphs: &[CodePoint], face: *const ffi::hb_face_t) -> bool {
        for lookup in &self.lookups {
            unsafe {
                let ok = ffi::hb_ot_layout_lookup_would_substitute(
                    face,
                    lookup.index.0 as u32,
                    glyphs.as_ptr() as *const _,
                    glyphs.len() as u32,
                    self.zero_context as i32,
                );

                if ok != 0 {
                    return true;
                }
            }
        }

        false
    }
}


#[no_mangle]
pub extern "C" fn rb_create_default_shaper() -> *const ffi::hb_ot_complex_shaper_t {
    let shaper = Box::new(ffi::hb_ot_complex_shaper_t {
        collect_features: None,
            override_features: None,
            data_create: None,
            data_destroy: None,
            preprocess_text: None,
            postprocess_glyphs: None,
            normalization_preference: ffi::HB_OT_SHAPE_NORMALIZATION_MODE_DEFAULT,
            decompose: None,
            compose: None,
            setup_masks: None,
            gpos_tag: 0,
            reorder_marks: None,
            zero_width_marks: ffi::HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE,
            fallback_position: true,
    });
    Box::into_raw(shaper)
}

#[no_mangle]
pub extern "C" fn rb_shaper_destroy(shaper: *mut ffi::hb_ot_complex_shaper_t) {
    unsafe { Box::from_raw(shaper as *mut ffi::hb_ot_complex_shaper_t) };
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_shaper_collect_features(
    shaper: *const ffi::hb_ot_complex_shaper_t,
    planner: *mut ffi::hb_ot_shape_planner_t,
) {
    unsafe {
        if let Some(f) = (*shaper).collect_features {
            f(planner)
        }
    }
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_shaper_override_features(
    shaper: *const ffi::hb_ot_complex_shaper_t,
    planner: *mut ffi::hb_ot_shape_planner_t,
) {
    unsafe {
        if let Some(f) = (*shaper).override_features {
            f(planner)
        }
    }
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_shaper_data_create(
    shaper: *const ffi::hb_ot_complex_shaper_t,
    plan: *mut ffi::hb_shape_plan_t,
) -> *mut c_void {
    unsafe {
        if let Some(f) = (*shaper).data_create {
            f(plan)
        } else {
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_shaper_data_destroy(
    shaper: *const ffi::hb_ot_complex_shaper_t,
    data: *mut c_void,
) {
    unsafe {
        if let Some(f) = (*shaper).data_destroy {
            f(data)
        }
    }
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_shaper_preprocess_text(
    shaper: *const ffi::hb_ot_complex_shaper_t,
    plan: *const ffi::hb_shape_plan_t,
    buffer: *mut ffi::rb_buffer_t,
    font: *mut ffi::hb_font_t,
) {
    unsafe {
        if let Some(f) = (*shaper).preprocess_text {
            f(plan, buffer, font)
        }
    }
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_shaper_postprocess_glyphs(
    shaper: *const ffi::hb_ot_complex_shaper_t,
    plan: *const ffi::hb_shape_plan_t,
    buffer: *mut ffi::rb_buffer_t,
    font: *mut ffi::hb_font_t,
) {
    unsafe {
        if let Some(f) = (*shaper).postprocess_glyphs {
            f(plan, buffer, font)
        }
    }
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_shaper_normalization_preference(
    shaper: *const ffi::hb_ot_complex_shaper_t,
) -> ffi::hb_ot_shape_normalization_mode_t {
    unsafe { (*shaper).normalization_preference }
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_shaper_decompose(
    shaper: *const ffi::hb_ot_complex_shaper_t,
    c: *const ffi::hb_ot_shape_normalize_context_t,
    ab: ffi::hb_codepoint_t,
    a: *mut ffi::hb_codepoint_t,
    b: *mut ffi::hb_codepoint_t,
) -> bool {
    unsafe {
        if let Some(f) = (*shaper).decompose {
            f(c, ab, a, b)
        } else {
            crate::unicode::rb_ucd_decompose(ab, a, b) != 0
        }
    }
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_shaper_compose(
    shaper: *const ffi::hb_ot_complex_shaper_t,
    c: *const ffi::hb_ot_shape_normalize_context_t,
    a: ffi::hb_codepoint_t,
    b: ffi::hb_codepoint_t,
    ab: *mut ffi::hb_codepoint_t,
) -> bool {
    unsafe {
        if let Some(f) = (*shaper).compose {
            f(c, a, b, ab)
        } else {
            crate::unicode::rb_ucd_compose(a, b, ab) != 0
        }
    }
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_shaper_setup_masks(
    shaper: *const ffi::hb_ot_complex_shaper_t,
    plan: *const ffi::hb_shape_plan_t,
    buffer: *mut ffi::rb_buffer_t,
    font: *mut ffi::hb_font_t,
) {
    unsafe {
        if let Some(f) = (*shaper).setup_masks {
            f(plan, buffer, font)
        }
    }
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_shaper_gpos_tag(
    shaper: *const ffi::hb_ot_complex_shaper_t,
) -> ffi::hb_tag_t {
    unsafe { (*shaper).gpos_tag }
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_shaper_reorder_marks(
    shaper: *const ffi::hb_ot_complex_shaper_t,
    plan: *const ffi::hb_shape_plan_t,
    buffer: *mut ffi::rb_buffer_t,
    start: u32,
    end: u32,
) {
    unsafe {
        if let Some(f) = (*shaper).reorder_marks {
            f(plan, buffer, start, end)
        }
    }
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_shaper_zero_width_marks(
    shaper: *const ffi::hb_ot_complex_shaper_t,
) -> ffi::hb_ot_shape_zero_width_marks_type_t {
    unsafe { (*shaper).zero_width_marks }
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_shaper_fallback_position(
    shaper: *const ffi::hb_ot_complex_shaper_t,
) -> bool {
    unsafe { (*shaper).fallback_position }
}
