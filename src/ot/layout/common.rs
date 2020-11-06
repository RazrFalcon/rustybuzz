//! Common tables for OpenType layout.

use std::cmp::Ordering;
use std::convert::TryFrom;

use ttf_parser::parser::{
    FromData, LazyArray16, LazyArray32, Offset, Offset16, Offset32, Offsets16, Stream, TryNumFrom,
};
use ttf_parser::GlyphId;

use crate::Tag;
use crate::font::Font;

/// A GSUB or GPOS table.
#[derive(Clone, Copy)]
pub struct SubstPosTable<'a> {
    scripts: RecordList<'a>,
    features: RecordList<'a>,
    lookups: LookupList<'a>,
    variations: Option<FeatureVariations<'a>>,
}

impl<'a> SubstPosTable<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);

        let major_version = s.read::<u16>()?;
        let minor_version = s.read::<u16>()?;
        if major_version != 1 {
            return None;
        }

        let scripts = RecordList::parse(s.read_offset16_data()?)?;
        let features = RecordList::parse(s.read_offset16_data()?)?;
        let lookups = LookupList::parse(s.read_offset16_data()?)?;

        let mut variations = None;
        if minor_version >= 1 {
            variations = FeatureVariations::parse(s.read_offset32_data()?);
        }

        Some(Self { scripts, features, lookups, variations })
    }

    pub fn feature_count(&self) -> u16 {
        self.features.len()
    }

    pub fn get_feature(&self, index: FeatureIndex) -> Option<Feature<'a>> {
        Feature::parse(self.features.get_data(index.0)?)
    }

    pub fn get_feature_tag(&self, index: FeatureIndex) -> Option<Tag> {
        self.features.get_tag(index.0)
    }

    pub fn get_feature_variation(
        &self,
        feature_index: FeatureIndex,
        var_index: VariationIndex,
    ) -> Option<Feature<'a>> {
        self.variations
            .and_then(|var| var.find_substitute(feature_index, var_index))
            .or_else(|| self.get_feature(feature_index))
    }

    pub fn find_variation_index(&self, coords: &[i32]) -> Option<VariationIndex> {
        self.variations?.find_index(coords)
    }
}

/// A type-safe wrapper for a lookup type.
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct LookupType(pub u16);

impl FromData for LookupType {
    const SIZE: usize = 2;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        u16::parse(data).map(Self)
    }
}

/// A type-safe wrapper for a lookup index.
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct LookupIndex(pub u16);

impl FromData for LookupIndex {
    const SIZE: usize = 2;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        u16::parse(data).map(Self)
    }
}

/// A type-safe wrapper for a feature index.
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct FeatureIndex(pub u16);

impl FromData for FeatureIndex {
    const SIZE: usize = 2;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        u16::parse(data).map(Self)
    }
}

/// A type-safe wrapper for a variation index used by GSUB/GPOS tables.
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct VariationIndex(pub u32);

#[derive(Clone, Copy, Debug)]
struct RecordList<'a> {
    data: &'a [u8],
    records: LazyArray16<'a, TagRecord>
}

impl<'a> RecordList<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let records = s.read_array16(count)?;
        Some(Self { data, records })
    }

    fn len(&self) -> u16 {
        self.records.len()
    }

    fn get_tag(&self, index: u16) -> Option<Tag> {
        self.records.get(index).map(|record| record.tag)
    }

    fn get_data(&self, index: u16) -> Option<&'a [u8]> {
        let offset = self.records.get(index)?.offset.to_usize();
        self.data.get(offset..)
    }
}

#[derive(Clone, Copy, Debug)]
struct TagRecord {
    tag: Tag,
    offset: Offset16,
}

impl FromData for TagRecord {
    const SIZE: usize = 6;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            tag: s.read()?,
            offset: s.read()?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Feature<'a> {
    pub params: Offset16,
    pub lookup_indices: LazyArray16<'a, LookupIndex>,
}

impl<'a> Feature<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let params = s.read::<Offset16>()?;
        let count = s.read::<u16>()?;
        let lookup_indices = s.read_array16(count)?;
        Some(Self { params, lookup_indices })
    }
}

#[derive(Clone, Copy)]
struct LookupList<'a> {
    offsets: Offsets16<'a, Offset16>,
}

impl<'a> LookupList<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let offsets = s.read_offsets16(count, data)?;
        Some(LookupList { offsets })
    }

    fn len(&self) -> usize {
        usize::from(self.offsets.len())
    }

    fn get(&self, index: LookupIndex) -> Option<Lookup<'a>> {
        Lookup::parse(self.offsets.slice(index.0)?)
    }
}

#[derive(Clone, Copy)]
struct Lookup<'a> {
    kind: LookupType,
    flags: LookupFlags,
    offsets: Offsets16<'a, Offset16>,
    mark_filtering_set: Option<u16>,
}

impl<'a> Lookup<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let kind = s.read::<LookupType>()?;
        let flags = s.read::<LookupFlags>()?;
        let count = s.read::<u16>()?;
        let offsets = s.read_offsets16(count, data)?;

        let mut mark_filtering_set: Option<u16> = None;
        if flags.contains(LookupFlags::USE_MARK_FILTERING_SET) {
            mark_filtering_set = Some(s.read()?);
        }

        Some(Self {
            kind,
            flags,
            offsets,
            mark_filtering_set,
        })
    }
}

bitflags::bitflags! {
    #[derive(Default)]
    pub struct LookupFlags: u16 {
        const RIGHT_TO_LEFT          = 0x0001;
        const IGNORE_BASE_GLYPHS     = 0x0002;
        const IGNORE_LIGATURES       = 0x0004;
        const IGNORE_MARKS           = 0x0008;
        const IGNORE_FLAGS           = 0x000E;
        const USE_MARK_FILTERING_SET = 0x0010;
        const MARK_ATTACHMENT_TYPE   = 0xFF00;
    }
}

impl FromData for LookupFlags {
    const SIZE: usize = 2;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        u16::parse(data).map(Self::from_bits_truncate)
    }
}

pub fn parse_extension_lookup<'a, T: 'a>(
    data: &'a [u8],
    parse: impl FnOnce(&'a [u8], u16) -> Option<T>,
) -> Option<T> {
    let mut s = Stream::new(data);
    let format: u16 = s.read()?;
    match format {
        1 => {
            let kind = s.read::<u16>()?;
            let offset = s.read::<Offset32>()?.to_usize();
            parse(data.get(offset..)?, kind)
        }
        _ => None,
    }
}

#[derive(Clone, Copy, Debug)]
struct FeatureVariations<'a> {
    data: &'a [u8],
    records: LazyArray32<'a, FeatureVariationRecord>,
}

impl<'a> FeatureVariations<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let major_version = s.read::<u16>()?;
        s.skip::<u16>(); // minor version
        if major_version != 1 {
            return None;
        }

        let count = s.read::<u32>()?;
        let records = s.read_array32(count)?;
        Some(Self { data, records })
    }

    fn find_index(&self, coords: &[i32]) -> Option<VariationIndex> {
        for i in 0..self.records.len() {
            let record = self.records.get(i)?;
            let offset = record.conditions.to_usize();
            let set = ConditionSet::parse(self.data.get(offset..)?)?;
            if set.evaluate(coords) {
                return Some(VariationIndex(i));
            }
        }
        None
    }

    fn find_substitute(
        &self,
        feature_index: FeatureIndex,
        var_index: VariationIndex,
    ) -> Option<Feature<'a>> {
        let offset = self.records.get(var_index.0)?.substitutions.to_usize();
        let subst = FeatureTableSubstitution::parse(self.data.get(offset..)?)?;
        subst.find_substitute(feature_index)
    }
}

#[derive(Clone, Copy, Debug)]
struct FeatureVariationRecord {
    conditions: Offset32,
    substitutions: Offset32,
}

impl FromData for FeatureVariationRecord {
    const SIZE: usize = 8;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            conditions: s.read()?,
            substitutions: s.read()?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct ConditionSet<'a> {
    data: &'a [u8],
    conditions: LazyArray16<'a, Offset32>,
}

impl<'a> ConditionSet<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let conditions = s.read_array16(count)?;
        Some(Self { data, conditions })
    }

    fn evaluate(&self, coords: &[i32]) -> bool {
        self.conditions.into_iter().all(|offset| {
            self.data.get(offset.to_usize()..)
                .and_then(Condition::parse)
                .map_or(false, |c| c.evaluate(coords))
        })
    }
}

#[derive(Clone, Copy, Debug)]
enum Condition {
    Format1 {
        axis_index: u16,
        filter_range_min: i16,
        filter_range_max: i16,
    }
}

impl Condition {
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let axis_index = s.read::<u16>()?;
                let filter_range_min = s.read::<i16>()?;
                let filter_range_max = s.read::<i16>()?;
                Self::Format1 { axis_index, filter_range_min, filter_range_max }
            }
            _ => return None,
        })
    }

    fn evaluate(&self, coords: &[i32]) -> bool {
        let Self::Format1 { axis_index, filter_range_min, filter_range_max } = *self;
        let coord = coords.get(usize::from(axis_index)).copied().unwrap_or(0);
        i32::from(filter_range_min) <= coord && coord <= i32::from(filter_range_max)
    }
}

#[derive(Clone, Copy, Debug)]
struct FeatureTableSubstitution<'a> {
    data: &'a [u8],
    records: LazyArray16<'a, FeatureTableSubstitutionRecord>,
}

impl<'a> FeatureTableSubstitution<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let major_version = s.read::<u16>()?;
        s.skip::<u16>(); // minor version
        if major_version != 1 {
            return None;
        }

        let count = s.read::<u16>()?;
        let records = s.read_array16(count)?;
        Some(Self { data, records })
    }

    fn find_substitute(&self, feature_index: FeatureIndex) -> Option<Feature<'a>> {
        for record in self.records {
            if record.feature_index == feature_index {
                let offset = record.feature.to_usize();
                return Feature::parse(self.data.get(offset..)?);
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug)]
struct FeatureTableSubstitutionRecord {
    feature_index: FeatureIndex,
    feature: Offset32,
}

impl FromData for FeatureTableSubstitutionRecord {
    const SIZE: usize = 6;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            feature_index: s.read()?,
            feature: s.read()?,
        })
    }
}

/// A table that defines which glyph ids are covered by some lookup.
///
/// https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#coverage-table
#[derive(Clone, Copy, Debug)]
pub enum Coverage<'a> {
    Format1 { glyphs: LazyArray16<'a, GlyphId> },
    Format2 { records: LazyArray16<'a, RangeRecord> },
}

impl<'a> Coverage<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let count = s.read::<u16>()?;
                let glyphs = s.read_array16(count)?;
                Self::Format1 { glyphs }
            }
            2 => {
                let count = s.read::<u16>()?;
                let records = s.read_array16(count)?;
                Self::Format2 { records }
            }
            _ => return None,
        })
    }

    /// Returns the coverage index of the glyph or `None` if it is not covered.
    pub fn get(&self, glyph: GlyphId) -> Option<u16> {
        match self {
            Self::Format1 { glyphs } => glyphs.binary_search(&glyph).map(|p| p.0),
            Self::Format2 { records } => {
                let record = RangeRecord::binary_search(records, glyph)?;
                let offset = glyph.0 - record.start.0;
                record.value.checked_add(offset)
            }
        }
    }
}

/// A table that defines which classes glyph ids belong to.
///
/// https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table
#[derive(Clone, Copy, Debug)]
pub enum ClassDef<'a> {
    Format1 {
        start: GlyphId,
        classes: LazyArray16<'a, Class>,
    },
    Format2 {
        records: LazyArray16<'a, RangeRecord>,
    },
}

impl<'a> ClassDef<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let start = s.read::<GlyphId>()?;
                let count = s.read::<u16>()?;
                let classes = s.read_array16(count)?;
                Self::Format1 { start, classes }
            },
            2 => {
                let count = s.read::<u16>()?;
                Self::Format2 { records: s.read_array16(count)? }
            },
            _ => return None,
        })
    }

    /// Returns the glyph class of the glyph (zero if it is not defined).
    pub fn get(&self, glyph: GlyphId) -> Class {
        let class = match self {
            Self::Format1 { start, classes } => {
                glyph.0.checked_sub(start.0)
                    .and_then(|index| classes.get(index))
            }
            Self::Format2 { records } => {
                RangeRecord::binary_search(records, glyph)
                    .map(|record| Class(record.value))
            }
        };
        class.unwrap_or(Class(0))
    }
}

/// A type-safe wrapper for a glyph class.
#[repr(transparent)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default, Debug)]
pub struct Class(pub u16);

impl FromData for Class {
    const SIZE: usize = 2;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        u16::parse(data).map(Self)
    }
}

/// A record that describes a range of glyph ids.
#[derive(Clone, Copy, Debug)]
pub struct RangeRecord {
    start: GlyphId,
    end: GlyphId,
    value: u16,
}

impl RangeRecord {
    fn binary_search(records: &LazyArray16<RangeRecord>, glyph: GlyphId) -> Option<RangeRecord> {
        records.binary_search_by(|record| {
            if glyph < record.start {
                Ordering::Greater
            } else if glyph <= record.end {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        }).map(|p| p.1)
    }
}

impl FromData for RangeRecord {
    const SIZE: usize = 6;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            start: s.read::<GlyphId>()?,
            end: s.read::<GlyphId>()?,
            value: s.read::<u16>()?,
        })
    }
}

/// A device table.
///
/// https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#device-and-variationindex-tables
#[derive(Clone, Copy, Debug)]
pub enum Device<'a> {
    Hinting(HintingDevice<'a>),
    Variation(VariationDevice),
}

impl<'a> Device<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let first = s.read::<u16>()?;
        let second = s.read::<u16>()?;
        let format = s.read::<u16>()?;
        Some(match format {
            1..=3 => {
                let start_size = first;
                let end_size = second;
                let count = 1 + (end_size - start_size) >> (4 - format);
                let delta_values = s.read_array16(count)?;
                Self::Hinting(HintingDevice {
                    start_size,
                    end_size,
                    delta_format: format,
                    delta_values,
                })
            }
            0x8000 => Self::Variation(VariationDevice {
                outer_index: first,
                inner_index: second,
            }),
            _ => return None,
        })
    }

    pub fn get_x_delta(&self, font: &Font) -> Option<i32> {
        match self {
            Self::Hinting(hinting) => hinting.get_x_delta(font),
            Self::Variation(variation) => variation.get_x_delta(font),
        }
    }

    pub fn get_y_delta(&self, font: &Font) -> Option<i32> {
        match self {
            Self::Hinting(hinting) => hinting.get_y_delta(font),
            Self::Variation(variation) => variation.get_y_delta(font),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HintingDevice<'a> {
    start_size: u16,
    end_size: u16,
    delta_format: u16,
    delta_values: LazyArray16<'a, u16>,
}

impl HintingDevice<'_> {
    pub fn get_x_delta(&self, font: &Font) -> Option<i32> {
        let ppem = font.pixels_per_em().map(|(x, _)| x)?;
        let scale = font.units_per_em();
        self.get_delta(ppem, scale)
    }

    pub fn get_y_delta(&self, font: &Font) -> Option<i32> {
        let ppem = font.pixels_per_em().map(|(_, y)| y)?;
        let scale = font.units_per_em();
        self.get_delta(ppem, scale)
    }

    fn get_delta(&self, ppem: u16, scale: i32) -> Option<i32> {
        let f = self.delta_format;
        debug_assert!(matches!(f, 1..=3));

        if ppem == 0 || ppem < self.start_size || ppem > self.end_size {
            return None;
        }

        let s = ppem - self.start_size;
        let byte = self.delta_values.get(s >> (4 - f)).unwrap();
        let bits = byte >> (16 - (((s & ((1 << (4 - f)) - 1)) + 1) << f));
        let mask = 0xFFFF >> (16 - (1 << f));

        let mut delta = i64::from(bits & mask);
        if delta >= i64::from(mask + 1 >> 1) {
            delta -= i64::from(mask + 1);
        }

        i32::try_from(delta * i64::from(scale) / i64::from(ppem)).ok()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VariationDevice {
    outer_index: u16,
    inner_index: u16,
}

impl VariationDevice {
    pub fn get_x_delta(&self, font: &Font) -> Option<i32> {
        self.get_delta(font)
    }

    pub fn get_y_delta(&self, font: &Font) -> Option<i32> {
        self.get_delta(font)
    }

    fn get_delta(&self, font: &Font) -> Option<i32> {
        font.ttfp_face
            .gdef_variation_delta(self.outer_index, self.inner_index)
            .and_then(|float| i32::try_num_from(float.round()))
    }
}
