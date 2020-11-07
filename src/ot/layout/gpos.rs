//! The Glyph Positioning Table.

use std::convert::TryFrom;

use ttf_parser::parser::{
    FromData, LazyArray16, LazyArray32, NumFrom, Offset, Offset16, Offsets16, Stream,
};
use ttf_parser::GlyphId;

use super::apply::ApplyContext;
use super::common::{
    parse_extension_lookup, ClassDef, Coverage, Device, Class, LookupFlags, SubstPosTable,
};
use super::context_lookups::{ContextLookup, ChainContextLookup};
use super::dyn_array::DynArray;
use super::matching::SkippyIter;
use crate::buffer::{Buffer, BufferScratchFlags, GlyphPosition};
use crate::common::Direction;
use crate::{Font, Tag};

#[derive(Clone, Copy, Debug)]
pub struct PosTable<'a>(SubstPosTable<'a>);

impl<'a> PosTable<'a> {
    pub const TAG: Tag = Tag::from_bytes(b"GPOS");

    pub fn parse(data: &'a [u8]) -> Option<Self> {
        SubstPosTable::parse(data).map(Self)
    }

    pub(crate) fn position_start(_: &Font, buffer: &mut Buffer) {
        for pos in &mut buffer.pos {
            pos.set_attach_chain(0);
            pos.set_attach_type(0);
        }
    }

    pub(crate) fn position_finish_advances(_: &Font, _: &mut Buffer) {}

    pub(crate) fn position_finish_offsets(_: &Font, buffer: &mut Buffer) {
        let len = buffer.len;
        let direction = buffer.direction;

        // Handle attachments
        if buffer.scratch_flags.contains(BufferScratchFlags::HAS_GPOS_ATTACHMENT) {
            for i in 0..len {
                propagate_attachment_offsets(&mut buffer.pos, len, i, direction);
            }
        }
    }
}

fn propagate_attachment_offsets(
    pos: &mut [GlyphPosition],
    len: usize,
    i: usize,
    direction: Direction,
) {
    // Adjusts offsets of attached glyphs (both cursive and mark) to accumulate
    // offset of glyph they are attached to.
    let chain = pos[i].attach_chain();
    let kind = pos[i].attach_type();
    if chain == 0 {
        return;
    }

    pos[i].set_attach_chain(0);

    let j = (i as isize + isize::from(chain)) as _;
    if j >= len {
        return;
    }

    propagate_attachment_offsets(pos, len, j, direction);

    match AttachType::from_raw(kind).unwrap() {
        AttachType::Mark => {
            pos[i].x_offset += pos[j].x_offset;
            pos[i].y_offset += pos[j].y_offset;

            assert!(j < i);
            if direction.is_forward() {
                for k in j..i {
                    pos[i].x_offset -= pos[k].x_advance;
                    pos[i].y_offset -= pos[k].y_advance;
                }
            } else {
                for k in j+1..i+1 {
                    pos[i].x_offset += pos[k].x_advance;
                    pos[i].y_offset += pos[k].y_advance;
                }
            }
        }

        AttachType::Cursive => {
            if direction.is_horizontal() {
                pos[i].y_offset += pos[j].y_offset;
            } else {
                pos[i].x_offset += pos[j].x_offset;
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum PosLookupSubtable<'a> {
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
    fn parse(data: &'a [u8], kind: u16) -> Option<Self> {
        match kind {
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

    #[allow(dead_code)]
    fn coverage(&self) -> &Coverage<'a> {
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

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        match self {
            Self::Single(t) => t.apply(ctx),
            Self::Pair(t) => t.apply(ctx),
            Self::Cursive(t) => t.apply(ctx),
            Self::MarkBase(t) => t.apply(ctx),
            Self::MarkLig(t) => t.apply(ctx),
            Self::MarkMark(t) => t.apply(ctx),
            Self::Context(t) => t.apply(ctx),
            Self::ChainContext(t) => t.apply(ctx),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum SinglePos<'a> {
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
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let flags = s.read::<ValueFormatFlags>()?;
                let value = ValueRecord::read(&mut s, flags)?;
                Self::Format1 { data, coverage, value }
            }
            2 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let flags = s.read::<ValueFormatFlags>()?;
                let count = s.read::<u16>()?;
                let values = DynArray::read(&mut s, usize::from(count), flags.size())?;
                Self::Format2 { data, coverage, flags, values }
            }
            _ => return None,
        })
    }

    fn coverage(&self) -> &Coverage<'a> {
        match self {
            Self::Format1 { coverage, .. } => coverage,
            Self::Format2 { coverage, .. } => coverage,
        }
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph_id = GlyphId(u16::try_from(ctx.buffer().cur(0).codepoint).unwrap());
        let (base, value) = match *self {
            Self::Format1 { data, coverage, value } => {
                coverage.get(glyph_id)?;
                (data, value)
            }
            Self::Format2 { data, coverage, flags, values } => {
                let index = coverage.get(glyph_id)?;
                let record = ValueRecord::new(values.get(usize::from(index))?, flags);
                (data, record)
            }
        };

        value.apply(ctx, base, ctx.buffer().idx);
        ctx.buffer_mut().idx += 1;

        Some(())
    }
}

#[derive(Clone, Copy, Debug)]
enum PairPos<'a> {
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
        counts: [u16; 2],
        matrix: DynArray<'a>,
    },
}

impl<'a> PairPos<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let flags = [
                    s.read::<ValueFormatFlags>()?,
                    s.read::<ValueFormatFlags>()?,
                ];
                let count = s.read::<u16>()?;
                let sets = s.read_offsets16(count, data)?;
                Self::Format1 { coverage, flags, sets }
            }
            2 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let flags = [
                    s.read::<ValueFormatFlags>()?,
                    s.read::<ValueFormatFlags>()?,
                ];
                let classes = [
                    ClassDef::parse(s.read_offset16_data()?)?,
                    ClassDef::parse(s.read_offset16_data()?)?,
                ];
                let counts = [s.read::<u16>()?, s.read::<u16>()?];
                let count = usize::num_from(u32::from(counts[0]) * u32::from(counts[1]));
                let stride = flags[0].size() + flags[1].size();
                let matrix = DynArray::read(&mut s, count, stride)?;
                Self::Format2 { data, coverage, flags, classes, counts, matrix }
            },
            _ => return None,
        })
    }

    fn coverage(&self) -> &Coverage<'a> {
        match self {
            Self::Format1 { coverage, .. } => coverage,
            Self::Format2 { coverage, .. } => coverage,
        }
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let first = GlyphId(u16::try_from(ctx.buffer().cur(0).codepoint).unwrap());
        let index = self.coverage().get(first)?;

        let mut iter = SkippyIter::new(ctx, ctx.buffer().idx, 1, false);
        if !iter.next() {
            return None;
        }

        let pos = iter.index();
        let second = GlyphId(u16::try_from(ctx.buffer().info[pos].codepoint).unwrap());

        let (base, flags, mut s) = match *self {
            Self::Format1 { flags, sets, .. } => {
                let data = sets.slice(index)?;
                let mut s = Stream::new(data);
                let count = s.read::<u16>()?;
                let stride = GlyphId::SIZE + flags[0].size() + flags[1].size();
                let records = DynArray::read(&mut s, usize::from(count), stride)?;
                let record = records.binary_search_by(|data| {
                    Stream::new(data).read::<GlyphId>().unwrap().cmp(&second)
                })?;

                let mut s = Stream::new(record);
                s.skip::<GlyphId>();
                (data, flags, s)
            }
            Self::Format2 { data, flags, classes, counts, matrix, .. } => {
                let classes = [classes[0].get(first).0, classes[1].get(second).0];
                if classes[0] >= counts[0] || classes[1] >= counts[1] {
                    return None;
                }

                let idx = usize::from(classes[0]) * usize::from(counts[1]) + usize::from(classes[1]);
                let record = matrix.get(idx)?;
                (data, flags, Stream::new(record))
            }
        };

        let records = [
            ValueRecord::read(&mut s, flags[0])?,
            ValueRecord::read(&mut s, flags[1])?,
        ];

        // Note the intentional use of "|" instead of short-circuit "||".
        if records[0].apply(ctx, base, ctx.buffer().idx) | records[1].apply(ctx, base, pos) {
            let start = ctx.buffer().idx;
            ctx.buffer_mut().unsafe_to_break(start, pos + 1);
        }

        ctx.buffer_mut().idx = pos + usize::from(!flags[1].is_empty());
        Some(())
    }
}

#[derive(Clone, Copy, Debug)]
enum CursivePos<'a> {
    Format1 {
        data: &'a [u8],
        coverage: Coverage<'a>,
        entry_exits: LazyArray16<'a, EntryExitRecord>,
    }
}

impl<'a> CursivePos<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let count = s.read::<u16>()?;
                let entry_exits = s.read_array16(count)?;
                Self::Format1 { data, coverage, entry_exits }
            }
            _ => return None,
        })
    }

    fn coverage(&self) -> &Coverage<'a> {
        match self {
            Self::Format1 { coverage, .. } => coverage,
        }
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let Self::Format1 { data, coverage, entry_exits } = *self;

        let this = GlyphId(u16::try_from(ctx.buffer().cur(0).codepoint).unwrap());
        let entry = entry_exits.get(coverage.get(this)?)?.entry_anchor;
        if entry.is_null() {
            return None;
        }

        let mut iter = SkippyIter::new(ctx, ctx.buffer().idx, 1, false);
        if !iter.prev() {
            return None;
        }

        let i = iter.index();
        let prev = GlyphId(u16::try_from(ctx.buffer().info[i].codepoint).unwrap());
        let exit = entry_exits.get(coverage.get(prev)?)?.exit_anchor;
        if exit.is_null() {
            return None;
        }

        let font = ctx.font();
        let (exit_x, exit_y) = Anchor::parse(data.get(exit.to_usize()..)?)?.get(font);
        let (entry_x, entry_y) = Anchor::parse(data.get(entry.to_usize()..)?)?.get(font);

        let direction = ctx.direction();
        let lookup_props = ctx.lookup_props();
        let buffer = ctx.buffer_mut();
        let j = buffer.idx;
        buffer.unsafe_to_break(i, j);

        let pos = &mut buffer.pos;
        match direction {
            Direction::LeftToRight => {
                pos[i].x_advance = exit_x + pos[i].x_offset;
                let d = entry_x + pos[j].x_offset;
                pos[j].x_advance -= d;
                pos[j].x_offset -= d;
            }
            Direction::RightToLeft => {
                let d = exit_x + pos[i].x_offset;
                pos[i].x_advance -= d;
                pos[i].x_offset -= d;
                pos[j].x_advance = entry_x + pos[j].x_offset;
            }
            Direction::TopToBottom => {
                pos[i].y_advance = exit_y + pos[i].y_offset;
                let d = entry_y + pos[j].y_offset;
                pos[j].y_advance -= d;
                pos[j].y_offset -= d;
            }
            Direction::BottomToTop => {
                let d = exit_y + pos[i].y_offset;
                pos[i].y_advance -= d;
                pos[i].y_offset -= d;
                pos[j].y_advance = entry_y;
            }
            Direction::Invalid => {}
        }

        // Cross-direction adjustment

        // We attach child to parent (think graph theory and rooted trees whereas
        // the root stays on baseline and each node aligns itself against its
        // parent.
        //
        // Optimize things for the case of RightToLeft, as that's most common in
        // Arabic.
        let mut child = i;
        let mut parent = j;
        let mut x_offset = entry_x - exit_x;
        let mut y_offset = entry_y - exit_y;

        // Low bits are lookup flags, so we want to truncate.
        if lookup_props as u16 & LookupFlags::RIGHT_TO_LEFT.bits() == 0 {
            std::mem::swap(&mut child, &mut parent);
            x_offset = -x_offset;
            y_offset = -y_offset;
        }

        // If child was already connected to someone else, walk through its old
        // chain and reverse the link direction, such that the whole tree of its
        // previous connection now attaches to new parent.  Watch out for case
        // where new parent is on the path from old chain...
        reverse_cursive_minor_offset(pos, child, direction, parent);

        pos[child].set_attach_type(AttachType::Cursive as u8);
        pos[child].set_attach_chain((parent as isize - child as isize) as i16);

        buffer.scratch_flags |= BufferScratchFlags::HAS_GPOS_ATTACHMENT;
        if direction.is_horizontal() {
            pos[child].y_offset = y_offset;
        } else {
            pos[child].x_offset = x_offset;
        }

        // If parent was attached to child, break them free.
        // https://github.com/harfbuzz/harfbuzz/issues/2469
        if pos[parent].attach_chain() == -pos[child].attach_chain() {
            pos[parent].set_attach_chain(0);
        }

        buffer.idx += 1;
        Some(())
    }
}

#[derive(Clone, Copy, Debug)]
struct EntryExitRecord {
    entry_anchor: Offset16,
    exit_anchor: Offset16,
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

fn reverse_cursive_minor_offset(
    pos: &mut [GlyphPosition],
    i: usize,
    direction: Direction,
    new_parent: usize,
) {
    let chain = pos[i].attach_chain();
    let attach_type = pos[i].attach_type();
    if chain == 0 || attach_type & AttachType::Cursive as u8 == 0 {
        return;
    }

    pos[i].set_attach_chain(0);

    // Stop if we see new parent in the chain.
    let j = (i as isize + isize::from(chain)) as _;
    if j == new_parent {
        return;
    }

    reverse_cursive_minor_offset(pos, j, direction, new_parent);

    if direction.is_horizontal() {
        pos[j].y_offset = -pos[i].y_offset;
    } else {
        pos[j].x_offset = -pos[i].x_offset;
    }

    pos[j].set_attach_chain(-chain);
    pos[j].set_attach_type(attach_type);
}

#[derive(Clone, Copy, Debug)]
enum MarkBasePos<'a> {
    Format1 {
        mark_coverage: Coverage<'a>,
        base_coverage: Coverage<'a>,
        marks: MarkArray<'a>,
        base_matrix: AnchorMatrix<'a>,
    }
}

impl<'a> MarkBasePos<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let mark_coverage = Coverage::parse(s.read_offset16_data()?)?;
                let base_coverage = Coverage::parse(s.read_offset16_data()?)?;
                let class_count = s.read::<u16>()?;
                let marks = MarkArray::parse(s.read_offset16_data()?)?;
                let base_matrix = AnchorMatrix::parse(s.read_offset16_data()?, class_count)?;
                Self::Format1 { mark_coverage, base_coverage, marks, base_matrix }
            }
            _ => return None,
        })
    }

    fn coverage(&self) -> &Coverage<'a> {
        match self {
            Self::Format1 { mark_coverage, .. } => mark_coverage,
        }
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let Self::Format1 { mark_coverage, base_coverage, marks, base_matrix } = *self;

        let buffer = ctx.buffer();
        let mark_glyph = GlyphId(u16::try_from(buffer.cur(0).codepoint).unwrap());
        let mark_index = mark_coverage.get(mark_glyph)?;

        // Now we search backwards for a non-mark glyph
        let mut iter = SkippyIter::new(ctx, buffer.idx, 1, false);
        iter.set_lookup_props(u32::from(LookupFlags::IGNORE_MARKS.bits()));

        let info = &buffer.info;
        loop {
            if !iter.prev() {
                return None;
            }

            // We only want to attach to the first of a MultipleSubst sequence.
            // https://github.com/harfbuzz/harfbuzz/issues/740
            // Reject others...
            // ...but stop if we find a mark in the MultipleSubst sequence:
            // https://github.com/harfbuzz/harfbuzz/issues/1020
            let idx = iter.index();
            if !info[idx].is_multiplied()
                || info[idx].lig_comp() == 0
                || idx == 0
                || info[idx - 1].is_mark()
                || info[idx].lig_id() != info[idx - 1].lig_id()
                || info[idx].lig_comp() != info[idx - 1].lig_comp() + 1
            {
                break;
            }
            iter.reject();
        }

        // Checking that matched glyph is actually a base glyph by GDEF is too strong; disabled

        let idx = iter.index();
        let base_glyph = GlyphId(u16::try_from(info[idx].codepoint).unwrap());
        let base_index = base_coverage.get(base_glyph)?;

        marks.apply(ctx, base_matrix, mark_index, base_index, idx)
    }
}

#[derive(Clone, Copy, Debug)]
enum MarkLigPos<'a> {
    Format1 {
        mark_coverage: Coverage<'a>,
        lig_coverage: Coverage<'a>,
        marks: MarkArray<'a>,
        lig_array: LigatureArray<'a>,
    }
}

impl<'a> MarkLigPos<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let mark_coverage = Coverage::parse(s.read_offset16_data()?)?;
                let lig_coverage = Coverage::parse(s.read_offset16_data()?)?;
                let class_count = s.read::<u16>()?;
                let marks = MarkArray::parse(s.read_offset16_data()?)?;
                let lig_array = LigatureArray::parse(s.read_offset16_data()?, class_count)?;
                Self::Format1 { mark_coverage, lig_coverage, marks, lig_array }
            }
            _ => return None,
        })
    }

    fn coverage(&self) -> &Coverage<'a> {
        match self {
            Self::Format1 { mark_coverage, .. } => mark_coverage,
        }
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let Self::Format1 { mark_coverage, lig_coverage, marks, lig_array } = *self;

        let buffer = ctx.buffer();
        let mark_glyph = GlyphId(u16::try_from(buffer.cur(0).codepoint).unwrap());
        let mark_index = mark_coverage.get(mark_glyph)?;

        // Now we search backwards for a non-mark glyph
        let mut iter = SkippyIter::new(ctx, buffer.idx, 1, false);
        iter.set_lookup_props(u32::from(LookupFlags::IGNORE_MARKS.bits()));
        if !iter.prev() {
            return None;
        }

        // Checking that matched glyph is actually a ligature by GDEF is too strong; disabled

        let idx = iter.index();
        let lig_glyph = GlyphId(u16::try_from(buffer.info[idx].codepoint).unwrap());
        let lig_index = lig_coverage.get(lig_glyph)?;
        let lig_attach = lig_array.get(lig_index)?;

        // Find component to attach to
        let comp_count = lig_attach.rows;
        if comp_count == 0 {
            return None;
        }

        // We must now check whether the ligature ID of the current mark glyph
        // is identical to the ligature ID of the found ligature.  If yes, we
        // can directly use the component index.  If not, we attach the mark
        // glyph to the last component of the ligature.
        let lig_id = buffer.info[idx].lig_id();
        let mark_id = buffer.cur(0).lig_id();
        let mark_comp = u16::from(buffer.cur(0).lig_comp());
        let matches = lig_id != 0 && lig_id == mark_id && mark_comp > 0;
        let comp_index = if matches { mark_comp.min(comp_count) } else { comp_count } - 1;

        marks.apply(ctx, lig_attach, mark_index, comp_index, idx)
    }
}

#[derive(Clone, Copy, Debug)]
struct LigatureArray<'a> {
    class_count: u16,
    attach: Offsets16<'a, Offset16>,
}

impl<'a> LigatureArray<'a> {
    fn parse(data: &'a [u8], class_count: u16) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let attach = s.read_offsets16(count, data)?;
        Some(Self { class_count, attach })
    }

    fn get(&self, idx: u16) -> Option<AnchorMatrix> {
        AnchorMatrix::parse(self.attach.slice(idx)?, self.class_count)
    }
}

#[derive(Clone, Copy, Debug)]
enum MarkMarkPos<'a> {
    Format1 {
        mark1_coverage: Coverage<'a>,
        mark2_coverage: Coverage<'a>,
        marks: MarkArray<'a>,
        mark2_matrix: AnchorMatrix<'a>,
    }
}

impl<'a> MarkMarkPos<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let mark1_coverage = Coverage::parse(s.read_offset16_data()?)?;
                let mark2_coverage = Coverage::parse(s.read_offset16_data()?)?;
                let class_count = s.read::<u16>()?;
                let marks = MarkArray::parse(s.read_offset16_data()?)?;
                let mark2_matrix = AnchorMatrix::parse(s.read_offset16_data()?, class_count)?;
                Self::Format1 { mark1_coverage, mark2_coverage, marks, mark2_matrix }
            }
            _ => return None,
        })
    }

    fn coverage(&self) -> &Coverage<'a> {
        match self {
            Self::Format1 { mark1_coverage, .. } => mark1_coverage,
        }
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let Self::Format1 { mark1_coverage, mark2_coverage, marks, mark2_matrix } = *self;

        let buffer = ctx.buffer();
        let mark1_glyph = GlyphId(u16::try_from(buffer.cur(0).codepoint).unwrap());
        let mark1_index = mark1_coverage.get(mark1_glyph)?;

        // Now we search backwards for a suitable mark glyph until a non-mark glyph
        let mut iter = SkippyIter::new(ctx, buffer.idx, 1, false);
        iter.set_lookup_props(ctx.lookup_props() & !u32::from(LookupFlags::IGNORE_FLAGS.bits()));
        if !iter.prev() {
            return None;
        }

        let idx = iter.index();
        if !buffer.info[idx].is_mark() {
            return None;
        }

        let id1 = buffer.cur(0).lig_id();
        let id2 = buffer.info[idx].lig_id();
        let comp1 = buffer.cur(0).lig_comp();
        let comp2 = buffer.info[idx].lig_comp();

        let matches = if id1 == id2 {
            // Marks belonging to the same base
            // or marks belonging to the same ligature component.
            id1 == 0 || comp1 == comp2
        } else {
            // If ligature ids don't match, it may be the case that one of the marks
            // itself is a ligature.  In which case match.
            (id1 > 0 && comp1 == 0) || (id2 > 0 && comp2 == 0)
        };

        if !matches {
            return None;
        }

        let mark2_glyph = GlyphId(u16::try_from(buffer.info[idx].codepoint).unwrap());
        let mark2_index = mark2_coverage.get(mark2_glyph)?;

        marks.apply(ctx, mark2_matrix, mark1_index, mark2_index, idx)
    }
}

#[derive(Clone, Copy, Debug)]
struct ValueRecord<'a> {
    data: &'a [u8],
    flags: ValueFormatFlags,
}

impl<'a> ValueRecord<'a> {
    fn new(data: &'a [u8], flags: ValueFormatFlags) -> Self {
        Self { data, flags }
    }

    fn read(s: &mut Stream<'a>, flags: ValueFormatFlags) -> Option<Self> {
        s.read_bytes(flags.size()).map(|data| Self { data, flags })
    }

    fn apply(&self, ctx: &mut ApplyContext, base: &[u8], idx: usize) -> bool {
        let mut s = Stream::new(self.data);

        let horizontal = ctx.direction().is_horizontal();
        let mut pos = ctx.buffer().pos[idx];
        let mut worked = false;

        if self.flags.contains(ValueFormatFlags::X_PLACEMENT) {
            if let Some(delta) = s.read::<i16>() {
                pos.x_offset += i32::from(delta);
                worked |= delta != 0;
            }
        }

        if self.flags.contains(ValueFormatFlags::Y_PLACEMENT) {
            if let Some(delta) = s.read::<i16>() {
                pos.y_offset += i32::from(delta);
                worked |= delta != 0;
            }
        }

        if self.flags.contains(ValueFormatFlags::X_ADVANCE) {
            if let Some(delta) = s.read::<i16>() {
                if horizontal {
                    pos.x_advance += i32::from(delta);
                    worked |= delta != 0;
                }
            }
        }

        if self.flags.contains(ValueFormatFlags::Y_ADVANCE) {
            if let Some(delta) = s.read::<i16>() {
                if !horizontal {
                    // y_advance values grow downward but font-space grows upward, hence negation
                    pos.y_advance -= i32::from(delta);
                    worked |= delta != 0;
                }
            }
        }

        if self.flags.intersects(ValueFormatFlags::DEVICES) {
            let font = ctx.font();
            let (ppem_x, ppem_y) = font.pixels_per_em().unwrap_or((0, 0));
            let coords = font.ttfp_face.variation_coordinates().len();
            let use_x_device = ppem_x != 0 || coords != 0;
            let use_y_device = ppem_y != 0 || coords != 0;

            if self.flags.contains(ValueFormatFlags::X_PLACEMENT_DEVICE) {
                if let Some(offset) = s.read::<Offset16>() {
                    if use_x_device && !offset.is_null() {
                        pos.x_offset += device_x_delta(base, offset, font);
                        worked = true;
                    }
                }
            }

            if self.flags.contains(ValueFormatFlags::Y_PLACEMENT_DEVICE) {
                if let Some(offset) = s.read::<Offset16>() {
                    if use_y_device && !offset.is_null() {
                        pos.y_offset += device_y_delta(base, offset, font);
                        worked = true;
                    }
                }
            }

            if self.flags.contains(ValueFormatFlags::X_ADVANCE_DEVICE) {
                if let Some(offset) = s.read::<Offset16>() {
                    if horizontal && use_x_device && !offset.is_null() {
                        pos.x_advance += device_x_delta(base, offset, font);
                        worked = true;
                    }
                }
            }

            if self.flags.contains(ValueFormatFlags::Y_ADVANCE_DEVICE) {
                if let Some(offset) = s.read::<Offset16>() {
                    if !horizontal && use_y_device && !offset.is_null() {
                        // y_advance values grow downward but font-space grows upward, hence negation
                        pos.y_advance -= device_y_delta(base, offset, font);
                        worked = true;
                    }
                }
            }
        }

        ctx.buffer_mut().pos[idx] = pos;
        worked
    }
}

fn device_x_delta(base: &[u8], offset: Offset16, font: &Font) -> i32 {
    device(base, offset).and_then(|device| device.get_x_delta(font)).unwrap_or(0)
}

fn device_y_delta(base: &[u8], offset: Offset16, font: &Font) -> i32 {
    device(base, offset).and_then(|device| device.get_y_delta(font)).unwrap_or(0)
}

fn device(base: &[u8], offset: Offset16) -> Option<Device> {
    base.get(offset.to_usize()..).and_then(Device::parse)
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
    fn size(self) -> usize {
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
struct Anchor<'a> {
    x: i16,
    y: i16,
    x_device: Option<Device<'a>>,
    y_device: Option<Device<'a>>,
}

impl<'a> Anchor<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
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

    fn get(&self, font: &Font) -> (i32, i32) {
        let mut x = i32::from(self.x);
        let mut y = i32::from(self.y);

        if self.x_device.is_some() || self.y_device.is_some() {
            let (ppem_x, ppem_y) = font.pixels_per_em().unwrap_or((0, 0));
            let coords = font.ttfp_face.variation_coordinates().len();

            if let Some(device) = self.x_device {
                if ppem_x != 0 || coords != 0 {
                    x += device.get_x_delta(font).unwrap_or(0);
                }
            }

            if let Some(device) = self.y_device {
                if ppem_y != 0 || coords != 0 {
                    y += device.get_y_delta(font).unwrap_or(0);
                }
            }
        }

        (x, y)
    }
}

#[derive(Clone, Copy, Debug)]
struct AnchorMatrix<'a> {
    data: &'a [u8],
    rows: u16,
    cols: u16,
    matrix: LazyArray32<'a, Offset16>,
}

impl<'a> AnchorMatrix<'a> {
    fn parse(data: &'a [u8], cols: u16) -> Option<Self> {
        let mut s = Stream::new(data);
        let rows = s.read::<u16>()?;
        let count = u32::from(rows) * u32::from(cols);
        let matrix = s.read_array32(count)?;
        Some(Self { data, rows, cols, matrix })
    }

    fn get(&self, row: u16, col: u16) -> Option<Anchor> {
        let idx = u32::from(row) * u32::from(self.cols) + u32::from(col);
        let offset = self.matrix.get(idx)?.to_usize();
        Anchor::parse(self.data.get(offset..)?)
    }
}

#[derive(Clone, Copy, Debug)]
struct MarkArray<'a> {
    data: &'a [u8],
    array: LazyArray16<'a, MarkRecord>,
}

impl<'a> MarkArray<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let array = s.read_array16(count)?;
        Some(Self { data, array })
    }

    fn apply(
        &self,
        ctx: &mut ApplyContext,
        anchors: AnchorMatrix,
        mark_index: u16,
        glyph_index: u16,
        glyph_pos: usize,
    ) -> Option<()> {
        // If this subtable doesn't have an anchor for this base and this class
        // return `None` such that the subsequent subtables have a chance at it.
        let record = self.array.get(mark_index)?;
        let mark_anchor = Anchor::parse(self.data.get(record.mark_anchor.to_usize()..)?)?;
        let base_anchor = anchors.get(glyph_index, record.class.0)?;

        let font = ctx.font();
        let (mark_x, mark_y) = mark_anchor.get(font);
        let (base_x, base_y) = base_anchor.get(font);

        let buffer = ctx.buffer_mut();
        buffer.unsafe_to_break(glyph_pos, buffer.idx);

        let idx = buffer.idx;
        let pos = buffer.cur_pos_mut();
        pos.x_offset = base_x - mark_x;
        pos.y_offset = base_y - mark_y;
        pos.set_attach_type(AttachType::Mark as u8);
        pos.set_attach_chain((glyph_pos as isize - idx as isize) as i16);

        buffer.scratch_flags |= BufferScratchFlags::HAS_GPOS_ATTACHMENT;
        buffer.idx += 1;

        Some(())
    }
}

#[derive(Clone, Copy, Debug)]
struct MarkRecord {
    class: Class,
    mark_anchor: Offset16,
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

#[derive(Clone, Copy, Debug)]
enum AttachType {
    Mark = 1,
    Cursive = 2,
}

impl AttachType {
    fn from_raw(kind: u8) -> Option<Self> {
        match kind {
            1 => Some(Self::Mark),
            2 => Some(Self::Cursive),
            _ => None,
        }
    }
}

#[no_mangle]
pub extern "C" fn rb_pos_lookup_apply(
    data: *const u8,
    ctx: *mut crate::ffi::rb_ot_apply_context_t,
    kind: u32,
) -> crate::ffi::rb_bool_t {
    let data = unsafe { std::slice::from_raw_parts(data, isize::MAX as usize) };
    let mut ctx = ApplyContext::from_ptr_mut(ctx);
    PosLookupSubtable::parse(data, kind as u16)
        .map(|table| table.apply(&mut ctx).is_some())
        .unwrap_or(false) as crate::ffi::rb_bool_t
}
