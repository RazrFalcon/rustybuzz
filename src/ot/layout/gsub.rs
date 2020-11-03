//! The Glyph Subsitution Table.

use std::convert::TryFrom;

use ttf_parser::parser::{LazyArray16, Offset16, Offsets16, Stream};
use ttf_parser::GlyphId;

use super::common::Coverage;
use super::matching::{
    match_backtrack, match_coverage, match_glyph, match_input, match_lookahead, Matched,
};
use super::{ApplyContext, WouldApplyContext, MAX_NESTING_LEVEL};
use crate::buffer::GlyphPropsFlags;
use crate::ot::Map;
use crate::unicode::GeneralCategory;

#[derive(Clone, Copy, Debug)]
enum SingleSubst<'a> {
    Format1 {
        coverage: Coverage<'a>,
        delta: i16,
    },
    Format2 {
        coverage: Coverage<'a>,
        substitutes: LazyArray16<'a, GlyphId>,
    },
}

impl<'a> SingleSubst<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let delta = s.read::<i16>()?;
                Self::Format1 { coverage, delta }
            }
            2 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let count = s.read::<u16>()?;
                let substitutes = s.read_array16(count)?;
                Self::Format2 { coverage, substitutes }
            }
            _ => return None,
        })
    }

    fn coverage(&self) -> &Coverage<'a> {
        match self {
            Self::Format1 { coverage, .. } => coverage,
            Self::Format2 { coverage, .. } => coverage,
        }
    }

    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        let glyph_id = GlyphId(u16::try_from(ctx.glyph(0)).unwrap());
        ctx.len() == 1 && self.coverage().get(glyph_id).is_some()
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph_id = GlyphId(u16::try_from(ctx.buffer().cur(0).codepoint).unwrap());
        let subst = match *self {
            Self::Format1 { coverage, delta } => {
                coverage.get(glyph_id)?;
                // According to the Adobe Annotated OpenType Suite, result is always
                // limited to 16bit, so we explicitly want to truncate.
                GlyphId((i32::from(glyph_id.0) + i32::from(delta)) as u16)
            }
            Self::Format2 { coverage, substitutes } => {
                let index = coverage.get(glyph_id)?;
                substitutes.get(index)?
            }
        };

        ctx.replace_glyph(subst);
        Some(())
    }
}

#[derive(Clone, Copy, Debug)]
enum MultipleSubst<'a> {
    Format1 {
        coverage: Coverage<'a>,
        sequences: Offsets16<'a, Offset16>,
    },
}

impl<'a> MultipleSubst<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let count = s.read::<u16>()?;
                let sequences = s.read_offsets16(count, data)?;
                Self::Format1 { coverage, sequences }
            }
            _ => return None,
        })
    }

    fn coverage(&self) -> &Coverage<'a> {
        match self {
            Self::Format1 { coverage, .. } => coverage,
        }
    }

    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        let glyph_id = GlyphId(u16::try_from(ctx.glyph(0)).unwrap());
        ctx.len() == 1 && self.coverage().get(glyph_id).is_some()
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph_id = GlyphId(u16::try_from(ctx.buffer().cur(0).codepoint).unwrap());
        match self {
            Self::Format1 { coverage, sequences } => {
                let index = coverage.get(glyph_id)?;
                let seq = Sequence::parse(sequences.slice(index)?)?;
                seq.apply(ctx)
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Sequence<'a> {
    substitutes: LazyArray16<'a, GlyphId>,
}

impl<'a> Sequence<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let substitutes = s.read_array16(count)?;
        Some(Self { substitutes })
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        match self.substitutes.len() {
            // Spec disallows this, but Uniscribe allows it.
            // https://github.com/harfbuzz/harfbuzz/issues/253
            0 => ctx.buffer_mut().delete_glyph(),

            // Special-case to make it in-place and not consider this
            // as a "multiplied" substitution.
            1 => ctx.replace_glyph(self.substitutes.get(0)?),

            _ => {
                let class = if ctx.buffer().cur(0).is_ligature() {
                    GlyphPropsFlags::BASE_GLYPH
                } else {
                    GlyphPropsFlags::empty()
                };

                for (i, subst) in self.substitutes.into_iter().enumerate() {
                    // Index is truncated to 4 bits anway, so we can safely cast to u8.
                    ctx.buffer_mut().cur_mut(0).set_lig_props_for_component(i as u8);
                    ctx.output_glyph_for_component(subst, class);
                }

                ctx.buffer_mut().skip_glyph();
            }
        }
        Some(())
    }
}

#[derive(Clone, Copy, Debug)]
enum AlternateSubst<'a> {
    Format1 {
        coverage: Coverage<'a>,
        alternate_sets: Offsets16<'a, Offset16>,
    },
}

impl<'a> AlternateSubst<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let count = s.read::<u16>()?;
                let alternate_sets = s.read_offsets16(count, data)?;
                Self::Format1 { coverage, alternate_sets }
            }
            _ => return None,
        })
    }

    fn coverage(&self) -> &Coverage<'a> {
        match self {
            Self::Format1 { coverage, .. } => coverage,
        }
    }

    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        let glyph_id = GlyphId(u16::try_from(ctx.glyph(0)).unwrap());
        ctx.len() == 1 && self.coverage().get(glyph_id).is_some()
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph_id = GlyphId(u16::try_from(ctx.buffer().cur(0).codepoint).unwrap());
        match self {
            Self::Format1 { coverage, alternate_sets } => {
                let index = coverage.get(glyph_id)?;
                let set = AlternateSet::parse(alternate_sets.slice(index)?)?;
                set.apply(ctx)
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct AlternateSet<'a> {
    alternates: LazyArray16<'a, GlyphId>,
}

impl<'a> AlternateSet<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let alternates = s.read_array16(count)?;
        Some(Self { alternates })
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let len = self.alternates.len();
        if len == 0 {
            return None;
        }

        let glyph_mask = ctx.buffer().cur(0).mask;
        let lookup_mask = ctx.lookup_mask();

        // Note: This breaks badly if two features enabled this lookup together.
        let shift = lookup_mask.trailing_zeros();
        let mut alt_index = (lookup_mask & glyph_mask) >> shift;

        // If alt_index is MAX_VALUE, randomize feature if it is the rand feature.
        if alt_index == Map::MAX_VALUE && ctx.random() {
            alt_index = ctx.random_number() % u32::from(len) + 1;
        }

        let idx = u16::try_from(alt_index).ok()?.checked_sub(1)?;
        ctx.replace_glyph(self.alternates.get(idx)?);

        Some(())
    }
}

#[derive(Clone, Copy, Debug)]
enum LigatureSubst<'a> {
    Format1 {
        coverage: Coverage<'a>,
        ligature_sets: Offsets16<'a, Offset16>,
    },
}

impl<'a> LigatureSubst<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let count = s.read::<u16>()?;
                let ligature_sets = s.read_offsets16(count, data)?;
                Self::Format1 { coverage, ligature_sets }
            }
            _ => return None,
        })
    }

    fn coverage(&self) -> &Coverage<'a> {
        match self {
            Self::Format1 { coverage, .. } => coverage,
        }
    }

    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        let glyph_id = GlyphId(u16::try_from(ctx.glyph(0)).unwrap());
        match self {
            Self::Format1 { coverage, ligature_sets } => {
                coverage.get(glyph_id)
                    .and_then(|index| ligature_sets.slice(index))
                    .and_then(LigatureSet::parse)
                    .map(|set| set.would_apply(ctx))
                    .unwrap_or(false)
            }
        }
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph_id = GlyphId(u16::try_from(ctx.buffer().cur(0).codepoint).unwrap());
        match self {
            Self::Format1 { coverage, ligature_sets } => {
                let index = coverage.get(glyph_id)?;
                let set = LigatureSet::parse(ligature_sets.slice(index)?)?;
                set.apply(ctx)
            }
        }
    }
}

struct LigatureSet<'a> {
    ligatures: Offsets16<'a, Offset16>,
}

impl<'a> LigatureSet<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let ligatures = s.read_offsets16(count, data)?;
        Some(Self { ligatures })
    }

    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        self.ligatures
            .into_iter()
            .filter_map(|data| Ligature::parse(data))
            .any(|lig| lig.would_apply(ctx))
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        for data in self.ligatures {
            let lig = Ligature::parse(data)?;
            if lig.apply(ctx).is_some() {
                return Some(());
            }
        }
        None
    }
}

struct Ligature<'a> {
    lig_glyph: GlyphId,
    components: LazyArray16<'a, u16>,
}

impl<'a> Ligature<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let lig_glyph = s.read::<GlyphId>()?;
        let count = s.read::<u16>()?;
        let components = s.read_array16(count.checked_sub(1)?)?;
        Some(Self { lig_glyph, components })
    }

    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        ctx.len() == 1 + usize::from(self.components.len())
            && self.components
                .into_iter()
                .enumerate()
                .all(|(i, comp)| ctx.glyph(1 + i) == u32::from(comp))
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        // Special-case to make it in-place and not consider this
        // as a "ligated" substitution.
        if self.components.is_empty() {
            ctx.replace_glyph(self.lig_glyph);
            Some(())
        } else {
            match_input(ctx, self.components, &match_glyph).map(|matched| {
                let count = 1 + usize::from(self.components.len());
                ligate(ctx, count, matched, self.lig_glyph);
            })
        }
    }
}

fn ligate(ctx: &mut ApplyContext, count: usize, matched: Matched, lig_glyph: GlyphId) {
    // - If a base and one or more marks ligate, consider that as a base, NOT
    //   ligature, such that all following marks can still attach to it.
    //   https://github.com/harfbuzz/harfbuzz/issues/1109
    //
    // - If all components of the ligature were marks, we call this a mark ligature.
    //   If it *is* a mark ligature, we don't allocate a new ligature id, and leave
    //   the ligature to keep its old ligature id.  This will allow it to attach to
    //   a base ligature in GPOS.  Eg. if the sequence is: LAM,LAM,SHADDA,FATHA,HEH,
    //   and LAM,LAM,HEH for a ligature, they will leave SHADDA and FATHA with a
    //   ligature id and component value of 2.  Then if SHADDA,FATHA form a ligature
    //   later, we don't want them to lose their ligature id/component, otherwise
    //   GPOS will fail to correctly position the mark ligature on top of the
    //   LAM,LAM,HEH ligature.  See:
    //     https://bugzilla.gnome.org/show_bug.cgi?id=676343
    //
    // - If a ligature is formed of components that some of which are also ligatures
    //   themselves, and those ligature components had marks attached to *their*
    //   components, we have to attach the marks to the new ligature component
    //   positions!  Now *that*'s tricky!  And these marks may be following the
    //   last component of the whole sequence, so we should loop forward looking
    //   for them and update them.
    //
    //   Eg. the sequence is LAM,LAM,SHADDA,FATHA,HEH, and the font first forms a
    //   'calt' ligature of LAM,HEH, leaving the SHADDA and FATHA with a ligature
    //   id and component == 1.  Now, during 'liga', the LAM and the LAM-HEH ligature
    //   form a LAM-LAM-HEH ligature.  We need to reassign the SHADDA and FATHA to
    //   the new ligature with a component value of 2.
    //
    //   This in fact happened to a font...  See:
    //   https://bugzilla.gnome.org/show_bug.cgi?id=437633
    //

    let mut buffer = ctx.buffer_mut();
    buffer.merge_clusters(buffer.idx, buffer.idx + matched.len);

    let mut is_base_ligature = buffer.info[matched.positions[0]].is_base_glyph();
    let mut is_mark_ligature = buffer.info[matched.positions[0]].is_mark();
    for i in 1..count {
        if !buffer.info[matched.positions[i]].is_mark() {
            is_base_ligature = false;
            is_mark_ligature = false;
        }
    }

    let is_ligature = !is_base_ligature && !is_mark_ligature;
    let class = if is_ligature { GlyphPropsFlags::LIGATURE } else { GlyphPropsFlags::empty() };
    let lig_id = if is_ligature { buffer.allocate_lig_id() } else { 0 };
    let first = buffer.cur_mut(0);
    let mut last_lig_id = first.lig_id();
    let mut last_num_comps = first.lig_num_comps();
    let mut comps_so_far = last_num_comps;

    if is_ligature {
        first.set_lig_props_for_ligature(lig_id, matched.total_component_count);
        if first.general_category() == GeneralCategory::NonspacingMark {
            first.set_general_category(GeneralCategory::OtherLetter);
        }
    }

    ctx.replace_glyph_with_ligature(lig_glyph, class);
    buffer = ctx.buffer_mut();

    for i in 1..count {
        while buffer.idx < matched.positions[i] && buffer.successful {
            if is_ligature {
                let cur = buffer.cur_mut(0);
                let mut this_comp = cur.lig_comp();
                if this_comp == 0 {
                    this_comp = last_num_comps;
                }
                let new_lig_comp = comps_so_far - last_num_comps + this_comp.min(last_num_comps);
                cur.set_lig_props_for_mark(lig_id, new_lig_comp);
            }
            buffer.next_glyph();
        }

        let cur = buffer.cur(0);
        last_lig_id = cur.lig_id();
        last_num_comps = cur.lig_num_comps();
        comps_so_far += last_num_comps;

        // Skip the base glyph.
        buffer.idx += 1;
    }

    if !is_mark_ligature && last_lig_id != 0 {
        // Re-adjust components for any marks following.
        for i in buffer.idx..buffer.len {
            let info = &mut buffer.info[i];
            if last_lig_id != info.lig_id() {
                break;
            }

            let this_comp = info.lig_comp();
            if this_comp == 0 {
                break;
            }

            let new_lig_comp = comps_so_far - last_num_comps + this_comp.min(last_num_comps);
            info.set_lig_props_for_mark(lig_id, new_lig_comp)
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ReverseChainSingleSubst<'a> {
    Format1 {
        data: &'a [u8],
        coverage: Coverage<'a>,
        backtrack_coverages: LazyArray16<'a, u16>,
        lookahead_coverages: LazyArray16<'a, u16>,
        substitutes: LazyArray16<'a, GlyphId>,
    },
}

impl<'a> ReverseChainSingleSubst<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let backtrack_count = s.read::<u16>()?;
                let backtrack_coverages = s.read_array16(backtrack_count)?;
                let lookahead_count = s.read::<u16>()?;
                let lookahead_coverages = s.read_array16(lookahead_count)?;
                let substitute_count = s.read::<u16>()?;
                let substitutes = s.read_array16(substitute_count)?;
                Self::Format1 {
                    data,
                    coverage,
                    backtrack_coverages,
                    lookahead_coverages,
                    substitutes,
                }
            }
            _ => return None,
        })
    }

    fn coverage(&self) -> &Coverage<'a> {
        match self {
            Self::Format1 { coverage, .. } => coverage,
        }
    }

    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        let glyph_id = GlyphId(u16::try_from(ctx.glyph(0)).unwrap());
        ctx.len() == 1 && self.coverage().get(glyph_id).is_some()
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        // No chaining to this type.
        if ctx.nesting_level_left() != MAX_NESTING_LEVEL {
            return None;
        }

        let glyph_id = GlyphId(u16::try_from(ctx.buffer().cur(0).codepoint).unwrap());
        match *self {
            Self::Format1 {
                data,
                coverage,
                backtrack_coverages,
                lookahead_coverages,
                substitutes,
            } => {
                let index = coverage.get(glyph_id)?;
                if index >= substitutes.len() {
                    return None;
                }

                let subst = substitutes.get(index)?;
                let match_func = &match_coverage(data);
                if let Some(start_idx) = match_backtrack(ctx, backtrack_coverages, match_func) {
                    if let Some(end_idx) = match_lookahead(ctx, lookahead_coverages, match_func, 1) {
                        ctx.buffer_mut().unsafe_to_break_from_outbuffer(start_idx, end_idx);
                        ctx.replace_glyph_inplace(subst);

                        // Note: We DON'T decrease buffer.idx.  The main loop does it
                        // for us.  This is useful for preventing surprises if someone
                        // calls us through a Context lookup.
                        return Some(());
                    }
                }

                None
            }
        }
    }
}

make_ffi_funcs!(SingleSubst, rb_single_subst_apply, rb_single_subst_would_apply);
make_ffi_funcs!(MultipleSubst, rb_multiple_subst_apply, rb_multiple_subst_would_apply);
make_ffi_funcs!(AlternateSubst, rb_alternate_subst_apply, rb_alternate_subst_would_apply);
make_ffi_funcs!(LigatureSubst, rb_ligature_subst_apply, rb_ligature_subst_would_apply);
make_ffi_funcs!(ReverseChainSingleSubst, rb_reverse_chain_single_subst_apply, rb_reverse_chain_single_subst_would_apply);
