//! The Glyph Positioning Table.

use alloc::vec::Vec;

use crate::Face;
use crate::glyph_set::GlyphSet;
use super::gsubgpos::*;
use super::*;

#[derive(Clone, Debug)]
pub struct PosTable<'a> {
    pub inner: SubstPosTable<'a>,
    pub lookups: Vec<Option<PosLookup<'a>>>,
}

impl<'a> PosTable<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let inner = SubstPosTable::parse(data)?;
        let lookups = (0..inner.lookup_count())
            .map(|i| inner.get_lookup(LookupIndex(i)).map(PosLookup::parse))
            .collect();

        Some(Self { inner, lookups})
    }
}

#[derive(Clone, Debug)]
pub struct PosLookup<'a> {
    pub subtables: Vec<PosLookupSubtable<'a>>,
    pub coverage: GlyphSet,
    pub props: u32,
}

impl<'a> PosLookup<'a> {
    pub fn parse(lookup: Lookup<'a>) -> Self {
        let subtables: Vec<_> = lookup
            .subtables
            .into_iter()
            .flat_map(|data| PosLookupSubtable::parse(data, lookup.kind))
            .collect();

        let mut coverage = GlyphSet::builder();
        for subtable in &subtables {
            subtable.coverage().collect(&mut coverage);
        }

        Self {
            subtables,
            coverage: coverage.finish(),
            props: lookup.props(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PosLookupSubtable<'a> {
    Single(SinglePos<'a>),
    Pair(PairPos<'a>),
    Cursive(CursivePos<'a>),
    MarkBase(MarkBasePos<'a>),
    MarkLig(MarkLigPos<'a>),
    MarkMark(MarkMarkPos<'a>),
    Context(ContextLookup<'a>),
    ChainContext(ChainContextLookup<'a>),
}

impl<'a> PosLookupSubtable<'a> {
    pub fn parse(data: &'a [u8], kind: LookupType) -> Option<Self> {
        match kind.0 {
            1 => SinglePos::parse(data).map(Self::Single),
            2 => PairPos::parse(data).map(Self::Pair),
            3 => CursivePos::parse(data).map(Self::Cursive),
            4 => MarkBasePos::parse(data).map(Self::MarkBase),
            5 => MarkLigPos::parse(data).map(Self::MarkLig),
            6 => MarkMarkPos::parse(data).map(Self::MarkMark),
            7 => ContextLookup::parse(data).map(Self::Context),
            8 => ChainContextLookup::parse(data).map(Self::ChainContext),
            9 => parse_extension_lookup(data, Self::parse),
            _ => None,
        }
    }

    pub fn coverage(&self) -> Coverage<'a> {
        match self {
            Self::Single(t) => t.coverage(),
            Self::Pair(t) => t.coverage(),
            Self::Cursive(t) => t.coverage(),
            Self::MarkBase(t) => t.coverage(),
            Self::MarkLig(t) => t.coverage(),
            Self::MarkMark(t) => t.coverage(),
            Self::Context(t) => t.coverage(),
            Self::ChainContext(t) => t.coverage(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SinglePos<'a> {
    Format1 {
        data: &'a [u8],
        coverage: Coverage<'a>,
        value: ValueRecord<'a>,
    },
    Format2 {
        data: &'a [u8],
        coverage: Coverage<'a>,
        flags: ValueFormatFlags,
        values: DynArray<'a>,
    },
}

impl<'a> SinglePos<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let flags = s.read::<ValueFormatFlags>()?;
                let value = ValueRecord::read(&mut s, flags)?;
                Self::Format1 { data, coverage, value }
            }
            2 => {
                let coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let flags = s.read::<ValueFormatFlags>()?;
                let count = s.read::<u16>()?;
                let values = s.read_dyn_array(usize::from(count), flags.size())?;
                Self::Format2 { data, coverage, flags, values }
            }
            _ => return None,
        })
    }

    pub fn coverage(&self) -> Coverage<'a> {
        match *self {
            Self::Format1 { coverage, .. } => coverage,
            Self::Format2 { coverage, .. } => coverage,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PairPos<'a> {
    Format1 {
        coverage: Coverage<'a>,
        flags: [ValueFormatFlags; 2],
        sets: Offsets16<'a, Offset16>,
    },
    Format2 {
        data: &'a [u8],
        coverage: Coverage<'a>,
        flags: [ValueFormatFlags; 2],
        classes: [ClassDef<'a>; 2],
        matrix: ClassMatrix<'a>,
    },
}

impl<'a> PairPos<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let flags = [
                    s.read::<ValueFormatFlags>()?,
                    s.read::<ValueFormatFlags>()?,
                ];
                let count = s.read::<u16>()?;
                let sets = s.read_offsets16(count, data)?;
                Self::Format1 { coverage, flags, sets }
            }
            2 => {
                let coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let flags = [
                    s.read::<ValueFormatFlags>()?,
                    s.read::<ValueFormatFlags>()?,
                ];
                let classes = [
                    ClassDef::parse(s.read_at_offset16(data)?)?,
                    ClassDef::parse(s.read_at_offset16(data)?)?,
                ];
                let counts = [s.read::<u16>()?, s.read::<u16>()?];
                let matrix = ClassMatrix::read(&mut s, counts, flags)?;
                Self::Format2 { data, coverage, flags, classes, matrix }
            },
            _ => return None,
        })
    }

    pub fn coverage(&self) -> Coverage<'a> {
        match *self {
            Self::Format1 { coverage, .. } => coverage,
            Self::Format2 { coverage, .. } => coverage,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PairSet<'a> {
    records: DynArray<'a>,
    flags: [ValueFormatFlags; 2],
}

impl<'a> PairSet<'a> {
    pub fn parse(data: &'a [u8], flags: [ValueFormatFlags; 2]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let stride = GlyphId::SIZE + flags[0].size() + flags[1].size();
        let records = s.read_dyn_array(usize::from(count), stride)?;
        Some(Self { records, flags })
    }

    pub fn get(&self, second: GlyphId) -> Option<[ValueRecord<'a>; 2]> {
        let record = self.records.binary_search_by(|data| {
            Stream::new(data).read::<GlyphId>().unwrap().cmp(&second)
        })?.1;

        let mut s = Stream::new(record);
        s.skip::<GlyphId>();
        Some([
            ValueRecord::read(&mut s, self.flags[0])?,
            ValueRecord::read(&mut s, self.flags[1])?,
        ])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ClassMatrix<'a> {
    matrix: DynArray<'a>,
    counts: [u16; 2],
    flags: [ValueFormatFlags; 2],
}

impl<'a> ClassMatrix<'a> {
    pub fn read(s: &mut Stream<'a>, counts: [u16; 2], flags: [ValueFormatFlags; 2]) -> Option<Self> {
        let count = usize::num_from(u32::from(counts[0]) * u32::from(counts[1]));
        let stride = flags[0].size() + flags[1].size();
        let matrix = s.read_dyn_array(count, stride)?;
        Some(Self { matrix, counts, flags })
    }

    pub fn get(&self, classes: [u16; 2]) -> Option<[ValueRecord<'a>; 2]> {
        if classes[0] >= self.counts[0] || classes[1] >= self.counts[1] {
            return None;
        }

        let idx = usize::from(classes[0]) * usize::from(self.counts[1]) + usize::from(classes[1]);
        let record = self.matrix.get(idx)?;

        let mut s = Stream::new(record);
        Some([
            ValueRecord::read(&mut s, self.flags[0])?,
            ValueRecord::read(&mut s, self.flags[1])?,
        ])
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CursivePos<'a> {
    Format1 {
        data: &'a [u8],
        coverage: Coverage<'a>,
        entry_exits: LazyArray16<'a, EntryExitRecord>,
    }
}

impl<'a> CursivePos<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let count = s.read::<u16>()?;
                let entry_exits = s.read_array16(count)?;
                Self::Format1 { data, coverage, entry_exits }
            }
            _ => return None,
        })
    }

    pub fn coverage(&self) -> Coverage<'a> {
        match *self {
            Self::Format1 { coverage, .. } => coverage,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EntryExitRecord {
    pub entry_anchor: Offset16,
    pub exit_anchor: Offset16,
}

impl FromData for EntryExitRecord {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            entry_anchor: s.read()?,
            exit_anchor: s.read()?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MarkBasePos<'a> {
    Format1 {
        mark_coverage: Coverage<'a>,
        base_coverage: Coverage<'a>,
        marks: MarkArray<'a>,
        base_matrix: AnchorMatrix<'a>,
    }
}

impl<'a> MarkBasePos<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let mark_coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let base_coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let class_count = s.read::<u16>()?;
                let marks = MarkArray::parse(s.read_at_offset16(data)?)?;
                let base_matrix = AnchorMatrix::parse(s.read_at_offset16(data)?, class_count)?;
                Self::Format1 { mark_coverage, base_coverage, marks, base_matrix }
            }
            _ => return None,
        })
    }

    pub fn coverage(&self) -> Coverage<'a> {
        match *self {
            Self::Format1 { mark_coverage, .. } => mark_coverage,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MarkLigPos<'a> {
    Format1 {
        mark_coverage: Coverage<'a>,
        lig_coverage: Coverage<'a>,
        marks: MarkArray<'a>,
        lig_array: LigatureArray<'a>,
    }
}

impl<'a> MarkLigPos<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let mark_coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let lig_coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let class_count = s.read::<u16>()?;
                let marks = MarkArray::parse(s.read_at_offset16(data)?)?;
                let lig_array = LigatureArray::parse(s.read_at_offset16(data)?, class_count)?;
                Self::Format1 { mark_coverage, lig_coverage, marks, lig_array }
            }
            _ => return None,
        })
    }

    pub fn coverage(&self) -> Coverage<'a> {
        match *self {
            Self::Format1 { mark_coverage, .. } => mark_coverage,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LigatureArray<'a> {
    pub class_count: u16,
    pub attach: Offsets16<'a, Offset16>,
}

impl<'a> LigatureArray<'a> {
    pub fn parse(data: &'a [u8], class_count: u16) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let attach = s.read_offsets16(count, data)?;
        Some(Self { class_count, attach })
    }

    pub fn get(&self, idx: u16) -> Option<AnchorMatrix> {
        AnchorMatrix::parse(self.attach.slice(idx)?, self.class_count)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MarkMarkPos<'a> {
    Format1 {
        mark1_coverage: Coverage<'a>,
        mark2_coverage: Coverage<'a>,
        marks: MarkArray<'a>,
        mark2_matrix: AnchorMatrix<'a>,
    }
}

impl<'a> MarkMarkPos<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let mark1_coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let mark2_coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let class_count = s.read::<u16>()?;
                let marks = MarkArray::parse(s.read_at_offset16(data)?)?;
                let mark2_matrix = AnchorMatrix::parse(s.read_at_offset16(data)?, class_count)?;
                Self::Format1 { mark1_coverage, mark2_coverage, marks, mark2_matrix }
            }
            _ => return None,
        })
    }

    pub fn coverage(&self) -> Coverage<'a> {
        match *self {
            Self::Format1 { mark1_coverage, .. } => mark1_coverage,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ValueRecord<'a> {
    pub data: &'a [u8],
    pub flags: ValueFormatFlags,
}

impl<'a> ValueRecord<'a> {
    pub fn new(data: &'a [u8], flags: ValueFormatFlags) -> Self {
        Self { data, flags }
    }

    pub fn read(s: &mut Stream<'a>, flags: ValueFormatFlags) -> Option<Self> {
        s.read_bytes(flags.size()).map(|data| Self { data, flags })
    }
}

bitflags::bitflags! {
    #[derive(Default)]
    pub struct ValueFormatFlags: u16 {
        const X_PLACEMENT        = 0x0001;
        const Y_PLACEMENT        = 0x0002;
        const X_ADVANCE          = 0x0004;
        const Y_ADVANCE          = 0x0008;
        const X_PLACEMENT_DEVICE = 0x0010;
        const Y_PLACEMENT_DEVICE = 0x0020;
        const X_ADVANCE_DEVICE   = 0x0040;
        const Y_ADVANCE_DEVICE   = 0x0080;
        const DEVICES            = Self::X_PLACEMENT_DEVICE.bits
                                 | Self::Y_PLACEMENT_DEVICE.bits
                                 | Self::X_ADVANCE_DEVICE.bits
                                 | Self::Y_ADVANCE_DEVICE.bits;
    }
}

impl ValueFormatFlags {
    pub fn size(self) -> usize {
        u16::SIZE * usize::num_from(self.bits.count_ones())
    }
}

impl FromData for ValueFormatFlags {
    const SIZE: usize = 2;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        u16::parse(data).map(Self::from_bits_truncate)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Anchor<'a> {
    pub x: i16,
    pub y: i16,
    pub x_device: Option<Device<'a>>,
    pub y_device: Option<Device<'a>>,
}

impl<'a> Anchor<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        if !matches!(format, 1..=3) {
            return None;
        }

        let mut table = Anchor {
            x: s.read::<i16>()?,
            y: s.read::<i16>()?,
            x_device: None,
            y_device: None,
        };

        // Note: Format 2 is not handled since there is currently no way to
        // get a glyph contour point by index.

        if format == 3 {
            table.x_device = s.read::<Option<Offset16>>()?
                .and_then(|offset| data.get(offset.to_usize()..))
                .and_then(Device::parse);

            table.y_device = s.read::<Option<Offset16>>()?
                .and_then(|offset| data.get(offset.to_usize()..))
                .and_then(Device::parse);
        }

        Some(table)
    }

    pub fn get(&self, face: &Face) -> (i32, i32) {
        let mut x = i32::from(self.x);
        let mut y = i32::from(self.y);

        if self.x_device.is_some() || self.y_device.is_some() {
            let (ppem_x, ppem_y) = face.pixels_per_em().unwrap_or((0, 0));
            let coords = face.ttfp_face.variation_coordinates().len();

            if let Some(device) = self.x_device {
                if ppem_x != 0 || coords != 0 {
                    x += device.get_x_delta(face).unwrap_or(0);
                }
            }

            if let Some(device) = self.y_device {
                if ppem_y != 0 || coords != 0 {
                    y += device.get_y_delta(face).unwrap_or(0);
                }
            }
        }

        (x, y)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AnchorMatrix<'a> {
    pub data: &'a [u8],
    pub rows: u16,
    pub cols: u16,
    pub matrix: LazyArray32<'a, Offset16>,
}

impl<'a> AnchorMatrix<'a> {
    pub fn parse(data: &'a [u8], cols: u16) -> Option<Self> {
        let mut s = Stream::new(data);
        let rows = s.read::<u16>()?;
        let count = u32::from(rows) * u32::from(cols);
        let matrix = s.read_array32(count)?;
        Some(Self { data, rows, cols, matrix })
    }

    pub fn get(&self, row: u16, col: u16) -> Option<Anchor> {
        let idx = u32::from(row) * u32::from(self.cols) + u32::from(col);
        let offset = self.matrix.get(idx)?.to_usize();
        Anchor::parse(self.data.get(offset..)?)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MarkArray<'a> {
    pub data: &'a [u8],
    pub array: LazyArray16<'a, MarkRecord>,
}

impl<'a> MarkArray<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let array = s.read_array16(count)?;
        Some(Self { data, array })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MarkRecord {
    pub class: Class,
    pub mark_anchor: Offset16,
}

impl FromData for MarkRecord {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            class: s.read()?,
            mark_anchor: s.read()?,
        })
    }
}
