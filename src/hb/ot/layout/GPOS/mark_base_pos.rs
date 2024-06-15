use crate::hb::ot::layout::GPOS::mark_array::MarkArrayExt;
use crate::hb::ot_layout::{
    _hb_glyph_info_get_lig_comp, _hb_glyph_info_get_lig_id, _hb_glyph_info_is_mark,
    _hb_glyph_info_multiplied,
};
use crate::hb::ot_layout_common::lookup_flags;
use crate::hb::ot_layout_gsubgpos::OT::hb_ot_apply_context_t;
use crate::hb::ot_layout_gsubgpos::{skipping_iterator_t, Apply};
use ttf_parser::gpos::MarkToBaseAdjustment;

impl Apply for MarkToBaseAdjustment<'_> {
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
        let buffer = &ctx.buffer;
        let mark_glyph = ctx.buffer.cur(0).as_glyph();
        let mark_index = self.mark_coverage.get(mark_glyph)?;

        // Now we search backwards for a non-mark glyph
        let mut iter = skipping_iterator_t::new(ctx, buffer.idx, 1, false);
        iter.set_lookup_props(u32::from(lookup_flags::IGNORE_MARKS));

        let info = &buffer.info;
        loop {
            let mut unsafe_from = 0;
            if !iter.prev(Some(&mut unsafe_from)) {
                ctx.buffer
                    .unsafe_to_concat_from_outbuffer(Some(unsafe_from), Some(ctx.buffer.idx + 1));
                return None;
            }

            // We only want to attach to the first of a MultipleSubst sequence.
            // https://github.com/harfbuzz/harfbuzz/issues/740
            // Reject others...
            // ...but stop if we find a mark in the MultipleSubst sequence:
            // https://github.com/harfbuzz/harfbuzz/issues/1020
            let idx = iter.index();
            if !_hb_glyph_info_multiplied(&info[idx])
                || _hb_glyph_info_get_lig_comp(&info[idx]) == 0
                || idx == 0
                || _hb_glyph_info_is_mark(&info[idx - 1])
                || !_hb_glyph_info_multiplied(&info[idx - 1])
                || _hb_glyph_info_get_lig_id(&info[idx])
                    != _hb_glyph_info_get_lig_id(&info[idx - 1])
                || _hb_glyph_info_get_lig_comp(&info[idx])
                    != _hb_glyph_info_get_lig_comp(&info[idx - 1]) + 1
            {
                break;
            }
            iter.reject();
        }

        // Checking that matched glyph is actually a base glyph by GDEF is too strong; disabled

        let iter_idx = iter.index();
        let base_glyph = info[iter_idx].as_glyph();
        let Some(base_index) = self.base_coverage.get(base_glyph) else {
            ctx.buffer
                .unsafe_to_concat_from_outbuffer(Some(iter_idx), Some(buffer.idx + 1));
            return None;
        };

        self.marks
            .apply(ctx, self.anchors, mark_index, base_index, iter_idx)
    }
}
