use std::ptr::NonNull;

use crate::{ffi, Tag, Mask};

pub struct Map(NonNull<ffi::hb_ot_map_t>);

impl Map {
    pub fn from_ptr(ptr: *const ffi::hb_ot_map_t) -> Self {
        Map(NonNull::new(ptr as _).unwrap())
    }

    pub fn get_1_mask(&self, feature_tag: Tag) -> Mask {
        unsafe { ffi::hb_ot_map_get_1_mask(self.0.as_ptr(), feature_tag) }
    }

    pub fn found_script(&self, index: usize) -> bool {
        unsafe { ffi::hb_ot_map_get_found_script(self.0.as_ptr(), index as u32) }
    }
}
