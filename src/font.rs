use std::str::FromStr;

use crate::ffi;
use crate::{Face, Tag};


/// A type representing a single font (i.e. a specific combination of typeface
/// and typesize).
#[derive(Debug)]
pub struct Font<'a> {
    ptr: *mut ffi::hb_font_t,
    face: Face<'a>,
}

impl<'a> Font<'a> {
    /// Create a new font from the specified `Face`.
    pub fn new(face: Face<'a>) -> Self {
        unsafe {
            Font {
                ptr: ffi::hb_font_create(face.as_ptr()),
                face,
            }
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut ffi::hb_font_t {
        self.ptr
    }

    /// Returns the EM scale of the font.
    pub fn scale(&self) -> (i32, i32) {
        let mut result = (0i32, 0i32);
        unsafe { ffi::hb_font_get_scale(self.ptr, &mut result.0, &mut result.1) };
        result
    }

    /// Sets the EM scale of the font.
    pub fn set_scale(&mut self, x: i32, y: i32) {
        unsafe { ffi::hb_font_set_scale(self.ptr, x, y) };
    }

    /// Returns font's PPEM.
    pub fn ppem(&self) -> (u32, u32) {
        let mut result = (0u32, 0u32);
        unsafe { ffi::hb_font_get_ppem(self.ptr, &mut result.0, &mut result.1) };
        result
    }

    /// Set font's PPEM.
    pub fn set_ppem(&mut self, x: u32, y: u32) {
        unsafe { ffi::hb_font_set_ppem(self.ptr, x, y) };
    }

    /// Sets *point size* of the font.
    ///
    /// Set to 0 to unset.
    ///
    /// There are 72 points in an inch.
    pub fn set_ptem(&mut self, ptem: f32) {
        unsafe { ffi::hb_font_set_ptem(self.ptr, ptem) };
    }

    /// Sets a font variations.
    pub fn set_variations(&mut self, variations: &[Variation]) {
        unsafe {
            ffi::hb_font_set_variations(
                self.ptr,
                variations.as_ptr() as *mut _,
                variations.len() as u32,
            )
        }
    }
}

impl<'a> Drop for Font<'a> {
    fn drop(&mut self) {
        unsafe { ffi::hb_font_destroy(self.ptr); }
    }
}


/// Font variation property.
pub struct Variation {
    /// Name.
    pub tag: Tag,
    /// Value.
    pub value: f32,
}

impl FromStr for Variation {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unsafe {
            let mut var: ffi::hb_variation_t = std::mem::MaybeUninit::zeroed().assume_init();
            let ok = ffi::hb_variation_from_string(s.as_ptr() as *const _, s.len() as i32, &mut var as *mut _);
            if ok == 1 {
                Ok(Variation {
                    tag: Tag(var.tag),
                    value: var.value,
                })
            } else {
                Err("invalid variation")
            }
        }
    }
}
