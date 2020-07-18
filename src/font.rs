use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::common::Variation;
use crate::ffi;


struct Blob<'a> {
    ptr: NonNull<ffi::hb_blob_t>,
    marker: PhantomData<&'a [u8]>,
}

impl<'a> Blob<'a> {
    fn from_data(bytes: &'a [u8]) -> Option<Self> {
        let ptr = NonNull::new(unsafe {
            ffi::hb_blob_create(
                bytes.as_ptr() as *const _,
                bytes.len() as u32,
                ffi::HB_MEMORY_MODE_READONLY,
                std::ptr::null_mut(),
                None,
            )
        })?;

        Some(Blob {
            ptr,
            marker: PhantomData,
        })
    }

    pub fn as_ptr(&self) -> *mut ffi::hb_blob_t {
        self.ptr.as_ptr()
    }
}

impl Drop for Blob<'_> {
    fn drop(&mut self) {
        unsafe { ffi::hb_blob_destroy(self.as_ptr()) }
    }
}


struct Face<'a> {
    ptr: NonNull<ffi::hb_face_t>,
    #[allow(dead_code)] blob: Blob<'a>,
}

impl<'a> Face<'a> {
    fn from_data(data: &'a [u8], face_index: u32) -> Option<Self> {
        let blob = Blob::from_data(data)?;
        let ptr = NonNull::new(unsafe { ffi::hb_face_create(blob.as_ptr(), face_index) })?;
        Some(Face {
            ptr,
            blob,
        })
    }

    pub fn as_ptr(&self) -> *mut ffi::hb_face_t {
        self.ptr.as_ptr()
    }
}

impl Drop for Face<'_> {
    fn drop(&mut self) {
        unsafe { ffi::hb_face_destroy(self.as_ptr()) }
    }
}


/// A font handle.
pub struct Font<'a> {
    ttfp_face: ttf_parser::Face<'a>,
    hb_face: Face<'a>,
    upem: i32,
    ppem: Option<(u32, u32)>,
    ptem: Option<f32>,
    coords: Vec<i32>,
}

impl<'a> Font<'a> {
    /// Creates a new `Font` from data.
    ///
    /// Data will be referenced, not owned.
    pub fn from_slice(data: &'a [u8], face_index: u32) -> Option<Self> {
        let ttfp_face = ttf_parser::Face::from_slice(data, face_index).ok()?;
        let upem = ttfp_face.units_per_em()? as i32;

        let face = Face::from_data(data, face_index)?;
        Some(Font {
            ttfp_face,
            hb_face: face,
            upem,
            ppem: None,
            ptem: None,
            coords: Vec::new(),
        })
    }

    pub(crate) fn as_ptr(&self) -> *const ffi::rb_font_t {
        self as *const _ as *const ffi::rb_font_t
    }

    /// Sets pixels per EM.
    ///
    /// Used during raster glyphs processing and hinting.
    ///
    /// `None` by default.
    pub fn set_ppem(&mut self, ppem: Option<(u32, u32)>) {
        self.ppem = ppem;
    }

    /// Sets point size per EM.
    ///
    /// Used for optical-sizing in Apple fonts.
    ///
    /// `None` by default.
    pub fn set_ptem(&mut self, ptem: Option<f32>) {
        self.ptem = ptem;
    }

    /// Sets font variations.
    pub fn set_variations(&mut self, variations: &[Variation]) {
        for variation in variations {
            self.ttfp_face.set_variation(variation.tag, variation.value);
        }

        self.coords.clear();
        for c in self.ttfp_face.variation_coordinates() {
            self.coords.push(c.get() as i32)
        }
    }
}

fn font_from_raw(font: *const ffi::rb_font_t) -> &'static Font<'static> {
    unsafe { &*(font as *const Font) }
}

#[no_mangle]
pub extern "C" fn hb_font_get_face(font: *const ffi::rb_font_t) -> *mut ffi::hb_face_t {
    font_from_raw(font).hb_face.as_ptr()
}

#[no_mangle]
pub extern "C" fn hb_font_get_upem(font: *const ffi::rb_font_t) -> i32 {
    font_from_raw(font).upem
}

#[no_mangle]
pub extern "C" fn hb_font_get_ptem(font: *const ffi::rb_font_t) -> f32 {
    font_from_raw(font).ptem.unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn hb_font_get_ppem_x(font: *const ffi::rb_font_t) -> u32 {
    font_from_raw(font).ppem.map(|ppem| ppem.0).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn hb_font_get_ppem_y(font: *const ffi::rb_font_t) -> u32 {
    font_from_raw(font).ppem.map(|ppem| ppem.1).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn hb_font_get_coords(font: *const ffi::rb_font_t) -> *const i32 {
    font_from_raw(font).coords.as_ptr() as _
}

#[no_mangle]
pub extern "C" fn hb_font_get_num_coords(font: *const ffi::rb_font_t) -> u32 {
    font_from_raw(font).coords.len() as u32
}
