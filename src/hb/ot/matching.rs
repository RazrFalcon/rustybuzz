//! Matching of glyph patterns.

use std::cmp::max;
use ttf_parser::GlyphId;

use crate::hb::buffer::hb_glyph_info_t;
use crate::hb::hb_mask_t;
use crate::hb::ot::apply::ApplyContext;
use crate::hb::ot_layout::*;

pub type MatchFunc<'a> = dyn Fn(GlyphId, u16) -> bool + 'a;

/// Value represents glyph id.
pub fn match_glyph(glyph: GlyphId, value: u16) -> bool {
    glyph == GlyphId(value)
}

pub fn match_input(
    ctx: &mut ApplyContext,
    input_len: u16,
    match_func: &MatchingFunc,
    end_position: &mut usize,
    match_positions: &mut [usize; MAX_CONTEXT_LENGTH],
    p_total_component_count: Option<&mut u8>,
) -> bool {
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

    let count = usize::from(input_len) + 1;
    if count > MAX_CONTEXT_LENGTH {
        return false;
    }

    let mut iter = SkippyIter::new(ctx, ctx.buffer.idx, input_len, false);
    iter.enable_matching(match_func);

    let first = ctx.buffer.cur(0);
    let first_lig_id = _hb_glyph_info_get_lig_id(first);
    let first_lig_comp = _hb_glyph_info_get_lig_comp(first);
    let mut total_component_count = _hb_glyph_info_get_lig_num_comps(first);
    let mut ligbase = Ligbase::NotChecked;

    match_positions[0] = ctx.buffer.idx;

    for position in &mut match_positions[1..count] {
        let mut unsafe_to = 0;
        if !iter.next(Some(&mut unsafe_to)) {
            *end_position = unsafe_to;
            return false;
        }

        *position = iter.index();

        let this = ctx.buffer.info[iter.index()];
        let this_lig_id = _hb_glyph_info_get_lig_id(&this);
        let this_lig_comp = _hb_glyph_info_get_lig_comp(&this);

        if first_lig_id != 0 && first_lig_comp != 0 {
            // If first component was attached to a previous ligature component,
            // all subsequent components should be attached to the same ligature
            // component, otherwise we shouldn't ligate them...
            if first_lig_id != this_lig_id || first_lig_comp != this_lig_comp {
                // ...unless, we are attached to a base ligature and that base
                // ligature is ignorable.
                if ligbase == Ligbase::NotChecked {
                    let out = ctx.buffer.out_info();
                    let mut j = ctx.buffer.out_len;
                    let mut found = false;
                    while j > 0 && _hb_glyph_info_get_lig_id(&out[j - 1]) == first_lig_id {
                        if _hb_glyph_info_get_lig_comp(&out[j - 1]) == 0 {
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
                    return false;
                }
            }
        } else {
            // If first component was NOT attached to a previous ligature component,
            // all subsequent components should also NOT be attached to any ligature
            // component, unless they are attached to the first component itself!
            if this_lig_id != 0 && this_lig_comp != 0 && (this_lig_id != first_lig_id) {
                return false;
            }
        }

        total_component_count += _hb_glyph_info_get_lig_num_comps(&this);
    }

    *end_position = iter.index() + 1;

    if let Some(p_total_component_count) = p_total_component_count {
        *p_total_component_count = total_component_count;
    }

    true
}

pub fn match_backtrack(
    ctx: &mut ApplyContext,
    backtrack_len: u16,
    match_func: &MatchingFunc,
    match_start: &mut usize,
) -> bool {
    let mut iter = SkippyIter::new(ctx, ctx.buffer.backtrack_len(), backtrack_len, true);
    iter.enable_matching(match_func);

    for _ in 0..backtrack_len {
        let mut unsafe_from = 0;
        if !iter.prev(Some(&mut unsafe_from)) {
            *match_start = unsafe_from;
            return false;
        }
    }

    *match_start = iter.index();
    true
}

pub fn match_lookahead(
    ctx: &mut ApplyContext,
    lookahead_len: u16,
    match_func: &MatchingFunc,
    start_index: usize,
    end_index: &mut usize,
) -> bool {
    let mut iter = SkippyIter::new(ctx, start_index - 1, lookahead_len, true);
    iter.enable_matching(match_func);

    for _ in 0..lookahead_len {
        let mut unsafe_to = 0;
        if !iter.next(Some(&mut unsafe_to)) {
            *end_index = unsafe_to;
            return false;
        }
    }

    *end_index = iter.index() + 1;
    true
}

pub type MatchingFunc<'a> = dyn Fn(GlyphId, u16) -> bool + 'a;

pub struct SkippyIter<'a, 'b> {
    ctx: &'a ApplyContext<'a, 'b>,
    lookup_props: u32,
    ignore_zwnj: bool,
    ignore_zwj: bool,
    mask: hb_mask_t,
    syllable: u8,
    matching: Option<&'a MatchingFunc<'a>>,
    buf_len: usize,
    buf_idx: usize,
    num_items: u16,
}

impl<'a, 'b> SkippyIter<'a, 'b> {
    pub fn new(
        ctx: &'a ApplyContext<'a, 'b>,
        start_buf_index: usize,
        num_items: u16,
        context_match: bool,
    ) -> Self {
        SkippyIter {
            ctx,
            lookup_props: ctx.lookup_props,
            // Ignore ZWNJ if we are matching GPOS, or matching GSUB context and asked to.
            ignore_zwnj: ctx.table_index == TableIndex::GPOS || (context_match && ctx.auto_zwnj),
            // Ignore ZWJ if we are matching context, or asked to.
            ignore_zwj: context_match || ctx.auto_zwj,
            mask: if context_match {
                u32::MAX
            } else {
                ctx.lookup_mask
            },
            syllable: if ctx.buffer.idx == start_buf_index {
                ctx.buffer.cur(0).syllable()
            } else {
                0
            },
            matching: None,
            buf_len: ctx.buffer.len,
            buf_idx: start_buf_index,
            num_items,
        }
    }

    pub fn set_lookup_props(&mut self, lookup_props: u32) {
        self.lookup_props = lookup_props;
    }

    pub fn enable_matching(&mut self, func: &'a MatchingFunc<'a>) {
        self.matching = Some(func);
    }

    pub fn index(&self) -> usize {
        self.buf_idx
    }

    pub fn next(&mut self, unsafe_to: Option<&mut usize>) -> bool {
        assert!(self.num_items > 0);
        while self.buf_idx + usize::from(self.num_items) < self.buf_len {
            self.buf_idx += 1;
            let info = &self.ctx.buffer.info[self.buf_idx];

            let skip = self.may_skip(info);
            if skip == Some(true) {
                continue;
            }

            let matched = self.may_match(info);
            if matched == Some(true) || (matched.is_none() && skip == Some(false)) {
                self.num_items -= 1;
                return true;
            }

            if skip == Some(false) {
                if let Some(unsafe_to) = unsafe_to {
                    *unsafe_to = self.buf_idx + 1;
                }

                return false;
            }
        }

        if let Some(unsafe_to) = unsafe_to {
            *unsafe_to = self.buf_idx + 1;
        }

        false
    }

    pub fn prev(&mut self, unsafe_from: Option<&mut usize>) -> bool {
        assert!(self.num_items > 0);
        while self.buf_idx >= usize::from(self.num_items) {
            self.buf_idx -= 1;
            let info = &self.ctx.buffer.out_info()[self.buf_idx];

            let skip = self.may_skip(info);
            if skip == Some(true) {
                continue;
            }

            let matched = self.may_match(info);
            if matched == Some(true) || (matched.is_none() && skip == Some(false)) {
                self.num_items -= 1;
                return true;
            }

            if skip == Some(false) {
                if let Some(unsafe_from) = unsafe_from {
                    *unsafe_from = max(1, self.buf_idx) - 1;
                }

                return false;
            }
        }

        if let Some(unsafe_from) = unsafe_from {
            *unsafe_from = 0;
        }

        false
    }

    pub fn reject(&mut self) {
        self.num_items += 1;
    }

    fn may_match(&self, info: &hb_glyph_info_t) -> Option<bool> {
        if (info.mask & self.mask) != 0 && (self.syllable == 0 || self.syllable == info.syllable())
        {
            self.matching.map(|f| f(info.as_glyph(), self.num_items))
        } else {
            Some(false)
        }
    }

    fn may_skip(&self, info: &hb_glyph_info_t) -> Option<bool> {
        if !self.ctx.check_glyph_property(info, self.lookup_props) {
            return Some(true);
        }

        if !_hb_glyph_info_is_default_ignorable(info)
            || info.is_hidden()
            || (!self.ignore_zwnj && _hb_glyph_info_is_zwnj(info))
            || (!self.ignore_zwj && _hb_glyph_info_is_zwj(info))
        {
            return Some(false);
        }

        None
    }
}
