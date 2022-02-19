// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6kerx.html

use core::num::NonZeroU16;

use ttf_parser::{aat, GlyphId};
use ttf_parser::parser::{Stream, FromData, NumFrom, Offset32, Offset};
use super::kern::KerningRecord;

const HEADER_SIZE: usize = 12;

pub fn parse(number_of_glyphs: NonZeroU16, data: &[u8]) -> Option<Subtables> {
    let mut s = Stream::new(data);
    s.skip::<u16>(); // version
    s.skip::<u16>(); // padding
    let number_of_tables: u32 = s.read()?;
    Some(Subtables {
        number_of_glyphs,
        table_index: 0,
        number_of_tables,
        stream: s,
    })
}


/// An iterator over extended kerning subtables.
#[allow(missing_debug_implementations)]
#[derive(Clone, Copy)]
pub struct Subtables<'a> {
    /// The number of glyphs from the `maxp` table.
    number_of_glyphs: NonZeroU16,
    /// The current table index.
    table_index: u32,
    /// The total number of tables.
    number_of_tables: u32,
    /// Actual data. Starts right after the `kern` header.
    stream: Stream<'a>,
}

impl<'a> Iterator for Subtables<'a> {
    type Item = (Coverage, Subtable<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.table_index == self.number_of_tables {
            return None;
        }

        if self.stream.at_end() {
            return None;
        }

        let s = &mut self.stream;

        let table_len: u32 = s.read()?;
        let coverage: Coverage = s.read()?;
        s.skip::<u16>(); // unused
        let format: u8 = s.read()?;
        let tuple_count: u32 = s.read()?;

        // Subtract the header size.
        let data_len = usize::num_from(table_len).checked_sub(HEADER_SIZE)?;

        let kind = match format {
            0 => {
                let data = s.read_bytes(data_len)?;
                Subtable::Format0(Subtable0(data))
            }
            1 => {
                let data = s.tail()?.get(0..data_len)?;
                let state_table = aat::ExtendedStateTable::parse(self.number_of_glyphs, s)?;

                // Actions offset is right after the state table.
                let actions_offset: Offset32 = s.read()?;
                // Actions offset is from the start of the state table and not from the start of subtable.
                // And since we don't know the length of the actions data,
                // simply store all the data after the offset.
                let actions_data = data.get(actions_offset.to_usize()..)?;

                Subtable::Format1(format1::StateTable {
                    state_table,
                    actions_data,
                    tuple_count,
                })
            }
            2 => {
                let data = s.read_bytes(data_len)?;
                Subtable::Format2(Subtable2(data))
            }
            4 => {
                let data = s.tail()?.get(0..data_len)?;
                let state_table = aat::ExtendedStateTable::parse(self.number_of_glyphs, s)?;
                let flags: u32 = s.read()?;
                let action_type = ((flags & 0xC0000000) >> 30) as u8;
                let points_offset = usize::num_from(flags & 0x00FFFFFF);

                let action_type = match action_type {
                    0 => format4::ActionType::ControlPointActions,
                    1 => format4::ActionType::AnchorPointActions,
                    2 => format4::ActionType::ControlPointCoordinateActions,
                    _ => return None,
                };

                Subtable::Format4(format4::StateTable {
                    state_table,
                    action_type,
                    control_points_data: data.get(points_offset..)?,
                })
            }
            6 => {
                let data = s.read_bytes(data_len)?;
                Subtable::Format6(Subtable6 {
                    data,
                    number_of_glyphs: self.number_of_glyphs,
                })
            }
            _ => {
                // Unknown format.
                return None;
            }
        };

        Some((coverage, kind))
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Coverage(u8);

impl Coverage {
    #[inline]
    pub fn is_horizontal(self) -> bool {
        self.0 & (1 << 7) == 0
    }

    #[inline]
    pub fn has_cross_stream(self) -> bool {
        self.0 & (1 << 6) != 0
    }

    #[inline]
    pub fn is_variable(self) -> bool {
        self.0 & (1 << 5) != 0
    }
}

impl FromData for Coverage {
    const SIZE: usize = 1;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        data.get(0).copied().map(Coverage)
    }
}


pub enum Subtable<'a> {
    Format0(Subtable0<'a>),
    Format1(format1::StateTable<'a>),
    Format2(Subtable2<'a>),
    Format4(format4::StateTable<'a>),
    Format6(Subtable6<'a>),
}


pub trait KerningPairs {
    fn glyphs_kerning(&self, left: GlyphId, right: GlyphId) -> Option<i16>;
}

/// A *Format 0 Kerning Subtable (Ordered List of Kerning Pairs)* implementation
/// from https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6kerx.html
pub struct Subtable0<'a>(&'a [u8]);

impl KerningPairs for Subtable0<'_> {
    fn glyphs_kerning(&self, left: GlyphId, right: GlyphId) -> Option<i16> {
        // Just like format0 in `kern`, but uses u32 instead of u16 in the header.

        let mut s = Stream::new(self.0);
        let number_of_pairs: u32 = s.read()?;
        s.advance(12); // search_range (u32) + entry_selector (u32) + range_shift (u32)
        let pairs = s.read_array32::<KerningRecord>(number_of_pairs)?;

        let needle = u32::from(left.0) << 16 | u32::from(right.0);
        pairs.binary_search_by(|v| v.pair.cmp(&needle)).map(|(_, v)| v.value)
    }
}


pub trait StateTableExt<T: FromData + Copy> {
    fn class(&self, glyph_id: GlyphId) -> Option<u16>;
    fn entry(&self, state: u16, class: u16) -> Option<aat::ExtendedStateEntry<T>>;
}

/// A state machine entry.
#[derive(Clone, Copy, Debug)]
pub struct EntryData {
    pub action_index: u16,
}

impl FromData for EntryData {
    const SIZE: usize = 2;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(EntryData {
            action_index: s.read()?,
        })
    }
}

pub mod format1 {
    use super::*;

    pub struct StateTable<'a> {
        pub state_table: aat::ExtendedStateTable<'a, EntryData>,
        pub actions_data: &'a [u8],
        pub tuple_count: u32,
    }

    impl StateTable<'_> {
        #[inline]
        pub fn kerning(&self, action_index: u16) -> Option<i16> {
            Stream::read_at(self.actions_data, usize::from(action_index) * i16::SIZE)
        }
    }

    impl StateTableExt<EntryData> for StateTable<'_> {
        fn class(&self, glyph_id: GlyphId) -> Option<u16> {
            self.state_table.class(glyph_id)
        }

        fn entry(&self, state: u16, class: u16) -> Option<aat::ExtendedStateEntry<EntryData>> {
            self.state_table.entry(state, class)
        }
    }

    impl<'a> core::ops::Deref for StateTable<'a> {
        type Target = aat::ExtendedStateTable<'a, EntryData>;

        fn deref(&self) -> &Self::Target {
            &self.state_table
        }
    }
}


/// A *Format 2 Kerning Table (Simple n x m Array of Kerning Values)* implementation
/// from https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6kerx.html
pub struct Subtable2<'a>(&'a [u8]);

impl KerningPairs for Subtable2<'_> {
    fn glyphs_kerning(&self, left: GlyphId, right: GlyphId) -> Option<i16> {
        let mut s = Stream::new(self.0);
        s.skip::<u32>(); // row_width

        // Offsets are from beginning of the subtable and not from the `data` start,
        // so we have to subtract the header.
        let left_hand_table_offset = s.read::<Offset32>()?.to_usize().checked_sub(HEADER_SIZE)?;
        let right_hand_table_offset = s.read::<Offset32>()?.to_usize().checked_sub(HEADER_SIZE)?;
        let array_offset = s.read::<Offset32>()?.to_usize().checked_sub(HEADER_SIZE)?;

        // 'The array can be indexed by completing the left-hand and right-hand class mappings,
        // adding the class values to the address of the subtable,
        // and fetching the kerning value to which the new address points.'

        let left_class = get_format2_class(left.0, left_hand_table_offset, self.0).unwrap_or(0);
        let right_class = get_format2_class(right.0, right_hand_table_offset, self.0).unwrap_or(0);

        // 'Values within the left-hand offset table should not be less than the kerning array offset.'
        if usize::from(left_class) < array_offset {
            return None;
        }

        // Classes are already premultiplied, so we only need to sum them.
        let index = usize::from(left_class) + usize::from(right_class);
        let value_offset = index.checked_sub(HEADER_SIZE)?;
        Stream::read_at(self.0, value_offset)
    }
}

fn get_format2_class(glyph_id: u16, offset: usize, data: &[u8]) -> Option<u16> {
    let mut s = Stream::new_at(data, offset)?;
    let first_glyph: u16 = s.read()?;
    let index = glyph_id.checked_sub(first_glyph)?;

    let number_of_classes: u16 = s.read()?;
    let classes = s.read_array16::<u16>(number_of_classes)?;
    classes.get(index)
}


pub mod format4 {
    use super::*;

    pub enum ActionType {
        ControlPointActions,
        AnchorPointActions,
        ControlPointCoordinateActions,
    }

    pub struct StateTable<'a> {
        pub state_table: aat::ExtendedStateTable<'a, EntryData>,
        pub action_type: ActionType,
        pub control_points_data: &'a [u8],
    }

    impl StateTableExt<EntryData> for StateTable<'_> {
        fn class(&self, glyph_id: GlyphId) -> Option<u16> {
            self.state_table.class(glyph_id)
        }

        fn entry(&self, state: u16, class: u16) -> Option<aat::ExtendedStateEntry<EntryData>> {
            self.state_table.entry(state, class)
        }
    }

    impl<'a> core::ops::Deref for StateTable<'a> {
        type Target = aat::ExtendedStateTable<'a, EntryData>;

        fn deref(&self) -> &Self::Target {
            &self.state_table
        }
    }
}


/// A *Format 6 Kerning Subtable (Simple Index-based n x m Array of Kerning Values)* implementation
/// from https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6kerx.html
pub struct Subtable6<'a>{
    data: &'a [u8],
    number_of_glyphs: NonZeroU16,
}

impl KerningPairs for Subtable6<'_> {
    fn glyphs_kerning(&self, left: GlyphId, right: GlyphId) -> Option<i16> {
        use core::convert::TryFrom;

        let mut s = Stream::new(self.data);
        let flags: u32 = s.read()?;
        s.skip::<u16>(); // row_count
        s.skip::<u16>(); // col_count
        // All offsets are from the start of the subtable.
        let row_index_table_offset = s.read::<Offset32>()?.to_usize().checked_sub(HEADER_SIZE)?;
        let column_index_table_offset = s.read::<Offset32>()?.to_usize().checked_sub(HEADER_SIZE)?;
        let kerning_array_offset = s.read::<Offset32>()?.to_usize().checked_sub(HEADER_SIZE)?;
        let kerning_vector_offset = s.read::<Offset32>()?.to_usize().checked_sub(HEADER_SIZE)?;

        let row_index_table_data = self.data.get(row_index_table_offset..)?;
        let column_index_table_data = self.data.get(column_index_table_offset..)?;
        let kerning_array_data = self.data.get(kerning_array_offset..)?;
        let kerning_vector_data = self.data.get(kerning_vector_offset..)?;

        let has_long_values = flags & 0x00000001 != 0;
        if has_long_values {
            let l: u32 = aat::Lookup::parse(self.number_of_glyphs, row_index_table_data)?
                .value(left).unwrap_or(0) as u32;

            let r: u32 = aat::Lookup::parse(self.number_of_glyphs, column_index_table_data)?
                .value(right).unwrap_or(0) as u32;

            let array_offset = usize::try_from(l + r).ok()?.checked_mul(i32::SIZE)?;
            let vector_offset: u32 = Stream::read_at(kerning_array_data, array_offset)?;

            Stream::read_at(kerning_vector_data, usize::num_from(vector_offset))
        } else {
            let l: u16 = aat::Lookup::parse(self.number_of_glyphs, row_index_table_data)?
                .value(left).unwrap_or(0);

            let r: u16 = aat::Lookup::parse(self.number_of_glyphs, column_index_table_data)?
                .value(right).unwrap_or(0);

            let array_offset = usize::try_from(l + r).ok()?.checked_mul(i16::SIZE)?;
            let vector_offset: u16 = Stream::read_at(kerning_array_data, array_offset)?;

            Stream::read_at(kerning_vector_data, usize::from(vector_offset))
        }
    }
}
