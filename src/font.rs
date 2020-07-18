use std::convert::TryFrom;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ptr::NonNull;

use ttf_parser::{Tag, GlyphId};

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
    units_per_em: i32,
    pixels_per_em: Option<(u16, u16)>,
    points_per_em: Option<f32>,
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
            units_per_em: upem,
            pixels_per_em: None,
            points_per_em: None,
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
    pub fn set_pixels_per_em(&mut self, ppem: Option<(u16, u16)>) {
        self.pixels_per_em = ppem;
    }

    /// Sets point size per EM.
    ///
    /// Used for optical-sizing in Apple fonts.
    ///
    /// `None` by default.
    pub fn set_points_per_em(&mut self, ptem: Option<f32>) {
        self.points_per_em = ptem;
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
    font_from_raw(font).units_per_em
}

#[no_mangle]
pub extern "C" fn hb_font_get_ptem(font: *const ffi::hb_font_t) -> f32 {
    font_from_raw(font).points_per_em.unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn hb_font_get_ppem_x(font: *const ffi::hb_font_t) -> u32 {
    font_from_raw(font).pixels_per_em.map(|ppem| ppem.0).unwrap_or(0) as u32
}

#[no_mangle]
pub extern "C" fn hb_font_get_ppem_y(font: *const ffi::hb_font_t) -> u32 {
    font_from_raw(font).pixels_per_em.map(|ppem| ppem.1).unwrap_or(0) as u32
}

#[no_mangle]
pub extern "C" fn hb_font_get_coords(font: *const ffi::hb_font_t) -> *const i32 {
    font_from_raw(font).coords.as_ptr() as _
}

#[no_mangle]
pub extern "C" fn hb_font_get_num_coords(font: *const ffi::hb_font_t) -> u32 {
    font_from_raw(font).coords.len() as u32
}

#[no_mangle]
pub extern "C" fn hb_ot_get_glyph_extents(
    font: *const ffi::hb_font_t,
    glyph: ffi::hb_codepoint_t,
    extents: *mut ffi::hb_glyph_extents_t,
) -> ffi::hb_bool_t {
    let font = font_from_raw(font);
    let glyph_id = GlyphId(u16::try_from(glyph).unwrap());

    let pixels_per_em = match font.pixels_per_em {
        Some(ppem) => ppem.0,
        None => std::u16::MAX,
    };

    if let Some(img) = font.ttfp_face.glyph_raster_image(glyph_id, pixels_per_em) {
        // HarfBuzz also supports only PNG.
        if img.format == ttf_parser::RasterImageFormat::PNG {
            let scale = font.units_per_em as f32 / img.pixels_per_em as f32;
            unsafe {
                *extents = ffi::hb_glyph_extents_t {
                    x_bearing: (f32::from(img.x) * scale).round() as i32,
                    y_bearing: ((f32::from(img.y) + f32::from(img.height)) * scale).round() as i32,
                    width: (f32::from(img.width) * scale).round() as i32,
                    height: (-f32::from(img.height) * scale).round() as i32,
                };
            }

            return 1;
        }
    }

    match font.ttfp_face.glyph_bounding_box(glyph_id) {
        Some(bbox) => unsafe {
            *extents = ffi::hb_glyph_extents_t {
                x_bearing: i32::from(bbox.x_min),
                y_bearing: i32::from(bbox.y_max),
                width: i32::from(bbox.width()),
                height: i32::from(bbox.y_min - bbox.y_max),
            };

            1
        }
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn hb_font_get_advance(
    font: *const ffi::hb_font_t,
    glyph: ffi::hb_codepoint_t,
    is_vertical: ffi::hb_bool_t,
) -> u32 {
    let face = &font_from_raw(font).ttfp_face;
    let glyph = GlyphId(u16::try_from(glyph).unwrap());

    if  face.is_variable() &&
        face.has_non_default_variation_coordinates() &&
       !face.has_table(ttf_parser::TableName::HorizontalMetricsVariations) &&
       !face.has_table(ttf_parser::TableName::VerticalMetricsVariations)
    {
        return match face.glyph_bounding_box(glyph) {
            Some(bbox) => {
                (if is_vertical == 1 {
                    bbox.y_max + bbox.y_min
                } else {
                    bbox.x_max + bbox.x_min
                }) as u32
            }
            None => 0,
        };
    }

    if is_vertical == 1 && face.has_table(ttf_parser::TableName::VerticalMetrics) {
        face.glyph_ver_advance(glyph).unwrap_or(0) as u32
    } else if is_vertical == 0 && face.has_table(ttf_parser::TableName::HorizontalMetrics) {
        face.glyph_hor_advance(glyph).unwrap_or(0) as u32
    } else {
        face.units_per_em().unwrap_or(1000) as u32
    }
}

#[no_mangle]
pub extern "C" fn hb_font_get_side_bearing(
    font: *const ffi::hb_font_t,
    glyph: ffi::hb_codepoint_t,
    is_vertical: ffi::hb_bool_t,
) -> i32 {
    let face = &font_from_raw(font).ttfp_face;
    let glyph = GlyphId(u16::try_from(glyph).unwrap());

    if  face.is_variable() &&
       !face.has_table(ttf_parser::TableName::HorizontalMetricsVariations) &&
       !face.has_table(ttf_parser::TableName::VerticalMetricsVariations)
    {
        return match face.glyph_bounding_box(glyph) {
            Some(bbox) => (if is_vertical == 1 { bbox.x_min } else { bbox.y_min }) as i32,
            None => 0,
        }
    }

    if is_vertical == 1 {
        face.glyph_ver_side_bearing(glyph).unwrap_or(0) as i32
    } else {
        face.glyph_hor_side_bearing(glyph).unwrap_or(0) as i32
    }
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

#[no_mangle]
pub extern "C" fn hb_ot_get_glyph_name(
    font: *const ffi::hb_font_t,
    glyph: ffi::hb_codepoint_t,
    mut raw_name: *mut c_char,
    len: u32,
) -> i32 {
    assert_ne!(len, 0);

    let face = &font_from_raw(font).ttfp_face;
    match face.glyph_name(GlyphId(u16::try_from(glyph).unwrap())) {
        Some(name) => unsafe {
            let len = std::cmp::min(name.len(), len as usize - 1);

            for b in &name.as_bytes()[0..len] {
                *raw_name = *b as c_char;
                raw_name = raw_name.offset(1);
            }

            *raw_name = b'\0' as c_char;

            1
        }
        _ => 0,
    }
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
