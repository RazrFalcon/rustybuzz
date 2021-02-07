use ttf_parser::{Tag, GlyphClass, GlyphId};

use crate::Variation;
use crate::ot::TableIndex;
use crate::buffer::GlyphPropsFlags;
use crate::tables::gpos::PosTable;
use crate::tables::gsub::SubstTable;
use crate::tables::gsubgpos::SubstPosTable;
use crate::tables::{ankr, feat, kern, kerx, morx, trak};


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


/// A font face handle.
pub struct Face<'a> {
    pub(crate) ttfp_face: ttf_parser::Face<'a>,
    units_per_em: i32,
    pixels_per_em: Option<(u16, u16)>,
    pub(crate) points_per_em: Option<f32>,
    prefered_cmap_encoding_subtable: Option<u16>,
    pub(crate) gsub: Option<SubstTable<'a>>,
    pub(crate) gpos: Option<PosTable<'a>>,
    pub(crate) kern: Option<kern::Subtables<'a>>,
    pub(crate) kerx: Option<kerx::Subtables<'a>>,
    pub(crate) ankr: Option<ankr::Table<'a>>,
    pub(crate) feat: Option<feat::Table<'a>>,
    pub(crate) trak: Option<trak::Table<'a>>,
    pub(crate) morx: Option<morx::Chains<'a>>,
}

impl<'a> Face<'a> {
    /// Creates a new `Face` from data.
    ///
    /// Data will be referenced, not owned.
    pub fn from_slice(data: &'a [u8], face_index: u32) -> Option<Self> {
        let ttfp_face = ttf_parser::Face::from_slice(data, face_index).ok()?;
        Some(Face {
            units_per_em: ttfp_face.units_per_em()? as i32,
            pixels_per_em: None,
            points_per_em: None,
            prefered_cmap_encoding_subtable: find_best_cmap_subtable(&ttfp_face),
            gsub: ttfp_face.table_data(Tag::from_bytes(b"GSUB")).and_then(SubstTable::parse),
            gpos: ttfp_face.table_data(Tag::from_bytes(b"GPOS")).and_then(PosTable::parse),
            kern: ttfp_face.table_data(Tag::from_bytes(b"kern")).and_then(kern::parse),
            kerx: ttfp_face.table_data(Tag::from_bytes(b"kerx"))
                .and_then(|data| kerx::parse(data, ttfp_face.number_of_glyphs())),
            ankr: ttfp_face.table_data(Tag::from_bytes(b"ankr"))
                .and_then(|data| ankr::Table::parse(data, ttfp_face.number_of_glyphs())),
            morx: ttfp_face.table_data(Tag::from_bytes(b"morx"))
                .and_then(|data| morx::Chains::parse(data, ttfp_face.number_of_glyphs())),
            trak: ttfp_face.table_data(Tag::from_bytes(b"trak")).and_then(trak::Table::parse),
            feat: ttfp_face.table_data(Tag::from_bytes(b"feat")).and_then(feat::Table::parse),
            ttfp_face,
        })
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

    pub(crate) fn glyph_extents(&self, glyph: GlyphId) -> Option<GlyphExtents> {
        let pixels_per_em = match self.pixels_per_em {
            Some(ppem) => ppem.0,
            None => core::u16::MAX,
        };

        if let Some(img) = self.ttfp_face.glyph_raster_image(glyph, pixels_per_em) {
            // HarfBuzz also supports only PNG.
            if img.format == ttf_parser::RasterImageFormat::PNG {
                let scale = self.units_per_em as f32 / img.pixels_per_em as f32;
                return Some(GlyphExtents {
                    x_bearing: (f32::from(img.x) * scale).round() as i32,
                    y_bearing: ((f32::from(img.y) + f32::from(img.height)) * scale).round() as i32,
                    width: (f32::from(img.width) * scale).round() as i32,
                    height: (-f32::from(img.height) * scale).round() as i32,
                });
            }
        }

        let bbox = self.ttfp_face.glyph_bounding_box(glyph)?;
        Some(GlyphExtents {
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

    pub(crate) fn layout_table(&self, table_index: TableIndex) -> Option<&SubstPosTable<'a>> {
        match table_index {
            TableIndex::GSUB => self.gsub.as_ref().map(|table| &table.inner),
            TableIndex::GPOS => self.gpos.as_ref().map(|table| &table.inner),
        }
    }

    pub(crate) fn layout_tables(&self) -> impl Iterator<Item = (TableIndex, &SubstPosTable<'a>)> + '_ {
        TableIndex::iter().filter_map(move |idx| self.layout_table(idx).map(|table| (idx, table)))
    }
}

#[derive(Clone, Copy, Default)]
pub struct GlyphExtents {
    pub x_bearing: i32,
    pub y_bearing: i32,
    pub width: i32,
    pub height: i32,
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
