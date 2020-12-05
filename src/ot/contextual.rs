use ttf_parser::parser::{LazyArray16, Offset};

use crate::tables::gsubgpos::*;
use super::MAX_CONTEXT_LENGTH;
use super::apply::{Apply, ApplyContext, WouldApply, WouldApplyContext};
use super::matching::{
    match_backtrack, match_class, match_coverage, match_glyph, match_input, match_lookahead,
    would_match_input, MatchFunc, Matched,
};

impl WouldApply for ContextLookup<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        let glyph_id = ctx.glyphs[0];
        match *self {
            Self::Format1 { coverage, sets } => {
                coverage.get(glyph_id)
                    .and_then(|index| sets.slice(index))
                    .and_then(RuleSet::parse)
                    .map_or(false, |set| set.would_apply(ctx, &match_glyph))
            }
            Self::Format2 { classes: class_def, sets, .. } => {
                let class = class_def.get(glyph_id);
                sets.get(class.0).map_or(false, |offset| !offset.is_null())
                    && sets.slice(class.0)
                        .and_then(RuleSet::parse)
                        .map_or(false, |set| set.would_apply(ctx, &match_class(class_def)))
            }
            Self::Format3 { data, coverages, .. } => {
                would_apply_context(ctx, coverages, &match_coverage(data))
            }
        }
    }
}

impl Apply for ContextLookup<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph = ctx.buffer.cur(0).as_glyph();
        match *self {
            Self::Format1 { coverage, sets } => {
                let index = coverage.get(glyph)?;
                let set = RuleSet::parse(sets.slice(index)?)?;
                set.apply(ctx, &match_glyph)
            }
            Self::Format2 { coverage, classes, sets } => {
                coverage.get(glyph)?;
                let class = classes.get(glyph);
                let offset = sets.get(class.0)?;
                if !offset.is_null() {
                    let set = RuleSet::parse(sets.slice(class.0)?)?;
                    set.apply(ctx, &match_class(classes))
                } else {
                    None
                }
            }
            Self::Format3 { data, coverage, coverages, lookups } => {
                coverage.get(glyph)?;
                apply_context(ctx, coverages, &match_coverage(data), lookups)
            }
        }
    }
}

impl<'a> RuleSet<'a> {
    fn would_apply(&self, ctx: &WouldApplyContext, match_func: &MatchFunc) -> bool {
        self.rules
            .into_iter()
            .filter_map(|data| Rule::parse(data))
            .any(|rule| rule.would_apply(ctx, match_func))
    }

    fn apply(&self, ctx: &mut ApplyContext, match_func: &MatchFunc) -> Option<()> {
        for data in self.rules {
            if let Some(rule) = Rule::parse(data) {
                if rule.apply(ctx, match_func).is_some() {
                    return Some(());
                }
            }
        }
        None
    }
}

impl<'a> Rule<'a> {
    fn would_apply(&self, ctx: &WouldApplyContext, match_func: &MatchFunc) -> bool {
        would_apply_context(ctx, self.input, match_func)
    }

    fn apply(&self, ctx: &mut ApplyContext, match_func: &MatchFunc) -> Option<()> {
        apply_context(ctx, self.input, match_func, self.lookups)
    }
}

impl WouldApply for ChainContextLookup<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        let glyph_id = ctx.glyphs[0];
        match *self {
            Self::Format1 { coverage, sets } => {
                coverage.get(glyph_id)
                    .and_then(|index| sets.slice(index))
                    .and_then(ChainRuleSet::parse)
                    .map_or(false, |set| set.would_apply(ctx, &match_glyph))
            }
            Self::Format2 { input_classes, sets, .. } => {
                let class = input_classes.get(glyph_id);
                sets.get(class.0).map_or(false, |offset| !offset.is_null())
                    && sets.slice(class.0)
                        .and_then(ChainRuleSet::parse)
                        .map_or(false, |set| set.would_apply(ctx, &match_class(input_classes)))
            }
            Self::Format3 { data, backtrack_coverages, input_coverages, lookahead_coverages, .. } => {
                would_apply_chain_context(
                    ctx,
                    backtrack_coverages,
                    input_coverages,
                    lookahead_coverages,
                    &match_coverage(data),
                )
            }
        }
    }
}

impl Apply for ChainContextLookup<'_> {
    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph = ctx.buffer.cur(0).as_glyph();
        match *self {
            Self::Format1 { coverage, sets } => {
                let index = coverage.get(glyph)?;
                let set = ChainRuleSet::parse(sets.slice(index)?)?;
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
                let offset = sets.get(class.0)?;
                if !offset.is_null() {
                    let set = ChainRuleSet::parse(sets.slice(class.0)?)?;
                    set.apply(ctx, [
                        &match_class(backtrack_classes),
                        &match_class(input_classes),
                        &match_class(lookahead_classes),
                    ])
                } else {
                    None
                }
            }
            Self::Format3 {
                data,
                coverage,
                backtrack_coverages,
                input_coverages,
                lookahead_coverages,
                lookups,
            } => {
                coverage.get(glyph)?;
                apply_chain_context(
                    ctx,
                    backtrack_coverages,
                    input_coverages,
                    lookahead_coverages,
                    [
                        &match_coverage(data),
                        &match_coverage(data),
                        &match_coverage(data),
                    ],
                    lookups,
                )
            }
        }
    }
}

impl<'a> ChainRuleSet<'a> {
    fn would_apply(&self, ctx: &WouldApplyContext, match_func: &MatchFunc) -> bool {
        self.rules
            .into_iter()
            .filter_map(|data| ChainRule::parse(data))
            .any(|rule| rule.would_apply(ctx, match_func))
    }

    fn apply(&self, ctx: &mut ApplyContext, match_funcs: [&MatchFunc; 3]) -> Option<()> {
        for data in self.rules {
            if let Some(rule) = ChainRule::parse(data) {
                if rule.apply(ctx, match_funcs).is_some() {
                    return Some(());
                }
            }
        }
        None
    }
}

impl<'a> ChainRule<'a> {
    fn would_apply(&self, ctx: &WouldApplyContext, match_func: &MatchFunc) -> bool {
        would_apply_chain_context(ctx, self.backtrack, self.input, self.lookahead, match_func)
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

fn would_apply_context(
    ctx: &WouldApplyContext,
    input: LazyArray16<u16>,
    match_func: &MatchFunc,
) -> bool {
    would_match_input(ctx, input, match_func)
}

fn would_apply_chain_context(
    ctx: &WouldApplyContext,
    backtrack: LazyArray16<u16>,
    input: LazyArray16<u16>,
    lookahead: LazyArray16<u16>,
    match_func: &MatchFunc,
) -> bool {
    (!ctx.zero_context || (backtrack.len() == 0 && lookahead.len() == 0))
        && would_match_input(ctx, input, match_func)
}

fn apply_context(
    ctx: &mut ApplyContext,
    input: LazyArray16<u16>,
    match_func: &MatchFunc,
    lookups: LazyArray16<LookupRecord>,
) -> Option<()> {
    match_input(ctx, input, match_func).map(|matched| {
        ctx.buffer.unsafe_to_break(ctx.buffer.idx, ctx.buffer.idx + matched.len);
        apply_lookup(ctx, input, matched, lookups);
    })
}

fn apply_chain_context(
    ctx: &mut ApplyContext,
    backtrack: LazyArray16<u16>,
    input: LazyArray16<u16>,
    lookahead: LazyArray16<u16>,
    match_funcs: [&MatchFunc; 3],
    lookups: LazyArray16<LookupRecord>,
) -> Option<()> {
    if let Some(matched) = match_input(ctx, input, match_funcs[1]) {
        if let Some(start_idx) = match_backtrack(ctx, backtrack, match_funcs[0]) {
            if let Some(end_idx) = match_lookahead(ctx, lookahead, match_funcs[2], matched.len) {
                ctx.buffer.unsafe_to_break_from_outbuffer(start_idx, end_idx);
                apply_lookup(ctx, input, matched, lookups);
                return Some(());
            }
        }
    }
    None
}

fn apply_lookup(
    ctx: &mut ApplyContext,
    input: LazyArray16<u16>,
    mut matched: Matched,
    lookups: LazyArray16<LookupRecord>,
) {
    let mut count = 1 + usize::from(input.len());

    // All positions are distance from beginning of *output* buffer.
    // Adjust.
    let mut end = {
        let backtrack_len = ctx.buffer.backtrack_len();
        let delta = backtrack_len as isize - ctx.buffer.idx as isize;

        // Convert positions to new indexing.
        for j in 0..count {
            matched.positions[j] = (matched.positions[j] as isize + delta) as _;
        }

        backtrack_len + matched.len
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
        if idx == 0 && record.lookup_index == ctx.lookup_index {
            continue;
        }

        if !ctx.buffer.move_to(matched.positions[idx]) {
            break;
        }

        if ctx.buffer.max_ops <= 0 {
            break;
        }

        let orig_len = ctx.buffer.backtrack_len() + ctx.buffer.lookahead_len();
        if ctx.recurse(record.lookup_index).is_none() {
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
        if end <= matched.positions[idx] {
            // End might end up being smaller than match_positions[idx] if the recursed
            // lookup ended up removing many items, more than we have had matched.
            // Just never rewind end back and get out of here.
            // https://bugs.chromium.org/p/chromium/issues/detail?id=659496
            end = matched.positions[idx];

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
        matched.positions.copy_within(next .. count, (next as isize + delta) as _);
        next = (next as isize + delta) as _;
        count = (count as isize + delta) as _;

        // Fill in new entries.
        for j in idx+1..next {
            matched.positions[j] = matched.positions[j - 1] + 1;
        }

        // And fixup the rest.
        while next < count {
            matched.positions[next] = (matched.positions[next] as isize + delta) as _;
            next += 1;
        }
    }

    ctx.buffer.move_to(end);
}
