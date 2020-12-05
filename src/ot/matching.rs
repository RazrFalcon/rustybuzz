//! Matching of glyph patterns.

use ttf_parser::parser::LazyArray16;
use ttf_parser::GlyphId;

use crate::Mask;
use crate::buffer::GlyphInfo;
use crate::tables::gsubgpos::{Class, ClassDef, Coverage};
use super::{TableIndex, MAX_CONTEXT_LENGTH};
use super::apply::{ApplyContext, WouldApplyContext};

pub type MatchFunc<'a> = dyn Fn(GlyphId, u16) -> bool + 'a;

/// Value represents glyph id.
pub fn match_glyph(glyph: GlyphId, value: u16) -> bool {
    glyph == GlyphId(value)
}

/// Value represents glyph class.
pub fn match_class<'a>(class_def: ClassDef<'a>) -> impl Fn(GlyphId, u16) -> bool + 'a {
    move |glyph, value| class_def.get(glyph) == Class(value)
}

/// Value represents offset to coverage table.
pub fn match_coverage<'a>(data: &'a [u8]) -> impl Fn(GlyphId, u16) -> bool + 'a {
    move |glyph, value| {
        data.get(usize::from(value)..)
            .and_then(Coverage::parse)
            .map_or(false, |coverage| coverage.get(glyph).is_some())
    }
}

pub fn would_match_input(
    ctx: &WouldApplyContext,
    input: LazyArray16<u16>,
    match_func: &MatchFunc,
) -> bool {
    ctx.glyphs.len() == 1 + usize::from(input.len())
        && input.into_iter().enumerate().all(|(i, value)| {
            match_func(ctx.glyphs[1 + i], value)
        })
}

// TODO: Find out whether returning this by value is slow.
pub struct Matched {
    pub len: usize,
    pub positions: [usize; MAX_CONTEXT_LENGTH],
    pub total_component_count: u8,
}

pub fn match_input(
    ctx: &ApplyContext,
    input: LazyArray16<u16>,
    match_func: &MatchFunc,
) -> Option<Matched> {
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

    let count = 1 + usize::from(input.len());
    if count > MAX_CONTEXT_LENGTH {
        return None;
    }

    let mut iter = SkippyIter::new(ctx, ctx.buffer.idx, input.len(), false);
    iter.enable_matching(input, match_func);

    let first = ctx.buffer.cur(0);
    let first_lig_id = first.lig_id();
    let first_lig_comp = first.lig_comp();
    let mut positions = [0; MAX_CONTEXT_LENGTH];
    let mut total_component_count = first.lig_num_comps();
    let mut ligbase = Ligbase::NotChecked;

    positions[0] = ctx.buffer.idx;

    for i in 1..count {
        if !iter.next() {
            return None;
        }

        positions[i] = iter.index();

        let this = ctx.buffer.info[iter.index()];
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
                    let out = ctx.buffer.out_info();
                    let mut j = ctx.buffer.out_len;
                    let mut found = false;
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

        total_component_count += this.lig_num_comps();
    }

    Some(Matched {
        len: iter.index() - ctx.buffer.idx + 1,
        positions,
        total_component_count,
    })
}

pub fn match_backtrack(
    ctx: &ApplyContext,
    backtrack: LazyArray16<u16>,
    match_func: &MatchFunc,
) -> Option<usize> {
    let mut iter = SkippyIter::new(ctx, ctx.buffer.backtrack_len(), backtrack.len(), true);
    iter.enable_matching(backtrack, match_func);

    for _ in 0..backtrack.len() {
        if !iter.prev() {
            return None;
        }
    }

    Some(iter.index())
}

pub fn match_lookahead(
    ctx: &ApplyContext,
    lookahead: LazyArray16<u16>,
    match_func: &MatchFunc,
    offset: usize,
) -> Option<usize> {
    let mut iter = SkippyIter::new(ctx, ctx.buffer.idx + offset - 1, lookahead.len(), true);
    iter.enable_matching(lookahead, match_func);

    for _ in 0..lookahead.len() {
        if !iter.next() {
            return None;
        }
    }

    Some(iter.index() + 1)
}

pub struct SkippyIter<'a, 'b> {
    ctx: &'a ApplyContext<'a, 'b>,
    lookup_props: u32,
    ignore_zwnj: bool,
    ignore_zwj: bool,
    mask: Mask,
    syllable: u8,
    matching: Option<(LazyArray16<'a, u16>, &'a MatchFunc<'a>)>,
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
            mask: if context_match { u32::MAX } else { ctx.lookup_mask },
            syllable: if ctx.buffer.idx == start_buf_index {
                ctx.buffer.cur(0).syllable()
            } else {
                0
            },
            matching: None,
            buf_len: ctx.buffer.len,
            buf_idx: start_buf_index,
            num_items
        }
    }

    pub fn set_lookup_props(&mut self, lookup_props: u32) {
        self.lookup_props = lookup_props;
    }

    pub fn enable_matching(&mut self, input: LazyArray16<'a, u16>, match_func: &'a MatchFunc) {
        self.matching = Some((input, match_func));
    }

    pub fn index(&self) -> usize {
        self.buf_idx
    }

    pub fn next(&mut self) -> bool {
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
                return false;
            }
        }

        false
    }

    pub fn prev(&mut self) -> bool {
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
                return false;
            }
        }

        false
    }

    pub fn reject(&mut self) {
        self.num_items += 1;
    }

    fn may_match(&self, info: &GlyphInfo) -> Option<bool> {
        if (info.mask & self.mask) != 0 && (self.syllable == 0 || self.syllable == info.syllable()) {
            self.matching.map(|(input, func)| {
                let index = input.len() - self.num_items;
                let value = input.get(index).unwrap();
                func(info.as_glyph(), value)
            })
        } else {
            Some(false)
        }
    }

    fn may_skip(&self, info: &GlyphInfo) -> Option<bool> {
        if !self.ctx.check_glyph_property(info, self.lookup_props) {
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
