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

    pub fn would_substitute(&self, glyphs: &[CodePoint], face: *mut ffi::hb_face_t) -> bool {
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
