use std::ptr::NonNull;

use crate::{ffi, ot, Script};

pub struct ShapePlanner {
    planner: NonNull<ffi::hb_ot_shape_planner_t>,
    pub ot_map: ot::MapBuilder,
}

impl ShapePlanner {
    #[inline]
    pub fn from_ptr_mut(ptr: *mut ffi::hb_ot_shape_planner_t) -> Self {
        unsafe {
            ShapePlanner {
                planner: NonNull::new(ptr).unwrap(),
                ot_map: ot::MapBuilder::from_ptr_mut(ffi::hb_ot_shape_planner_get_ot_map(ptr)),
            }
        }
    }

    #[inline]
    pub fn script(&self) -> Script {
        unsafe {
            Script::from_raw(ffi::hb_ot_shape_planner_get_script(self.planner.as_ptr()))
        }
    }
}
