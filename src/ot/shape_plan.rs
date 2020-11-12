use std::os::raw::c_void;
use std::ptr::NonNull;

use crate::{ffi, Script, Direction};
use super::{Map, ComplexShaper};

pub struct ShapePlan {
    plan: NonNull<ffi::rb_ot_shape_plan_t>,
    pub map: Map,
    pub shaper: &'static ComplexShaper,
}

impl ShapePlan {
    #[inline]
    pub fn from_ptr(plan: *const ffi::rb_ot_shape_plan_t) -> Self {
        assert!(!plan.is_null());
        unsafe {
            ShapePlan {
                plan: NonNull::new(plan as _).unwrap(),
                map: Map::from_ptr(ffi::rb_ot_shape_plan_get_ot_map(plan)),
                shaper: ComplexShaper::from_ptr(ffi::rb_ot_shape_plan_get_ot_complex_shaper(plan)),
            }
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const ffi::rb_ot_shape_plan_t {
        self.plan.as_ptr()
    }

    #[inline]
    pub fn data(&self) -> *const c_void {
        unsafe {
            ffi::rb_ot_shape_plan_get_data(self.as_ptr())
        }
    }

    #[inline]
    pub fn script(&self) -> Script {
        unsafe {
            Script::from_raw(ffi::rb_ot_shape_plan_get_script(self.as_ptr()))
        }
    }

    #[inline]
    pub fn direction(&self) -> Direction {
        unsafe {
            Direction::from_raw(ffi::rb_ot_shape_plan_get_direction(self.as_ptr()))
        }
    }

    #[inline]
    pub fn has_gpos_mark(&self) -> bool {
        unsafe {
            ffi::rb_ot_shape_plan_has_gpos_mark(self.as_ptr())
        }
    }
}
