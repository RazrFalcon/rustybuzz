use std::ptr::NonNull;

use crate::{ffi, ot};

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
    pub fn normalization_preference(&self) -> ot::ShapeNormalizationMode {
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
