//! Common tables for OpenType layout.

use std::cmp::Ordering;
use std::convert::TryFrom;

use ttf_parser::GlyphId;
use ttf_parser::parser::{FromData, LazyArray16, Offset, Offset16, Offsets16, Offset32, Stream};

/// A GSUB or GPOS table.
#[derive(Clone, Copy)]
pub struct SubstPosTable<'a> {
    lookups: LookupList<'a>,
}

impl<'a> SubstPosTable<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);

        let major_version = s.read::<u16>()?;
        let minor_version = s.read::<u16>()?;
        if major_version != 1 {
            return None;
        }

        s.skip::<Offset16>(); // TODO: script list
        s.skip::<Offset16>(); // TODO: feature list
        let lookups_offset = s.read::<Offset16>()?.to_usize();
        if minor_version >= 1 {
            s.skip::<Offset32>(); // TODO: feature variations
        }

        Some(Self {
            lookups: LookupList::parse(data.get(lookups_offset..)?)?,
        })
    }

    pub fn lookups(&self) -> LookupList {
        self.lookups
    }
}

#[derive(Clone, Copy)]
pub struct LookupList<'a> {
    offsets: Offsets16<'a, Offset16>,
}

impl<'a> LookupList<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let offsets = s.read_offsets16(count, data)?;
        Some(LookupList { offsets })
    }

    pub fn len(&self) -> usize {
        self.offsets.len() as usize
    }

    pub fn get(&self, index: usize) -> Option<Lookup<'a>> {
        Lookup::parse(self.offsets.slice(u16::try_from(index).ok()?)?)
    }
}

#[derive(Clone, Copy)]
pub struct Lookup<'a> {
    pub type_: u16,
    pub flags: LookupFlags,
    pub offsets: Offsets16<'a, Offset16>,
    pub mark_filtering_set: Option<u16>,
}

impl<'a> Lookup<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let type_ = s.read::<u16>()?;
        let flags = LookupFlags(s.read()?);
        let count = s.read::<u16>()?;
        let offsets = s.read_offsets16(count, data)?;

        let mut mark_filtering_set: Option<u16> = None;
        if flags.use_mark_filtering_set() {
            mark_filtering_set = Some(s.read()?);
        }

        Some(Self {
            type_,
            flags,
            offsets,
            mark_filtering_set,
        })
    }
}

#[derive(Clone, Copy)]
pub struct LookupFlags(u16);

impl LookupFlags {
    #[inline]
    pub fn use_mark_filtering_set(self) -> bool {
        self.0 & (1 << 4) != 0
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
        let count: u16 = s.read()?;
        Some(match format {
            1 => Self::Format1 { glyphs: s.read_array16(count)? },
            2 => Self::Format2 { records: s.read_array16(count)? },
            _ => return None,
        })
    }

    /// Returns the coverage index of the glyph or `None` if it is not covered.
    pub fn get(&self, glyph_id: GlyphId) -> Option<u16> {
        match self {
            Self::Format1 { glyphs } => glyphs.binary_search(&glyph_id).map(|p| p.0),
            Self::Format2 { records } => {
                let record = records.binary_search_by(|record| {
                    if glyph_id < record.start {
                        Ordering::Greater
                    } else if glyph_id <= record.end {
                        Ordering::Equal
                    } else {
                        Ordering::Less
                    }
                })?.1;

                // Can't underflow because we got back `Ordering::Equal`.
                let offset = glyph_id.0 - record.start.0;
                record.value.checked_add(offset)
            }
        }
    }
}

/// A record that describes a range of glyph ids.
#[derive(Clone, Copy, Debug)]
pub struct RangeRecord {
    start: GlyphId,
    end: GlyphId,
    value: u16,
}

impl FromData for RangeRecord {
    const SIZE: usize = 6;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(RangeRecord {
            start: s.read::<GlyphId>()?,
            end: s.read::<GlyphId>()?,
            value: s.read::<u16>()?,
        })
    }
}
