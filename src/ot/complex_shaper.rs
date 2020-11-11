use std::ptr::NonNull;

use crate::ffi;
use crate::buffer::Buffer;
use crate::Face;
use super::{ShapeNormalizationMode, ShapePlan};

pub const MAX_COMBINING_MARKS: usize = 32;

pub struct ComplexShaper(NonNull<ffi::rb_ot_complex_shaper_t>);

impl ComplexShaper {
    #[inline]
    pub fn from_ptr(ptr: *const ffi::rb_ot_complex_shaper_t) -> Self {
        Self(NonNull::new(ptr as _).unwrap())
    }

    #[inline]
    pub fn as_ptr(&self) -> *const ffi::rb_ot_complex_shaper_t {
        self.0.as_ptr()
    }

    #[inline]
    pub fn normalization_preference(&self) -> ShapeNormalizationMode {
        unsafe {
            let n = ffi::rb_ot_complex_shaper_get_normalization_preference(self.as_ptr());
            std::mem::transmute(n as u8)
        }
    }

    #[inline]
    pub fn get_decompose(&self) -> Option<ffi::rb_ot_decompose_func_t> {
        unsafe { ffi::rb_ot_complex_shaper_get_decompose(self.as_ptr()) }
    }

    #[inline]
    pub fn get_compose(&self) -> Option<ffi::rb_ot_compose_func_t> {
        unsafe { ffi::rb_ot_complex_shaper_get_compose(self.as_ptr()) }
    }

    #[inline]
    pub fn get_reorder_marks(&self) -> Option<ffi::rb_ot_reorder_marks_func_t> {
        unsafe { ffi::rb_ot_complex_shaper_get_reorder_marks(self.as_ptr()) }
    }
}

#[no_mangle]
pub extern "C" fn rb_clear_substitution_flags(
    plan: *const ffi::rb_ot_shape_plan_t,
    face: *const ffi::rb_face_t,
    buffer: *mut ffi::rb_buffer_t,
) {
    let plan = ShapePlan::from_ptr(plan);
    let face = Face::from_ptr(face);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    clear_substitution_flags(&plan, face, &mut buffer);
}

fn clear_substitution_flags(_: &ShapePlan, _: &Face, buffer: &mut Buffer) {
    let len = buffer.len;
    for info in &mut buffer.info[..len] {
        info.clear_substituted();
    }
}

#[no_mangle]
pub extern "C" fn rb_clear_syllables(
    plan: *const ffi::rb_ot_shape_plan_t,
    face: *const ffi::rb_face_t,
    buffer: *mut ffi::rb_buffer_t,
) {
    let plan = ShapePlan::from_ptr(plan);
    let face = Face::from_ptr(face);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    clear_syllables(&plan, face, &mut buffer);
}

fn clear_syllables(_: &ShapePlan, _: &Face, buffer: &mut Buffer) {
    let len = buffer.len;
    for info in &mut buffer.info[..len] {
        info.set_syllable(0);
    }
}
