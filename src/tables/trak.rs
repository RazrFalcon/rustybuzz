// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6trak.html

use ttf_parser::parser::{Stream, FromData, Fixed, LazyArray16, Offset, Offset16, Offset32};


#[derive(Clone, Copy)]
pub struct Table<'a> {
    data: &'a [u8], // The whole table.
    hor: Option<TrackData<'a>>,
    ver: Option<TrackData<'a>>,
}

impl<'a> Table<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);

        let version: Fixed = s.read()?;
        if version.0 != 1.0 {
            return None;
        }

        let format: u16 = s.read()?;
        if format != 0 {
            return None;
        }

        let hor_offset: Option<Offset16> = s.read()?;
        let ver_offset: Option<Offset16> = s.read()?;
        s.skip::<u16>(); // reserved

        let hor = if let Some(offset) = hor_offset {
            let mut s = Stream::new_at(data, offset.to_usize())?;
            Some(TrackData::parse(data, &mut s)?)
        } else {
            None
        };

        let ver = if let Some(offset) = ver_offset {
            let mut s = Stream::new_at(data, offset.to_usize())?;
            Some(TrackData::parse(data, &mut s)?)
        } else {
            None
        };

        Some(Table {
            data,
            hor,
            ver,
        })
    }

    pub fn hor_tracking(&self, ptem: f32) -> Option<i32> {
        self.hor.and_then(|d| d.tracking(ptem, self.data))
    }

    pub fn ver_tracking(&self, ptem: f32) -> Option<i32> {
        self.ver.and_then(|d| d.tracking(ptem, self.data))
    }
}


#[derive(Clone, Copy)]
pub struct TrackData<'a> {
    tracks: LazyArray16<'a, TrackTableEntry>,
    sizes: LazyArray16<'a, Fixed>,
}

impl<'a> TrackData<'a> {
    fn parse(table_data: &'a [u8], s: &mut Stream<'a>) -> Option<Self> {
        let tracks_count: u16 = s.read()?;
        let sizes_count: u16 = s.read()?;
        let size_table_offset: Offset32 = s.read()?;

        let sizes = {
            let mut s = Stream::new_at(table_data, size_table_offset.to_usize())?;
            s.read_array16(sizes_count)?
        };

        Some(TrackData {
            tracks: s.read_array16(tracks_count)?,
            sizes,
        })
    }

    fn tracking(&self, ptem: f32, table_data: &[u8]) -> Option<i32> {
        // Choose track.
        let track = self.tracks.into_iter().find(|t| t.track.0 == 0.0)?;

        // Choose size.
        if self.sizes.is_empty() {
            return None;
        }

        let mut idx = self.sizes.into_iter().position(|s| s.0 >= ptem)
            .unwrap_or(self.sizes.len() as usize - 1);

        if idx > 0 {
            idx -= 1;
        }

        self.interpolate_at(idx as u16, ptem, &track, table_data).map(|n| n.round() as i32)
    }

    fn interpolate_at(
        &self,
        idx: u16,
        target_size: f32,
        track: &TrackTableEntry,
        table_data: &[u8],
    ) -> Option<f32> {
        debug_assert!(idx < self.sizes.len() - 1);

        let s0 = self.sizes.get(idx)?.0;
        let s1 = self.sizes.get(idx + 1)?.0;

        let t = if s0 == s1 { 0.0 } else { (target_size - s0) / (s1 - s0) };

        let n = t * (track.value(table_data, idx + 1)? as f32)
            + (1.0 - t) * (track.value(table_data, idx)? as f32);

        Some(n)
    }
}


#[derive(Clone, Copy)]
pub struct TrackTableEntry {
    track: Fixed,
    _name_id: u16,
    offset: Offset16,
}

impl TrackTableEntry {
    fn value(&self, table_data: &[u8], idx: u16) -> Option<i16> {
        Stream::read_at(table_data, self.offset.to_usize() + (idx as usize) * u16::SIZE)
    }
}

impl FromData for TrackTableEntry {
    const SIZE: usize = 8;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(TrackTableEntry {
            track: s.read()?,
            _name_id: s.read()?,
            offset: s.read()?,
        })
    }
}
