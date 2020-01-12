use std::marker::PhantomData;

use crate::ffi;

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
}

impl<'a> Face<'a> {
    /// Create a new `Face` from the data.
    pub fn new(data: &'a [u8], index: u32) -> Face<'a> {
        unsafe {
            let blob = Blob::with_bytes(data);
            Face {
                ptr: ffi::hb_face_create(blob.as_ptr(), index),
                blob,
            }
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
        unsafe { ffi::hb_face_destroy(self.ptr); }
    }
}


#[derive(Debug)]
pub(crate) struct Blob<'a> {
    ptr: *mut ffi::hb_blob_t,
    marker: PhantomData<&'a [u8]>,
}

impl<'a> Blob<'a> {
    /// Create a new `Blob` from the slice `bytes`. The blob will not own the
    /// slice's data.
    pub fn with_bytes(bytes: &'a [u8]) -> Blob<'a> {
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

    pub(crate) fn as_ptr(&self) -> *mut ffi::hb_blob_t {
        self.ptr
    }
}

impl<'a> Drop for Blob<'a> {
    fn drop(&mut self) {
        unsafe { ffi::hb_blob_destroy(self.ptr); }
    }
}
