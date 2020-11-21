use ttf_parser::parser::*;
use ttf_parser::{GlyphId, Class};

pub mod ankr;
pub mod gpos;
pub mod gsub;
pub mod gsubgpos;
pub mod kern;
pub mod kerx;

trait StreamExt<'a> {
    fn read_dyn_array(&mut self, count: usize, stride: usize) -> Option<DynArray<'a>>;
    fn read_at_offset16(&mut self, data: &'a [u8]) -> Option<&'a [u8]>;
    fn read_at_offset32(&mut self, data: &'a [u8]) -> Option<&'a [u8]>;
    fn read_offsets16(&mut self, count: u16, data: &'a [u8]) -> Option<Offsets16<'a, Offset16>>;
}

impl<'a> StreamExt<'a> for Stream<'a> {
    #[inline]
    fn read_dyn_array(&mut self, count: usize, stride: usize) -> Option<DynArray<'a>> {
        let len = count * stride;
        self.read_bytes(len).map(|data| DynArray::new(data, stride))
    }

    #[inline]
    fn read_at_offset16(&mut self, data: &'a [u8]) -> Option<&'a [u8]> {
        let offset = self.read::<Offset16>()?.to_usize();
        data.get(offset..)
    }

    #[inline]
    fn read_at_offset32(&mut self, data: &'a [u8]) -> Option<&'a [u8]> {
        let offset = self.read::<Offset32>()?.to_usize();
        data.get(offset..)
    }

    #[inline]
    fn read_offsets16(&mut self, count: u16, data: &'a [u8]) -> Option<Offsets16<'a, Offset16>> {
        let offsets = self.read_array16(count)?;
        Some(Offsets16 { data, offsets })
    }
}

/// A slice-like container with runtime-defined stride.
#[derive(Clone, Copy, Debug)]
pub struct DynArray<'a> {
    data: &'a [u8],
    stride: usize,
}

impl<'a> DynArray<'a> {
    #[inline]
    pub fn new(data: &'a [u8], stride: usize) -> Self {
        Self { data, stride }
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&'a [u8]> {
        let start = index * self.stride;
        let end = start + self.stride;
        self.data.get(start..end)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len() / self.stride
    }

    #[inline]
    pub fn binary_search_by<F>(&self, mut f: F) -> Option<(usize, &'a [u8])>
        where F: FnMut(&[u8]) -> core::cmp::Ordering
    {
        // Based on Rust std implementation.

        use core::cmp::Ordering;

        let mut size = self.len();
        if size == 0 {
            return None;
        }

        let mut base = 0;
        while size > 1 {
            let half = size / 2;
            let mid = base + half;
            // mid is always in [0, size), that means mid is >= 0 and < size.
            // mid >= 0: by definition
            // mid < size: mid = size / 2 + size / 4 + size / 8 ...
            let cmp = f(&self.get(mid)?);
            base = if cmp == Ordering::Greater { base } else { mid };
            size -= half;
        }

        // base is always in [0, size) because base <= mid.
        let value = self.get(base)?;
        if f(&value) == Ordering::Equal { Some((base, value)) } else { None }
    }
}

/// Array of offsets from beginning of `data`.
#[derive(Clone, Copy)]
pub struct Offsets16<'a, T: Offset> {
    data: &'a [u8],
    offsets: LazyArray16<'a, T>, // [Offset16/Offset32]
}

impl<'a, T: Offset + FromData> Offsets16<'a, T> {
    pub fn len(&self) -> u16 {
        self.offsets.len() as u16
    }

    pub fn get(&self, index: u16) -> Option<T> {
        self.offsets.get(index)
    }

    pub fn slice(&self, index: u16) -> Option<&'a [u8]> {
        let offset = self.offsets.get(index)?.to_usize();
        self.data.get(offset..)
    }
}

impl<'a, T: Offset + FromData + Copy + core::fmt::Debug> core::fmt::Debug for Offsets16<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self.offsets)
    }
}

/// An iterator over `Offset16`.
#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
pub struct OffsetsIter16<'a, T: Offset + FromData> {
    offsets: Offsets16<'a, T>,
    index: u16,
}

impl<'a, T: Offset + FromData> IntoIterator for Offsets16<'a, T> {
    type Item = &'a [u8];
    type IntoIter = OffsetsIter16<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        OffsetsIter16 {
            offsets: self,
            index: 0,
        }
    }
}

impl<'a, T: Offset + FromData> Iterator for OffsetsIter16<'a, T> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.offsets.len() {
            let idx = self.index;
            self.index += 1;

            // Skip NULL offsets.
            if self.offsets.get(idx)?.is_null() {
                return self.next();
            }

            self.offsets.slice(idx)
        } else {
            None
        }
    }
}

pub mod aat {
    /*!
    A collection of [Apple Advanced Typography](
    https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6AATIntro.html)
    related types.
    */

    // https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6Tables.html

    use ttf_parser::GlyphId;
    use ttf_parser::parser::{Stream, FromData, NumFrom, Offset32, Offset, LazyArray16};

    /// An [Extended State Table](
    /// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6Tables.html#ExtendedStateHeader).
    pub mod extended_state_table {
        // The implementation of this table is very similar to the `kern` one,
        // but it uses wider types (u8 -> u16, u16 -> u32) for some values.
        // And `Entry` has three fields instead of two.

        use super::*;

        /// Predefined classes.
        pub mod class {
            #![allow(missing_docs)]
            pub const END_OF_TEXT: u16 = 0;
            pub const OUT_OF_BOUNDS: u16 = 1;
            pub const DELETED_GLYPH: u16 = 2;
        }


        /// A type-safe wrapper for a state machine state.
        #[derive(Clone, Copy, PartialEq, Debug)]
        pub struct State(pub(crate) u16);

        pub const START_OF_TEXT: State = State(0);

        pub trait Entry: FromData + Copy {
            fn new_state(&self) -> State;
            fn is_actionable(&self) -> bool;

            /// If set, don't advance to the next glyph before going to the new state
            fn has_advance(&self) -> bool;
        }


        pub trait StateTable<T: Entry> {
            fn class(&self, glyph_id: GlyphId) -> Option<u16>;
            fn entry(&self, state: State, class: u16) -> Option<T>;
        }

        /// An extended state table.
        pub struct Table<'a> {
            number_of_glyphs: u16, // From `maxp`.
            number_of_classes: u32,
            lookup: Lookup<'a>,
            state_array: &'a [u8],
            entry_table: &'a [u8],
        }

        impl<'a> Table<'a> {
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

                Some(Table {
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
            pub fn entry<T: Entry>(&self, state: State, mut class: u16) -> Option<T> {
                if u32::from(class) >= self.number_of_classes {
                    class = class::OUT_OF_BOUNDS;
                }

                let state_idx =
                    usize::from(state.0) * usize::num_from(self.number_of_classes) + usize::from(class);

                let entry_idx: u16 = Stream::read_at(self.state_array, state_idx * u16::SIZE)?;
                Stream::read_at(self.entry_table, usize::from(entry_idx) * T::SIZE)
            }
        }

        impl core::fmt::Debug for Table<'_> {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_str("Machine(...)")
            }
        }
    }


    /// A lookup table as defined at
    /// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6Tables.html
    #[derive(Clone)]
    pub(crate) enum Lookup<'a> {
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
        pub(crate) fn parse(data: &'a [u8]) -> Option<Self> {
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

        pub(crate) fn value(&self, glyph_id: GlyphId, number_of_glyphs: u16) -> Option<u16> {
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
                        // TODO: we should return u32 here, but this is not supported
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
    pub(crate) struct BinarySearchTable<'a, T: BinarySearchValue> {
        len: u16,
        values: LazyArray16<'a, T>,
    }

    impl<'a, T: BinarySearchValue> BinarySearchTable<'a, T> {
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
            let mut max = self.len - 1;
            while min <= max {
                let mid = (min + max) / 2;
                let v = self.values.get(mid)?;
                match v.contains(key) {
                    core::cmp::Ordering::Less    => max = mid - 1,
                    core::cmp::Ordering::Greater => min = mid + 1,
                    core::cmp::Ordering::Equal   => return Some(v),
                }
            }

            None
        }
    }


    pub(crate) trait BinarySearchValue: FromData {
        fn is_termination(&self) -> bool;
        fn contains(&self, glyph_id: GlyphId) -> core::cmp::Ordering;
    }


    #[derive(Clone, Copy)]
    pub(crate) struct LookupSegment {
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


    #[derive(Clone, Copy)]
    pub(crate) struct LookupSingle {
        glyph: u16,
        value: u16,
    }

    impl BinarySearchValue for LookupSingle {
        fn is_termination(&self) -> bool {
            self.glyph == 0xFFFF
        }

        fn contains(&self, id: GlyphId) -> core::cmp::Ordering {
            self.glyph.cmp(&id.0)
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
}
