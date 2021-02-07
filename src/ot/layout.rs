//! OpenType layout.

use core::ops::{Index, IndexMut};

use ttf_parser::GlyphId;

use crate::{Face, Tag};
use crate::buffer::Buffer;
use crate::common::TagExt;
use crate::plan::ShapePlan;
use crate::tables::gsubgpos::{FeatureIndex, LangIndex, LookupIndex, ScriptIndex, SubstPosTable};
use super::apply::{Apply, ApplyContext};

pub const MAX_NESTING_LEVEL: usize = 6;
pub const MAX_CONTEXT_LENGTH: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableIndex {
    GSUB = 0,
    GPOS = 1,
}

impl TableIndex {
    pub fn iter() -> impl Iterator<Item = TableIndex> {
        [Self::GSUB, Self::GPOS].iter().copied()
    }
}

impl<T> Index<TableIndex> for [T] {
    type Output = T;

    fn index(&self, table_index: TableIndex) -> &Self::Output {
        &self[table_index as usize]
    }
}

impl<T> IndexMut<TableIndex> for [T] {
    fn index_mut(&mut self, table_index: TableIndex) -> &mut Self::Output {
        &mut self[table_index as usize]
    }
}

/// A lookup-based layout table (GSUB or GPOS).
pub trait LayoutTable {
    /// The index of this table.
    const INDEX: TableIndex;

    /// Whether lookups in this table can be applied to the buffer in-place.
    const IN_PLACE: bool;

    /// The kind of lookup stored in this table.
    type Lookup: LayoutLookup;

    /// Get the lookup at the specified index.
    fn get_lookup(&self, index: LookupIndex) -> Option<&Self::Lookup>;
}

/// A lookup in a layout table.
pub trait LayoutLookup: Apply {
    /// The lookup's lookup_props.
    fn props(&self) -> u32;

    /// Whether the lookup has to be applied backwards.
    fn is_reverse(&self) -> bool;

    /// Whether any subtable of the lookup could apply at a specific glyph.
    fn covers(&self, glyph: GlyphId) -> bool;
}

impl SubstPosTable<'_> {
    /// Returns true + index and tag of the first found script tag in the given GSUB or GPOS table
    /// or false + index and tag if falling back to a default script.
    pub fn select_script(&self, script_tags: &[Tag]) -> Option<(bool, ScriptIndex, Tag)> {
        for &tag in script_tags {
            if let Some(index) = self.find_script_index(tag) {
                return Some((true, index, tag));
            }
        }

        for &tag in &[
            // try finding 'DFLT'
            Tag::default_script(),
            // try with 'dflt'; MS site has had typos and many fonts use it now :(
            Tag::default_language(),
            // try with 'latn'; some old fonts put their features there even though
            // they're really trying to support Thai, for example :(
            Tag::from_bytes(b"latn"),
        ] {
            if let Some(index) = self.find_script_index(tag) {
                return Some((false, index, tag));
            }
        }

        None
    }

    /// Returns the index of the first found language tag in the given GSUB or GPOS table,
    /// underneath the specified script index.
    pub fn select_script_language(
        &self,
        script_index: ScriptIndex,
        lang_tags: &[Tag],
    ) -> Option<LangIndex> {
        let script = self.get_script(script_index)?;

        for &tag in lang_tags {
            if let Some(index) = script.find_lang_index(tag) {
                return Some(index);
            }
        }

        // try finding 'dflt'
        if let Some(index) = script.find_lang_index(Tag::default_language()) {
            return Some(index);
        }

        None
    }

    /// Returns the index and tag of a required feature in the given GSUB or GPOS table,
    /// underneath the specified script and language.
    pub fn get_required_language_feature(
        &self,
        script_index: ScriptIndex,
        lang_index: Option<LangIndex>,
    ) -> Option<(FeatureIndex, Tag)> {
        let script = self.get_script(script_index)?;
        let sys = match lang_index {
            Some(index) => script.get_lang(index)?,
            None => script.default_lang()?,
        };
        let idx = sys.required_feature?;
        let tag = self.get_feature_tag(idx)?;
        Some((idx, tag))
    }

    /// Returns the index of a given feature tag in the given GSUB or GPOS table,
    /// underneath the specified script and language.
    pub fn find_language_feature(
        &self,
        script_index: ScriptIndex,
        lang_index: Option<LangIndex>,
        feature_tag: Tag,
    ) -> Option<FeatureIndex> {
        let script = self.get_script(script_index)?;
        let sys = match lang_index {
            Some(index) => script.get_lang(index)?,
            None => script.default_lang()?,
        };

        for i in 0..sys.feature_count() {
            if let Some(index) = sys.get_feature_index(i) {
                if self.get_feature_tag(index) == Some(feature_tag) {
                    return Some(index);
                }
            }
        }

        None
    }
}

/// Applies the lookups in the given GSUB or GPOS table.
pub fn apply_layout_table<T: LayoutTable>(
    plan: &ShapePlan,
    face: &Face,
    buffer: &mut Buffer,
    table: Option<&T>,
) {
    let mut ctx = ApplyContext::new(T::INDEX, face, buffer);

    for (stage_index, stage) in plan.ot_map.stages(T::INDEX).into_iter().enumerate() {
        for lookup in plan.ot_map.stage_lookups(T::INDEX, stage_index) {
            ctx.lookup_index = lookup.index;
            ctx.lookup_mask = lookup.mask;
            ctx.auto_zwj = lookup.auto_zwj;
            ctx.auto_zwnj = lookup.auto_zwnj;

            if lookup.random {
                ctx.random = true;
                ctx.buffer.unsafe_to_break(0, ctx.buffer.len);
            }

            if let Some(table) = &table {
                if let Some(lookup) =  table.get_lookup(lookup.index) {
                    apply_string::<T>(&mut ctx, lookup);
                }
            }
        }

        if let Some(func) = stage.pause_func {
            ctx.buffer.clear_output();
            func(plan, face, ctx.buffer);
        }
    }
}

fn apply_string<T: LayoutTable>(ctx: &mut ApplyContext, lookup: &T::Lookup) {
    if ctx.buffer.is_empty() || ctx.lookup_mask == 0 {
        return;
    }

    ctx.lookup_props = lookup.props();

    if !lookup.is_reverse() {
        // in/out forward substitution/positioning
        if T::INDEX == TableIndex::GSUB {
            ctx.buffer.clear_output();
        }
        ctx.buffer.idx = 0;

        if apply_forward(ctx, lookup) {
            if !T::IN_PLACE {
                ctx.buffer.swap_buffers();
            } else {
                assert!(!ctx.buffer.have_separate_output);
            }
        }
    } else {
        // in-place backward substitution/positioning
        if T::INDEX == TableIndex::GSUB {
            ctx.buffer.remove_output();
        }

        ctx.buffer.idx = ctx.buffer.len - 1;
        apply_backward(ctx, lookup);
    }
}

fn apply_forward(ctx: &mut ApplyContext, lookup: &impl Apply) -> bool {
    let mut ret = false;
    while ctx.buffer.idx < ctx.buffer.len && ctx.buffer.successful {
        let cur = ctx.buffer.cur(0);
        if (cur.mask & ctx.lookup_mask) != 0
            && ctx.check_glyph_property(cur, ctx.lookup_props)
            && lookup.apply(ctx).is_some()
        {
            ret = true;
        } else {
            ctx.buffer.next_glyph();
        }
    }
    ret
}

fn apply_backward(ctx: &mut ApplyContext, lookup: &impl Apply) -> bool {
    let mut ret = false;
    loop {
        let cur = ctx.buffer.cur(0);
        ret |= (cur.mask & ctx.lookup_mask) != 0
            && ctx.check_glyph_property(cur, ctx.lookup_props)
            && lookup.apply(ctx).is_some();

        if ctx.buffer.idx == 0 {
            break;
        }

        ctx.buffer.idx -= 1;
    }
    ret
}

pub fn clear_substitution_flags(_: &ShapePlan, _: &Face, buffer: &mut Buffer) {
    let len = buffer.len;
    for info in &mut buffer.info[..len] {
        info.clear_substituted();
    }
}

pub fn clear_syllables(_: &ShapePlan, _: &Face, buffer: &mut Buffer) {
    let len = buffer.len;
    for info in &mut buffer.info[..len] {
        info.set_syllable(0);
    }
}
