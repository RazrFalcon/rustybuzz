use crate::hb::ot_layout::LayoutLookup;
use crate::hb::ot_layout_common::SubstLookup;
use crate::hb::ot_layout_gsubgpos::OT::hb_ot_apply_context_t;
use crate::hb::ot_layout_gsubgpos::{Apply, WouldApply, WouldApplyContext};
use crate::hb::set_digest::hb_set_digest_ext;
use ttf_parser::GlyphId;

impl LayoutLookup for SubstLookup<'_> {
    fn props(&self) -> u32 {
        self.props
    }

    fn is_reverse(&self) -> bool {
        self.reverse
    }

    fn may_have(&self, glyph: GlyphId) -> bool {
        self.set_digest.may_have_glyph(glyph)
    }
}

impl WouldApply for SubstLookup<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        self.may_have(ctx.glyphs[0])
            && self
                .subtables
                .iter()
                .any(|subtable| subtable.would_apply(ctx))
    }
}

impl Apply for SubstLookup<'_> {
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
        if self.may_have(ctx.buffer.cur(0).as_glyph()) {
            for subtable in &self.subtables {
                if subtable.apply(ctx).is_some() {
                    return Some(());
                }
            }
        }

        None
    }
}
