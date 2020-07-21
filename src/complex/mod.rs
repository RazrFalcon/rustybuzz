mod arabic;
mod arabic_table;
mod hangul;
mod hebrew;

#[inline]
pub const fn hb_flag(x: u32) -> u32 {
    1 << x
}

#[inline]
pub fn hb_flag_unsafe(x: u32) -> u32 {
    if x < 32 { 1 << x } else { 0 }
}
