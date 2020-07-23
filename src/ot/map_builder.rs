use std::ptr::NonNull;

use crate::{ffi, Tag};

bitflags::bitflags! {
    /// Flags used for serialization with a `BufferSerializer`.
    #[derive(Default)]
    pub struct FeatureFlags: u32 {
        const NONE                      = 0x00;

        /// Feature applies to all characters; results in no mask allocated for it.
        const GLOBAL                    = 0x01;

        /// Has fallback implementation, so include mask bit even if feature not found.
        const HAS_FALLBACK              = 0x02;

        /// Don't skip over ZWNJ when matching **context**.
        const MANUAL_ZWNJ               = 0x04;

        /// Don't skip over ZWJ when matching **input**.
        const MANUAL_ZWJ                = 0x08;

        const MANUAL_JOINERS            = Self::MANUAL_ZWNJ.bits | Self::MANUAL_ZWJ.bits;
        const GLOBAL_MANUAL_JOINERS     = Self::GLOBAL.bits | Self::MANUAL_JOINERS.bits;

        /// If feature not found in LangSys, look for it in global feature list and pick one.
        const GLOBAL_SEARCH             = 0x10;

        /// Randomly select a glyph from an AlternateSubstFormat1 subtable.
        const RANDOM                    = 0x20;
    }
}


pub struct MapBuilder(NonNull<ffi::rb_ot_map_builder_t>);

impl MapBuilder {
    #[inline]
    pub fn from_ptr_mut(ptr: *mut ffi::rb_ot_map_builder_t) -> Self {
        MapBuilder(NonNull::new(ptr).unwrap())
    }

    #[inline]
    pub fn add_feature(&mut self, tag: Tag, flags: FeatureFlags, value: u32) {
        unsafe { ffi::rb_ot_map_builder_add_feature(self.0.as_ptr(), tag, flags.bits, value) };
    }

    #[inline]
    pub fn enable_feature(&mut self, tag: Tag, flags: FeatureFlags, value: u32) {
        self.add_feature(tag, flags | FeatureFlags::GLOBAL, value);
    }

    #[inline]
    pub fn disable_feature(&mut self, tag: Tag) {
        self.add_feature(tag, FeatureFlags::GLOBAL, 0);
    }

    #[inline]
    pub fn add_gsub_pause(&mut self, pause: ffi::rb_ot_pause_func_t) {
        unsafe { ffi::rb_ot_map_builder_add_gsub_pause(self.0.as_ptr(), pause) }
    }
}
