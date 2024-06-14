use crate::hb::ot::layout::GPOS::mark_array::MarkArrayExt;
use crate::hb::ot_layout::{_hb_glyph_info_get_lig_comp, _hb_glyph_info_get_lig_id};
use crate::hb::ot_layout_common::lookup_flags;
use crate::hb::ot_layout_gsubgpos::OT::hb_ot_apply_context_t;
use crate::hb::ot_layout_gsubgpos::{skipping_iterator_t, Apply};
use ttf_parser::gpos::MarkToLigatureAdjustment;

impl Apply for MarkToLigatureAdjustment<'_> {
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
        let buffer = &ctx.buffer;
        let mark_glyph = ctx.buffer.cur(0).as_glyph();
        let mark_index = self.mark_coverage.get(mark_glyph)?;

        // Now we search backwards for a non-mark glyph
        let mut iter = skipping_iterator_t::new(ctx, buffer.idx, 1, false);
        iter.set_lookup_props(u32::from(lookup_flags::IGNORE_MARKS));

        let mut unsafe_from = 0;
        if !iter.prev(Some(&mut unsafe_from)) {
            ctx.buffer
                .unsafe_to_concat_from_outbuffer(Some(unsafe_from), Some(ctx.buffer.idx + 1));
            return None;
        }

        // Checking that matched glyph is actually a ligature by GDEF is too strong; disabled

        let iter_idx = iter.index();
        let lig_glyph = buffer.info[iter_idx].as_glyph();
        let Some(lig_index) = self.ligature_coverage.get(lig_glyph) else {
            ctx.buffer
                .unsafe_to_concat_from_outbuffer(Some(iter_idx), Some(buffer.idx + 1));
            return None;
        };
        let lig_attach = self.ligature_array.get(lig_index)?;

        // Find component to attach to
        let comp_count = lig_attach.rows;
        if comp_count == 0 {
            ctx.buffer
                .unsafe_to_concat_from_outbuffer(Some(iter_idx), Some(buffer.idx + 1));
            return None;
        }

        // We must now check whether the ligature ID of the current mark glyph
        // is identical to the ligature ID of the found ligature.  If yes, we
        // can directly use the component index.  If not, we attach the mark
        // glyph to the last component of the ligature.
        let lig_id = _hb_glyph_info_get_lig_id(&buffer.info[iter_idx]);
        let mark_id = _hb_glyph_info_get_lig_id(&buffer.cur(0));
        let mark_comp = u16::from(_hb_glyph_info_get_lig_comp(buffer.cur(0)));
        let matches = lig_id != 0 && lig_id == mark_id && mark_comp > 0;
        let comp_index = if matches {
            mark_comp.min(comp_count)
        } else {
            comp_count
        } - 1;

        self.marks
            .apply(ctx, lig_attach, mark_index, comp_index, iter_idx)
    }
}
