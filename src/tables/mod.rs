use core::cmp::Ordering;

use ttf_parser::parser::*;
use ttf_parser::{GlyphId, Class};

pub mod ankr;
pub mod feat;
pub mod gpos;
pub mod gsub;
pub mod gsubgpos;
pub mod kern;
pub mod kerx;
pub mod morx;
pub mod trak;
pub mod aat;

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
        where F: FnMut(&[u8]) -> Ordering
    {
        // Based on Rust std implementation.

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
