use std::marker::PhantomData;

use crate::{ffi, Face, Tag};

#[repr(C)]
pub struct Map {
    // This has to have the exact same memory size as the C++ variant!
    inner: ffi::rb_aat_map_t
}

impl Map {
    pub fn new() -> Self {
        // Initialized on the C++ side.
        let mut map = Map {
            inner: ffi::rb_aat_map_t {
                _chain_flags: ffi::rb_vector_t::zero(),
            }
        };

        unsafe { ffi::rb_aat_map_init(&mut map.inner); }

        map
    }

    pub fn as_ptr(&self) -> *const ffi::rb_aat_map_t {
        self as *const _ as *const ffi::rb_aat_map_t
    }

    pub fn as_ptr_mut(&mut self) -> *mut ffi::rb_aat_map_t {
        self as *mut _ as *mut ffi::rb_aat_map_t
    }
}

impl Drop for Map {
    fn drop(&mut self) {
        unsafe { ffi::rb_aat_map_fini(&mut self.inner); }
    }
}

#[repr(C)]
pub struct MapBuilder<'a> {
    // This has to have the exact same memory size as the C++ variant!
    inner: ffi::rb_aat_map_builder_t,
    phantom: PhantomData<&'a Face<'a>>,
}

impl<'a> MapBuilder<'a> {
    pub fn new(face: &'a Face<'a>) -> Self {
        // Initialized on the C++ side.
        let mut builder = MapBuilder {
            inner: ffi::rb_aat_map_builder_t {
                _face: std::ptr::null(),
                _features: ffi::rb_vector_t::zero(),
            },
            phantom: PhantomData,
        };

        unsafe { ffi::rb_aat_map_builder_init(&mut builder.inner, face.as_ptr()); }

        builder
    }

    pub fn as_ptr(&mut self) -> *mut ffi::rb_aat_map_builder_t {
        self as *mut _ as *mut ffi::rb_aat_map_builder_t
    }

    pub fn add_feature(&mut self, tag: Tag, value: u32) {
        unsafe { ffi::rb_aat_map_builder_add_feature(self.as_ptr(), tag, value); }
    }

    pub fn compile(&mut self, map: &mut Map) {
        unsafe { ffi::rb_aat_map_builder_compile(self.as_ptr(), map.as_ptr_mut()); }
    }
}

impl Drop for MapBuilder<'_> {
    fn drop(&mut self) {
        unsafe { ffi::rb_aat_map_builder_fini(&mut self.inner); }
    }
}
