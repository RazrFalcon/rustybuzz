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
///
/// Combines `hb_font_t`, `hb_face_t` and `hb_blob_t`.
pub struct Font<'a> {
    ptr: NonNull<ffi::hb_font_t>,
    #[allow(dead_code)] face: Face<'a>,
}

impl<'a> Font<'a> {
    /// Creates a new `Font` from data.
    ///
    /// Data will be referenced, not owned.
    pub fn from_data(data: &'a [u8], face_index: u32) -> Option<Self> {
        let face = Face::from_data(data, face_index)?;
        let ptr = NonNull::new(unsafe { ffi::hb_font_create(face.as_ptr()) })?;
        Some(Font {
            ptr,
            face,
        })
    }

    pub(crate) fn as_ptr(&self) -> *mut ffi::hb_font_t {
        self.ptr.as_ptr()
    }

    /// Returns the EM scale of the font.
    pub fn scale(&self) -> (i32, i32) {
        let mut result = (0i32, 0i32);
        unsafe { ffi::hb_font_get_scale(self.as_ptr(), &mut result.0, &mut result.1) };
        result
    }

    /// Sets the EM scale of the font.
    pub fn set_scale(&mut self, x: i32, y: i32) {
        unsafe { ffi::hb_font_set_scale(self.as_ptr(), x, y) };
    }

    /// Sets point size per EM.
    ///
    /// Used for optical-sizing in Apple fonts.
    /// A value of zero means "not set".
    pub fn set_ptem(&mut self, v: f32) {
        unsafe { ffi::hb_font_set_ptem(self.as_ptr(), v) };
    }

    /// Sets font variations.
    pub fn set_variations(&mut self, variations: &[Variation]) {
        unsafe {
            ffi::hb_font_set_variations(
                self.as_ptr(),
                variations.as_ptr() as _,
                variations.len() as u32,
            )
        };
    }
}

impl Drop for Font<'_> {
    fn drop(&mut self) {
        unsafe { ffi::hb_font_destroy(self.as_ptr()) }
    }
}
