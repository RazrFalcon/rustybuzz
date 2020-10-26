use std::ptr::NonNull;

use ttf_parser::GlyphId;

use crate::ffi;
use crate::buffer::{Buffer, GlyphPropsFlags};

pub struct WouldApplyContext(NonNull<ffi::rb_would_apply_context_t>);

impl WouldApplyContext {
    pub fn from_ptr(ptr: *const ffi::rb_would_apply_context_t) -> Self {
        Self(NonNull::new(ptr as _).unwrap())
    }

    pub fn len(&self) -> usize {
        unsafe {
            ffi::rb_would_apply_context_get_len(self.0.as_ptr()) as usize
        }
    }

    pub fn glyph(&self, index: usize) -> u32 {
        unsafe {
            ffi::rb_would_apply_context_get_glyph(self.0.as_ptr(), index as u32)
        }
    }

    pub fn zero_context(&self) -> bool {
        unsafe {
            ffi::rb_would_apply_context_get_zero_context(self.0.as_ptr()) != 0
        }
    }
}

pub struct ApplyContext(NonNull<ffi::rb_ot_apply_context_t>);

impl ApplyContext {
    pub fn from_ptr_mut(ptr: *mut ffi::rb_ot_apply_context_t) -> Self {
        Self(NonNull::new(ptr).unwrap())
    }

    pub(crate) fn buffer(&mut self) -> &mut Buffer {
        unsafe {
            Buffer::from_ptr_mut(ffi::rb_ot_apply_context_get_buffer(self.0.as_ptr()))
        }
    }

    pub fn replace_glyph(&mut self, glyph_id: GlyphId) {
        unsafe {
            ffi::rb_ot_apply_context_replace_glyph(self.0.as_ptr(), glyph_id.0 as u32);
        }
    }

    pub fn output_glyph_for_component(&mut self, glyph_id: GlyphId, class_guess: GlyphPropsFlags) {
        unsafe {
            ffi::rb_ot_apply_context_output_glyph_for_component(
                self.0.as_ptr(),
                glyph_id.0 as u32,
                class_guess.bits() as u32,
            );
        }
    }
}
