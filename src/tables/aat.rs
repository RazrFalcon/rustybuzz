/*!
A collection of [Apple Advanced Typography](
https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6AATIntro.html)
related types.
*/

use ttf_parser::GlyphId;
use ttf_parser::parser::{Stream, FromData, NumFrom, Offset32, Offset, LazyArray16};

use crate::buffer::Buffer;

/// Predefined classes.
pub mod class {
    pub const END_OF_TEXT: u16 = 0;
    pub const OUT_OF_BOUNDS: u16 = 1;
    pub const DELETED_GLYPH: u16 = 2;
}

pub const START_OF_TEXT: u16 = 0;

pub trait Entry: FromData + Copy {
    fn new_state(&self) -> u16;
    fn flags(&self) -> u16;
    fn is_actionable(&self) -> bool;

    /// If set, don't advance to the next glyph before going to the new state.
    fn has_advance(&self) -> bool;
}

pub trait StateTable2<T: Entry> {
    fn class(&self, glyph_id: GlyphId) -> Option<u16>;
    fn entry(&self, state: u16, class: u16) -> Option<T>;
}

/// An extended state table.
pub struct StateTable<'a> {
    number_of_glyphs: u16, // From `maxp`.
    number_of_classes: u32,
    lookup: Lookup<'a>,
    state_array: &'a [u8],
    entry_table: &'a [u8],
}

impl<'a> StateTable<'a> {
    pub const SIZE: usize = 16;

    pub(crate) fn parse(data: &'a [u8], number_of_glyphs: u16) -> Option<Self> {
        let mut s = Stream::new(data);

        let number_of_classes: u32 = s.read()?;
        // Note that in format1 subtable, offsets are not from the subtable start,
        // but from subtable start + `header_size`.
        // So there is not need to subtract the `header_size`.
        let lookup_table_offset = s.read::<Offset32>()?.to_usize();
        let state_array_offset = s.read::<Offset32>()?.to_usize();
        let entry_table_offset = s.read::<Offset32>()?.to_usize();

        Some(StateTable {
            number_of_glyphs,
            number_of_classes,
            lookup: Lookup::parse(data.get(lookup_table_offset..)?)?,
            // We don't know the actual data size and it's kinda expensive to calculate.
            // So we are simply storing all the data past the offset.
            // Despite the fact that they may overlap.
            state_array: data.get(state_array_offset..)?,
            entry_table: data.get(entry_table_offset..)?,
        })
    }

    /// Returns a glyph class.
    #[inline]
    pub fn class(&self, glyph_id: GlyphId) -> Option<u16> {
        if glyph_id.0 == 0xFFFF {
            return Some(class::DELETED_GLYPH);
        }

        self.lookup.value(glyph_id, self.number_of_glyphs)
    }

    /// Returns a class entry.
    #[inline]
    pub fn entry<T: Entry>(&self, state: u16, mut class: u16) -> Option<T> {
        if u32::from(class) >= self.number_of_classes {
            class = class::OUT_OF_BOUNDS;
        }

        let state_idx =
            usize::from(state) * usize::num_from(self.number_of_classes) + usize::from(class);

        let entry_idx: u16 = Stream::read_at(self.state_array, state_idx * u16::SIZE)?;
        Stream::read_at(self.entry_table, usize::from(entry_idx) * T::SIZE)
    }

    /// Returns a class entry.
    #[inline]
    pub fn entry2<T: FromData>(&self, state: u16, mut class: u16) -> Option<T> {
        if u32::from(class) >= self.number_of_classes {
            class = class::OUT_OF_BOUNDS;
        }

        let state_idx =
            usize::from(state) * usize::num_from(self.number_of_classes) + usize::from(class);

        let entry_idx: u16 = Stream::read_at(self.state_array, state_idx * u16::SIZE)?;
        Stream::read_at(self.entry_table, usize::from(entry_idx) * T::SIZE)
    }
}


pub struct Entry2<T: FromData> {
    pub new_state: u16,
    pub flags: u16,
    pub extra: T,
}

impl<T: FromData> FromData for Entry2<T> {
    const SIZE: usize = 4 + T::SIZE;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Entry2 {
            new_state: s.read()?,
            flags: s.read()?,
            extra: s.read()?,
        })
    }
}


pub trait Driver<T: FromData> {
    fn in_place(&self) -> bool;
    fn can_advance(&self, entry: &Entry2<T>) -> bool;
    fn is_actionable(&self, entry: &Entry2<T>, buffer: &Buffer) -> bool;
    fn transition(&mut self, entry: &Entry2<T>, buffer: &mut Buffer) -> Option<()>;
}

pub fn drive<T: FromData>(machine: &StateTable, c: &mut dyn Driver<T>, buffer: &mut Buffer) {
    if !c.in_place() {
        buffer.clear_output();
    }

    let mut state = START_OF_TEXT;
    buffer.idx = 0;
    loop {
        let class = if buffer.idx < buffer.len {
            machine.class(buffer.info[buffer.idx].as_glyph()).unwrap_or(1)
        } else {
            class::END_OF_TEXT
        };

        let entry: Entry2<T> = match machine.entry2(state, class) {
            Some(v) => v,
            None => break,
        };

        // Unsafe-to-break before this if not in state 0, as things might
        // go differently if we start from state 0 here.
        if state != START_OF_TEXT &&
            buffer.backtrack_len() != 0 &&
            buffer.idx < buffer.len
        {
            // If there's no value and we're just epsilon-transitioning to state 0, safe to break.
            if  c.is_actionable(&entry, buffer) ||
                !(entry.new_state == START_OF_TEXT && !c.can_advance(&entry))
            {
                buffer.unsafe_to_break_from_outbuffer(buffer.backtrack_len() - 1, buffer.idx + 1);
            }
        }

        // Unsafe-to-break if end-of-text would kick in here.
        if buffer.idx + 2 <= buffer.len {
            let end_entry: Entry2<T> = match machine.entry2(state, class::END_OF_TEXT) {
                Some(v) => v,
                None => break,
            };

            if c.is_actionable(&end_entry, buffer) {
                buffer.unsafe_to_break(buffer.idx, buffer.idx + 2);
            }
        }

        c.transition(&entry, buffer);

        state = entry.new_state;

        if buffer.idx >= buffer.len {
            break;
        }

        if c.can_advance(&entry) {
            buffer.next_glyph();
        } else {
            if buffer.max_ops <= 0 {
                buffer.next_glyph();
            }
            buffer.max_ops -= 1;
        }
    }

    if !c.in_place() {
        while buffer.idx < buffer.len {
            buffer.next_glyph();
        }

        buffer.swap_buffers();
    }
}

/// A lookup table as defined at
/// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6Tables.html
#[derive(Clone)]
pub enum Lookup<'a> {
    Format1(&'a [u8]),
    Format2(BinarySearchTable<'a, LookupSegment>),
    Format4(BinarySearchTable<'a, LookupSegment>, &'a [u8]),
    Format6(BinarySearchTable<'a, LookupSingle>),
    Format8 {
        first_glyph: u16,
        values: LazyArray16<'a, u16>
    },
    Format10 {
        value_size: u16,
        first_glyph: u16,
        glyph_count: u16,
        data: &'a [u8],
    },
}

impl<'a> Lookup<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        match format {
            0 => {
                Some(Lookup::Format1(s.tail()?))
            }
            2 => {
                let bsearch = BinarySearchTable::parse(s.tail()?)?;
                Some(Lookup::Format2(bsearch))
            }
            4 => {
                let bsearch = BinarySearchTable::parse(s.tail()?)?;
                Some(Lookup::Format4(bsearch, data))
            }
            6 => {
                let bsearch = BinarySearchTable::parse(s.tail()?)?;
                Some(Lookup::Format6(bsearch))
            }
            8 => {
                let first_glyph: u16 = s.read()?;
                let glyph_count: u16 = s.read()?;
                let values = s.read_array16(glyph_count)?;
                Some(Lookup::Format8 { first_glyph, values })
            }
            10 => {
                let value_size: u16 = s.read()?;
                let first_glyph: u16 = s.read()?;
                let glyph_count: u16 = s.read()?;
                Some(Lookup::Format10 {
                    value_size, first_glyph, glyph_count, data: s.tail()?
                })
            }
            _ => {
                None
            }
        }
    }

    pub fn value(&self, glyph_id: GlyphId, number_of_glyphs: u16) -> Option<u16> {
        match self {
            Lookup::Format1(data) => {
                if glyph_id.0 >= number_of_glyphs {
                    None
                } else {
                    // Format 0 is just an unsized u16 array.
                    Stream::read_at(data, usize::from(glyph_id.0) * u16::SIZE)
                }
            }
            Lookup::Format2(ref bsearch) => {
                bsearch.get(glyph_id).map(|v| v.value)
            }
            Lookup::Format4(ref bsearch, data) => {
                let offset = bsearch.get(glyph_id)?.value;
                Stream::read_at(data, usize::from(offset))
            }
            Lookup::Format6(ref bsearch) => {
                bsearch.get(glyph_id).map(|v| v.value)
            }
            Lookup::Format8 { first_glyph, values } => {
                let idx = glyph_id.0.checked_sub(*first_glyph)?;
                values.get(idx)
            }
            Lookup::Format10 { value_size, first_glyph, glyph_count, data } => {
                let idx = glyph_id.0.checked_sub(*first_glyph)?;
                let mut s = Stream::new(data);
                match value_size {
                    1 => s.read_array16::<u8>(*glyph_count)?.get(idx).map(u16::from),
                    2 => s.read_array16::<u16>(*glyph_count)?.get(idx),
                    // TODO: we should return u32 here, but this is not supported yet
                    4 => s.read_array16::<u32>(*glyph_count)?.get(idx).map(|n| n as u16),
                    _ => None, // 8 is also supported
                }
            }
        }
    }
}

/// A binary searching table as defined at
/// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6Tables.html
#[derive(Clone)]
pub struct BinarySearchTable<'a, T: BinarySearchValue> {
    len: u16,
    values: LazyArray16<'a, T>,
}

impl<'a, T: BinarySearchValue + core::fmt::Debug> BinarySearchTable<'a, T> {
    #[inline(never)]
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let unit_size: u16 = s.read()?;
        let number_of_units: u16 = s.read()?;
        s.advance(6); // search_range + entry_selector + range_shift

        if usize::from(unit_size) != T::SIZE {
            return None;
        }

        if number_of_units < 2 {
            return None;
        }

        let values: LazyArray16<T> = s.read_array16(number_of_units)?;

        // 'The number of termination values that need to be included is table-specific.
        // The value that indicates binary search termination is 0xFFFF.'
        let mut len = number_of_units;
        if values.last()?.is_termination() {
            len -= 1;
        }

        Some(BinarySearchTable {
            len,
            values,
        })
    }

    fn get(&self, key: GlyphId) -> Option<T> {
        let mut min = 0;
        let mut max = (self.len as isize) - 1;
        while min <= max {
            let mid = (min + max) / 2;
            let v = self.values.get(mid as u16)?;
            match v.contains(key) {
                core::cmp::Ordering::Less    => max = mid - 1,
                core::cmp::Ordering::Greater => min = mid + 1,
                core::cmp::Ordering::Equal   => return Some(v),
            }
        }

        None
    }
}


pub trait BinarySearchValue: FromData {
    fn is_termination(&self) -> bool;
    fn contains(&self, glyph_id: GlyphId) -> core::cmp::Ordering;
}


#[derive(Clone, Copy, Debug)]
pub struct LookupSegment {
    last_glyph: u16,
    first_glyph: u16,
    value: u16,
}

impl BinarySearchValue for LookupSegment {
    fn is_termination(&self) -> bool {
        self.last_glyph == 0xFFFF && self.first_glyph == 0xFFFF
    }

    fn contains(&self, id: GlyphId) -> core::cmp::Ordering {
        if id.0 < self.first_glyph {
            core::cmp::Ordering::Less
        } else if id.0 <= self.last_glyph {
            core::cmp::Ordering::Greater
        } else {
            core::cmp::Ordering::Equal
        }
    }
}

impl FromData for LookupSegment {
    const SIZE: usize = 6;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(LookupSegment {
            last_glyph: s.read()?,
            first_glyph: s.read()?,
            value: s.read()?,
        })
    }
}


#[derive(Clone, Copy, Debug)]
pub struct LookupSingle {
    glyph: u16,
    value: u16,
}

impl BinarySearchValue for LookupSingle {
    fn is_termination(&self) -> bool {
        self.glyph == 0xFFFF
    }

    fn contains(&self, id: GlyphId) -> core::cmp::Ordering {
        id.0.cmp(&self.glyph)
    }
}

impl FromData for LookupSingle {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(LookupSingle {
            glyph: s.read()?,
            value: s.read()?,
        })
    }
}
