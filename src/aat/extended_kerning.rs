use core::convert::TryFrom;

use crate::Face;
use crate::buffer::{BufferScratchFlags, Buffer};
use crate::ot::{attach_type, ApplyContext, TableIndex};
use crate::tables::aat;
use crate::tables::{ankr, kerx};
use crate::tables::gsubgpos::LookupFlags;
use crate::plan::ShapePlan;
use crate::ot::matching::SkippyIter;

pub(crate) fn apply(plan: &ShapePlan, face: &Face, buffer: &mut Buffer) -> Option<()> {
    let mut seen_cross_stream = false;
    for (coverage, subtable) in face.kerx? {
        if coverage.is_variable() {
            continue;
        }

        if buffer.direction.is_horizontal() != coverage.is_horizontal() {
            continue;
        }

        let reverse = buffer.direction.is_backward();

        if !seen_cross_stream && coverage.has_cross_stream() {
            seen_cross_stream = true;

            // Attach all glyphs into a chain.
            for pos in &mut buffer.pos {
                pos.set_attach_type(attach_type::CURSIVE);
                pos.set_attach_chain(if buffer.direction.is_forward() { -1 } else { 1 });
                // We intentionally don't set BufferScratchFlags::HAS_GPOS_ATTACHMENT,
                // since there needs to be a non-zero attachment for post-positioning to
                // be needed.
            }
        }

        if reverse {
            buffer.reverse();
        }

        match subtable {
            kerx::Subtable::Format0(ref sub) => {
                if !plan.requested_kerning {
                    continue;
                }

                apply_simple_kerning(coverage, sub, plan, face, buffer);
            }
            kerx::Subtable::Format1(ref sub) => {
                let mut driver = Driver1 {
                    stack: [0; 8],
                    depth: 0,
                };

                apply_state_machine_kerning(coverage, sub, &mut driver, plan, buffer);
            }
            kerx::Subtable::Format2(ref sub) => {
                if !plan.requested_kerning {
                    continue;
                }

                apply_simple_kerning(coverage, sub, plan, face, buffer);
            }
            kerx::Subtable::Format4(ref sub) => {
                let mut driver = Driver4 {
                    mark_set: false,
                    mark: 0,
                    ankr_table: face.ankr.clone(),
                };

                apply_state_machine_kerning(coverage, sub, &mut driver, plan, buffer);
            }
            kerx::Subtable::Format6(ref sub) => {
                if !plan.requested_kerning {
                    continue;
                }

                apply_simple_kerning(coverage, sub, plan, face, buffer);
            }
        }

        if reverse {
            buffer.reverse();
        }
    }

    Some(())
}

fn apply_simple_kerning(
    coverage: kerx::Coverage,
    subtable: &dyn kerx::KerningPairs,
    plan: &ShapePlan,
    face: &Face,
    buffer: &mut Buffer,
) {
    let mut ctx = ApplyContext::new(TableIndex::GPOS, face, buffer);
    ctx.lookup_mask = plan.kern_mask;
    ctx.lookup_props = u32::from(LookupFlags::IGNORE_MARKS.bits());

    let horizontal = ctx.buffer.direction.is_horizontal();

    let mut i = 0;
    while i < ctx.buffer.len {
        if (ctx.buffer.info[i].mask & plan.kern_mask) == 0 {
            i += 1;
            continue;
        }

        let mut iter = SkippyIter::new(&ctx, i, 1, false);
        if !iter.next() {
            i += 1;
            continue;
        }

        let j = iter.index();

        let info = &ctx.buffer.info;
        let kern = subtable.glyphs_kerning(info[i].as_glyph(), info[j].as_glyph()).unwrap_or(0);
        let kern = i32::from(kern);

        let pos = &mut ctx.buffer.pos;
        if kern != 0 {
            if horizontal {
                if coverage.has_cross_stream() {
                    pos[j].y_offset = kern;
                    ctx.buffer.scratch_flags |= BufferScratchFlags::HAS_GPOS_ATTACHMENT;
                } else {
                    let kern1 = kern >> 1;
                    let kern2 = kern - kern1;
                    pos[i].x_advance += kern1;
                    pos[j].x_advance += kern2;
                    pos[j].x_offset += kern2;
                }
            } else {
                if coverage.has_cross_stream() {
                    pos[j].x_offset = kern;
                    ctx.buffer.scratch_flags |= BufferScratchFlags::HAS_GPOS_ATTACHMENT;
                } else {
                    let kern1 = kern >> 1;
                    let kern2 = kern - kern1;
                    pos[i].y_advance += kern1;
                    pos[j].y_advance += kern2;
                    pos[j].y_offset += kern2;
                }
            }

            ctx.buffer.unsafe_to_break(i, j + 1)
        }

        i = j;
    }
}

// TODO: use crate::tables::aat instead

fn apply_state_machine_kerning<E: aat::Entry, T: aat::StateTable2<E>>(
    coverage: kerx::Coverage,
    aat: &T,
    driver: &mut dyn StateTableDriver<T, E>,
    plan: &ShapePlan,
    buffer: &mut Buffer,
) {
    let mut state = aat::START_OF_TEXT;
    buffer.idx = 0;
    loop {
        let class = if buffer.idx < buffer.len {
            aat.class(buffer.info[buffer.idx].as_glyph()).unwrap_or(1)
        } else {
            aat::class::END_OF_TEXT
        };

        let entry: E = match aat.entry(state, class) {
            Some(v) => v,
            None => break,
        };

        // Unsafe-to-break before this if not in state 0, as things might
        // go differently if we start from state 0 here.
        if state != aat::START_OF_TEXT &&
            buffer.backtrack_len() != 0 &&
            buffer.idx < buffer.len
        {
            // If there's no value and we're just epsilon-transitioning to state 0, safe to break.
            if   entry.is_actionable() ||
                !(entry.new_state() == aat::START_OF_TEXT && !entry.has_advance())
            {
                buffer.unsafe_to_break_from_outbuffer(buffer.backtrack_len() - 1, buffer.idx + 1);
            }
        }

        // Unsafe-to-break if end-of-text would kick in here.
        if buffer.idx + 2 <= buffer.len {
            let end_entry: E = match aat.entry(state, aat::class::END_OF_TEXT) {
                Some(v) => v,
                None => break,
            };

            if end_entry.is_actionable() {
                buffer.unsafe_to_break(buffer.idx, buffer.idx + 2);
            }
        }

        let _ = driver.transition(aat, entry, coverage.has_cross_stream(), plan, buffer);

        state = entry.new_state();

        if buffer.idx >= buffer.len {
            break;
        }

        if entry.has_advance() || buffer.max_ops <= 0 {
            buffer.next_glyph();
        }
        buffer.max_ops -= 1;
    }
}


trait StateTableDriver<Table, Entry> {
    fn is_actionable(&self, entry: Entry) -> bool;
    fn transition(&mut self, aat: &Table, entry: Entry,
                  has_cross_stream: bool, plan: &ShapePlan, buffer: &mut Buffer) -> Option<()>;
}


struct Driver1 {
    stack: [usize; 8],
    depth: usize,
}

impl StateTableDriver<kerx::format1::StateTable<'_>, kerx::format1::Entry> for Driver1 {
    fn is_actionable(&self, entry: kerx::format1::Entry) -> bool {
        use aat::Entry;
        entry.is_actionable()
    }

    fn transition(
        &mut self,
        aat: &kerx::format1::StateTable,
        entry: kerx::format1::Entry,
        has_cross_stream: bool,
        plan: &ShapePlan,
        buffer: &mut Buffer,
    ) -> Option<()> {
        use aat::Entry;

        if entry.has_reset() {
            self.depth = 0;
        }

        if entry.has_push() {
            if self.depth < self.stack.len() {
                self.stack[self.depth] = buffer.idx;
                self.depth += 1;
            } else {
                self.depth = 0; // Probably not what CoreText does, but better?
            }
        }

        if entry.is_actionable() && self.depth != 0 {
            let tuple_count = u16::try_from(aat.tuple_count.max(1)).ok()?;

            let mut action_index = entry.action_index;

            // From Apple 'kern' spec:
            // "Each pops one glyph from the kerning stack and applies the kerning value to it.
            // The end of the list is marked by an odd value...
            let mut last = false;
            while !last && self.depth != 0 {
                self.depth -= 1;
                let idx = self.stack[self.depth];
                let mut v = aat.kerning(action_index)? as i32;
                action_index = action_index.checked_add(tuple_count)?;
                if idx >= buffer.len {
                    continue;
                }

                // "The end of the list is marked by an odd value..."
                last = v & 1 != 0;
                v &= !1;

                // Testing shows that CoreText only applies kern (cross-stream or not)
                // if none has been applied by previous subtables. That is, it does
                // NOT seem to accumulate as otherwise implied by specs.

                let mut has_gpos_attachment = false;
                let glyph_mask = buffer.info[idx].mask;
                let pos = &mut buffer.pos[idx];

                if buffer.direction.is_horizontal() {
                    if has_cross_stream {
                        // The following flag is undocumented in the spec, but described
                        // in the 'kern' table example.
                        if v == -0x8000 {
                            pos.set_attach_type(0);
                            pos.set_attach_chain(0);
                            pos.y_offset = 0;
                        } else if pos.attach_type() != 0 {
                            pos.y_offset += v;
                            has_gpos_attachment = true;
                        }
                    } else if glyph_mask & plan.kern_mask != 0 {
                        pos.x_advance += v;
                        pos.x_offset += v;
                    }
                } else {
                    if has_cross_stream {
                        // CoreText doesn't do crossStream kerning in vertical. We do.
                        if v == -0x8000 {
                            pos.set_attach_type(0);
                            pos.set_attach_chain(0);
                            pos.x_offset = 0;
                        } else if pos.attach_type() != 0 {
                            pos.x_offset += v;
                            has_gpos_attachment = true;
                        }
                    } else if glyph_mask & plan.kern_mask != 0 {
                        if pos.y_offset == 0 {
                            pos.y_advance += v;
                            pos.y_offset += v;
                        }
                    }
                }

                if has_gpos_attachment {
                    buffer.scratch_flags |= BufferScratchFlags::HAS_GPOS_ATTACHMENT;
                }
            }
        }

        Some(())
    }
}


struct Driver4<'a> {
    mark_set: bool,
    mark: usize,
    ankr_table: Option<ankr::Table<'a>>,
}

impl StateTableDriver<kerx::format4::StateTable<'_>, kerx::format4::Entry> for Driver4<'_> {
    // TODO: remove
    fn is_actionable(&self, entry: kerx::format4::Entry) -> bool {
        use aat::Entry;
        entry.is_actionable()
    }

    fn transition(
        &mut self,
        aat: &kerx::format4::StateTable,
        entry: kerx::format4::Entry,
        _has_cross_stream: bool,
        _opt: &ShapePlan,
        buffer: &mut Buffer,
    ) -> Option<()> {
        use ttf_parser::parser::{Stream, FromData};
        use aat::Entry;

        if self.mark_set && entry.is_actionable() && buffer.idx < buffer.len {
            let points_data_offset = usize::from(entry.action_index) * u16::SIZE;
            let mut s = Stream::new_at(aat.control_points_data, points_data_offset)?;

            // Note: I wasn't able to find any fonts that actually use
            // ControlPointActions and ControlPointCoordinateActions.
            // So they are commented out for now.
            match aat.action_type {
                kerx::format4::ActionType::ControlPointActions => {
                    // let mark_control_point: u16 = s.read()?;
                    // let curr_control_point: u16 = s.read()?;
                }
                kerx::format4::ActionType::AnchorPointActions => {
                    if let Some(ref ankr_table) = self.ankr_table {
                        let mark_anchor_point: u16 = s.read()?;
                        let curr_anchor_point: u16 = s.read()?;

                        let mark_idx = buffer.info[self.mark].as_glyph();
                        let mark_anchor = ankr_table.anchor(mark_idx, mark_anchor_point).unwrap_or_default();

                        let curr_idx = buffer.cur(0).as_glyph();
                        let curr_anchor = ankr_table.anchor(curr_idx, curr_anchor_point).unwrap_or_default();

                        let pos = buffer.cur_pos_mut();
                        pos.x_offset = i32::from(mark_anchor.x - curr_anchor.x);
                        pos.y_offset = i32::from(mark_anchor.y - curr_anchor.y);
                    }
                }
                kerx::format4::ActionType::ControlPointCoordinateActions => {
                    // let mark_x: i16 = s.read()?;
                    // let mark_y: i16 = s.read()?;
                    // let curr_x: i16 = s.read()?;
                    // let curr_y: i16 = s.read()?;
                    //
                    // let pos = buffer.cur_pos_mut();
                    // pos.x_offset = mark_x - curr_x;
                    // pos.y_offset = mark_y - curr_y;
                }
            }

            buffer.cur_pos_mut().set_attach_type(attach_type::MARK);
            let idx = buffer.idx;
            buffer.cur_pos_mut().set_attach_chain(self.mark as i16 - idx as i16);
            buffer.scratch_flags |= BufferScratchFlags::HAS_GPOS_ATTACHMENT;
        }

        if entry.has_mark() {
            self.mark_set = true;
            self.mark = buffer.idx;
        }

        Some(())
    }
}
