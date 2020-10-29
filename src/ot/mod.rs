#![allow(dead_code)]

macro_rules! make_ffi_funcs {
    ($table:ident, $would_apply:ident, $apply:ident) => {
        #[no_mangle]
        pub extern "C" fn $would_apply(
            ctx: *const crate::ffi::rb_would_apply_context_t,
            data_ptr: *const u8,
            data_len: u32,
        ) -> crate::ffi::rb_bool_t {
            let ctx = WouldApplyContext::from_ptr(ctx);
            let data = unsafe { std::slice::from_raw_parts(data_ptr, data_len as usize) };
            match $table::parse(data) {
                Some(table) => table.would_apply(&ctx) as crate::ffi::rb_bool_t,
                None => 0,
            }
        }

        #[no_mangle]
        pub extern "C" fn $apply(
            ctx: *mut crate::ffi::rb_ot_apply_context_t,
            data_ptr: *const u8,
            data_len: u32,
        ) -> crate::ffi::rb_bool_t {
            let mut ctx = ApplyContext::from_ptr_mut(ctx);
            let data = unsafe { std::slice::from_raw_parts(data_ptr, data_len as usize) };
            match $table::parse(data) {
                Some(table) => table.apply(&mut ctx).is_some() as crate::ffi::rb_bool_t,
                None => 0,
            }
        }
    }
}

pub mod feature;
mod fallback;
mod map;
mod map_builder;
mod normalize;
mod shape_plan;
mod shape_planner;
mod complex_shaper;
mod ggg;
mod gsub;
mod layout;
mod matching;

pub use map::*;
pub use map_builder::*;
pub use normalize::*;
pub use shape_plan::*;
pub use shape_planner::*;
pub use complex_shaper::*;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TableIndex {
    GSUB = 0,
    GPOS = 1,
}
