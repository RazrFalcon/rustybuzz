pub mod arabic;
pub mod hangul;
pub mod hebrew;
pub mod indic;
pub mod khmer;
pub mod myanmar;
pub mod thai;
pub mod universal;
mod arabic_table;
mod indic_machine;
mod indic_table;
mod khmer_machine;
mod myanmar_machine;
mod universal_machine;
mod universal_table;
mod vowel_constraints;


#[inline]
pub const fn rb_flag(x: u32) -> u32 {
    1 << x
}

#[inline]
pub fn rb_flag_unsafe(x: u32) -> u32 {
    if x < 32 { 1 << x } else { 0 }
}

#[inline]
pub fn rb_flag_range(x: u32, y: u32) -> u32 {
    (x < y) as u32 + rb_flag(y + 1) - rb_flag(x)
}

#[inline]
pub const fn rb_flag64(x: u32) -> u64 {
    1 << x as u64
}

#[inline]
pub fn rb_flag64_unsafe(x: u32) -> u64 {
    if x < 64 { 1 << (x as u64) } else { 0 }
}
