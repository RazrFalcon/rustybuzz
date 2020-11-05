//! OpenType layout.

#![allow(dead_code)]

macro_rules! make_ffi_funcs {
    ($table:ident, $apply:ident $(, $would_apply:ident)?) => {
        #[no_mangle]
        pub extern "C" fn $apply(
            data: *const u8,
            ctx: *mut crate::ffi::rb_ot_apply_context_t,
        ) -> crate::ffi::rb_bool_t {
            let data = unsafe { std::slice::from_raw_parts(data, isize::MAX as usize) };
            let mut ctx = ApplyContext::from_ptr_mut(ctx);
            $table::parse(data)
                .map(|table| table.apply(&mut ctx).is_some())
                .unwrap_or(false) as crate::ffi::rb_bool_t
        }

        $(
            #[no_mangle]
            pub extern "C" fn $would_apply(
                data: *const u8,
                ctx: *const crate::ffi::rb_would_apply_context_t,
            ) -> crate::ffi::rb_bool_t {
                let data = unsafe { std::slice::from_raw_parts(data, isize::MAX as usize) };
                let ctx = WouldApplyContext::from_ptr(ctx);
                $table::parse(data)
                    .map(|table| table.would_apply(&ctx))
                    .unwrap_or(false) as crate::ffi::rb_bool_t
            }
        )?
    }
}

mod apply;
mod common;
mod context_lookups;
mod dyn_array;
mod gpos;
mod gsub;
mod matching;

pub const MAX_NESTING_LEVEL: usize = 6;
pub const MAX_CONTEXT_LENGTH: usize = 64;
