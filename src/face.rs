use std::marker::PhantomData;
use std::ptr::NonNull;

use ttf_parser::{Tag, GlyphClass, GlyphId};

use crate::buffer::GlyphPropsFlags;
use crate::tables::gpos::PosTable;
use crate::tables::gsub::SubstTable;
use crate::tables::gsubgpos::SubstPosTable;
use crate::ot::TableIndex;
use crate::{ffi, Variation};


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
    fn from_data(bytes: &'a [u8]) -> Self {
        Blob {
            ptr: NonNull::new(unsafe {
                ffi::rb_blob_create(
                    bytes.as_ptr() as *const _,
                    bytes.len() as u32,
                    std::ptr::null_mut(),
                    None,
                )
            }).unwrap(),
            marker: PhantomData,
        }
    }

    pub fn from_ptr(ptr: *mut ffi::rb_blob_t) -> Blob<'static> {
        Blob {
            ptr: NonNull::new(ptr).unwrap(),
            marker: PhantomData,
        }
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


/// A font face handle.
pub struct Face<'a> {
    pub(crate) ttfp_face: ttf_parser::Face<'a>,
    units_per_em: i32,
    pixels_per_em: Option<(u16, u16)>,
    points_per_em: Option<f32>,
    prefered_cmap_encoding_subtable: Option<u16>,
    pub(crate) gsub: Option<SubstTable<'a>>,
    pub(crate) gpos: Option<PosTable<'a>>,
    kern: Blob<'a>,
    morx: Blob<'a>,
    mort: Blob<'a>,
    kerx: Blob<'a>,
    ankr: Blob<'a>,
    trak: Blob<'a>,
    feat: Blob<'a>,
}

impl<'a> Face<'a> {
    /// Creates a new `Face` from data.
    ///
    /// Data will be referenced, not owned.
    pub fn from_slice(data: &'a [u8], face_index: u32) -> Option<Self> {
        let ttfp_face = ttf_parser::Face::from_slice(data, face_index).ok()?;
        let upem = ttfp_face.units_per_em()? as i32;
        let prefered_cmap_encoding_subtable = find_best_cmap_subtable(&ttfp_face);
        let gsub = ttfp_face.table_data(Tag::from_bytes(b"GSUB")).and_then(SubstTable::parse);
        let gpos = ttfp_face.table_data(Tag::from_bytes(b"GPOS")).and_then(PosTable::parse);
        let kern = load_sanitized_table(&ttfp_face, Tag::from_bytes(b"kern"));
        let morx = load_sanitized_table(&ttfp_face, Tag::from_bytes(b"morx"));
        let mort = load_sanitized_table(&ttfp_face, Tag::from_bytes(b"mort"));
        let kerx = load_sanitized_table(&ttfp_face, Tag::from_bytes(b"kerx"));
        let ankr = load_sanitized_table(&ttfp_face, Tag::from_bytes(b"ankr"));
        let trak = load_sanitized_table(&ttfp_face, Tag::from_bytes(b"trak"));
        let feat = load_sanitized_table(&ttfp_face, Tag::from_bytes(b"feat"));
        Some(Face {
            ttfp_face,
            units_per_em: upem,
            pixels_per_em: None,
            points_per_em: None,
            prefered_cmap_encoding_subtable,
            gsub,
            gpos,
            kern,
            morx,
            mort,
            kerx,
            ankr,
            trak,
            feat,
        })
    }

    #[inline]
    pub(crate) fn from_ptr(face: *const ffi::rb_face_t) -> &'static Face<'static> {
        unsafe { &*(face as *const Face) }
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const ffi::rb_face_t {
        self as *const _ as *const ffi::rb_face_t
    }

    #[inline]
    pub(crate) fn units_per_em(&self) -> i32 {
        self.units_per_em
    }

    #[inline]
    pub(crate) fn pixels_per_em(&self) -> Option<(u16, u16)> {
        self.pixels_per_em
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
    }

    pub(crate) fn has_glyph(&self, c: u32) -> bool {
        self.glyph_index(c).is_some()
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

    pub(crate) fn glyph_h_advance(&self, glyph: GlyphId) -> i32 {
        self.glyph_advance(glyph, false) as i32
    }

    pub(crate) fn glyph_v_advance(&self, glyph: GlyphId) -> i32 {
        -(self.glyph_advance(glyph, true) as i32)
    }

    fn glyph_advance(&self, glyph: GlyphId, is_vertical: bool) -> u32 {
        let face = &self.ttfp_face;
        if face.is_variable() &&
           face.has_non_default_variation_coordinates() &&
          !face.has_table(ttf_parser::TableName::HorizontalMetricsVariations) &&
          !face.has_table(ttf_parser::TableName::VerticalMetricsVariations)
        {
            return match face.glyph_bounding_box(glyph) {
                Some(bbox) => {
                    (if is_vertical {
                        bbox.y_max + bbox.y_min
                    } else {
                        bbox.x_max + bbox.x_min
                    }) as u32
                }
                None => 0,
            };
        }

        if is_vertical && face.has_table(ttf_parser::TableName::VerticalMetrics) {
            face.glyph_ver_advance(glyph).unwrap_or(0) as u32
        } else if !is_vertical && face.has_table(ttf_parser::TableName::HorizontalMetrics) {
            face.glyph_hor_advance(glyph).unwrap_or(0) as u32
        } else {
            face.units_per_em().unwrap_or(1000) as u32
        }
    }

    pub(crate) fn glyph_h_origin(&self, glyph: GlyphId) -> i32 {
        self.glyph_h_advance(glyph) / 2
    }

    pub(crate) fn glyph_v_origin(&self, glyph: GlyphId) -> i32 {
        match self.ttfp_face.glyph_y_origin(glyph) {
            Some(y) => i32::from(y),
            None => self.glyph_extents(glyph).map_or(0, |ext| ext.y_bearing)
                + self.glyph_side_bearing(glyph, true)
        }
    }

    pub(crate) fn glyph_side_bearing(&self, glyph: GlyphId, is_vertical: bool) -> i32 {
        let face = &self.ttfp_face;
        if  face.is_variable() &&
           !face.has_table(ttf_parser::TableName::HorizontalMetricsVariations) &&
           !face.has_table(ttf_parser::TableName::VerticalMetricsVariations)
        {
            return match face.glyph_bounding_box(glyph) {
                Some(bbox) => (if is_vertical { bbox.x_min } else { bbox.y_min }) as i32,
                None => 0,
            }
        }

        if is_vertical {
            face.glyph_ver_side_bearing(glyph).unwrap_or(0) as i32
        } else {
            face.glyph_hor_side_bearing(glyph).unwrap_or(0) as i32
        }
    }

    pub(crate) fn glyph_extents(&self, glyph: GlyphId) -> Option<ffi::rb_glyph_extents_t> {
        let pixels_per_em = match self.pixels_per_em {
            Some(ppem) => ppem.0,
            None => std::u16::MAX,
        };

        if let Some(img) = self.ttfp_face.glyph_raster_image(glyph, pixels_per_em) {
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

        let bbox = self.ttfp_face.glyph_bounding_box(glyph)?;
        Some(ffi::rb_glyph_extents_t {
            x_bearing: i32::from(bbox.x_min),
            y_bearing: i32::from(bbox.y_max),
            width: i32::from(bbox.width()),
            height: i32::from(bbox.y_min - bbox.y_max),
        })
    }

    pub(crate) fn glyph_name(&self, glyph: GlyphId) -> Option<&str> {
        self.ttfp_face.glyph_name(glyph)
    }

    pub(crate) fn glyph_props(&self, glyph: GlyphId) -> u16 {
        match self.ttfp_face.glyph_class(glyph) {
            Some(GlyphClass::Base) => GlyphPropsFlags::BASE_GLYPH.bits(),
            Some(GlyphClass::Ligature) => GlyphPropsFlags::LIGATURE.bits(),
            Some(GlyphClass::Mark) => {
                let class = self.ttfp_face.glyph_mark_attachment_class(glyph).0;
                (class << 8) | GlyphPropsFlags::MARK.bits()
            }
            _ => 0,
        }
    }

    pub(crate) fn layout_table(&self, table_index: TableIndex) -> Option<SubstPosTable<'a>> {
        match table_index {
            TableIndex::GSUB => self.gsub.map(|table| table.0),
            TableIndex::GPOS => self.gpos.map(|table| table.0),
        }
    }

    pub(crate) fn layout_tables(&self) -> impl Iterator<Item = (TableIndex, SubstPosTable<'a>)> + '_ {
        TableIndex::iter().filter_map(move |idx| self.layout_table(idx).map(|table| (idx, table)))
    }

    fn get_table_blob(&self, tag: Tag) -> &Blob<'a> {
        match &tag.to_bytes() {
            b"kern" => &self.kern,
            b"morx" => &self.morx,
            b"mort" => &self.mort,
            b"kerx" => &self.kerx,
            b"ankr" => &self.ankr,
            b"trak" => &self.trak,
            b"feat" => &self.feat,
            _ => panic!("invalid table"),
        }
    }
}

fn load_sanitized_table<'a>(face: &ttf_parser::Face<'a>, tag: Tag) -> Blob<'a> {
    let data = face.table_data(tag).unwrap_or(&[]);
    unsafe {
        let input = Blob::from_data(data);
        let ptr = ffi::rb_face_sanitize_table(input.as_ptr(), tag, u32::from(face.number_of_glyphs()));

        // Sanitization takes ownership and may return the same blob we pass in.
        std::mem::forget(input);
        Blob::from_ptr(ptr)
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
pub extern "C" fn rb_face_get_glyph_count(face: *const ffi::rb_face_t) -> i32 {
    Face::from_ptr(face).ttfp_face.number_of_glyphs() as i32
}

#[no_mangle]
pub extern "C" fn rb_face_get_ptem(face: *const ffi::rb_face_t) -> f32 {
    Face::from_ptr(face).points_per_em.unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn rb_face_get_table_blob(face: *const ffi::rb_face_t, tag: Tag) -> *mut ffi::rb_blob_t {
    Face::from_ptr(face).get_table_blob(tag).as_ptr()
}

#[no_mangle]
pub extern "C" fn rb_face_get_glyph_contour_point_for_origin(
    _: *const ffi::rb_face_t,
    _: ffi::rb_codepoint_t ,
    _: u32,
    _: ffi::rb_direction_t ,
    x: *mut ffi::rb_position_t,
    y: *mut ffi::rb_position_t,
) -> ffi::rb_bool_t {
    unsafe {
        *x = 0;
        *y = 0;
    }
    0
}
