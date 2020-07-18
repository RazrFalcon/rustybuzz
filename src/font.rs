use std::marker::PhantomData;
use std::ptr::NonNull;

use ttf_parser::Tag;

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

    pub(crate) fn as_ptr(&self) -> *const ffi::hb_font_t {
        self as *const _ as *const ffi::hb_font_t
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

fn font_from_raw(font: *const ffi::hb_font_t) -> &'static Font<'static> {
    unsafe { &*(font as *const Font) }
}

#[no_mangle]
pub extern "C" fn hb_font_get_face(font: *const ffi::hb_font_t) -> *mut ffi::hb_face_t {
    font_from_raw(font).hb_face.as_ptr()
}

#[no_mangle]
pub extern "C" fn hb_font_get_upem(font: *const ffi::hb_font_t) -> i32 {
    font_from_raw(font).upem
}

#[no_mangle]
pub extern "C" fn hb_font_get_ptem(font: *const ffi::hb_font_t) -> f32 {
    font_from_raw(font).ptem.unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn hb_font_get_ppem_x(font: *const ffi::hb_font_t) -> u32 {
    font_from_raw(font).ppem.map(|ppem| ppem.0).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn hb_font_get_ppem_y(font: *const ffi::hb_font_t) -> u32 {
    font_from_raw(font).ppem.map(|ppem| ppem.1).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn hb_font_get_coords(font: *const ffi::hb_font_t) -> *const i32 {
    font_from_raw(font).coords.as_ptr() as _
}

#[no_mangle]
pub extern "C" fn hb_font_get_num_coords(font: *const ffi::hb_font_t) -> u32 {
    font_from_raw(font).coords.len() as u32
}

mod metrics {
    use crate::Tag;

    pub const HORIZONTAL_ASCENDER: Tag  = Tag::from_bytes(b"hasc");
    pub const HORIZONTAL_DESCENDER: Tag = Tag::from_bytes(b"hdsc");
    pub const HORIZONTAL_LINE_GAP: Tag  = Tag::from_bytes(b"hlgp");
    pub const VERTICAL_ASCENDER: Tag    = Tag::from_bytes(b"vasc");
    pub const VERTICAL_DESCENDER: Tag   = Tag::from_bytes(b"vdsc");
    pub const VERTICAL_LINE_GAP: Tag    = Tag::from_bytes(b"vlgp");
}

#[no_mangle]
pub extern "C" fn hb_ot_metrics_get_position_common(
    font: *const ffi::hb_font_t,
    tag: Tag,
    position: *mut i32,
) -> ffi::hb_bool_t {
    let face = &font_from_raw(font).ttfp_face;
    let pos = match tag {
        metrics::HORIZONTAL_ASCENDER => {
            i32::from(face.ascender())
        }
        metrics::HORIZONTAL_DESCENDER => {
            -i32::from(face.descender()).abs()
        }
        metrics::HORIZONTAL_LINE_GAP => {
            i32::from(face.line_gap())
        }
        metrics::VERTICAL_ASCENDER if face.has_table(ttf_parser::TableName::VerticalHeader) => {
            i32::from(face.vertical_ascender().unwrap_or(0))
        }
        metrics::VERTICAL_DESCENDER if face.has_table(ttf_parser::TableName::VerticalHeader) => {
            -i32::from(face.vertical_descender().unwrap_or(0)).abs()
        }
        metrics::VERTICAL_LINE_GAP if face.has_table(ttf_parser::TableName::VerticalHeader) => {
            i32::from(face.vertical_line_gap().unwrap_or(0))
        }
        _ => return 0,
    };

    unsafe { *position = pos; }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_get_position_common_vertical() {
        // Vertical font.
        let font_data = std::fs::read("tests/fonts/text-rendering-tests/TestGVAROne.ttf").unwrap();
        let font = Font::from_slice(&font_data, 0).unwrap();

        unsafe {
            let pos = &mut 0i32 as _;

            // Horizontal.
            assert_eq!(hb_ot_metrics_get_position_common(font.as_ptr(), metrics::HORIZONTAL_ASCENDER, pos), 1);
            assert_eq!(*pos, 967);

            assert_eq!(hb_ot_metrics_get_position_common(font.as_ptr(), metrics::HORIZONTAL_DESCENDER, pos), 1);
            assert_eq!(*pos, -253);

            assert_eq!(hb_ot_metrics_get_position_common(font.as_ptr(), metrics::HORIZONTAL_LINE_GAP, pos), 1);
            assert_eq!(*pos, 0);

            // Vertical.
            assert_eq!(hb_ot_metrics_get_position_common(font.as_ptr(), metrics::VERTICAL_ASCENDER, pos), 1);
            assert_eq!(*pos, 500);

            assert_eq!(hb_ot_metrics_get_position_common(font.as_ptr(), metrics::VERTICAL_DESCENDER, pos), 1);
            assert_eq!(*pos, -500);

            assert_eq!(hb_ot_metrics_get_position_common(font.as_ptr(), metrics::VERTICAL_LINE_GAP, pos), 1);
            assert_eq!(*pos, 0);

            // TODO: find font with variable metrics
        }
    }

    #[test]
    fn metrics_get_position_common_use_typo() {
        // A font with OS/2.useTypographicMetrics flag set.
        let font_data = std::fs::read("tests/fonts/in-house/1a3d8f381387dd29be1e897e4b5100ac8b4829e1.ttf").unwrap();
        let font = Font::from_slice(&font_data, 0).unwrap();

        unsafe {
            let pos = &mut 0i32 as _;

            // Horizontal.
            assert_eq!(hb_ot_metrics_get_position_common(font.as_ptr(), metrics::HORIZONTAL_ASCENDER, pos), 1);
            assert_eq!(*pos, 800);

            assert_eq!(hb_ot_metrics_get_position_common(font.as_ptr(), metrics::HORIZONTAL_DESCENDER, pos), 1);
            assert_eq!(*pos, -200);

            assert_eq!(hb_ot_metrics_get_position_common(font.as_ptr(), metrics::HORIZONTAL_LINE_GAP, pos), 1);
            assert_eq!(*pos, 90);

            // Vertical.
            assert_eq!(hb_ot_metrics_get_position_common(font.as_ptr(), metrics::VERTICAL_ASCENDER, pos), 0);
            assert_eq!(hb_ot_metrics_get_position_common(font.as_ptr(), metrics::VERTICAL_DESCENDER, pos), 0);
            assert_eq!(hb_ot_metrics_get_position_common(font.as_ptr(), metrics::VERTICAL_LINE_GAP, pos), 0);
        }
    }
}
