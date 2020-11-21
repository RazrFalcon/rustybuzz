// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6ankr.html

use ttf_parser as ttf;
use ttf::GlyphId;
use ttf::parser::{Stream, FromData, Offset32, Offset};

use super::aat;

#[derive(Clone)]
pub(crate) struct Table<'a> {
    lookup: aat::Lookup<'a>,
    glyph_data_table: &'a [u8],
    number_of_glyphs: u16,
}

impl<'a> Table<'a> {
    pub fn parse(data: &'a [u8], number_of_glyphs: u16) -> Option<Self> {
        let mut s = Stream::new(data);

        let version: u16 = s.read()?;
        if version != 0 {
            return None;
        }

        s.skip::<u16>(); // reserved
        let lookup_table_offset: Offset32 = s.read()?;
        let glyph_data_table_offset: Offset32 = s.read()?;

        Some(Table {
            lookup: aat::Lookup::parse(data.get(lookup_table_offset.to_usize()..)?)?,
            glyph_data_table: data.get(glyph_data_table_offset.to_usize()..)?,
            number_of_glyphs,
        })
    }

    pub fn anchor(&self, glyph_id: GlyphId, idx: u16) -> Option<Anchor> {
        let offset = self.lookup.value(glyph_id, self.number_of_glyphs)?;

        let mut s = Stream::new_at(self.glyph_data_table, usize::from(offset))?;
        let number_of_points: u32 = s.read()?;
        let points = s.read_array32(number_of_points)?;
        points.get(u32::from(idx))
    }
}


#[derive(Clone, Copy, Default)]
pub struct Anchor {
    pub x: i16,
    pub y: i16,
}

impl FromData for Anchor {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Anchor {
            x: s.read()?,
            y: s.read()?,
        })
    }
}
