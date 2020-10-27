use std::ptr::NonNull;

use ttf_parser::GlyphId;

use crate::{ffi, Mask};
use crate::buffer::{Buffer, GlyphPropsFlags, GlyphInfo};
use super::ggg::LookupFlags;

pub const MAX_CONTEXT_LENGTH: usize = 64;

pub struct WouldApplyContext(NonNull<ffi::rb_would_apply_context_t>);

impl WouldApplyContext {
    pub fn from_ptr(ptr: *const ffi::rb_would_apply_context_t) -> Self {
        Self(NonNull::new(ptr as _).unwrap())
    }

    pub fn len(&self) -> usize {
        unsafe { ffi::rb_would_apply_context_get_len(self.0.as_ptr()) as usize }
    }

    pub fn glyph(&self, index: usize) -> u32 {
        unsafe { ffi::rb_would_apply_context_get_glyph(self.0.as_ptr(), index as u32) }
    }

    pub fn zero_context(&self) -> bool {
        unsafe { ffi::rb_would_apply_context_get_zero_context(self.0.as_ptr()) != 0 }
    }
}

pub struct ApplyContext(NonNull<ffi::rb_ot_apply_context_t>);

impl ApplyContext {
    pub fn from_ptr_mut(ptr: *mut ffi::rb_ot_apply_context_t) -> Self {
        Self(NonNull::new(ptr).unwrap())
    }

    pub(crate) fn buffer(&self) -> &Buffer {
        unsafe { Buffer::from_ptr(ffi::rb_ot_apply_context_get_buffer(self.0.as_ptr())) }
    }

    pub(crate) fn buffer_mut(&mut self) -> &mut Buffer {
        unsafe { Buffer::from_ptr_mut(ffi::rb_ot_apply_context_get_buffer(self.0.as_ptr())) }
    }

    pub fn lookup_mask(&self) -> Mask {
        unsafe { ffi::rb_ot_apply_context_get_lookup_mask(self.0.as_ptr()) }
    }

    pub fn table_index(&self) -> u32 {
        unsafe { ffi::rb_ot_apply_context_get_table_index(self.0.as_ptr()) }
    }

    pub fn auto_zwnj(&self) -> bool {
        unsafe { ffi::rb_ot_apply_context_get_auto_zwnj(self.0.as_ptr()) != 0 }
    }

    pub fn auto_zwj(&self) -> bool {
        unsafe { ffi::rb_ot_apply_context_get_auto_zwj(self.0.as_ptr()) != 0 }
    }

    pub fn random(&self) -> bool {
        unsafe { ffi::rb_ot_apply_context_get_random(self.0.as_ptr()) != 0 }
    }

    pub fn random_number(&self) -> u32 {
        unsafe { ffi::rb_ot_apply_context_random_number(self.0.as_ptr()) }
    }

    pub fn check_glyph_property(&self, info: &GlyphInfo, match_props: u16) -> bool {
        let glyph_props = info.glyph_props();

        // Not covered, if, for example, glyph class is ligature and
        // match_props includes LookupFlags::IgnoreLigatures
        if glyph_props & match_props & LookupFlags::IGNORE_FLAGS.bits() != 0 {
            return false;
        }

        if glyph_props & GlyphPropsFlags::MARK.bits() != 0 {
            // If using mark filtering sets, the high short of
            // match_props has the set index.
            if match_props & LookupFlags::USE_MARK_FILTERING_SET.bits() != 0 {
                return unsafe {
                    ffi::rb_ot_apply_context_gdef_mark_set_covers(
                        self.0.as_ptr(),
                        (match_props >> 16) as u32,
                        info.codepoint,
                    ) != 0
                };
            }

            // The second byte of match_props has the meaning
            // "ignore marks of attachment type different than
            // the attachment type specified."
            if match_props & LookupFlags::MARK_ATTACHMENT_TYPE.bits() != 0 {
                return (match_props & LookupFlags::MARK_ATTACHMENT_TYPE.bits())
                    == (glyph_props & LookupFlags::MARK_ATTACHMENT_TYPE.bits());
            }
        }

        true
    }

    pub fn replace_glyph(&mut self, glyph_id: GlyphId) {
        unsafe { ffi::rb_ot_apply_context_replace_glyph(self.0.as_ptr(), glyph_id.0 as u32); }
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
