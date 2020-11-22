// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6feat.html

use ttf_parser::parser::{Stream, FromData, Offset32, Fixed, LazyArray16};


#[derive(Clone, Copy)]
pub struct Table<'a> {
    names: LazyArray16<'a, FeatureName>,
}

impl<'a> Table<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);

        let version: Fixed = s.read()?;
        if version.0 != 1.0 {
            return None;
        }

        let count: u16 = s.read()?;
        s.advance_checked(6)?; // reserved
        let names = s.read_array16(count)?;

        Some(Table {
            names,
        })
    }

    pub fn exposes_feature(&self, kind: u16) -> bool {
        match self.feature(kind) {
            Some(feature) => feature.has_data(),
            None => false,
        }
    }

    pub fn feature(&self, kind: u16) -> Option<FeatureName> {
        self.names.binary_search_by(|name| name.kind.cmp(&kind)).map(|(_, f)| f)
    }
}


#[derive(Clone, Copy)]
pub struct FeatureName {
    kind: u16,
    records_count: u16,
    _setting_table_offset: Offset32,
    flags: u16,
    _index: i16,
}

impl FeatureName {
    pub fn has_data(&self) -> bool {
        self.records_count > 0
    }

    pub fn is_exclusive(&self) -> bool {
        self.flags & 0x8000 != 0
    }
}

impl FromData for FeatureName {
    const SIZE: usize = 12;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(FeatureName {
            kind: s.read()?,
            records_count: s.read()?,
            _setting_table_offset: s.read()?,
            flags: s.read()?,
            _index: s.read()?,
        })
    }
}

