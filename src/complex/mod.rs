mod arabic;
mod arabic_table;
mod hangul;
mod hebrew;
mod indic;
mod indic_table;
mod indic_machine;
mod vowel_constraints;

use crate::ffi;

#[inline]
pub const fn hb_flag(x: u32) -> u32 {
    1 << x
}

#[inline]
pub fn hb_flag_unsafe(x: u32) -> u32 {
    if x < 32 { 1 << x } else { 0 }
}

#[inline]
pub fn hb_flag_range(x: u32, y: u32) -> u32 {
    (x < y) as u32 + hb_flag(y + 1) - hb_flag(x)
}

#[no_mangle]
pub extern "C" fn rb_preprocess_text_vowel_constraints(buffer: *mut ffi::rb_buffer_t) {
    let buffer = crate::Buffer::from_ptr_mut(buffer);
    vowel_constraints::preprocess_text_vowel_constraints(buffer);
}
