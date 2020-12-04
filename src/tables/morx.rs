// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6morx.html

use ttf_parser::parser::{Stream, FromData, LazyArray32, NumFrom, Offset32, Offset};
use ttf_parser::GlyphId;

use crate::Mask;
use crate::tables::aat;


#[derive(Clone, Copy)]
pub struct Chains<'a> {
    index: u32,
    len: u32,
    stream: Stream<'a>,
    number_of_glyphs: u16,
}

impl<'a> Chains<'a> {
    pub fn parse(data: &'a [u8], number_of_glyphs: u16) -> Option<Self> {
        let mut s = Stream::new(data);

        s.skip::<u16>(); // version
        s.skip::<u16>(); // reserved
        let count: u32 = s.read()?;

        Some(Chains {
            index: 0,
            len: count,
            stream: s,
            number_of_glyphs,
        })
    }
}

impl<'a> Iterator for Chains<'a> {
    type Item = Chain<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.len {
            return None;
        }

        if self.stream.at_end() {
            return None;
        }

        let default_flags: u32 = self.stream.read()?;
        let len: u32 = self.stream.read()?;
        let features_count: u32 = self.stream.read()?;
        let subtables_count: u32 = self.stream.read()?;

        let features: LazyArray32<Feature> = self.stream.read_array32(features_count)?;

        const HEADER_LEN: usize = 16;
        let len = usize::num_from(len)
            .checked_sub(HEADER_LEN)?
            .checked_sub(Feature::SIZE * usize::num_from(features_count))?;

        let subtables_data = self.stream.read_bytes(len)?;

        Some(Chain {
            default_flags,
            features,
            subtables_count,
            subtables_data,
            number_of_glyphs: self.number_of_glyphs,
        })
    }
}


#[derive(Clone, Copy)]
pub struct Chain<'a> {
    default_flags: Mask,
    features: LazyArray32<'a, Feature>,
    subtables_count: u32,
    subtables_data: &'a [u8],
    number_of_glyphs: u16,
}

impl<'a> Chain<'a> {
    pub fn default_flags(&self) -> Mask {
        self.default_flags
    }

    pub fn features(&self) -> LazyArray32<'a, Feature> {
        self.features
    }

    pub fn subtables(&self) -> Subtables<'a> {
        Subtables {
            index: 0,
            len: self.subtables_count,
            stream: Stream::new(self.subtables_data),
            number_of_glyphs: self.number_of_glyphs,
        }
    }
}


#[derive(Clone, Copy)]
pub struct Feature {
    pub kind: u16,
    pub setting: u16,
    pub enable_flags: u32,
    pub disable_flags: u32,
}

impl FromData for Feature {
    const SIZE: usize = 12;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Feature {
            kind: s.read()?,
            setting: s.read()?,
            enable_flags: s.read()?,
            disable_flags: s.read()?,
        })
    }
}


#[derive(Clone, Copy)]
pub struct Subtables<'a> {
    index: u32,
    len: u32,
    stream: Stream<'a>,
    number_of_glyphs: u16,
}

impl<'a> Iterator for Subtables<'a> {
    type Item = Subtable<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.len {
            return None;
        }

        if self.stream.at_end() {
            return None;
        }

        let len: u32 = self.stream.read()?;
        let coverage: u8 = self.stream.read()?;
        self.stream.skip::<u16>(); // reserved
        let kind: u8 = self.stream.read()?;
        let feature_flags: u32 = self.stream.read()?;

        const HEADER_LEN: usize = 12;
        let len = usize::num_from(len).checked_sub(HEADER_LEN)?;
        let subtables_data = self.stream.read_bytes(len)?;

        let kind = match kind {
            0 => {
                let table = aat::StateTable::parse(subtables_data, self.number_of_glyphs)?;
                SubtableKind::Rearrangement(table)
            }
            1 => {
                let table = ContextualSubtable::parse(subtables_data, self.number_of_glyphs)?;
                SubtableKind::Contextual(table)
            }
            2 => {
                let table = LigatureSubtable::parse(subtables_data, self.number_of_glyphs)?;
                SubtableKind::Ligature(table)
            }
            // 3 - reserved
            4 => {
                SubtableKind::NonContextual(aat::Lookup::parse(subtables_data)?)
            }
            5 => {
                let table = InsertionSubtable::parse(subtables_data, self.number_of_glyphs)?;
                SubtableKind::Insertion(table)
            }
            _ => return None,
        };

        Some(Subtable {
            kind,
            coverage,
            feature_flags,
        })
    }
}


pub struct Subtable<'a> {
    pub kind: SubtableKind<'a>,
    pub coverage: u8,
    pub feature_flags: u32,
}

impl Subtable<'_> {
    pub fn is_logical(&self) -> bool {
        self.coverage & 0x10 != 0
    }

    pub fn is_all_directions(&self) -> bool {
        self.coverage & 0x20 != 0
    }

    pub fn is_backwards(&self) -> bool {
        self.coverage & 0x40 != 0
    }

    pub fn is_vertical(&self) -> bool {
        self.coverage & 0x80 != 0
    }
}


pub enum SubtableKind<'a> {
    Rearrangement(aat::StateTable<'a>),
    Contextual(ContextualSubtable<'a>),
    Ligature(LigatureSubtable<'a>),
    NonContextual(aat::Lookup<'a>),
    Insertion(InsertionSubtable<'a>),
}


pub struct ContextualSubtable<'a> {
    pub offsets_data: &'a [u8],
    pub machine: aat::StateTable<'a>,
    pub offsets: LazyArray32<'a, Offset32>,
}

impl<'a> ContextualSubtable<'a> {
    fn parse(data: &'a [u8], number_of_glyphs: u16) -> Option<Self> {
        let mut s = Stream::new(data);

        let machine = aat::StateTable::parse(data, number_of_glyphs)?;
        s.advance(aat::StateTable::SIZE);
        let offset = s.read::<Offset32>()?.to_usize();

        // The offsets list is unsized.
        let offsets_data = data.get(offset..)?;
        let offsets = LazyArray32::new(offsets_data);

        Some(ContextualSubtable {
            offsets_data,
            machine,
            offsets,
        })
    }
}


#[derive(Copy, Clone)]
pub struct ContextualEntry {
    pub mark_index: u16,
    pub current_index: u16,
}

impl FromData for ContextualEntry {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(ContextualEntry {
            mark_index: s.read()?,
            current_index: s.read()?,
        })
    }
}


pub struct LigatureSubtable<'a> {
    pub machine: aat::StateTable<'a>,
    pub ligature_actions: LazyArray32<'a, u32>,
    pub components: LazyArray32<'a, u16>,
    pub ligatures: LazyArray32<'a, GlyphId>,
}

impl<'a> LigatureSubtable<'a> {
    fn parse(data: &'a [u8], number_of_glyphs: u16) -> Option<Self> {
        let mut s = Stream::new(data);

        let machine = aat::StateTable::parse(data, number_of_glyphs)?;
        s.advance(aat::StateTable::SIZE);
        let ligature_action_offset = s.read::<Offset32>()?.to_usize();
        let component_offset = s.read::<Offset32>()?.to_usize();
        let ligature_offset = s.read::<Offset32>()?.to_usize();

        // All three arrays are unsized, so we're simply reading/mapping all the data past offset.
        let ligature_actions = LazyArray32::new(data.get(ligature_action_offset..)?);
        let components = LazyArray32::new(data.get(component_offset..)?);
        let ligatures = LazyArray32::new(data.get(ligature_offset..)?);

        Some(LigatureSubtable {
            machine,
            ligature_actions,
            components,
            ligatures,
        })
    }
}


#[derive(Copy, Clone)]
pub struct InsertionEntry {
    pub current_insert_index: u16,
    pub marked_insert_index: u16,
}

impl FromData for InsertionEntry {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(InsertionEntry {
            current_insert_index: s.read()?,
            marked_insert_index: s.read()?,
        })
    }
}


pub struct InsertionSubtable<'a> {
    pub machine: aat::StateTable<'a>,
    pub glyphs: LazyArray32<'a, GlyphId>,
}

impl<'a> InsertionSubtable<'a> {
    fn parse(data: &'a [u8], number_of_glyphs: u16) -> Option<Self> {
        let mut s = Stream::new(data);

        let machine = aat::StateTable::parse(data, number_of_glyphs)?;
        s.advance(aat::StateTable::SIZE);
        let offset = s.read::<Offset32>()?.to_usize();

        // The list is unsized.
        let glyphs = LazyArray32::new(data.get(offset..)?);

        Some(InsertionSubtable {
            machine,
            glyphs,
        })
    }
}
