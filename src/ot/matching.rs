use std::convert::TryFrom;

use ttf_parser::GlyphId;
use ttf_parser::parser::LazyArray16;

use crate::Mask;
use crate::buffer::GlyphInfo;
use super::layout::{ApplyContext, MAX_CONTEXT_LENGTH};

pub type MatchFunc = dyn Fn(GlyphId, GlyphId) -> bool;

pub struct MatchedInput {
    pub end_offset: usize,
    pub match_positions: [usize; MAX_CONTEXT_LENGTH],
    pub total_component_count: u32,
}

pub fn match_input(
    ctx: &ApplyContext,
    input: LazyArray16<GlyphId>,
    match_func: &MatchFunc,
) -> Option<MatchedInput> {
    let count = 1 + input.len() as usize;
    if count > MAX_CONTEXT_LENGTH {
        return None;
    }

    // This is perhaps the trickiest part of OpenType...  Remarks:
    //
    // - If all components of the ligature were marks, we call this a mark ligature.
    //
    // - If there is no GDEF, and the ligature is NOT a mark ligature, we categorize
    //   it as a ligature glyph.
    //
    // - Ligatures cannot be formed across glyphs attached to different components
    //   of previous ligatures.  Eg. the sequence is LAM,SHADDA,LAM,FATHA,HEH, and
    //   LAM,LAM,HEH form a ligature, leaving SHADDA,FATHA next to eachother.
    //   However, it would be wrong to ligate that SHADDA,FATHA sequence.
    //   There are a couple of exceptions to this:
    //
    //   o If a ligature tries ligating with marks that belong to it itself, go ahead,
    //     assuming that the font designer knows what they are doing (otherwise it can
    //     break Indic stuff when a matra wants to ligate with a conjunct,
    //
    //   o If two marks want to ligate and they belong to different components of the
    //     same ligature glyph, and said ligature glyph is to be ignored according to
    //     mark-filtering rules, then allow.
    //     https://github.com/harfbuzz/harfbuzz/issues/545

    #[derive(PartialEq)]
    enum Ligbase {
        NotChecked,
        MayNotSkip,
        MaySkip,
    }

    let buffer = ctx.buffer();
    let first = buffer.cur(0);
    let first_lig_id = first.lig_id();
    let first_lig_comp = first.lig_comp();
    let mut match_positions = [0; MAX_CONTEXT_LENGTH];
    let mut total_component_count = first.lig_num_comps() as u32;
    let mut ligbase = Ligbase::NotChecked;
    let mut iter = SkippyIter::new(ctx, input, false, buffer.idx);
    iter.set_match_func(Some(match_func));

    match_positions[0] = buffer.idx;

    for i in 1 .. count {
        if !iter.next() {
            return None;
        }

        match_positions[i] = iter.buf_idx;

        let this = buffer.info[iter.buf_idx];
        let this_lig_id = this.lig_id();
        let this_lig_comp = this.lig_comp();

        if first_lig_id != 0 && first_lig_comp != 0 {
            // If first component was attached to a previous ligature component,
            // all subsequent components should be attached to the same ligature
            // component, otherwise we shouldn't ligate them...
            if first_lig_id != this_lig_id || first_lig_comp != this_lig_comp {
                // ...unless, we are attached to a base ligature and that base
                // ligature is ignorable.
                if ligbase == Ligbase::NotChecked {
                    let mut found = false;
                    let out = buffer.out_info();
                    let mut j = buffer.out_len;
                    while j > 0 && out[j - 1].lig_id() == first_lig_id {
                        if out[j - 1].lig_comp() == 0 {
                            j -= 1;
                            found = true;
                            break;
                        }
                        j -= 1;
                    }

                    ligbase = if found && iter.may_skip(&out[j]) == Some(true) {
                        Ligbase::MaySkip
                    } else {
                        Ligbase::MayNotSkip
                    };
                }

                if ligbase == Ligbase::MayNotSkip {
                    return None;
                }
            }
        } else {
            // If first component was NOT attached to a previous ligature component,
            // all subsequent components should also NOT be attached to any ligature
            // component, unless they are attached to the first component itself!
            if this_lig_id != 0 && this_lig_comp != 0 && (this_lig_id != first_lig_id) {
                return None;
            }
        }

        total_component_count += buffer.info[iter.buf_idx].lig_num_comps() as u32;
    }

    Some(MatchedInput {
        end_offset: iter.buf_idx - buffer.idx + 1,
        match_positions,
        total_component_count,
    })
}

pub fn match_backtrack(
    ctx: &ApplyContext,
    backtrack: LazyArray16<GlyphId>,
    match_func: &MatchFunc,
) -> Option<usize> {
    let mut iter = SkippyIter::new(ctx, backtrack, true, ctx.buffer().backtrack_len());
    iter.set_match_func(Some(match_func));

    for _ in 0 .. backtrack.len() {
        if !iter.prev() {
            return None;
        }
    }

    Some(iter.buf_idx)
}

pub fn match_lookahead(
    ctx: &ApplyContext,
    lookahead: LazyArray16<GlyphId>,
    match_func: &MatchFunc,
    offset: usize,
) -> Option<usize> {
    let mut iter = SkippyIter::new(ctx, lookahead, true, ctx.buffer().idx + offset - 1);
    iter.set_match_func(Some(match_func));

    for _ in 0 .. lookahead.len() {
        if !iter.next() {
            return None;
        }
    }

    Some(iter.buf_idx + 1)
}

pub struct SkippyIter<'a> {
    ctx: &'a ApplyContext,
    lookup_props: u16,
    ignore_zwnj: bool,
    ignore_zwj: bool,
    mask: Mask,
    syllable: u8,
    match_func: Option<&'a MatchFunc>,
    input: LazyArray16<'a, GlyphId>,
    buf_len: usize,
    buf_idx: usize,
    input_idx: u16,
}

impl<'a> SkippyIter<'a> {
    pub fn new(
        ctx: &'a ApplyContext,
        input: LazyArray16<'a, GlyphId>,
        context_match: bool,
        start_buf_index: usize,
    ) -> Self {
        SkippyIter {
            ctx,
            lookup_props: 0,
            // Ignore ZWNJ if we are matching GPOS, or matching GSUB context and asked to.
            ignore_zwnj: ctx.table_index() == 1 || (context_match && ctx.auto_zwnj()),
            // Ignore ZWJ if we are matching context, or asked to.
            ignore_zwj: context_match || ctx.auto_zwj(),
            mask: if context_match { u32::MAX } else { ctx.lookup_mask() },
            syllable: if ctx.buffer().idx > 0 { ctx.buffer().cur(0).syllable() } else { 0 },
            match_func: None,
            input,
            buf_len: ctx.buffer().len,
            buf_idx: start_buf_index,
            input_idx: 0,
        }
    }

    pub fn set_lookup_props(&mut self, lookup_props: u16) {
        self.lookup_props = lookup_props;
    }

    pub fn set_match_func(&mut self, match_func: Option<&'a MatchFunc>) {
        self.match_func = match_func;
    }
}

macro_rules! advance_impl {
    ($self:expr, $glyph:expr) => {{
        let info = &$self.ctx.buffer().info[$self.buf_idx];

        let skip = $self.may_skip(info);
        if skip == Some(true) {
            continue;
        }

        let matched = $self.may_match(info, $glyph);
        if matched == Some(true) || (matched.is_none() && skip == Some(false)) {
            $self.input_idx += 1;
            return true;
        }

        if skip == Some(false) {
            return false;
        }
    }}
}

impl SkippyIter<'_> {
    pub fn next(&mut self) -> bool {
        let glyph = self.input.get(self.input_idx).unwrap();
        let num_items = (self.input.len() - self.input_idx) as usize;

        while self.buf_idx + num_items < self.buf_len {
            self.buf_idx += 1;
            advance_impl!(self, glyph);
        }

        false
    }

    pub fn prev(&mut self) -> bool {
        let glyph = self.input.get(self.input_idx).unwrap();
        let num_items = (self.input.len() - self.input_idx) as usize;

        while self.buf_idx > num_items - 1 {
            self.buf_idx -= 1;
            advance_impl!(self, glyph);
        }

        false
    }

    fn may_match(&self, info: &GlyphInfo, glyph: GlyphId) -> Option<bool> {
        if (info.mask & self.mask) != 0 && (self.syllable == 0 || self.syllable == info.syllable()) {
            self.match_func.map(|func| {
                func(GlyphId(u16::try_from(info.codepoint).unwrap()), glyph)
            })
        } else {
            Some(false)
        }
    }

    fn may_skip(&self, info: &GlyphInfo) -> Option<bool> {
        if self.ctx.check_glyph_property(info, self.lookup_props) {
            return Some(true);
        }

        if !info.is_default_ignorable()
            || info.is_hidden()
            || (!self.ignore_zwnj && info.is_zwnj())
            || (!self.ignore_zwj && info.is_zwj())
        {
            return Some(false);
        }

        None
    }
}
