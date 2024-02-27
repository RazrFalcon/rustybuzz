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
