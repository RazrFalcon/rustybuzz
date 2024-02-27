use ttf_parser::gsub::*;
use ttf_parser::opentype_layout::LookupIndex;
use ttf_parser::GlyphId;

use super::buffer::hb_buffer_t;
use super::hb_font_t;
use super::ot_layout::*;
use super::ot_layout_common::{SubstLookup, SubstitutionTable};
use super::ot_layout_gsubgpos::*;
use super::ot_shape_plan::hb_ot_shape_plan_t;
use OT::hb_ot_apply_context_t;

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
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
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
                ctx.buffer
                    .unsafe_to_concat(Some(ctx.buffer.idx), Some(match_end));
                return None;
            }

            let count = usize::from(self.components.len()) + 1;
            ligate_input(
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

impl WouldApply for LigatureSet<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        self.into_iter().any(|lig| lig.would_apply(ctx))
    }
}

impl Apply for LigatureSet<'_> {
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
        for lig in self.into_iter() {
            if lig.apply(ctx).is_some() {
                return Some(());
            }
        }
        None
    }
}

// LigatureSubstFormat1::would_apply
impl WouldApply for LigatureSubstitution<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        self.coverage
            .get(ctx.glyphs[0])
            .and_then(|index| self.ligature_sets.get(index))
            .map_or(false, |set| set.would_apply(ctx))
    }
}

// LigatureSubstFormat1::apply
impl Apply for LigatureSubstitution<'_> {
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
        let glyph = ctx.buffer.cur(0).as_glyph();
        self.coverage
            .get(glyph)
            .and_then(|index| self.ligature_sets.get(index))
            .and_then(|set| set.apply(ctx))
    }
}

// ReverseChainSingleSubstFormat1::would_apply
impl WouldApply for ReverseChainSingleSubstitution<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        ctx.glyphs.len() == 1 && self.coverage.get(ctx.glyphs[0]).is_some()
    }
}

// ReverseChainSingleSubstFormat1::apply
impl Apply for ReverseChainSingleSubstitution<'_> {
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
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
                    .unsafe_to_break_from_outbuffer(Some(start_index), Some(end_index));
                ctx.replace_glyph_inplace(subst);

                // Note: We DON'T decrease buffer.idx.  The main loop does it
                // for us.  This is useful for preventing surprises if someone
                // calls us through a Context lookup.
                return Some(());
            }
        }

        ctx.buffer
            .unsafe_to_concat_from_outbuffer(Some(start_index), Some(end_index));
        return None;
    }
}

pub fn substitute(plan: &hb_ot_shape_plan_t, face: &hb_font_t, buffer: &mut hb_buffer_t) {
    apply_layout_table(plan, face, buffer, face.gsub.as_ref());
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
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
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
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
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
