mod map;

pub use map::*;

use crate::{ffi, Face};
use crate::buffer::Buffer;
use crate::plan::ShapePlan;

pub fn has_substitution(face: &Face) -> bool {
    unsafe { ffi::rb_aat_layout_has_substitution(face.as_ptr()) != 0 }
}

pub fn has_positioning(face: &Face) -> bool {
    unsafe { ffi::rb_aat_layout_has_positioning(face.as_ptr()) != 0 }
}

pub fn has_tracking(face: &Face) -> bool {
    unsafe { ffi::rb_aat_layout_has_tracking(face.as_ptr()) != 0 }
}

pub fn substitute(plan: &ShapePlan, face: &Face, buffer: &mut Buffer) {
    unsafe { ffi::rb_aat_layout_substitute(plan.as_ptr(), face.as_ptr(), buffer.as_ptr()); }
}

pub fn position(plan: &ShapePlan, face: &Face, buffer: &mut Buffer) {
    unsafe { ffi::rb_aat_layout_position(plan.as_ptr(), face.as_ptr(), buffer.as_ptr()); }
}

pub fn track(plan: &ShapePlan, face: &Face, buffer: &mut Buffer) {
    unsafe { ffi::rb_aat_layout_track(plan.as_ptr(), face.as_ptr(), buffer.as_ptr()); }
}

pub fn zero_width_deleted_glyphs(buffer: &mut Buffer) {
    unsafe { ffi::rb_aat_layout_zero_width_deleted_glyphs(buffer.as_ptr()); }
}

pub fn remove_deleted_glyphs(buffer: &mut Buffer) {
    unsafe { ffi::rb_aat_layout_remove_deleted_glyphs(buffer.as_ptr()); }
}
