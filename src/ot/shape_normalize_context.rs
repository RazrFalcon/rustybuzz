use std::ptr::NonNull;

use crate::{ffi, ot};

pub struct ShapeNormalizeContext {
    #[allow(dead_code)]
    ptr: NonNull<ffi::hb_ot_shape_normalize_context_t>,
    pub plan: ot::ShapePlan,
}

impl ShapeNormalizeContext {
    pub fn from_ptr(ptr: *const ffi::hb_ot_shape_normalize_context_t) -> Self {
        unsafe {
            ShapeNormalizeContext {
                ptr: NonNull::new(ptr as _).unwrap(),
                plan: ot::ShapePlan::from_ptr(ffi::hb_ot_shape_normalize_context_get_plan(ptr)),
            }
        }
    }
}
