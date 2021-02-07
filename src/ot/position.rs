use ttf_parser::GlyphId;
use ttf_parser::parser::{Offset, Offset16};

use crate::{Direction, Face};
use crate::buffer::{Buffer, BufferScratchFlags, GlyphPosition};
use crate::plan::ShapePlan;
use crate::tables::gpos::*;
use crate::tables::gsubgpos::*;

use super::{LayoutLookup, LayoutTable, TableIndex};
use super::apply::{Apply, ApplyContext};
use super::matching::SkippyIter;

pub fn position_start(_: &Face, buffer: &mut Buffer) {
    let len = buffer.len;
    for pos in &mut buffer.pos[..len] {
        pos.set_attach_chain(0);
        pos.set_attach_type(0);
    }
}

pub fn position(plan: &ShapePlan, face: &Face, buffer: &mut Buffer) {
    super::apply_layout_table(plan, face, buffer, face.gpos.as_ref());
}

pub fn position_finish_advances(_: &Face, _: &mut Buffer) {}

pub fn position_finish_offsets(_: &Face, buffer: &mut Buffer) {
    let len = buffer.len;
    let direction = buffer.direction;

    // Handle attachments
    if buffer.scratch_flags.contains(BufferScratchFlags::HAS_GPOS_ATTACHMENT) {
        for i in 0..len {
            propagate_attachment_offsets(&mut buffer.pos, len, i, direction);
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

    match kind {
        attach_type::MARK => {
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
        attach_type::CURSIVE => {
            if direction.is_horizontal() {
                pos[i].y_offset += pos[j].y_offset;
            } else {
                pos[i].x_offset += pos[j].x_offset;
            }
        }
        _ => {}
    }
}

impl<'a> LayoutTable for PosTable<'a> {
    const INDEX: TableIndex = TableIndex::GPOS;
    const IN_PLACE: bool = true;

    type Lookup = PosLookup<'a>;

    fn get_lookup(&self, index: LookupIndex) -> Option<&Self::Lookup> {
        self.lookups.get(usize::from(index.0))?.as_ref()
    }
}

impl LayoutLookup for PosLookup<'_> {
    fn props(&self) -> u32 {
        self.props
    }

    fn is_reverse(&self) -> bool {
        false
    }

    fn covers(&self, glyph: GlyphId) -> bool {
        self.coverage.contains(glyph)
    }
}

impl Apply for PosLookup<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        if self.covers(ctx.buffer.cur(0).as_glyph()) {
            for subtable in &self.subtables {
                if subtable.apply(ctx).is_some() {
                    return Some(());
                }
            }
        }

        None
    }
}

impl Apply for PosLookupSubtable<'_> {
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

impl Apply for SinglePos<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph = ctx.buffer.cur(0).as_glyph();
        let (base, value) = match *self {
            Self::Format1 { data, coverage, value } => {
                coverage.get(glyph)?;
                (data, value)
            }
            Self::Format2 { data, coverage, flags, values } => {
                let index = coverage.get(glyph)?;
                let record = ValueRecord::new(values.get(usize::from(index))?, flags);
                (data, record)
            }
        };

        value.apply(ctx, base, ctx.buffer.idx);
        ctx.buffer.idx += 1;

        Some(())
    }
}

impl Apply for PairPos<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let first = ctx.buffer.cur(0).as_glyph();
        let index = self.coverage().get(first)?;

        let mut iter = SkippyIter::new(ctx, ctx.buffer.idx, 1, false);
        if !iter.next() {
            return None;
        }

        let pos = iter.index();
        let second = ctx.buffer.info[pos].as_glyph();

        let (base, flags, records) = match *self {
            Self::Format1 { flags, sets, .. } => {
                let data = sets.slice(index)?;
                let set = PairSet::parse(data, flags)?;
                let records = set.get(second)?;
                (data, flags, records)
            }
            Self::Format2 { data, flags, classes, matrix, .. } => {
                let classes = [classes[0].get(first).0, classes[1].get(second).0];
                let records = matrix.get(classes)?;
                (data, flags, records)
            }
        };

        // Note the intentional use of "|" instead of short-circuit "||".
        if records[0].apply(ctx, base, ctx.buffer.idx) | records[1].apply(ctx, base, pos) {
            ctx.buffer.unsafe_to_break(ctx.buffer.idx, pos + 1);
        }

        ctx.buffer.idx = pos + usize::from(!flags[1].is_empty());
        Some(())
    }
}

impl Apply for CursivePos<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let Self::Format1 { data, coverage, entry_exits } = *self;

        let this = ctx.buffer.cur(0).as_glyph();
        let entry = entry_exits.get(coverage.get(this)?)?.entry_anchor;
        if entry.is_null() {
            return None;
        }

        let mut iter = SkippyIter::new(ctx, ctx.buffer.idx, 1, false);
        if !iter.prev() {
            return None;
        }

        let i = iter.index();
        let prev = ctx.buffer.info[i].as_glyph();
        let exit = entry_exits.get(coverage.get(prev)?)?.exit_anchor;
        if exit.is_null() {
            return None;
        }

        let (exit_x, exit_y) = Anchor::parse(data.get(exit.to_usize()..)?)?.get(ctx.face);
        let (entry_x, entry_y) = Anchor::parse(data.get(entry.to_usize()..)?)?.get(ctx.face);

        let direction = ctx.buffer.direction;
        let j = ctx.buffer.idx;
        ctx.buffer.unsafe_to_break(i, j);

        let pos = &mut ctx.buffer.pos;
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
        if ctx.lookup_props as u16 & LookupFlags::RIGHT_TO_LEFT.bits() == 0 {
            core::mem::swap(&mut child, &mut parent);
            x_offset = -x_offset;
            y_offset = -y_offset;
        }

        // If child was already connected to someone else, walk through its old
        // chain and reverse the link direction, such that the whole tree of its
        // previous connection now attaches to new parent.  Watch out for case
        // where new parent is on the path from old chain...
        reverse_cursive_minor_offset(pos, child, direction, parent);

        pos[child].set_attach_type(attach_type::CURSIVE);
        pos[child].set_attach_chain((parent as isize - child as isize) as i16);

        ctx.buffer.scratch_flags |= BufferScratchFlags::HAS_GPOS_ATTACHMENT;
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

        ctx.buffer.idx += 1;
        Some(())
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
    if chain == 0 || attach_type & attach_type::CURSIVE == 0 {
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

impl Apply for MarkBasePos<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let Self::Format1 { mark_coverage, base_coverage, marks, base_matrix } = *self;

        let buffer = &ctx.buffer;
        let mark_glyph = ctx.buffer.cur(0).as_glyph();
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
        let base_glyph = info[idx].as_glyph();
        let base_index = base_coverage.get(base_glyph)?;

        marks.apply(ctx, base_matrix, mark_index, base_index, idx)
    }
}

impl Apply for MarkLigPos<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let Self::Format1 { mark_coverage, lig_coverage, marks, lig_array } = *self;

        let buffer = &ctx.buffer;
        let mark_glyph = ctx.buffer.cur(0).as_glyph();
        let mark_index = mark_coverage.get(mark_glyph)?;

        // Now we search backwards for a non-mark glyph
        let mut iter = SkippyIter::new(ctx, buffer.idx, 1, false);
        iter.set_lookup_props(u32::from(LookupFlags::IGNORE_MARKS.bits()));
        if !iter.prev() {
            return None;
        }

        // Checking that matched glyph is actually a ligature by GDEF is too strong; disabled

        let idx = iter.index();
        let lig_glyph = buffer.info[idx].as_glyph();
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

impl Apply for MarkMarkPos<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let Self::Format1 { mark1_coverage, mark2_coverage, marks, mark2_matrix } = *self;

        let buffer = &ctx.buffer;
        let mark1_glyph = ctx.buffer.cur(0).as_glyph();
        let mark1_index = mark1_coverage.get(mark1_glyph)?;

        // Now we search backwards for a suitable mark glyph until a non-mark glyph
        let mut iter = SkippyIter::new(ctx, buffer.idx, 1, false);
        iter.set_lookup_props(ctx.lookup_props & !u32::from(LookupFlags::IGNORE_FLAGS.bits()));
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

        let mark2_glyph = buffer.info[idx].as_glyph();
        let mark2_index = mark2_coverage.get(mark2_glyph)?;

        marks.apply(ctx, mark2_matrix, mark1_index, mark2_index, idx)
    }
}

impl<'a> ValueRecord<'a> {
    pub fn apply(&self, ctx: &mut ApplyContext, base: &[u8], idx: usize) -> bool {
        let mut s = ttf_parser::parser::Stream::new(self.data);

        let horizontal = ctx.buffer.direction.is_horizontal();
        let mut pos = ctx.buffer.pos[idx];
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
            let (ppem_x, ppem_y) = ctx.face.pixels_per_em().unwrap_or((0, 0));
            let coords = ctx.face.ttfp_face.variation_coordinates().len();
            let use_x_device = ppem_x != 0 || coords != 0;
            let use_y_device = ppem_y != 0 || coords != 0;

            if self.flags.contains(ValueFormatFlags::X_PLACEMENT_DEVICE) {
                if let Some(offset) = s.read::<Offset16>() {
                    if use_x_device && !offset.is_null() {
                        pos.x_offset += device_x_delta(base, offset, ctx.face);
                        worked = true;
                    }
                }
            }

            if self.flags.contains(ValueFormatFlags::Y_PLACEMENT_DEVICE) {
                if let Some(offset) = s.read::<Offset16>() {
                    if use_y_device && !offset.is_null() {
                        pos.y_offset += device_y_delta(base, offset, ctx.face);
                        worked = true;
                    }
                }
            }

            if self.flags.contains(ValueFormatFlags::X_ADVANCE_DEVICE) {
                if let Some(offset) = s.read::<Offset16>() {
                    if horizontal && use_x_device && !offset.is_null() {
                        pos.x_advance += device_x_delta(base, offset, ctx.face);
                        worked = true;
                    }
                }
            }

            if self.flags.contains(ValueFormatFlags::Y_ADVANCE_DEVICE) {
                if let Some(offset) = s.read::<Offset16>() {
                    if !horizontal && use_y_device && !offset.is_null() {
                        // y_advance values grow downward but face-space grows upward, hence negation
                        pos.y_advance -= device_y_delta(base, offset, ctx.face);
                        worked = true;
                    }
                }
            }
        }

        ctx.buffer.pos[idx] = pos;
        worked
    }
}

fn device_x_delta(base: &[u8], offset: Offset16, face: &Face) -> i32 {
    device(base, offset).and_then(|device| device.get_x_delta(face)).unwrap_or(0)
}

fn device_y_delta(base: &[u8], offset: Offset16, face: &Face) -> i32 {
    device(base, offset).and_then(|device| device.get_y_delta(face)).unwrap_or(0)
}

fn device(base: &[u8], offset: Offset16) -> Option<Device> {
    base.get(offset.to_usize()..).and_then(Device::parse)
}

impl<'a> MarkArray<'a> {
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

        let (mark_x, mark_y) = mark_anchor.get(ctx.face);
        let (base_x, base_y) = base_anchor.get(ctx.face);

        ctx.buffer.unsafe_to_break(glyph_pos, ctx.buffer.idx);

        let idx = ctx.buffer.idx;
        let pos = ctx.buffer.cur_pos_mut();
        pos.x_offset = base_x - mark_x;
        pos.y_offset = base_y - mark_y;
        pos.set_attach_type(attach_type::MARK);
        pos.set_attach_chain((glyph_pos as isize - idx as isize) as i16);

        ctx.buffer.scratch_flags |= BufferScratchFlags::HAS_GPOS_ATTACHMENT;
        ctx.buffer.idx += 1;

        Some(())
    }
}

pub mod attach_type {
    pub const MARK: u8 = 1;
    pub const CURSIVE: u8 = 2;
}
