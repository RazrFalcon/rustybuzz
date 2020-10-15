use std::os::raw::c_void;
use std::ptr::NonNull;

use crate::{ffi, ot, Script, Direction};

pub struct ShapePlan {
    #[allow(dead_code)]
    plan: NonNull<ffi::rb_ot_shape_plan_t>,
    pub ot_map: ot::Map,
}

impl ShapePlan {
    #[inline]
    pub fn from_ptr(ptr: *const ffi::rb_ot_shape_plan_t) -> Self {
        assert!(!ptr.is_null());
        unsafe {
            ShapePlan {
                plan: NonNull::new(ptr as _).unwrap(),
                ot_map: ot::Map::from_ptr(ffi::rb_ot_shape_plan_get_ot_map(ptr)),
            }
        }
    }

    #[inline]
    pub fn data(&self) -> *const c_void {
        unsafe {
            ffi::rb_ot_shape_plan_get_data(self.plan.as_ptr())
        }
    }

    #[inline]
    pub fn script(&self) -> Script {
        unsafe {
            Script::from_raw(ffi::rb_ot_shape_plan_get_script(self.plan.as_ptr()))
        }
    }

    #[inline]
    pub fn direction(&self) -> Direction {
        unsafe {
            Direction::from_raw(ffi::rb_ot_shape_plan_get_direction(self.plan.as_ptr()))
        }
    }

    #[inline]
    pub fn has_gpos_mark(&self) -> bool {
        unsafe {
            ffi::rb_ot_shape_plan_has_gpos_mark(self.plan.as_ptr())
        }
    }
}
