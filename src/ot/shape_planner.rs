use std::ptr::NonNull;

use super::MapBuilder;
use crate::{ffi, Direction, Script};

pub struct ShapePlanner {
    planner: NonNull<ffi::rb_ot_shape_planner_t>,
    pub map: &'static mut MapBuilder,
}

impl ShapePlanner {
    #[inline]
    pub fn from_ptr(planner: *const ffi::rb_ot_shape_planner_t) -> Self {
        Self::from_ptr_mut(planner as _)
    }

    #[inline]
    pub fn from_ptr_mut(planner: *mut ffi::rb_ot_shape_planner_t) -> Self {
        unsafe {
            ShapePlanner {
                planner: NonNull::new(planner).unwrap(),
                map: MapBuilder::from_ptr_mut(ffi::rb_ot_shape_planner_get_ot_map(planner)),
            }
        }
    }

    #[inline]
    pub fn script(&self) -> Script {
        unsafe { Script::from_raw(ffi::rb_ot_shape_planner_get_script(self.planner.as_ptr())) }
    }

    #[inline]
    pub fn direction(&self) -> Direction {
        unsafe { Direction::from_raw(ffi::rb_ot_shape_planner_get_direction(self.planner.as_ptr())) }
    }
}
