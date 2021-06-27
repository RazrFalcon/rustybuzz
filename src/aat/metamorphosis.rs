use ttf_parser::parser::{LazyArray32, Offset32, Offset};

use crate::{Face, GlyphInfo};
use crate::buffer::Buffer;
use crate::plan::ShapePlan;
use crate::aat::{Map, MapBuilder, FeatureType};
use crate::aat::feature_selector;
use crate::tables::aat;
use crate::tables::morx;
use ttf_parser::GlyphId;

pub fn compile_flags(face: &Face, builder: &MapBuilder) -> Option<Map> {
    let mut map = Map::default();

    for chain in face.morx? {
        let mut flags = chain.default_flags();
        for feature in chain.features() {
            // Check whether this type/setting pair was requested in the map,
            // and if so, apply its flags.

            if builder.has_feature(feature.kind, feature.setting) {
                flags &= feature.disable_flags;
                flags |= feature.enable_flags;
            } else if feature.kind == FeatureType::LetterCase as u16 &&
                feature.setting == u16::from(feature_selector::SMALL_CAPS) {

                // Deprecated. https://github.com/harfbuzz/harfbuzz/issues/1342
                let ok = builder.has_feature(
                    FeatureType::LowerCase as u16,
                    u16::from(feature_selector::LOWER_CASE_SMALL_CAPS),
                );
                if ok {
                    flags &= feature.disable_flags;
                    flags |= feature.enable_flags;
                }
            }
        }

        map.chain_flags.push(flags);
    }

    Some(map)
}

pub fn apply(plan: &ShapePlan, face: &Face, buffer: &mut Buffer) -> Option<()> {
    for (chain_idx, chain) in face.morx?.enumerate() {
        let flags = plan.aat_map.chain_flags[chain_idx];
        for subtable in chain.subtables() {
            if subtable.feature_flags & flags == 0 {
                continue;
            }

            if !subtable.is_all_directions() &&
                buffer.direction.is_vertical() != subtable.is_vertical()
            {
                continue;
            }

            // Buffer contents is always in logical direction.  Determine if
            // we need to reverse before applying this subtable.  We reverse
            // back after if we did reverse indeed.
            //
            // Quoting the spec:
            // """
            // Bits 28 and 30 of the coverage field control the order in which
            // glyphs are processed when the subtable is run by the layout engine.
            // Bit 28 is used to indicate if the glyph processing direction is
            // the same as logical order or layout order. Bit 30 is used to
            // indicate whether glyphs are processed forwards or backwards within
            // that order.
            //
            // Bit 30   Bit 28   Interpretation for Horizontal Text
            //      0        0   The subtable is processed in layout order
            //                   (the same order as the glyphs, which is
            //                   always left-to-right).
            //      1        0   The subtable is processed in reverse layout order
            //                   (the order opposite that of the glyphs, which is
            //                   always right-to-left).
            //      0        1   The subtable is processed in logical order
            //                   (the same order as the characters, which may be
            //                   left-to-right or right-to-left).
            //      1        1   The subtable is processed in reverse logical order
            //                   (the order opposite that of the characters, which
            //                   may be right-to-left or left-to-right).

            let reverse = if subtable.is_logical() {
                subtable.is_backwards()
            } else {
                subtable.is_backwards() != buffer.direction.is_backward()
            };

            if reverse {
                buffer.reverse();
            }

            apply_subtable(&subtable.kind, face, buffer);

            if reverse {
                buffer.reverse();
            }
        }
    }

    Some(())
}

fn apply_subtable(kind: &morx::SubtableKind, face: &Face, buffer: &mut Buffer) {
    match kind {
        morx::SubtableKind::Rearrangement(ref table) => {
            let mut c = RearrangementCtx {
                start: 0,
                end: 0,
            };

            aat::drive::<()>(table, &mut c, buffer);
        }
        morx::SubtableKind::Contextual(ref table) => {
            let mut c = ContextualCtx {
                offsets_data: table.offsets_data,
                number_of_glyphs: face.ttfp_face.number_of_glyphs(),
                mark_set: false,
                mark: 0,
                offsets: table.offsets,
            };

            aat::drive::<morx::ContextualEntry>(&table.machine, &mut c, buffer);
        }
        morx::SubtableKind::Ligature(ref table) => {
            let mut c = LigatureCtx {
                table,
                match_length: 0,
                match_positions: [0; LIGATURE_MAX_MATCHES],
            };

            aat::drive::<u16>(&table.machine, &mut c, buffer);
        }
        morx::SubtableKind::NonContextual(ref lookup) => {
            let number_of_glyphs = face.ttfp_face.number_of_glyphs();
            for info in &mut buffer.info {
                if let Some(replacement) = lookup.value(info.as_glyph(), number_of_glyphs) {
                    info.glyph_id = u32::from(replacement);
                }
            }
        }
        morx::SubtableKind::Insertion(ref table) => {
            let mut c = InsertionCtx {
                mark: 0,
                glyphs: table.glyphs,
            };

            aat::drive::<morx::InsertionEntry>(&table.machine, &mut c, buffer);
        }
    }
}


struct RearrangementCtx {
    start: usize,
    end: usize,
}

impl RearrangementCtx {
    const MARK_FIRST: u16   = 0x8000;
    const DONT_ADVANCE: u16 = 0x4000;
    const MARK_LAST: u16    = 0x2000;
    const VERB: u16         = 0x000F;
}

impl aat::Driver<()> for RearrangementCtx {
    fn in_place(&self) -> bool {
        true
    }

    fn can_advance(&self, entry: &aat::Entry2<()>) -> bool {
        entry.flags & Self::DONT_ADVANCE == 0
    }

    fn is_actionable(&self, entry: &aat::Entry2<()>, _: &Buffer) -> bool {
        entry.flags & Self::VERB != 0 && self.start < self.end
    }

    fn transition(&mut self, entry: &aat::Entry2<()>, buffer: &mut Buffer) -> Option<()> {
        let flags = entry.flags;

        if flags & Self::MARK_FIRST != 0 {
            self.start = buffer.idx;
        }

        if flags & Self::MARK_LAST != 0 {
            self.end = (buffer.idx + 1).min(buffer.len);
        }

        if flags & Self::VERB != 0 && self.start < self.end {
            // The following map has two nibbles, for start-side
            // and end-side. Values of 0,1,2 mean move that many
            // to the other side. Value of 3 means move 2 and
            // flip them.
            const MAP: [u8; 16] = [
                0x00, // 0  no change
                0x10, // 1  Ax => xA
                0x01, // 2  xD => Dx
                0x11, // 3  AxD => DxA
                0x20, // 4  ABx => xAB
                0x30, // 5  ABx => xBA
                0x02, // 6  xCD => CDx
                0x03, // 7  xCD => DCx
                0x12, // 8  AxCD => CDxA
                0x13, // 9  AxCD => DCxA
                0x21, // 10 ABxD => DxAB
                0x31, // 11 ABxD => DxBA
                0x22, // 12 ABxCD => CDxAB
                0x32, // 13 ABxCD => CDxBA
                0x23, // 14 ABxCD => DCxAB
                0x33, // 15 ABxCD => DCxBA
            ];

            let m = MAP[usize::from(flags & Self::VERB)];
            let l = 2.min(m >> 4) as usize;
            let r = 2.min(m & 0x0F) as usize;
            let reverse_l = 3 == (m >> 4);
            let reverse_r = 3 == (m & 0x0F);

            if self.end - self.start >= usize::from(l + r) {
                buffer.merge_clusters(self.start, (buffer.idx + 1).min(buffer.len));
                buffer.merge_clusters(self.start, self.end);

                let mut buf = [GlyphInfo::default(); 4];

                for i in 0..l {
                    buf[i] = buffer.info[self.start + i];
                }

                for i in 0..r {
                    buf[i + 2] = buffer.info[self.end - r + i];
                }

                if l > r {
                    for i in 0..(self.end - self.start - l - r) {
                        buffer.info[self.start + r + i] = buffer.info[self.start + l + i];
                    }
                } else if l < r {
                    for i in (0..(self.end - self.start - l - r)).rev() {
                        buffer.info[self.start + r + i] = buffer.info[self.start + l + i];
                    }
                }

                for i in 0..r {
                    buffer.info[self.start + i] = buf[2 + i];
                }

                for i in 0..l {
                    buffer.info[self.end - l + i] = buf[i];
                }

                if reverse_l {
                    buffer.info.swap(self.end - 1, self.end - 2);
                }

                if reverse_r {
                    buffer.info.swap(self.start, self.start + 1);
                }
            }
        }

        Some(())
    }
}


struct ContextualCtx<'a> {
    offsets_data: &'a [u8],
    number_of_glyphs: u16,
    mark_set: bool,
    mark: usize,
    offsets: LazyArray32<'a, Offset32>,
}

impl ContextualCtx<'_> {
    const SET_MARK: u16         = 0x8000;
    const DONT_ADVANCE: u16     = 0x4000;
}

impl aat::Driver<morx::ContextualEntry> for ContextualCtx<'_> {
    fn in_place(&self) -> bool {
        true
    }

    fn can_advance(&self, entry: &aat::Entry2<morx::ContextualEntry>) -> bool {
        entry.flags & Self::DONT_ADVANCE == 0
    }

    fn is_actionable(&self, entry: &aat::Entry2<morx::ContextualEntry>, buffer: &Buffer) -> bool {
        if buffer.idx == buffer.len && !self.mark_set {
            return false;
        }

        return entry.extra.mark_index != 0xFFFF || entry.extra.current_index != 0xFFFF;
    }

    fn transition(&mut self, entry: &aat::Entry2<morx::ContextualEntry>, buffer: &mut Buffer) -> Option<()> {
        // Looks like CoreText applies neither mark nor current substitution for
        // end-of-text if mark was not explicitly set.
        if buffer.idx == buffer.len && !self.mark_set {
            return Some(());
        }

        let mut replacement = None;

        if entry.extra.mark_index != 0xFFFF {
            let offset = self.offsets.get(u32::from(entry.extra.mark_index))?.to_usize();
            let lookup = aat::Lookup::parse(&self.offsets_data[offset..])?;
            replacement = lookup.value(buffer.info[self.mark].as_glyph(), self.number_of_glyphs);
        }

        if let Some(replacement) = replacement {
            buffer.unsafe_to_break(self.mark, (buffer.idx +1 ).min(buffer.len));
            buffer.info[self.mark].glyph_id = u32::from(replacement);
        }

        replacement = None;
        let idx = buffer.idx.min(buffer.len - 1);
        if entry.extra.current_index != 0xFFFF {
            let offset = self.offsets.get(u32::from(entry.extra.current_index))?.to_usize();
            let lookup = aat::Lookup::parse(&self.offsets_data[offset..])?;
            replacement = lookup.value(buffer.info[idx].as_glyph(), self.number_of_glyphs);
        }

        if let Some(replacement) = replacement {
            buffer.info[idx].glyph_id = u32::from(replacement);
        }

        if entry.flags & Self::SET_MARK != 0 {
            self.mark_set = true;
            self.mark = buffer.idx;
        }

        Some(())
    }
}

struct InsertionCtx<'a> {
    mark: u32,
    glyphs: LazyArray32<'a, GlyphId>,
}

impl InsertionCtx<'_> {
    const SET_MARK: u16                 = 0x8000;
    const DONT_ADVANCE: u16             = 0x4000;
    const CURRENT_INSERT_BEFORE: u16    = 0x0800;
    const MARKED_INSERT_BEFORE: u16     = 0x0400;
    const CURRENT_INSERT_COUNT: u16     = 0x03E0;
    const MARKED_INSERT_COUNT: u16      = 0x001F;
}

impl aat::Driver<morx::InsertionEntry> for InsertionCtx<'_> {
    fn in_place(&self) -> bool {
        false
    }

    fn can_advance(&self, entry: &aat::Entry2<morx::InsertionEntry>) -> bool {
        entry.flags & Self::DONT_ADVANCE == 0
    }

    fn is_actionable(&self, entry: &aat::Entry2<morx::InsertionEntry>, _: &Buffer) -> bool {
        (entry.flags & (Self::CURRENT_INSERT_COUNT | Self::MARKED_INSERT_COUNT) != 0) &&
            (entry.extra.current_insert_index != 0xFFFF || entry.extra.marked_insert_index != 0xFFFF)
    }

    fn transition(&mut self, entry: &aat::Entry2<morx::InsertionEntry>, buffer: &mut Buffer) -> Option<()> {
        let flags = entry.flags;
        let mark_loc = buffer.out_len;

        if entry.extra.marked_insert_index != 0xFFFF {
            let count = flags & Self::MARKED_INSERT_COUNT;
            buffer.max_ops -= i32::from(count);
            if buffer.max_ops < 0 {
                return Some(());
            }

            let start = entry.extra.marked_insert_index;
            let before = flags & Self::MARKED_INSERT_BEFORE != 0;

            let end = buffer.out_len;
            buffer.move_to(self.mark as usize);

            if buffer.idx < buffer.len && !before {
                buffer.copy_glyph();
            }

            // TODO We ignore KashidaLike setting.
            for i in 0..count {
                let i = u32::from(start + i);
                buffer.output_glyph(u32::from(self.glyphs.get(i)?.0));
            }

            if buffer.idx < buffer.len && !before {
                buffer.skip_glyph();
            }

            buffer.move_to(end + usize::from(count));

            buffer.unsafe_to_break_from_outbuffer(self.mark as usize, (buffer.idx + 1).min(buffer.len));
        }

        if flags & Self::SET_MARK != 0 {
            self.mark = mark_loc as u32;
        }

        if entry.extra.current_insert_index != 0xFFFF {
            let count = (flags & Self::CURRENT_INSERT_COUNT) >> 5;
            buffer.max_ops -= i32::from(count);
            if buffer.max_ops < 0 {
                return Some(());
            }

            let start = entry.extra.current_insert_index;
            let before = flags & Self::CURRENT_INSERT_BEFORE != 0;
            let end = buffer.out_len;

            if buffer.idx < buffer.len && !before {
                buffer.copy_glyph();
            }

            // TODO We ignore KashidaLike setting.
            for i in 0..count {
                let i = u32::from(start + i);
                buffer.output_glyph(u32::from(self.glyphs.get(i)?.0));
            }

            if buffer.idx < buffer.len && !before {
                buffer.skip_glyph();
            }

            // Humm. Not sure where to move to. There's this wording under
            // DontAdvance flag:
            //
            // "If set, don't update the glyph index before going to the new state.
            // This does not mean that the glyph pointed to is the same one as
            // before. If you've made insertions immediately downstream of the
            // current glyph, the next glyph processed would in fact be the first
            // one inserted."
            //
            // This suggests that if DontAdvance is NOT set, we should move to
            // end+count. If it *was*, then move to end, such that newly inserted
            // glyphs are now visible.
            //
            // https://github.com/harfbuzz/harfbuzz/issues/1224#issuecomment-427691417
            buffer.move_to(if flags & Self::DONT_ADVANCE != 0 { end } else { end + usize::from(count) });
        }

        Some(())
    }
}



const LIGATURE_MAX_MATCHES: usize = 64;

struct LigatureCtx<'a> {
    table: &'a morx::LigatureSubtable<'a>,
    match_length: usize,
    match_positions: [usize; LIGATURE_MAX_MATCHES],
}

impl LigatureCtx<'_> {
    const SET_COMPONENT: u16    = 0x8000;
    const DONT_ADVANCE: u16     = 0x4000;
    const PERFORM_ACTION: u16   = 0x2000;

    const LIG_ACTION_LAST: u32      = 0x80000000;
    const LIG_ACTION_STORE: u32     = 0x40000000;
    const LIG_ACTION_OFFSET: u32    = 0x3FFFFFFF;
}

impl aat::Driver<u16> for LigatureCtx<'_> {
    fn in_place(&self) -> bool {
        false
    }

    fn can_advance(&self, entry: &aat::Entry2<u16>) -> bool {
        entry.flags & Self::DONT_ADVANCE == 0
    }

    fn is_actionable(&self, entry: &aat::Entry2<u16>, _: &Buffer) -> bool {
        entry.flags & Self::PERFORM_ACTION != 0
    }

    fn transition(&mut self, entry: &aat::Entry2<u16>, buffer: &mut Buffer) -> Option<()> {
        if entry.flags & Self::SET_COMPONENT != 0 {
            // Never mark same index twice, in case DONT_ADVANCE was used...
            if self.match_length != 0 &&
                self.match_positions[(self.match_length - 1) % LIGATURE_MAX_MATCHES] == buffer.out_len
            {
                self.match_length -= 1;
            }

            self.match_positions[self.match_length % LIGATURE_MAX_MATCHES] = buffer.out_len;
            self.match_length += 1;
        }

        if entry.flags & Self::PERFORM_ACTION != 0 {
            let end = buffer.out_len;

            if self.match_length == 0 {
                return Some(());
            }

            if buffer.idx >= buffer.len {
                return Some(()); // TODO: Work on previous instead?
            }

            let mut cursor = self.match_length;

            let mut ligature_actions_index = entry.extra;
            let mut ligature_idx = 0;
            loop {
                if cursor == 0 {
                    // Stack underflow. Clear the stack.
                    self.match_length = 0;
                    break;
                }

                cursor -= 1;
                buffer.move_to(self.match_positions[cursor % LIGATURE_MAX_MATCHES]);

                // We cannot use ? in this loop, because we must call
                // buffer.move_to(end) in the end.
                let action = match self.table.ligature_actions.get(u32::from(ligature_actions_index)) {
                    Some(v) => v,
                    None => break,
                };

                let mut uoffset = action & Self::LIG_ACTION_OFFSET;
                if uoffset & 0x20000000 != 0 {
                    uoffset |= 0xC0000000; // Sign-extend.
                }

                let offset = uoffset as i32;
                let component_idx = (buffer.cur(0).glyph_id as i32 + offset) as u32;
                ligature_idx += match self.table.components.get(component_idx) {
                    Some(v) => v,
                    None => break,
                };

                if (action & (Self::LIG_ACTION_STORE | Self::LIG_ACTION_LAST)) != 0 {
                    let lig = match self.table.ligatures.get(u32::from(ligature_idx)) {
                        Some(v) => v,
                        None => break,
                    };

                    buffer.replace_glyph(u32::from(lig.0));

                    let lig_end = self.match_positions[(self.match_length - 1) % LIGATURE_MAX_MATCHES] + 1;
                    // Now go and delete all subsequent components.
                    while self.match_length - 1 > cursor {
                        self.match_length -= 1;
                        buffer.move_to(self.match_positions[self.match_length % LIGATURE_MAX_MATCHES]);
                        buffer.replace_glyph(0xFFFF);
                    }

                    buffer.move_to(lig_end);
                    buffer.merge_out_clusters(
                        self.match_positions[cursor % LIGATURE_MAX_MATCHES],
                        buffer.out_len,
                    );
                }

                ligature_actions_index += 1;

                if action & Self::LIG_ACTION_LAST != 0 {
                    break;
                }
            }

            buffer.move_to(end);
        }

        Some(())
    }
}
