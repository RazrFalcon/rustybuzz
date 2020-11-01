use std::ptr::NonNull;

use crate::{ffi, Tag, Mask};
use super::TableIndex;


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MapLookup {
    pub index: u16,
    // TODO: to bitflags
    auto_zwnj: bool,
    auto_zwj: bool,
    random: bool,
    mask: Mask,
}


pub struct Map(NonNull<ffi::rb_ot_map_t>);

impl Map {
    pub const MAX_BITS: u32 = 8;
    pub const MAX_VALUE: u32 = (1 << Self::MAX_BITS) - 1;

    #[inline]
    pub fn from_ptr(ptr: *const ffi::rb_ot_map_t) -> Self {
        Map(NonNull::new(ptr as _).unwrap())
    }

    #[inline]
    pub fn as_ptr(&self) -> *const ffi::rb_ot_map_t {
        self.0.as_ptr()
    }

    #[inline]
    pub fn global_mask(&self) -> Mask {
        unsafe { ffi::rb_ot_map_global_mask(self.0.as_ptr()) }
    }

    #[inline]
    pub fn get_1_mask(&self, feature_tag: Tag) -> Mask {
        unsafe { ffi::rb_ot_map_get_1_mask(self.0.as_ptr(), feature_tag) }
    }

    #[inline]
    pub fn found_script(&self, table_index: TableIndex) -> bool {
        unsafe { ffi::rb_ot_map_get_found_script(self.0.as_ptr(), table_index as u32) }
    }

    #[inline]
    pub fn chosen_script(&self, table_index: TableIndex) -> Tag {
        unsafe { ffi::rb_ot_map_get_chosen_script(self.0.as_ptr(), table_index as u32) }
    }

    pub fn feature_stage(&self, table_index: TableIndex, feature_tag: Tag) -> usize {
        unsafe {
            ffi::rb_ot_map_get_feature_stage(self.as_ptr(), table_index as u32, feature_tag) as usize
        }
    }

    pub fn collect_stage_lookups(
        &self,
        table_index: TableIndex,
        stage: usize,
    ) -> &'static [ffi::rb_ot_map_lookup_map_t] {
        unsafe {
            let mut plookups: *const ffi::rb_ot_map_lookup_map_t = std::ptr::null();
            let mut lookup_count: u32 = 0;

            ffi::rb_ot_map_get_stage_lookups(
                self.as_ptr(),
                table_index as u32,
                stage as u32,
                &mut plookups as *mut _,
                &mut lookup_count as *mut _,
            );

            if plookups.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(plookups, lookup_count as usize)
            }
        }
    }
}
