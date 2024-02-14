use ttf_parser::opentype_layout::*;
use ttf_parser::{GlyphId, LazyArray16};

use super::apply::{Apply, ApplyContext, WouldApply, WouldApplyContext};
use super::matching::{match_backtrack, match_glyph, match_input, match_lookahead, MatchFunc};
use super::MAX_CONTEXT_LENGTH;

impl WouldApply for ContextLookup<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        let glyph = ctx.glyphs[0];
        match *self {
            Self::Format1 { coverage, sets } => coverage
                .get(glyph)
                .and_then(|index| sets.get(index))
                .map_or(false, |set| set.would_apply(ctx, &match_glyph)),
            Self::Format2 { classes, sets, .. } => {
                let class = classes.get(glyph);
                sets.get(class)
                    .map_or(false, |set| set.would_apply(ctx, &match_class(classes)))
            }
            Self::Format3 { coverages, .. } => {
                ctx.glyphs.len() == usize::from(coverages.len()) + 1
                    && coverages
                        .into_iter()
                        .enumerate()
                        .all(|(i, coverage)| coverage.get(ctx.glyphs[i + 1]).is_some())
            }
        }
    }
}

impl Apply for ContextLookup<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph = ctx.buffer.cur(0).as_glyph();
        match *self {
            Self::Format1 { coverage, sets } => {
                coverage.get(glyph)?;
                let set = coverage.get(glyph).and_then(|index| sets.get(index))?;
                set.apply(ctx, &match_glyph)
            }
            Self::Format2 {
                coverage,
                classes,
                sets,
            } => {
                coverage.get(glyph)?;
                let class = classes.get(glyph);
                let set = sets.get(class)?;
                set.apply(ctx, &match_class(classes))
            }
            Self::Format3 {
                coverage,
                coverages,
                lookups,
            } => {
                coverage.get(glyph)?;
                let coverages_len = coverages.len();

                let match_func = |glyph, num_items| {
                    let index = coverages_len - num_items;
                    let coverage = coverages.get(index).unwrap();
                    coverage.get(glyph).is_some()
                };

                let mut match_end = 0;
                let mut match_positions = [0; MAX_CONTEXT_LENGTH];

                if match_input(
                    ctx,
                    coverages_len,
                    &match_func,
                    &mut match_end,
                    &mut match_positions,
                    None,
                ) {
                    ctx.buffer.unsafe_to_break(Some(ctx.buffer.idx), Some(match_end));
                    apply_lookup(
                        ctx,
                        usize::from(coverages_len),
                        &mut match_positions,
                        match_end,
                        lookups,
                    );
                    return Some(());
                } else {
                    ctx.buffer.unsafe_to_concat(Some(ctx.buffer.idx), Some(match_end));
                    return None;
                }
            }
        }
    }
}

trait SequenceRuleSetExt {
    fn would_apply(&self, ctx: &WouldApplyContext, match_func: &MatchFunc) -> bool;
    fn apply(&self, ctx: &mut ApplyContext, match_func: &MatchFunc) -> Option<()>;
}

impl SequenceRuleSetExt for SequenceRuleSet<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext, match_func: &MatchFunc) -> bool {
        self.into_iter()
            .any(|rule| rule.would_apply(ctx, match_func))
    }

    fn apply(&self, ctx: &mut ApplyContext, match_func: &MatchFunc) -> Option<()> {
        if self
            .into_iter()
            .any(|rule| rule.apply(ctx, match_func).is_some())
        {
            Some(())
        } else {
            None
        }
    }
}

trait SequenceRuleExt {
    fn would_apply(&self, ctx: &WouldApplyContext, match_func: &MatchFunc) -> bool;
    fn apply(&self, ctx: &mut ApplyContext, match_func: &MatchFunc) -> Option<()>;
}

impl SequenceRuleExt for SequenceRule<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext, match_func: &MatchFunc) -> bool {
        ctx.glyphs.len() == usize::from(self.input.len()) + 1
            && self
                .input
                .into_iter()
                .enumerate()
                .all(|(i, value)| match_func(ctx.glyphs[i + 1], value))
    }

    fn apply(&self, ctx: &mut ApplyContext, match_func: &MatchFunc) -> Option<()> {
        apply_context(ctx, self.input, match_func, self.lookups)
    }
}

impl WouldApply for ChainedContextLookup<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        let glyph_id = ctx.glyphs[0];
        match *self {
            Self::Format1 { coverage, sets } => coverage
                .get(glyph_id)
                .and_then(|index| sets.get(index))
                .map_or(false, |set| set.would_apply(ctx, &match_glyph)),
            Self::Format2 {
                input_classes,
                sets,
                ..
            } => {
                let class = input_classes.get(glyph_id);
                sets.get(class).map_or(false, |set| {
                    set.would_apply(ctx, &match_class(input_classes))
                })
            }
            Self::Format3 {
                backtrack_coverages,
                input_coverages,
                lookahead_coverages,
                ..
            } => {
                (!ctx.zero_context
                    || (backtrack_coverages.len() == 0 && lookahead_coverages.len() == 0))
                    && (ctx.glyphs.len() == usize::from(input_coverages.len()) + 1
                        && input_coverages
                            .into_iter()
                            .enumerate()
                            .all(|(i, coverage)| coverage.contains(ctx.glyphs[i + 1])))
            }
        }
    }
}

impl Apply for ChainedContextLookup<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph = ctx.buffer.cur(0).as_glyph();
        match *self {
            Self::Format1 { coverage, sets } => {
                let index = coverage.get(glyph)?;
                let set = sets.get(index)?;
                set.apply(ctx, [&match_glyph, &match_glyph, &match_glyph])
            }
            Self::Format2 {
                coverage,
                backtrack_classes,
                input_classes,
                lookahead_classes,
                sets,
            } => {
                coverage.get(glyph)?;
                let class = input_classes.get(glyph);
                let set = sets.get(class)?;
                set.apply(
                    ctx,
                    [
                        &match_class(backtrack_classes),
                        &match_class(input_classes),
                        &match_class(lookahead_classes),
                    ],
                )
            }
            Self::Format3 {
                coverage,
                backtrack_coverages,
                input_coverages,
                lookahead_coverages,
                lookups,
            } => {
                coverage.get(glyph)?;

                let back = |glyph, num_items| {
                    let index = backtrack_coverages.len() - num_items;
                    let coverage = backtrack_coverages.get(index).unwrap();
                    coverage.contains(glyph)
                };

                let ahead = |glyph, num_items| {
                    let index = lookahead_coverages.len() - num_items;
                    let coverage = lookahead_coverages.get(index).unwrap();
                    coverage.contains(glyph)
                };

                let input = |glyph, num_items| {
                    let index = input_coverages.len() - num_items;
                    let coverage = input_coverages.get(index).unwrap();
                    coverage.contains(glyph)
                };

                let mut end_index = ctx.buffer.idx;
                let mut match_end = 0;
                let mut match_positions = [0; MAX_CONTEXT_LENGTH];

                let input_matches = match_input(
                    ctx,
                    input_coverages.len(),
                    &input,
                    &mut match_end,
                    &mut match_positions,
                    None,
                );

                if input_matches {
                    end_index = match_end;
                }

                if !(input_matches
                    && match_lookahead(
                        ctx,
                        lookahead_coverages.len(),
                        &ahead,
                        match_end,
                        &mut end_index,
                    ))
                {
                    ctx.buffer.unsafe_to_concat(Some(ctx.buffer.idx), Some(end_index));
                    return None;
                }

                let mut start_index = ctx.buffer.out_len;

                if !match_backtrack(ctx, backtrack_coverages.len(), &back, &mut start_index) {
                    ctx.buffer
                        .unsafe_to_concat_from_outbuffer(Some(start_index), Some(end_index));
                    return None;
                }

                ctx.buffer
                    .unsafe_to_break_from_outbuffer(Some(start_index), Some(end_index));
                apply_lookup(
                    ctx,
                    usize::from(input_coverages.len()),
                    &mut match_positions,
                    match_end,
                    lookups,
                );

                Some(())
            }
        }
    }
}

trait ChainRuleSetExt {
    fn would_apply(&self, ctx: &WouldApplyContext, match_func: &MatchFunc) -> bool;
    fn apply(&self, ctx: &mut ApplyContext, match_funcs: [&MatchFunc; 3]) -> Option<()>;
}

impl ChainRuleSetExt for ChainedSequenceRuleSet<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext, match_func: &MatchFunc) -> bool {
        self.into_iter()
            .any(|rule| rule.would_apply(ctx, match_func))
    }

    fn apply(&self, ctx: &mut ApplyContext, match_funcs: [&MatchFunc; 3]) -> Option<()> {
        if self
            .into_iter()
            .any(|rule| rule.apply(ctx, match_funcs).is_some())
        {
            Some(())
        } else {
            None
        }
    }
}

trait ChainRuleExt {
    fn would_apply(&self, ctx: &WouldApplyContext, match_func: &MatchFunc) -> bool;
    fn apply(&self, ctx: &mut ApplyContext, match_funcs: [&MatchFunc; 3]) -> Option<()>;
}

impl ChainRuleExt for ChainedSequenceRule<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext, match_func: &MatchFunc) -> bool {
        (!ctx.zero_context || (self.backtrack.len() == 0 && self.lookahead.len() == 0))
            && (ctx.glyphs.len() == usize::from(self.input.len()) + 1
                && self
                    .input
                    .into_iter()
                    .enumerate()
                    .all(|(i, value)| match_func(ctx.glyphs[i + 1], value)))
    }

    fn apply(&self, ctx: &mut ApplyContext, match_funcs: [&MatchFunc; 3]) -> Option<()> {
        apply_chain_context(
            ctx,
            self.backtrack,
            self.input,
            self.lookahead,
            match_funcs,
            self.lookups,
        )
    }
}

fn apply_context(
    ctx: &mut ApplyContext,
    input: LazyArray16<u16>,
    match_func: &MatchFunc,
    lookups: LazyArray16<SequenceLookupRecord>,
) -> Option<()> {
    let match_func = |glyph, num_items| {
        let index = input.len() - num_items;
        let value = input.get(index).unwrap();
        match_func(glyph, value)
    };

    let mut match_end = 0;
    let mut match_positions = [0; MAX_CONTEXT_LENGTH];

    if match_input(
        ctx,
        input.len(),
        &match_func,
        &mut match_end,
        &mut match_positions,
        None,
    ) {
        ctx.buffer.unsafe_to_break(Some(ctx.buffer.idx), Some(match_end));
        apply_lookup(
            ctx,
            usize::from(input.len()),
            &mut match_positions,
            match_end,
            lookups,
        );
        return Some(());
    }

    None
}

fn apply_chain_context(
    ctx: &mut ApplyContext,
    backtrack: LazyArray16<u16>,
    input: LazyArray16<u16>,
    lookahead: LazyArray16<u16>,
    match_funcs: [&MatchFunc; 3],
    lookups: LazyArray16<SequenceLookupRecord>,
) -> Option<()> {
    // NOTE: Whenever something in this method changes, we also need to
    // change it in the `apply` implementation for ChainedContextLookup.
    let f1 = |glyph, num_items| {
        let index = backtrack.len() - num_items;
        let value = backtrack.get(index).unwrap();
        match_funcs[0](glyph, value)
    };

    let f2 = |glyph, num_items| {
        let index = lookahead.len() - num_items;
        let value = lookahead.get(index).unwrap();
        match_funcs[2](glyph, value)
    };

    let f3 = |glyph, num_items| {
        let index = input.len() - num_items;
        let value = input.get(index).unwrap();
        match_funcs[1](glyph, value)
    };

    let mut end_index = ctx.buffer.idx;
    let mut match_end = 0;
    let mut match_positions = [0; MAX_CONTEXT_LENGTH];

    let input_matches = match_input(
        ctx,
        input.len(),
        &f3,
        &mut match_end,
        &mut match_positions,
        None,
    );

    if input_matches {
        end_index = match_end;
    }

    if !(input_matches && match_lookahead(ctx, lookahead.len(), &f2, match_end, &mut end_index)) {
        ctx.buffer.unsafe_to_concat(Some(ctx.buffer.idx), Some(end_index));
        return None;
    }

    let mut start_index = ctx.buffer.out_len;

    if !match_backtrack(ctx, backtrack.len(), &f1, &mut start_index) {
        ctx.buffer
            .unsafe_to_concat_from_outbuffer(Some(start_index), Some(end_index));
        return None;
    }

    ctx.buffer
        .unsafe_to_break_from_outbuffer(Some(start_index), Some(end_index));
    apply_lookup(
        ctx,
        usize::from(input.len()),
        &mut match_positions,
        match_end,
        lookups,
    );

    Some(())
}

fn apply_lookup(
    ctx: &mut ApplyContext,
    input_len: usize,
    match_positions: &mut [usize; MAX_CONTEXT_LENGTH],
    match_end: usize,
    lookups: LazyArray16<SequenceLookupRecord>,
) {
    let mut count = input_len + 1;

    // All positions are distance from beginning of *output* buffer.
    // Adjust.
    let mut end = {
        let backtrack_len = ctx.buffer.backtrack_len();
        let delta = backtrack_len as isize - ctx.buffer.idx as isize;

        // Convert positions to new indexing.
        for j in 0..count {
            match_positions[j] = (match_positions[j] as isize + delta) as _;
        }

        backtrack_len + match_end - ctx.buffer.idx
    };

    for record in lookups {
        if !ctx.buffer.successful {
            break;
        }

        let idx = usize::from(record.sequence_index);
        if idx >= count {
            continue;
        }

        // Don't recurse to ourself at same position.
        // Note that this test is too naive, it doesn't catch longer loops.
        if idx == 0 && record.lookup_list_index == ctx.lookup_index {
            continue;
        }

        if !ctx.buffer.move_to(match_positions[idx]) {
            break;
        }

        if ctx.buffer.max_ops <= 0 {
            break;
        }

        let orig_len = ctx.buffer.backtrack_len() + ctx.buffer.lookahead_len();
        if ctx.recurse(record.lookup_list_index).is_none() {
            continue;
        }

        let new_len = ctx.buffer.backtrack_len() + ctx.buffer.lookahead_len();
        let mut delta = new_len as isize - orig_len as isize;
        if delta == 0 {
            continue;
        }

        // Recursed lookup changed buffer len.  Adjust.
        //
        // TODO:
        //
        // Right now, if buffer length increased by n, we assume n new glyphs
        // were added right after the current position, and if buffer length
        // was decreased by n, we assume n match positions after the current
        // one where removed.  The former (buffer length increased) case is
        // fine, but the decrease case can be improved in at least two ways,
        // both of which are significant:
        //
        //   - If recursed-to lookup is MultipleSubst and buffer length
        //     decreased, then it's current match position that was deleted,
        //     NOT the one after it.
        //
        //   - If buffer length was decreased by n, it does not necessarily
        //     mean that n match positions where removed, as there might
        //     have been marks and default-ignorables in the sequence.  We
        //     should instead drop match positions between current-position
        //     and current-position + n instead.
        //
        // It should be possible to construct tests for both of these cases.

        end = (end as isize + delta) as _;
        if end <= match_positions[idx] {
            // End might end up being smaller than match_positions[idx] if the recursed
            // lookup ended up removing many items, more than we have had matched.
            // Just never rewind end back and get out of here.
            // https://bugs.chromium.org/p/chromium/issues/detail?id=659496
            end = match_positions[idx];

            // There can't be any further changes.
            break;
        }

        // next now is the position after the recursed lookup.
        let mut next = idx + 1;

        if delta > 0 {
            if delta as usize + count > MAX_CONTEXT_LENGTH {
                break;
            }
        } else {
            // NOTE: delta is negative.
            delta = delta.max(next as isize - count as isize);
            next = (next as isize - delta) as _;
        }

        // Shift!
        match_positions.copy_within(next..count, (next as isize + delta) as _);
        next = (next as isize + delta) as _;
        count = (count as isize + delta) as _;

        // Fill in new entries.
        for j in idx + 1..next {
            match_positions[j] = match_positions[j - 1] + 1;
        }

        // And fixup the rest.
        while next < count {
            match_positions[next] = (match_positions[next] as isize + delta) as _;
            next += 1;
        }
    }

    ctx.buffer.move_to(end);
}

/// Value represents glyph class.
fn match_class<'a>(class_def: ClassDefinition<'a>) -> impl Fn(GlyphId, u16) -> bool + 'a {
    move |glyph, value| class_def.get(glyph) == value
}
