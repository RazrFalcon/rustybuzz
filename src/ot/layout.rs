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

    pub fn table_index(&self) -> usize {
        unsafe { ffi::rb_ot_apply_context_get_table_index(self.0.as_ptr()) as usize }
    }

    pub fn lookup_index(&self) -> usize {
        unsafe { ffi::rb_ot_apply_context_get_lookup_index(self.0.as_ptr()) as usize }
    }

    pub fn lookup_props(&self) -> u32 {
        unsafe { ffi::rb_ot_apply_context_get_lookup_props(self.0.as_ptr()) }
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

    pub fn recurse(&self, sub_lookup_index: usize) -> bool {
        unsafe { ffi::rb_ot_apply_context_recurse(self.0.as_ptr(), sub_lookup_index as u32) != 0 }
    }

    pub fn check_glyph_property(&self, info: &GlyphInfo, match_props: u32) -> bool {
        let glyph_props = info.glyph_props();
        let lookup_flags = match_props as u16;

        // Not covered, if, for example, glyph class is ligature and
        // match_props includes LookupFlags::IgnoreLigatures
        if glyph_props & lookup_flags & LookupFlags::IGNORE_FLAGS.bits() != 0 {
            return false;
        }

        if glyph_props & GlyphPropsFlags::MARK.bits() != 0 {
            // If using mark filtering sets, the high short of
            // match_props has the set index.
            if lookup_flags & LookupFlags::USE_MARK_FILTERING_SET.bits() != 0 {
                let set_index = match_props >> 16;
                let glyph = info.codepoint;
                return unsafe {
                    ffi::rb_ot_apply_context_gdef_mark_set_covers(
                        self.0.as_ptr(),
                        set_index,
                        glyph,
                    ) != 0
                };
            }

            // The second byte of match_props has the meaning
            // "ignore marks of attachment type different than
            // the attachment type specified."
            if lookup_flags & LookupFlags::MARK_ATTACHMENT_TYPE.bits() != 0 {
                return (lookup_flags & LookupFlags::MARK_ATTACHMENT_TYPE.bits())
                    == (glyph_props & LookupFlags::MARK_ATTACHMENT_TYPE.bits());
            }
        }

        true
    }

    fn set_glyph_class(
        &mut self,
        glyph_id: GlyphId,
        class_guess: GlyphPropsFlags,
        ligature: bool,
        component: bool,
    ) {
        let ptr = self.0.as_ptr();
        let cur = self.buffer_mut().cur_mut(0);
        let mut props = cur.glyph_props();

        props |= GlyphPropsFlags::SUBSTITUTED.bits();

        if ligature {
            // In the only place that the MULTIPLIED bit is used, Uniscribe
            // seems to only care about the "last" transformation between
            // Ligature and Multiple substitutions.  Ie. if you ligate, expand,
            // and ligate again, it forgives the multiplication and acts as
            // if only ligation happened.  As such, clear MULTIPLIED bit.
            props &= !GlyphPropsFlags::MULTIPLIED.bits();
            props |= GlyphPropsFlags::LIGATED.bits();
        }

        if component {
            props |= GlyphPropsFlags::MULTIPLIED.bits();
        }

        if unsafe { ffi::rb_ot_apply_context_get_has_glyph_classes(ptr) != 0 } {
            props = (props & !GlyphPropsFlags::MARK.bits()) | unsafe {
                ffi::rb_ot_apply_context_gdef_get_glyph_props(ptr, glyph_id.0 as u32) as u16
            };
        } else if !class_guess.is_empty() {
            props = (props & !GlyphPropsFlags::MARK.bits()) | class_guess.bits();
        }

        cur.set_glyph_props(props);
    }

    pub fn replace_glyph(&mut self, glyph_id: GlyphId) {
        self.set_glyph_class(glyph_id, GlyphPropsFlags::empty(), false, false);
        self.buffer_mut().replace_glyph(glyph_id.0 as u32);
    }

    pub fn replace_glyph_inplace(&mut self, glyph_id: GlyphId) {
        self.set_glyph_class(glyph_id, GlyphPropsFlags::empty(), false, false);
        self.buffer_mut().cur_mut(0).codepoint = glyph_id.0 as u32;
    }

    pub fn replace_glyph_with_ligature(&mut self, glyph_id: GlyphId, class_guess: GlyphPropsFlags) {
        self.set_glyph_class(glyph_id, class_guess, true, false);
        self.buffer_mut().replace_glyph(glyph_id.0 as u32);
    }

    pub fn output_glyph_for_component(&mut self, glyph_id: GlyphId, class_guess: GlyphPropsFlags) {
        self.set_glyph_class(glyph_id, class_guess, false, true);
        self.buffer_mut().output_glyph(glyph_id.0 as u32);
    }
}
