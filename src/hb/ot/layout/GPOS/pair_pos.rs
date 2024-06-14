use ttf_parser::gpos::{PairAdjustment, ValueRecord};
use crate::hb::ot_layout_gpos_table::ValueRecordExt;
use crate::hb::ot_layout_gsubgpos::{Apply, skipping_iterator_t};
use crate::hb::ot_layout_gsubgpos::OT::hb_ot_apply_context_t;

impl Apply for PairAdjustment<'_> {
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
        let first_glyph = ctx.buffer.cur(0).as_glyph();
        let first_glyph_coverage_index = self.coverage().get(first_glyph)?;

        let mut iter = skipping_iterator_t::new(ctx, ctx.buffer.idx, 1, false);

        let mut unsafe_to = 0;
        if !iter.next(Some(&mut unsafe_to)) {
            ctx.buffer
                .unsafe_to_concat(Some(ctx.buffer.idx), Some(unsafe_to));
            return None;
        }

        let second_glyph_index = iter.index();
        let second_glyph = ctx.buffer.info[second_glyph_index].as_glyph();

        let finish = |ctx: &mut hb_ot_apply_context_t, has_record2| {
            ctx.buffer.idx = second_glyph_index;

            if has_record2 {
                ctx.buffer.idx += 1;
            }

            Some(())
        };

        let boring = |ctx: &mut hb_ot_apply_context_t, has_record2| {
            ctx.buffer
                .unsafe_to_concat(Some(ctx.buffer.idx), Some(second_glyph_index + 1));
            finish(ctx, has_record2)
        };

        let success = |ctx: &mut hb_ot_apply_context_t, flag1, flag2, has_record2| {
            if flag1 || flag2 {
                ctx.buffer
                    .unsafe_to_break(Some(ctx.buffer.idx), Some(second_glyph_index + 1));
                finish(ctx, has_record2)
            } else {
                boring(ctx, has_record2)
            }
        };

        let bail = |ctx: &mut hb_ot_apply_context_t, records: (ValueRecord, ValueRecord)| {
            let flag1 = records.0.apply(ctx, ctx.buffer.idx);
            let flag2 = records.1.apply(ctx, second_glyph_index);

            let has_record2 = !records.1.is_empty();
            success(ctx, flag1, flag2, has_record2)
        };

        let records = match self {
            Self::Format1 { sets, .. } => {
                sets.get(first_glyph_coverage_index)?.get(second_glyph)?
            }
            Self::Format2 {
                classes, matrix, ..
            } => {
                let classes = (classes.0.get(first_glyph), classes.1.get(second_glyph));

                let records = match matrix.get(classes) {
                    Some(v) => v,
                    None => {
                        ctx.buffer
                            .unsafe_to_concat(Some(ctx.buffer.idx), Some(iter.index() + 1));
                        return None;
                    }
                };

                return bail(ctx, records);
            }
        };

        bail(ctx, records)
    }
}
