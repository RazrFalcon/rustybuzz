use ttf_parser::parser::Stream;

/// A slice-like container with runtime-defined stride.
#[derive(Clone, Copy, Debug)]
pub struct DynArray<'a> {
    data: &'a [u8],
    stride: usize,
}

impl<'a> DynArray<'a> {
    /// Read a `DynArray` from a stream.
    #[inline]
    pub fn read(
        s: &mut Stream<'a>,
        count: usize,
        stride: usize,
    ) -> Option<Self> {
        let len = usize::from(count) * stride;
        s.read_bytes(len).map(|data| Self { data, stride })
    }

    /// Returns the value at `index`.
    pub fn get(&self, index: usize) -> Option<&'a [u8]> {
        let start = index * self.stride;
        let end = start + self.stride;
        self.data.get(start..end)
    }

    /// Returns the array's length.
    pub fn len(&self) -> usize {
        self.data.len() / self.stride
    }

    /// Performs a binary search using the specified closure.
    pub fn binary_search_by<F>(&self, mut f: F) -> Option<&'a [u8]>
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
        if f(&value) == Ordering::Equal { Some(value) } else { None }
    }
}
