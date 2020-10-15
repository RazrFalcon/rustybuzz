use std::convert::TryFrom;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ptr::NonNull;

use ttf_parser::{Tag, GlyphId};

use crate::common::Variation;
use crate::ffi;


// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#windows-platform-platform-id--3
const WINDOWS_SYMBOL_ENCODING: u16 = 0;
const WINDOWS_UNICODE_BMP_ENCODING: u16 = 1;
const WINDOWS_UNICODE_FULL_ENCODING: u16 = 10;

// https://docs.microsoft.com/en-us/typography/opentype/spec/name#platform-specific-encoding-and-language-ids-unicode-platform-platform-id--0
const UNICODE_1_0_ENCODING: u16 = 0;
const UNICODE_1_1_ENCODING: u16 = 1;
const UNICODE_ISO_ENCODING: u16 = 2;
const UNICODE_2_0_BMP_ENCODING: u16 = 3;
const UNICODE_2_0_FULL_ENCODING: u16 = 4;
//const UNICODE_VARIATION_ENCODING: u16 = 5;
const UNICODE_FULL_ENCODING: u16 = 6;


struct Blob<'a> {
    ptr: NonNull<ffi::rb_blob_t>,
    marker: PhantomData<&'a [u8]>,
}

impl<'a> Blob<'a> {
    fn from_data(bytes: &'a [u8]) -> Option<Self> {
        let ptr = NonNull::new(unsafe {
            ffi::rb_blob_create(
                bytes.as_ptr() as *const _,
                bytes.len() as u32,
                std::ptr::null_mut(),
                None,
            )
        })?;

        Some(Blob {
            ptr,
            marker: PhantomData,
        })
    }

    pub fn as_ptr(&self) -> *mut ffi::rb_blob_t {
        self.ptr.as_ptr()
    }
}

impl Drop for Blob<'_> {
    fn drop(&mut self) {
        unsafe { ffi::rb_blob_destroy(self.as_ptr()) }
    }
}


struct Face<'a> {
    ptr: NonNull<ffi::rb_face_t>,
    #[allow(dead_code)] blob: Blob<'a>,
}

impl<'a> Face<'a> {
    fn from_data(data: &'a [u8], face_index: u32) -> Option<Self> {
        let blob = Blob::from_data(data)?;
        let ptr = NonNull::new(unsafe { ffi::rb_face_create(blob.as_ptr(), face_index) })?;
        Some(Face {
            ptr,
            blob,
        })
    }

    pub fn as_ptr(&self) -> *mut ffi::rb_face_t {
        self.ptr.as_ptr()
    }
}

impl Drop for Face<'_> {
    fn drop(&mut self) {
        unsafe { ffi::rb_face_destroy(self.as_ptr()) }
    }
}


/// A font handle.
pub struct Font<'a> {
    ttfp_face: ttf_parser::Face<'a>,
    rb_face: Face<'a>,
    units_per_em: i32,
    pixels_per_em: Option<(u16, u16)>,
    points_per_em: Option<f32>,
    coords: Vec<i32>,
    prefered_cmap_encoding_subtable: Option<u16>,
}

impl<'a> Font<'a> {
    /// Creates a new `Font` from data.
    ///
    /// Data will be referenced, not owned.
    pub fn from_slice(data: &'a [u8], face_index: u32) -> Option<Self> {
        let ttfp_face = ttf_parser::Face::from_slice(data, face_index).ok()?;
        let upem = ttfp_face.units_per_em()? as i32;
        let prefered_cmap_encoding_subtable = find_best_cmap_subtable(&ttfp_face);
        let face = Face::from_data(data, face_index)?;
        Some(Font {
            ttfp_face,
            rb_face: face,
            units_per_em: upem,
            pixels_per_em: None,
            points_per_em: None,
            coords: Vec::new(),
            prefered_cmap_encoding_subtable,
        })
    }

    #[inline]
    pub(crate) fn from_ptr(font: *const ffi::rb_font_t) -> &'static Font<'static> {
        unsafe { &*(font as *const Font) }
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const ffi::rb_font_t {
        self as *const _ as *const ffi::rb_font_t
    }

    #[inline]
    pub(crate) fn face_ptr(&self) -> *mut ffi::rb_face_t {
        self.rb_face.as_ptr()
    }

    #[inline]
    pub(crate) fn units_per_em(&self) -> i32 {
        self.units_per_em
    }

    /// Sets pixels per EM.
    ///
    /// Used during raster glyphs processing and hinting.
    ///
    /// `None` by default.
    #[inline]
    pub fn set_pixels_per_em(&mut self, ppem: Option<(u16, u16)>) {
        self.pixels_per_em = ppem;
    }

    /// Sets point size per EM.
    ///
    /// Used for optical-sizing in Apple fonts.
    ///
    /// `None` by default.
    #[inline]
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

    pub(crate) fn glyph_index(&self, c: u32) -> Option<GlyphId> {
        let subtable_idx = self.prefered_cmap_encoding_subtable?;
        let subtable = self.ttfp_face.character_mapping_subtables().nth(subtable_idx as usize)?;
        match subtable.glyph_index(c) {
            Some(gid) => Some(gid),
            None => {
                // Special case for Windows Symbol fonts.
                // TODO: add tests
                if  subtable.platform_id() == ttf_parser::PlatformId::Windows &&
                    subtable.encoding_id() == WINDOWS_SYMBOL_ENCODING
                {
                    if c <= 0x00FF {
                        // For symbol-encoded OpenType fonts, we duplicate the
                        // U+F000..F0FF range at U+0000..U+00FF.  That's what
                        // Windows seems to do, and that's hinted about at:
                        // https://docs.microsoft.com/en-us/typography/opentype/spec/recom
                        // under "Non-Standard (Symbol) Fonts".
                        return self.glyph_index(0xF000 + c);
                    }
                }

                None
            }
        }
    }

    pub(crate) fn glyph_variation_index(&self, c: char, variation: char) -> Option<GlyphId> {
        let res = self.ttfp_face.character_mapping_subtables()
            .find(|e| e.format() == ttf_parser::cmap::Format::UnicodeVariationSequences)
            .and_then(|e| e.glyph_variation_index(c, variation))?;
        match res {
            ttf_parser::cmap::GlyphVariationResult::Found(v) => Some(v),
            ttf_parser::cmap::GlyphVariationResult::UseDefault => self.glyph_index(c as u32),
        }
    }

    pub(crate) fn glyph_h_advance(&self, glyph: u32) -> u32 {
        rb_font_get_advance(self.as_ptr(), glyph, 0)
    }

    pub(crate) fn glyph_v_advance(&self, glyph: u32) -> i32 {
        // NOTE(laurmaedje): Is it correct to negate here?
        // Seems like it's done this way it `rb_ot_get_glyph_v_advances`.
        -(rb_font_get_advance(self.as_ptr(), glyph, 1) as i32)
    }

    pub(crate) fn glyph_extents(&self, glyph: u32) -> Option<ffi::rb_glyph_extents_t> {
        let glyph_id = GlyphId(u16::try_from(glyph).unwrap());

        let pixels_per_em = match self.pixels_per_em {
            Some(ppem) => ppem.0,
            None => std::u16::MAX,
        };

        if let Some(img) = self.ttfp_face.glyph_raster_image(glyph_id, pixels_per_em) {
            // HarfBuzz also supports only PNG.
            if img.format == ttf_parser::RasterImageFormat::PNG {
                let scale = self.units_per_em as f32 / img.pixels_per_em as f32;
                return Some(ffi::rb_glyph_extents_t {
                    x_bearing: (f32::from(img.x) * scale).round() as i32,
                    y_bearing: ((f32::from(img.y) + f32::from(img.height)) * scale).round() as i32,
                    width: (f32::from(img.width) * scale).round() as i32,
                    height: (-f32::from(img.height) * scale).round() as i32,
                });
            }
        }

        let bbox = self.ttfp_face.glyph_bounding_box(glyph_id)?;
        Some(ffi::rb_glyph_extents_t {
            x_bearing: i32::from(bbox.x_min),
            y_bearing: i32::from(bbox.y_max),
            width: i32::from(bbox.width()),
            height: i32::from(bbox.y_min - bbox.y_max),
        })
    }

    pub(crate) fn glyph_name(&self, glyph: u32) -> Option<&str> {
        let glyph_id = GlyphId(u16::try_from(glyph).unwrap());
        self.ttfp_face.glyph_name(glyph_id)
    }
}

fn find_best_cmap_subtable(face: &ttf_parser::Face) -> Option<u16> {
    use ttf_parser::PlatformId;

    // Symbol subtable.
    // Prefer symbol if available.
    // https://github.com/harfbuzz/harfbuzz/issues/1918
    find_cmap_subtable(face, PlatformId::Windows, WINDOWS_SYMBOL_ENCODING)
        // 32-bit subtables:
        .or(find_cmap_subtable(face, PlatformId::Windows, WINDOWS_UNICODE_FULL_ENCODING))
        .or(find_cmap_subtable(face, PlatformId::Unicode, UNICODE_FULL_ENCODING))
        .or(find_cmap_subtable(face, PlatformId::Unicode, UNICODE_2_0_FULL_ENCODING))
        // 16-bit subtables:
        .or(find_cmap_subtable(face, PlatformId::Windows, WINDOWS_UNICODE_BMP_ENCODING))
        .or(find_cmap_subtable(face, PlatformId::Unicode, UNICODE_2_0_BMP_ENCODING))
        .or(find_cmap_subtable(face, PlatformId::Unicode, UNICODE_ISO_ENCODING))
        .or(find_cmap_subtable(face, PlatformId::Unicode, UNICODE_1_1_ENCODING))
        .or(find_cmap_subtable(face, PlatformId::Unicode, UNICODE_1_0_ENCODING))
}

fn find_cmap_subtable(
    face: &ttf_parser::Face,
    platform_id: ttf_parser::PlatformId,
    encoding_id: u16,
) -> Option<u16> {
    for (i, subtable) in face.character_mapping_subtables().enumerate() {
        if subtable.platform_id() == platform_id && subtable.encoding_id() == encoding_id {
            return Some(i as u16)
        }
    }

    None
}

#[no_mangle]
pub extern "C" fn rb_font_get_face(font: *const ffi::rb_font_t) -> *mut ffi::rb_face_t {
    Font::from_ptr(font).rb_face.as_ptr()
}

#[no_mangle]
pub extern "C" fn rb_font_get_upem(font: *const ffi::rb_font_t) -> i32 {
    Font::from_ptr(font).units_per_em
}

#[no_mangle]
pub extern "C" fn rb_font_get_ptem(font: *const ffi::rb_font_t) -> f32 {
    Font::from_ptr(font).points_per_em.unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn rb_font_get_ppem_x(font: *const ffi::rb_font_t) -> u32 {
    Font::from_ptr(font).pixels_per_em.map(|ppem| ppem.0).unwrap_or(0) as u32
}

#[no_mangle]
pub extern "C" fn rb_font_get_ppem_y(font: *const ffi::rb_font_t) -> u32 {
    Font::from_ptr(font).pixels_per_em.map(|ppem| ppem.1).unwrap_or(0) as u32
}

#[no_mangle]
pub extern "C" fn rb_font_get_coords(font: *const ffi::rb_font_t) -> *const i32 {
    Font::from_ptr(font).coords.as_ptr() as _
}

#[no_mangle]
pub extern "C" fn rb_font_get_num_coords(font: *const ffi::rb_font_t) -> u32 {
    Font::from_ptr(font).coords.len() as u32
}

#[no_mangle]
pub extern "C" fn rb_ot_get_glyph_extents(
    font: *const ffi::rb_font_t,
    glyph: ffi::rb_codepoint_t,
    extents: *mut ffi::rb_glyph_extents_t,
) -> ffi::rb_bool_t {
    let font = Font::from_ptr(font);
    match font.glyph_extents(glyph) {
        Some(bbox) => {
            unsafe { *extents = bbox; }
            1
        }
        None => 0,
    }
}

#[no_mangle]
pub extern "C" fn rb_font_get_advance(
    font: *const ffi::rb_font_t,
    glyph: ffi::rb_codepoint_t,
    is_vertical: ffi::rb_bool_t,
) -> u32 {
    let face = &Font::from_ptr(font).ttfp_face;
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
pub extern "C" fn rb_font_get_side_bearing(
    font: *const ffi::rb_font_t,
    glyph: ffi::rb_codepoint_t,
    is_vertical: ffi::rb_bool_t,
) -> i32 {
    let face = &Font::from_ptr(font).ttfp_face;
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
pub extern "C" fn rb_ot_metrics_get_position_common(
    font: *const ffi::rb_font_t,
    tag: Tag,
    position: *mut i32,
) -> ffi::rb_bool_t {
    let face = &Font::from_ptr(font).ttfp_face;
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
pub extern "C" fn rb_ot_get_glyph_name(
    font: *const ffi::rb_font_t,
    glyph: ffi::rb_codepoint_t,
    mut raw_name: *mut c_char,
    len: u32,
) -> i32 {
    assert_ne!(len, 0);

    let face = &Font::from_ptr(font).ttfp_face;
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

#[no_mangle]
pub extern "C" fn rb_font_has_vorg_data(font: *const ffi::rb_font_t) -> ffi::rb_bool_t {
    Font::from_ptr(font).ttfp_face.has_table(ttf_parser::TableName::VerticalOrigin) as i32
}

#[no_mangle]
pub extern "C" fn rb_font_get_y_origin(font: *const ffi::rb_font_t, glyph: ffi::rb_codepoint_t) -> i32 {
    let glyph_id = GlyphId(u16::try_from(glyph).unwrap());
    Font::from_ptr(font).ttfp_face.glyph_y_origin(glyph_id).unwrap_or(0) as i32
}

#[no_mangle]
pub extern "C" fn rb_ot_get_nominal_glyph(
    font: *const ffi::rb_font_t,
    u: ffi::rb_codepoint_t,
    glyph: *mut ffi::rb_codepoint_t,
) -> ffi::rb_bool_t {
    match Font::from_ptr(font).glyph_index(u) {
        Some(gid) => {
            unsafe { *glyph = gid.0 as u32 };
            1
        }
        None => 0,
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_get_variation_glyph(
    font: *const ffi::rb_font_t,
    u: ffi::rb_codepoint_t,
    variation: ffi::rb_codepoint_t,
    glyph: *mut ffi::rb_codepoint_t,
) -> ffi::rb_bool_t {
    let u = char::try_from(u).unwrap();
    let variation = char::try_from(variation).unwrap();
    match Font::from_ptr(font).glyph_variation_index(u, variation) {
        Some(gid) => {
            unsafe { *glyph = gid.0 as u32 };
            1
        }
        None => 0,
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
            assert_eq!(rb_ot_metrics_get_position_common(font.as_ptr(), metrics::HORIZONTAL_ASCENDER, pos), 1);
            assert_eq!(*pos, 967);

            assert_eq!(rb_ot_metrics_get_position_common(font.as_ptr(), metrics::HORIZONTAL_DESCENDER, pos), 1);
            assert_eq!(*pos, -253);

            assert_eq!(rb_ot_metrics_get_position_common(font.as_ptr(), metrics::HORIZONTAL_LINE_GAP, pos), 1);
            assert_eq!(*pos, 0);

            // Vertical.
            assert_eq!(rb_ot_metrics_get_position_common(font.as_ptr(), metrics::VERTICAL_ASCENDER, pos), 1);
            assert_eq!(*pos, 500);

            assert_eq!(rb_ot_metrics_get_position_common(font.as_ptr(), metrics::VERTICAL_DESCENDER, pos), 1);
            assert_eq!(*pos, -500);

            assert_eq!(rb_ot_metrics_get_position_common(font.as_ptr(), metrics::VERTICAL_LINE_GAP, pos), 1);
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
            assert_eq!(rb_ot_metrics_get_position_common(font.as_ptr(), metrics::HORIZONTAL_ASCENDER, pos), 1);
            assert_eq!(*pos, 800);

            assert_eq!(rb_ot_metrics_get_position_common(font.as_ptr(), metrics::HORIZONTAL_DESCENDER, pos), 1);
            assert_eq!(*pos, -200);

            assert_eq!(rb_ot_metrics_get_position_common(font.as_ptr(), metrics::HORIZONTAL_LINE_GAP, pos), 1);
            assert_eq!(*pos, 90);

            // Vertical.
            assert_eq!(rb_ot_metrics_get_position_common(font.as_ptr(), metrics::VERTICAL_ASCENDER, pos), 0);
            assert_eq!(rb_ot_metrics_get_position_common(font.as_ptr(), metrics::VERTICAL_DESCENDER, pos), 0);
            assert_eq!(rb_ot_metrics_get_position_common(font.as_ptr(), metrics::VERTICAL_LINE_GAP, pos), 0);
        }
    }
}
