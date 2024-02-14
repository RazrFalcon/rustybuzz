use core::convert::TryFrom;

use ttf_parser::gsub::*;
use ttf_parser::GlyphId;

use crate::buffer::{Buffer, GlyphPropsFlags};
use crate::plan::ShapePlan;
use crate::unicode::GeneralCategory;
use crate::Face;

use super::apply::{Apply, ApplyContext, WouldApply, WouldApplyContext};
use super::matching::{match_backtrack, match_glyph, match_input, match_lookahead};
use super::{
    LayoutLookup, LayoutTable, Map, SubstLookup, SubstitutionTable, TableIndex, MAX_CONTEXT_LENGTH,
    MAX_NESTING_LEVEL,
};
use ttf_parser::opentype_layout::LookupIndex;

/// Called before substitution lookups are performed, to ensure that glyph
/// class and other properties are set on the glyphs in the buffer.
pub fn substitute_start(face: &Face, buffer: &mut Buffer) {
    set_glyph_props(face, buffer)
}

pub fn substitute(plan: &ShapePlan, face: &Face, buffer: &mut Buffer) {
    super::apply_layout_table(plan, face, buffer, face.gsub.as_ref());
}

fn set_glyph_props(face: &Face, buffer: &mut Buffer) {
    let len = buffer.len;
    for info in &mut buffer.info[..len] {
        info.set_glyph_props(face.glyph_props(info.as_glyph()));
        info.set_lig_props(0);
        info.set_syllable(0);
    }
}

impl<'a> LayoutTable for SubstitutionTable<'a> {
    const INDEX: TableIndex = TableIndex::GSUB;
    const IN_PLACE: bool = false;

    type Lookup = SubstLookup<'a>;

    fn get_lookup(&self, index: LookupIndex) -> Option<&Self::Lookup> {
        self.lookups.get(usize::from(index))
    }
}

impl LayoutLookup for SubstLookup<'_> {
    fn props(&self) -> u32 {
        self.props
    }

    fn is_reverse(&self) -> bool {
        self.reverse
    }

    fn covers(&self, glyph: GlyphId) -> bool {
        self.coverage.contains(glyph)
    }
}

impl WouldApply for SubstLookup<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        self.covers(ctx.glyphs[0])
            && self
                .subtables
                .iter()
                .any(|subtable| subtable.would_apply(ctx))
    }
}

impl Apply for SubstLookup<'_> {
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

impl WouldApply for SubstitutionSubtable<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        match self {
            Self::Single(t) => t.would_apply(ctx),
            Self::Multiple(t) => t.would_apply(ctx),
            Self::Alternate(t) => t.would_apply(ctx),
            Self::Ligature(t) => t.would_apply(ctx),
            Self::Context(t) => t.would_apply(ctx),
            Self::ChainContext(t) => t.would_apply(ctx),
            Self::ReverseChainSingle(t) => t.would_apply(ctx),
        }
    }
}

impl Apply for SubstitutionSubtable<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        match self {
            Self::Single(t) => t.apply(ctx),
            Self::Multiple(t) => t.apply(ctx),
            Self::Alternate(t) => t.apply(ctx),
            Self::Ligature(t) => t.apply(ctx),
            Self::Context(t) => t.apply(ctx),
            Self::ChainContext(t) => t.apply(ctx),
            Self::ReverseChainSingle(t) => t.apply(ctx),
        }
    }
}

impl WouldApply for SingleSubstitution<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        ctx.glyphs.len() == 1 && self.coverage().get(ctx.glyphs[0]).is_some()
    }
}

impl Apply for SingleSubstitution<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph = ctx.buffer.cur(0).as_glyph();
        let subst = match *self {
            Self::Format1 { coverage, delta } => {
                coverage.get(glyph)?;
                // According to the Adobe Annotated OpenType Suite, result is always
                // limited to 16bit, so we explicitly want to truncate.
                GlyphId((i32::from(glyph.0) + i32::from(delta)) as u16)
            }
            Self::Format2 {
                coverage,
                substitutes,
            } => {
                let index = coverage.get(glyph)?;
                substitutes.get(index)?
            }
        };

        ctx.replace_glyph(subst);
        Some(())
    }
}

impl WouldApply for MultipleSubstitution<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        ctx.glyphs.len() == 1 && self.coverage.get(ctx.glyphs[0]).is_some()
    }
}

impl Apply for MultipleSubstitution<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph = ctx.buffer.cur(0).as_glyph();
        let index = self.coverage.get(glyph)?;
        let seq = self.sequences.get(index)?;
        seq.apply(ctx)
    }
}

impl Apply for Sequence<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        match self.substitutes.len() {
            // Spec disallows this, but Uniscribe allows it.
            // https://github.com/harfbuzz/harfbuzz/issues/253
            0 => ctx.buffer.delete_glyph(),

            // Special-case to make it in-place and not consider this
            // as a "multiplied" substitution.
            1 => ctx.replace_glyph(self.substitutes.get(0)?),

            _ => {
                let class = if ctx.buffer.cur(0).is_ligature() {
                    GlyphPropsFlags::BASE_GLYPH
                } else {
                    GlyphPropsFlags::empty()
                };
                let lig_id = ctx.buffer.cur(0).lig_id();

                for (i, subst) in self.substitutes.into_iter().enumerate() {
                    // If is attached to a ligature, don't disturb that.
                    // https://github.com/harfbuzz/harfbuzz/issues/3069
                    if lig_id == 0 {
                        // Index is truncated to 4 bits anway, so we can safely cast to u8.
                        ctx.buffer.cur_mut(0).set_lig_props_for_component(i as u8);
                    }
                    ctx.output_glyph_for_component(subst, class);
                }

                ctx.buffer.skip_glyph();
            }
        }
        Some(())
    }
}

impl WouldApply for AlternateSubstitution<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        ctx.glyphs.len() == 1 && self.coverage.get(ctx.glyphs[0]).is_some()
    }
}

impl Apply for AlternateSubstitution<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph = ctx.buffer.cur(0).as_glyph();
        let index = self.coverage.get(glyph)?;
        let set = self.alternate_sets.get(index)?;
        set.apply(ctx)
    }
}

impl Apply for AlternateSet<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let len = self.alternates.len();
        if len == 0 {
            return None;
        }

        let glyph_mask = ctx.buffer.cur(0).mask;

        // Note: This breaks badly if two features enabled this lookup together.
        let shift = ctx.lookup_mask.trailing_zeros();
        let mut alt_index = (ctx.lookup_mask & glyph_mask) >> shift;

        // If alt_index is MAX_VALUE, randomize feature if it is the rand feature.
        if alt_index == Map::MAX_VALUE && ctx.random {
            // Maybe we can do better than unsafe-to-break all; but since we are
            // changing random state, it would be hard to track that.  Good 'nough.
            ctx.buffer.unsafe_to_break(0, ctx.buffer.len);
            alt_index = ctx.random_number() % u32::from(len) + 1;
        }

        let idx = u16::try_from(alt_index).ok()?.checked_sub(1)?;
        ctx.replace_glyph(self.alternates.get(idx)?);

        Some(())
    }
}

impl WouldApply for LigatureSubstitution<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        self.coverage
            .get(ctx.glyphs[0])
            .and_then(|index| self.ligature_sets.get(index))
            .map_or(false, |set| set.would_apply(ctx))
    }
}

impl Apply for LigatureSubstitution<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph = ctx.buffer.cur(0).as_glyph();
        self.coverage
            .get(glyph)
            .and_then(|index| self.ligature_sets.get(index))
            .and_then(|set| set.apply(ctx))
    }
}

impl WouldApply for LigatureSet<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        self.into_iter().any(|lig| lig.would_apply(ctx))
    }
}

impl Apply for LigatureSet<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        for lig in self.into_iter() {
            if lig.apply(ctx).is_some() {
                return Some(());
            }
        }
        None
    }
}

impl WouldApply for Ligature<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        ctx.glyphs.len() == usize::from(self.components.len()) + 1
            && self
                .components
                .into_iter()
                .enumerate()
                .all(|(i, comp)| ctx.glyphs[i + 1] == comp)
    }
}

impl Apply for Ligature<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        // Special-case to make it in-place and not consider this
        // as a "ligated" substitution.
        if self.components.is_empty() {
            ctx.replace_glyph(self.glyph);
            Some(())
        } else {
            let f = |glyph, num_items| {
                let index = self.components.len() - num_items;
                let value = self.components.get(index).unwrap();
                match_glyph(glyph, value.0)
            };

            let mut match_end = 0;
            let mut match_positions = [0; MAX_CONTEXT_LENGTH];
            let mut total_component_count = 0;

            if !match_input(
                ctx,
                self.components.len(),
                &f,
                &mut match_end,
                &mut match_positions,
                Some(&mut total_component_count),
            ) {
                ctx.buffer.unsafe_to_concat(ctx.buffer.idx, match_end);
                return None;
            }

            let count = usize::from(self.components.len()) + 1;
            ligate(
                ctx,
                count,
                &match_positions,
                match_end,
                total_component_count,
                self.glyph,
            );
            return Some(());
        }
    }
}

fn ligate(
    ctx: &mut ApplyContext,
    // Including the first glyph
    count: usize,
    // Including the first glyph
    match_positions: &[usize; MAX_CONTEXT_LENGTH],
    match_end: usize,
    total_component_count: u8,
    lig_glyph: GlyphId,
) {
    // - If a base and one or more marks ligate, consider that as a base, NOT
    //   ligature, such that all following marks can still attach to it.
    //   https://github.com/harfbuzz/harfbuzz/issues/1109
    //
    // - If all components of the ligature were marks, we call this a mark ligature.
    //   If it *is* a mark ligature, we don't allocate a new ligature id, and leave
    //   the ligature to keep its old ligature id.  This will allow it to attach to
    //   a base ligature in GPOS.  Eg. if the sequence is: LAM,LAM,SHADDA,FATHA,HEH,
    //   and LAM,LAM,HEH for a ligature, they will leave SHADDA and FATHA with a
    //   ligature id and component value of 2.  Then if SHADDA,FATHA form a ligature
    //   later, we don't want them to lose their ligature id/component, otherwise
    //   GPOS will fail to correctly position the mark ligature on top of the
    //   LAM,LAM,HEH ligature.  See:
    //     https://bugzilla.gnome.org/show_bug.cgi?id=676343
    //
    // - If a ligature is formed of components that some of which are also ligatures
    //   themselves, and those ligature components had marks attached to *their*
    //   components, we have to attach the marks to the new ligature component
    //   positions!  Now *that*'s tricky!  And these marks may be following the
    //   last component of the whole sequence, so we should loop forward looking
    //   for them and update them.
    //
    //   Eg. the sequence is LAM,LAM,SHADDA,FATHA,HEH, and the font first forms a
    //   'calt' ligature of LAM,HEH, leaving the SHADDA and FATHA with a ligature
    //   id and component == 1.  Now, during 'liga', the LAM and the LAM-HEH ligature
    //   form a LAM-LAM-HEH ligature.  We need to reassign the SHADDA and FATHA to
    //   the new ligature with a component value of 2.
    //
    //   This in fact happened to a font...  See:
    //   https://bugzilla.gnome.org/show_bug.cgi?id=437633
    //

    let mut buffer = &mut ctx.buffer;
    buffer.merge_clusters(buffer.idx, match_end);

    let mut is_base_ligature = buffer.info[match_positions[0]].is_base_glyph();
    let mut is_mark_ligature = buffer.info[match_positions[0]].is_mark();
    for i in 1..count {
        if !buffer.info[match_positions[i]].is_mark() {
            is_base_ligature = false;
            is_mark_ligature = false;
        }
    }

    let is_ligature = !is_base_ligature && !is_mark_ligature;
    let class = if is_ligature {
        GlyphPropsFlags::LIGATURE
    } else {
        GlyphPropsFlags::empty()
    };
    let lig_id = if is_ligature {
        buffer.allocate_lig_id()
    } else {
        0
    };
    let first = buffer.cur_mut(0);
    let mut last_lig_id = first.lig_id();
    let mut last_num_comps = first.lig_num_comps();
    let mut comps_so_far = last_num_comps;

    if is_ligature {
        first.set_lig_props_for_ligature(lig_id, total_component_count);
        if first.general_category() == GeneralCategory::NonspacingMark {
            first.set_general_category(GeneralCategory::OtherLetter);
        }
    }

    ctx.replace_glyph_with_ligature(lig_glyph, class);
    buffer = &mut ctx.buffer;

    for i in 1..count {
        while buffer.idx < match_positions[i] && buffer.successful {
            if is_ligature {
                let cur = buffer.cur_mut(0);
                let mut this_comp = cur.lig_comp();
                if this_comp == 0 {
                    this_comp = last_num_comps;
                }
                let new_lig_comp = comps_so_far - last_num_comps + this_comp.min(last_num_comps);
                cur.set_lig_props_for_mark(lig_id, new_lig_comp);
            }
            buffer.next_glyph();
        }

        let cur = buffer.cur(0);
        last_lig_id = cur.lig_id();
        last_num_comps = cur.lig_num_comps();
        comps_so_far += last_num_comps;

        // Skip the base glyph.
        buffer.idx += 1;
    }

    if !is_mark_ligature && last_lig_id != 0 {
        // Re-adjust components for any marks following.
        for i in buffer.idx..buffer.len {
            let info = &mut buffer.info[i];
            if last_lig_id != info.lig_id() {
                break;
            }

            let this_comp = info.lig_comp();
            if this_comp == 0 {
                break;
            }

            let new_lig_comp = comps_so_far - last_num_comps + this_comp.min(last_num_comps);
            info.set_lig_props_for_mark(lig_id, new_lig_comp)
        }
    }
}

impl WouldApply for ReverseChainSingleSubstitution<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        ctx.glyphs.len() == 1 && self.coverage.get(ctx.glyphs[0]).is_some()
    }
}

impl Apply for ReverseChainSingleSubstitution<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        // No chaining to this type.
        if ctx.nesting_level_left != MAX_NESTING_LEVEL {
            return None;
        }

        let glyph = ctx.buffer.cur(0).as_glyph();
        let index = self.coverage.get(glyph)?;
        if index >= self.substitutes.len() {
            return None;
        }

        let subst = self.substitutes.get(index)?;

        let f1 = |glyph, num_items| {
            let index = self.backtrack_coverages.len() - num_items;
            let value = self.backtrack_coverages.get(index).unwrap();
            value.contains(glyph)
        };

        let f2 = |glyph, num_items| {
            let index = self.lookahead_coverages.len() - num_items;
            let value = self.lookahead_coverages.get(index).unwrap();
            value.contains(glyph)
        };

        let mut start_index = 0;
        let mut end_index = 0;

        if match_backtrack(ctx, self.backtrack_coverages.len(), &f1, &mut start_index) {
            if match_lookahead(
                ctx,
                self.lookahead_coverages.len(),
                &f2,
                ctx.buffer.idx + 1,
                &mut end_index,
            ) {
                ctx.buffer
                    .unsafe_to_break_from_outbuffer(start_index, end_index);
                ctx.replace_glyph_inplace(subst);

                // Note: We DON'T decrease buffer.idx.  The main loop does it
                // for us.  This is useful for preventing surprises if someone
                // calls us through a Context lookup.
                return Some(());
            }
        }

        ctx.buffer
            .unsafe_to_concat_from_outbuffer(start_index, end_index);
        return None;
    }
}
