//! OpenType layout.

use std::slice;

use ttf_parser::parser::NumFrom;

use crate::{ffi, Face, Tag};
use crate::buffer::Buffer;
use crate::common::TagExt;
use crate::plan::ShapePlan;
use crate::tables::gsubgpos::{
    FeatureIndex, LangIndex, LookupIndex, ScriptIndex, SubstPosTable, VariationIndex
};
use super::apply::{Apply, ApplyContext, WouldApply, WouldApplyContext};

pub const MAX_NESTING_LEVEL: usize = 6;
pub const MAX_CONTEXT_LENGTH: usize = 64;
pub const MAX_TAGS_PER_SCRIPT: usize = 3;
pub const MAX_TAGS_PER_LANGUAGE: usize = 3;

pub const SCRIPT_NOT_FOUND_INDEX: u32 = 0xFFFF;
pub const LANGUAGE_NOT_FOUND_INDEX: u32 = 0xFFFF;
pub const FEATURE_NOT_FOUND_INDEX: u32 = 0xFFFF;
pub const FEATURE_VARIATION_NOT_FOUND_INDEX: u32 = 0xFFFFFFFF;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TableIndex {
    GSUB = 0,
    GPOS = 1,
}

/// A lookup-based layout table (GSUB or GPOS).
pub trait LayoutTable {
    /// The tag of this table.
    const TAG: Tag;

    /// The index of this table.
    const INDEX: TableIndex;

    /// Whether lookups in this table can be applied to the buffer in-place.
    const IN_PLACE: bool;

    /// The kind of lookup stored in this table.
    type Lookup: LayoutLookup;

    /// Get the lookup at the specified index.
    fn get_lookup(&self, index: LookupIndex) -> Option<Self::Lookup>;
}

/// A lookup in a layout table.
pub trait LayoutLookup: Apply {
    /// The lookup's lookup_props.
    fn props(&self) -> u32;

    /// Whether the lookup has to be applied backwards.
    fn is_reverse(&self) -> bool;
}

// GSUB/GPOS

pub fn rb_ot_layout_table_select_script(
    face: *const ffi::rb_face_t,
    table_tag: Tag,
    script_count: u32,
    script_tags: *const Tag,
    script_index: *mut u32,
    chosen_script: *mut Tag,
) -> ffi::rb_bool_t {
    unsafe {
        *script_index = SCRIPT_NOT_FOUND_INDEX;
        *chosen_script = Tag(SCRIPT_NOT_FOUND_INDEX);
    }

    let scripts = unsafe { slice::from_raw_parts(script_tags, usize::num_from(script_count)) };
    if let Some((found, index, tag)) = Face::from_ptr(face)
        .layout_table(table_tag)
        .and_then(|table| select_script(table, scripts))
    {
        unsafe {
            *script_index = u32::from(index.0);
            *chosen_script = tag;
        }
        return found as ffi::rb_bool_t;
    }

    0
}

fn select_script(table: SubstPosTable, script_tags: &[Tag]) -> Option<(bool, ScriptIndex, Tag)> {
    for &tag in script_tags {
        if let Some(index) = table.find_script_index(tag) {
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
        if let Some(index) = table.find_script_index(tag) {
            return Some((false, index, tag));
        }
    }

    None
}

/// Fetches the index of a given feature tag in the specified face's GSUB table
/// or GPOS table, underneath the specified script and language.
///
/// Returns true if the feature is found, false otherwise.
pub fn rb_ot_layout_language_find_feature(
    face: *const ffi::rb_face_t,
    table_tag: Tag,
    script_index: u32,
    language_index: u32,
    feature_tag: Tag,
    feature_index: *mut u32,
) -> ffi::rb_bool_t {
    unsafe { *feature_index = FEATURE_NOT_FOUND_INDEX; }

    let script_index = ScriptIndex(script_index as u16);
    let lang_index = match language_index {
        LANGUAGE_NOT_FOUND_INDEX => None,
        _ => Some(LangIndex(language_index as u16)),
    };

    if let Some(index) = Face::from_ptr(face)
        .layout_table(table_tag)
        .and_then(|table| language_find_feature(table, script_index, lang_index, feature_tag))
    {
        unsafe { *feature_index = u32::from(index.0); }
        return 1;
    }

    0
}

fn language_find_feature(
    table: SubstPosTable,
    script_index: ScriptIndex,
    lang_index: Option<LangIndex>,
    feature_tag: Tag,
) -> Option<FeatureIndex> {
    let script = table.get_script(script_index)?;
    let sys = match lang_index {
        Some(index) => script.get_lang(index)?,
        None => script.default_lang()?,
    };

    for i in 0..sys.feature_count() {
        if let Some(index) = sys.get_feature_index(i) {
            if table.get_feature_tag(index) == Some(feature_tag) {
               return Some(index);
            }
        }
    }

    None
}

/// Fetches the index of a given language tag in the specified face's GSUB table
/// or GPOS table, underneath the specified script index.
///
/// Returns true if the language tag is found, false otherwise.
pub fn rb_ot_layout_script_select_language(
    face: *const ffi::rb_face_t,
    table_tag: Tag,
    script_index: u32,
    language_count: u32,
    language_tags: *const Tag,
    language_index: *mut u32,
) -> ffi::rb_bool_t {
    unsafe { *language_index = LANGUAGE_NOT_FOUND_INDEX; }

    let langs = unsafe { slice::from_raw_parts(language_tags, usize::num_from(language_count)) };
    let script_index = ScriptIndex(script_index as u16);
    if let Some((found, index)) = Face::from_ptr(face)
        .layout_table(table_tag)
        .and_then(|table| script_select_language(table, script_index, langs))
    {
        unsafe { *language_index = u32::from(index.0); }
        return found as ffi::rb_bool_t;
    }

    0
}

fn script_select_language(
    table: SubstPosTable,
    script_index: ScriptIndex,
    lang_tags: &[Tag],
) -> Option<(bool, LangIndex)> {
    let script = table.get_script(script_index)?;

    for &tag in lang_tags {
        if let Some(index) = script.find_lang_index(tag) {
            return Some((true, index));
        }
    }

    // try finding 'dflt'
    if let Some(index) = script.find_lang_index(Tag::default_language()) {
        return Some((false, index));
    }

    None
}

/// Fetches the tag of a requested feature index in the given face's GSUB or GPOS table,
/// underneath the specified script and language.
///
/// Returns true if the feature is found, false otherwise.
pub fn rb_ot_layout_language_get_required_feature(
    face: *const ffi::rb_face_t,
    table_tag: Tag,
    script_index: u32,
    language_index: u32,
    feature_index: *mut u32,
    feature_tag: *mut Tag,
) -> ffi::rb_bool_t {
    unsafe {
        *feature_index = FEATURE_NOT_FOUND_INDEX;
        *feature_tag = Tag(0);
    }

    let script_index = ScriptIndex(script_index as u16);
    let lang_index = match language_index {
        LANGUAGE_NOT_FOUND_INDEX => None,
        _ => Some(LangIndex(language_index as u16)),
    };

    if let Some((index, tag)) = Face::from_ptr(face)
        .layout_table(table_tag)
        .and_then(|table| language_get_required_feature(table, script_index, lang_index))
    {
        unsafe {
            *feature_index = u32::from(index.0);
            *feature_tag = tag;
        }
        return 1;
    }

    0
}

fn language_get_required_feature(
    table: SubstPosTable,
    script_index: ScriptIndex,
    lang_index: Option<LangIndex>,
) -> Option<(FeatureIndex, Tag)> {
    let script = table.get_script(script_index)?;
    let sys = match lang_index {
        Some(index) => script.get_lang(index)?,
        None => script.default_lang()?,
    };
    let idx = sys.required_feature?;
    let tag = table.get_feature_tag(idx)?;
    Some((idx, tag))
}

/// Fetches the index for a given feature tag in the specified face's GSUB table
/// or GPOS table.
///
/// Returns true if the feature is found, false otherwise.
pub fn rb_ot_layout_table_find_feature(
    face: *const ffi::rb_face_t,
    table_tag: Tag,
    feature_tag: Tag,
    feature_index: *mut u32,
) -> ffi::rb_bool_t {
    unsafe { *feature_index = FEATURE_NOT_FOUND_INDEX };

    if let Some(feature) = Face::from_ptr(face)
        .layout_table(table_tag)
        .and_then(|table| table.find_feature_index(feature_tag))
    {
        unsafe { *feature_index = u32::from(feature.0); }
        return 1;
    }

    0
}

/// Fetches the total number of lookups enumerated in the specified
/// face's GSUB table or GPOS table.
pub fn rb_ot_layout_table_get_lookup_count(
    face: *const ffi::rb_face_t,
    table_tag: Tag,
) -> u32 {
    Face::from_ptr(face)
        .layout_table(table_tag)
        .map_or(0, |table| u32::from(table.lookup_count()))
}

// Variations support

/// Fetches a list of feature variations in the specified face's GSUB table
/// or GPOS table, at the specified variation coordinates.
pub fn rb_ot_layout_table_find_feature_variations(
    face: *const ffi::rb_face_t,
    table_tag: Tag,
    coords: *const i32,
    num_coords: u32,
    variations_index: *mut u32,
) -> ffi::rb_bool_t {
    unsafe { *variations_index = FEATURE_VARIATION_NOT_FOUND_INDEX; }

    let coords = unsafe { slice::from_raw_parts(coords, usize::num_from(num_coords)) };
    if let Some(table) = Face::from_ptr(face).layout_table(table_tag) {
        if let Some(index) = table.find_variation_index(coords) {
            unsafe { *variations_index = index.0; }
            return 1;
        }
    }

    0
}

/// Fetches a list of all lookups enumerated for the specified feature, in
/// the specified face's GSUB table or GPOS table, enabled at the specified
/// variations index. The list returned will begin at the offset provided.
pub fn rb_ot_layout_feature_with_variations_get_lookups(
    face: *const ffi::rb_face_t,
    table_tag: Tag,
    feature_index: u32,
    var_index: u32,
    start: u32,
    lookup_count: *mut u32,
    lookup_indices: *mut u32,
) {
    if let Some(feature) = Face::from_ptr(face)
        .layout_table(table_tag)
        .and_then(|table| {
            let feature_index = FeatureIndex(feature_index as u16);
            match var_index {
                FEATURE_VARIATION_NOT_FOUND_INDEX => table.get_feature(feature_index),
                _ => table.get_variation(feature_index, VariationIndex(var_index)),
            }
        })
    {
        unsafe {
            let mut i = 0;
            for index in feature.lookup_indices.into_iter().skip(usize::num_from(start)) {
                if i == *lookup_count {
                    break;
                }
                *lookup_indices.offset(i as isize) = u32::from(index.0);
                i += 1;
            }
            *lookup_count = i;
        }
    } else {
        unsafe { *lookup_count = 0; }
    }
}

// GSUB

/// Tests whether a specified lookup in the specified face would
/// trigger a substitution on the given glyph sequence.
///
/// Returns true if a substitution would be triggered, false otherwise
pub fn rb_ot_layout_lookup_would_substitute(
    face: *const ffi::rb_face_t,
    lookup_index: u32,
    glyphs: *const u32,
    glyphs_length: u32,
    zero_context: ffi::rb_bool_t,
) -> ffi::rb_bool_t {
    let glyphs = unsafe { slice::from_raw_parts(glyphs, usize::num_from(glyphs_length)) };
    let zero_context = zero_context != 0;
    let ctx = WouldApplyContext { glyphs, zero_context };
    Face::from_ptr(face).gsub
        .and_then(|table| table.get_lookup(LookupIndex(lookup_index as u16)))
        .map_or(false, |lookup| lookup.would_apply(&ctx)) as ffi::rb_bool_t
}

// Substitution and positioning

pub fn apply_layout_table<T: LayoutTable>(
    plan: &ShapePlan,
    face: &Face,
    buffer: &mut Buffer,
    table: Option<T>,
) {
    let mut ctx = ApplyContext::new(T::INDEX, face, buffer);

    for (stage_index, stage) in plan.ot_map.stages(T::INDEX).into_iter().enumerate() {
        for lookup in plan.ot_map.stage_lookups(T::INDEX, stage_index) {
            let lookup_index = LookupIndex(lookup.index as u16);

            ctx.lookup_index = lookup_index;
            ctx.lookup_mask = lookup.mask;
            ctx.auto_zwj = lookup.auto_zwj;
            ctx.auto_zwnj = lookup.auto_zwnj;

            if lookup.random {
                ctx.random = true;
                ctx.buffer.unsafe_to_break(0, ctx.buffer.len);
            }

            if let Some(table) = &table {
                if let Some(lookup) =  table.get_lookup(lookup_index) {
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

fn apply_string<T: LayoutTable>(ctx: &mut ApplyContext, lookup: T::Lookup) {
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

fn apply_forward(ctx: &mut ApplyContext, lookup: impl Apply) -> bool {
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

fn apply_backward(ctx: &mut ApplyContext, lookup: impl Apply) -> bool {
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

// General

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
