use std::convert::TryFrom;
use std::os::raw::{c_void, c_char};
use std::marker::PhantomData;

use ttf_parser::GlyphId;

use crate::ffi;
use crate::common::f32_bound;
use crate::Variation;


#[derive(Debug)]
struct Blob<'a> {
    ptr: *mut ffi::hb_blob_t,
    marker: PhantomData<&'a [u8]>,
}

impl<'a> Blob<'a> {
    fn with_bytes(bytes: &'a [u8]) -> Blob<'a> {
        unsafe {
            let hb_blob = ffi::hb_blob_create(
                bytes.as_ptr() as *const _,
                bytes.len() as u32,
                ffi::HB_MEMORY_MODE_READONLY,
                std::ptr::null_mut(),
                None,
            );

            Blob {
                ptr: hb_blob,
                marker: PhantomData,
            }
        }
    }

    fn as_ptr(&self) -> *mut ffi::hb_blob_t {
        self.ptr
    }
}

impl<'a> Drop for Blob<'a> {
    fn drop(&mut self) {
        unsafe { ffi::hb_blob_destroy(self.ptr); }
    }
}


/// A wrapper around `hb_face_t`.
///
/// Font face is objects represent a single face in a font family. More
/// exactly, a font face represents a single face in a binary font file. Font
/// faces are typically built from a binary blob and a face index. Font faces
/// are used to create fonts.
#[derive(Debug)]
pub struct Face<'a> {
    ptr: *mut ffi::hb_face_t,
    blob: Blob<'a>,
    ttf: *const ttf_parser::Font<'a>,
}

impl<'a> Face<'a> {
    /// Creates a new `Face` from the data.
    pub fn new(data: &'a [u8], index: u32) -> Result<Face<'a>, ttf_parser::Error> {
        unsafe {
            let ttf = Box::new(ttf_parser::Font::from_data(data, index)?);
            let ttf = Box::into_raw(ttf);
            let blob = Blob::with_bytes(data);
            Ok(Face {
                ptr: ffi::hb_face_create(blob.as_ptr(), ttf as *const _, index),
                blob,
                ttf,
            })
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut ffi::hb_face_t {
        self.ptr
    }

    /// Returns face's UPEM.
    pub fn upem(&self) -> u32 {
        unsafe { ffi::hb_face_get_upem(self.ptr) }
    }

    /// Sets face's UPEM.
    pub fn set_upem(&mut self, upem: u32) {
        unsafe { ffi::hb_face_set_upem(self.ptr, upem) };
    }
}

impl<'a> Drop for Face<'a> {
    fn drop(&mut self) {
        unsafe {
            Box::from_raw(self.ttf as *mut ttf_parser::Font<'a>);
            ffi::hb_face_destroy(self.ptr);
        }
    }
}


/// A type representing a single font (i.e. a specific combination of typeface and typesize).
#[derive(Debug)]
pub struct Font<'a> {
    ptr: *mut ffi::hb_font_t,
    face: Face<'a>,
}

impl<'a> Font<'a> {
    /// Creates a new font from the specified `Face`.
    pub fn new(face: Face<'a>) -> Self {
        unsafe {
            Font {
                ptr: ffi::hb_font_create(face.as_ptr(), face.ttf as *const _),
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
        let ttf = unsafe { &*self.face.ttf };
        let coords_len = try_opt!(ttf.variation_axes_count()) as usize;
        let mut coords = vec![0; coords_len];

        for variation in variations {
            if let Some(axis) = ttf.variation_axis(variation.tag) {
                let mut v = f32_bound(axis.min_value, variation.value, axis.max_value);

                if v == axis.default_value {
                    v = 0.0;
                } else if v < axis.default_value {
                    v = (v - axis.default_value) / (axis.default_value - axis.min_value);
                } else {
                    v = (v - axis.default_value) / (axis.max_value - axis.default_value)
                }

                coords[axis.index as usize] = (v * 16384.0).round() as i32;
            }
        }

        ttf.map_variation_coordinates(&mut coords);

        unsafe {
            ffi::hb_font_set_variations(
                self.ptr,
                coords.as_ptr() as *mut _,
                coords.len() as u32,
            )
        }
    }
}

impl<'a> Drop for Font<'a> {
    fn drop(&mut self) {
        unsafe { ffi::hb_font_destroy(self.ptr); }
    }
}


#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn rb_ot_get_nominal_glyph(font_data: *const c_void, c: u32, glyph: *mut u32) -> i32 {
    let font = unsafe { &*(font_data as *const ttf_parser::Font) };
    match font.glyph_index(char::try_from(c).unwrap()) {
        Ok(g) => unsafe { *glyph = g.0 as u32; 1 }
        Err(_) => 0,
    }
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn rb_ot_get_variation_glyph(font_data: *const c_void, c: u32, variant: u32, glyph: *mut u32) -> i32 {
    let font = unsafe { &*(font_data as *const ttf_parser::Font) };
    match font.glyph_variation_index(char::try_from(c).unwrap(), char::try_from(variant).unwrap()) {
        Ok(g) => unsafe { *glyph = g.0 as u32; 1 }
        Err(_) => 0,
    }
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn rb_ot_get_glyph_bbox(font_data: *const c_void, glyph: u32, extents: *mut ffi::hb_glyph_bbox_t) -> i32 {
    let font = unsafe { &*(font_data as *const ttf_parser::Font) };
    match font.glyph_bounding_box(GlyphId(u16::try_from(glyph).unwrap())) {
        Ok(bbox) => unsafe {
            (*extents).x_min = bbox.x_min;
            (*extents).y_min = bbox.y_min;
            (*extents).x_max = bbox.x_max;
            (*extents).y_max = bbox.y_max;
            1
        }
        Err(_) => 0,
    }
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn rb_ot_get_glyph_name(font_data: *const c_void, glyph: u32, mut raw_name: *mut c_char, len: u32) -> i32 {
    assert_ne!(len, 0);

    let font = unsafe { &*(font_data as *const ttf_parser::Font) };
    match font.glyph_name(GlyphId(u16::try_from(glyph).unwrap())) {
        Some(name) => unsafe {
            let len = std::cmp::min(name.len(), len as usize - 1);

            for b in &name.as_bytes()[0..len] {
                *raw_name = *b as c_char;
                raw_name = raw_name.offset(1);
            }

            *raw_name = b'\0' as c_char;

            1
        }
        None => 0,
    }
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn rb_ot_has_glyph_classes(font_data: *const c_void) -> i32 {
    let font = unsafe { &*(font_data as *const ttf_parser::Font) };
    font.has_glyph_classes() as i32
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn rb_ot_get_glyph_class(font_data: *const c_void, glyph: u32) -> u32 {
    let font = unsafe { &*(font_data as *const ttf_parser::Font) };
    match font.glyph_class(GlyphId(u16::try_from(glyph).unwrap())) {
        Ok(c) => c as u32,
        Err(_) => 0,
    }
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn rb_ot_get_mark_attachment_class(font_data: *const c_void, glyph: u32) -> u32 {
    let font = unsafe { &*(font_data as *const ttf_parser::Font) };
    match font.glyph_mark_attachment_class(GlyphId(u16::try_from(glyph).unwrap())) {
        Ok(c) => c.0 as u32,
        Err(_) => 0,
    }
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn rb_ot_is_mark_glyph(font_data: *const c_void, set_index: u32, glyph: u32) -> i32 {
    let font = unsafe { &*(font_data as *const ttf_parser::Font) };
    match font.is_mark_glyph(GlyphId(u16::try_from(glyph).unwrap()), Some(set_index)) {
        Ok(c) => c as i32,
        Err(_) => 0,
    }
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn hb_ot_get_var_axis_count(font_data: *const c_void) -> u16 {
    let font = unsafe { &*(font_data as *const ttf_parser::Font) };
    font.variation_axes_count().unwrap_or(0)
}
