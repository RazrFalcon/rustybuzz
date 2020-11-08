//! OpenType layout.

mod apply;
mod common;
mod context_lookups;
mod gpos;
mod gsub;
mod kern;
mod matching;

use std::convert::TryFrom;
use std::slice;

use ttf_parser::parser::{NumFrom, Offset, Offset16, Stream};
use ttf_parser::GlyphId;

use super::{Map, ShapePlan, TableIndex};
use crate::buffer::Buffer;
use crate::common::TagExt;
use crate::{ffi, Font, Tag};
use apply::{Apply, ApplyContext, WouldApply, WouldApplyContext};
use common::{FeatureIndex, LangIndex, LookupIndex, ScriptIndex, SubstPosTable, VariationIndex};
use gpos::PosTable;
use gsub::SubstTable;

pub const MAX_NESTING_LEVEL: usize = 6;
pub const MAX_CONTEXT_LENGTH: usize = 64;

pub const SCRIPT_NOT_FOUND_INDEX: u32 = 0xFFFF;
pub const LANGUAGE_NOT_FOUND_INDEX: u32 = 0xFFFF;
pub const FEATURE_NOT_FOUND_INDEX: u32 = 0xFFFF;
pub const FEATURE_VARIATION_NOT_FOUND_INDEX: u32 = 0xFFFFFFFF;

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

// GDEF
// Note: GDEF blocklisting was removed for now because we use
//       ttf_parser's GDEF parsing routines.

/// rb_ot_layout_has_glyph_classes:
/// @face: #rb_face_t to work upon
///
/// Tests whether a face has any glyph classes defined in its GDEF table.
///
/// Return value: true if data found, false otherwise
#[no_mangle]
pub extern "C" fn rb_ot_layout_has_glyph_classes(face: *const ffi::rb_face_t) -> ffi::rb_bool_t {
    // TODO: Find out through ttfp_face when that's reachable.
    (|| {
        let data = table_data(face, Tag::from_bytes(b"GDEF"));
        let mut s = Stream::new(data);
        let major_version = s.read::<u16>()?;
        if major_version != 1 {
            return None;
        }
        s.skip::<u16>();
        let glyph_class_offset = s.read::<Offset16>()?;
        Some(!glyph_class_offset.is_null())
    })().unwrap_or(false) as ffi::rb_bool_t
}

// GSUB/GPOS

/// rb_ot_layout_table_select_script:
///
/// @face: #rb_face_t to work upon
/// @table_tag: RB_OT_TAG_GSUB or RB_OT_TAG_GPOS
/// @script_count: Number of script tags in the array
/// @script_tags: Array of #rb_tag_t script tags
/// @script_index: (out): The index of the requested script
/// @chosen_script: (out): #rb_tag_t of the requested script
///
/// Since: 2.0.0
#[no_mangle]
pub extern "C" fn rb_ot_layout_table_select_script(
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
    if let Some((found, index, tag)) = SubstPosTable::parse(table_data(face, table_tag))
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
    const LATIN_SCRIPT: Tag = Tag::from_bytes(b"latn");

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
        LATIN_SCRIPT,
    ] {
        if let Some(index) = table.find_script_index(tag) {
            return Some((false, index, tag));
        }
    }

    None
}

/// rb_ot_layout_language_find_feature:
///
/// @face: #rb_face_t to work upon
/// @table_tag: RB_OT_TAG_GSUB or RB_OT_TAG_GPOS
/// @script_index: The index of the requested script tag
/// @language_index: The index of the requested language tag
/// @feature_tag: #rb_tag_t of the feature tag requested
/// @feature_index: (out): The index of the requested feature
///
/// Fetches the index of a given feature tag in the specified face's GSUB table
/// or GPOS table, underneath the specified script and language.
///
/// Return value: true if the feature is found, false otherwise
#[no_mangle]
pub extern "C" fn rb_ot_layout_language_find_feature(
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

    if let Some(index) = SubstPosTable::parse(table_data(face, table_tag))
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

/// rb_ot_layout_script_select_language:
///
/// @face: #rb_face_t to work upon
/// @table_tag: RB_OT_TAG_GSUB or RB_OT_TAG_GPOS
/// @script_index: The index of the requested script tag
/// @language_count: The number of languages in the specified script
/// @language_tags: The array of language tags
/// @language_index: (out): The index of the requested language
///
/// Fetches the index of a given language tag in the specified face's GSUB table
/// or GPOS table, underneath the specified script index.
///
/// Return value: true if the language tag is found, false otherwise
///
/// Since: 2.0.0
#[no_mangle]
pub extern "C" fn rb_ot_layout_script_select_language(
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
    if let Some((found, index)) = SubstPosTable::parse(table_data(face, table_tag))
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

/// rb_ot_layout_language_get_required_feature:
///
/// @face: #rb_face_t to work upon
/// @table_tag: RB_OT_TAG_GSUB or RB_OT_TAG_GPOS
/// @script_index: The index of the requested script tag
/// @language_index: The index of the requested language tag
/// @feature_index: (out): The index of the requested feature
/// @feature_tag: (out): The #rb_tag_t of the requested feature
///
/// Fetches the tag of a requested feature index in the given face's GSUB or GPOS table,
/// underneath the specified script and language.
///
/// Return value: true if the feature is found, false otherwise
///
/// Since: 0.9.30
#[no_mangle]
pub extern "C" fn rb_ot_layout_language_get_required_feature(
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

    if let Some((index, tag)) = SubstPosTable::parse(table_data(face, table_tag))
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

/// rb_ot_layout_table_find_feature:
///
/// @face: #rb_face_t to work upon
/// @table_tag: RB_OT_TAG_GSUB or RB_OT_TAG_GPOS
/// @feature_tag: The #rb_tag_t og the requested feature tag
/// @feature_index: (out): The index of the requested feature
///
/// Fetches the index for a given feature tag in the specified face's GSUB table
/// or GPOS table.
///
/// Return value: true if the feature is found, false otherwise
#[no_mangle]
pub extern "C" fn rb_ot_layout_table_find_feature(
    face: *const ffi::rb_face_t,
    table_tag: Tag,
    feature_tag: Tag,
    feature_index: *mut u32,
) -> ffi::rb_bool_t {
    unsafe { *feature_index = FEATURE_NOT_FOUND_INDEX };

    if let Some(feature) = SubstPosTable::parse(table_data(face, table_tag))
        .and_then(|table| table.find_feature_index(feature_tag))
    {
        unsafe { *feature_index = u32::from(feature.0); }
        return 1;
    }

    0
}

/// rb_ot_layout_table_get_lookup_count:
///
/// @face: #rb_face_t to work upon
/// @table_tag: RB_OT_TAG_GSUB or RB_OT_TAG_GPOS
///
/// Fetches the total number of lookups enumerated in the specified
/// face's GSUB table or GPOS table.
///
/// Since: 0.9.22
#[no_mangle]
pub extern "C" fn rb_ot_layout_table_get_lookup_count(
    face: *const ffi::rb_face_t,
    table_tag: Tag,
) -> u32 {
    SubstPosTable::parse(table_data(face, table_tag))
        .map_or(0, |table| u32::from(table.lookup_count()))
}

// Variations support

/// rb_ot_layout_table_find_feature_variations:
///
/// @face: #rb_face_t to work upon
/// @table_tag: RB_OT_TAG_GSUB or RB_OT_TAG_GPOS
/// @coords: The variation coordinates to query
/// @num_coords: The number of variation coorinates
/// @variations_index: (out): The array of feature variations found for the query
///
/// Fetches a list of feature variations in the specified face's GSUB table
/// or GPOS table, at the specified variation coordinates.
#[no_mangle]
pub extern "C" fn rb_ot_layout_table_find_feature_variations(
    face: *const ffi::rb_face_t,
    table_tag: Tag,
    coords: *const i32,
    num_coords: u32,
    variations_index: *mut u32,
) -> ffi::rb_bool_t {
    unsafe { *variations_index = FEATURE_VARIATION_NOT_FOUND_INDEX; }

    let coords = unsafe { slice::from_raw_parts(coords, usize::num_from(num_coords)) };
    if let Some(table) = SubstPosTable::parse(table_data(face, table_tag)) {
        if let Some(index) = table.find_variation_index(coords) {
            unsafe { *variations_index = index.0; }
            return 1;
        }
    }

    0
}

/// rb_ot_layout_feature_with_variations_get_lookups:
///
/// @face: #rb_face_t to work upon
/// @table_tag: RB_OT_TAG_GSUB or RB_OT_TAG_GPOS
/// @feature_index: The index of the feature to query
/// @var_index: The index of the feature variation to query
/// @start: offset of the first lookup to retrieve
/// @lookup_count: (inout) (allow-none): Input = the maximum number of lookups to return;
///                Output = the actual number of lookups returned (may be zero)
/// @lookup_indices: (out) (array length=lookup_count): The array of lookups found for the query
///
/// Fetches a list of all lookups enumerated for the specified feature, in
/// the specified face's GSUB table or GPOS table, enabled at the specified
/// variations index. The list returned will begin at the offset provided.
#[no_mangle]
pub extern "C" fn rb_ot_layout_feature_with_variations_get_lookups(
    face: *const ffi::rb_face_t,
    table_tag: Tag,
    feature_index: u32,
    var_index: u32,
    start: u32,
    lookup_count: *mut u32,
    lookup_indices: *mut u32,
) {
    if let Some(feature) = SubstPosTable::parse(table_data(face, table_tag))
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

/// rb_ot_layout_has_substitution:
///
/// @face: #rb_face_t to work upon
///
/// Tests whether the specified face includes any GSUB substitutions.
///
/// Return value: true if data found, false otherwise
#[no_mangle]
pub extern "C" fn rb_ot_layout_has_substitution(face: *const ffi::rb_face_t) -> ffi::rb_bool_t {
    SubstTable::parse(table_data(face, SubstTable::TAG)).is_some() as ffi::rb_bool_t
}

/// rb_ot_layout_lookup_would_substitute:
///
/// @face: #rb_face_t to work upon
/// @lookup_index: The index of the lookup to query
/// @glyphs: The sequence of glyphs to query for substitution
/// @glyphs_length: The length of the glyph sequence
/// @zero_context: #rb_bool_t indicating whether substitutions should be context-free
///
/// Tests whether a specified lookup in the specified face would
/// trigger a substitution on the given glyph sequence.
///
/// Return value: true if a substitution would be triggered, false otherwise
///
/// Since: 0.9.7
#[no_mangle]
pub extern "C" fn rb_ot_layout_lookup_would_substitute(
    face: *const ffi::rb_face_t,
    lookup_index: u32,
    glyphs: *const u32,
    glyphs_length: u32,
    zero_context: ffi::rb_bool_t,
) -> ffi::rb_bool_t {
    let glyphs = unsafe { slice::from_raw_parts(glyphs, usize::num_from(glyphs_length)) };
    let zero_context = zero_context != 0;
    let ctx = WouldApplyContext { glyphs, zero_context };
    SubstTable::parse(table_data(face, SubstTable::TAG))
        .and_then(|table| table.get_lookup(LookupIndex(lookup_index as u16)))
        .map_or(false, |lookup| lookup.would_apply(&ctx)) as ffi::rb_bool_t
}

/// rb_ot_layout_substitute_start:
///
/// @font: #rb_font_t to use
/// @buffer: #rb_buffer_t buffer to work upon
///
/// Called before substitution lookups are performed, to ensure that glyph
/// class and other properties are set on the glyphs in the buffer.
#[no_mangle]
pub extern "C" fn rb_ot_layout_substitute_start(
    font: *const ffi::rb_font_t,
    buffer: *mut ffi::rb_buffer_t,
) {
    let font = Font::from_ptr(font);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    set_glyph_props(font, &mut buffer);
}

fn set_glyph_props(font: &Font, buffer: &mut Buffer) {
    let len = buffer.len;
    for info in &mut buffer.info[..len] {
        let glyph = GlyphId(u16::try_from(info.codepoint).unwrap());
        info.set_glyph_props(font.glyph_props(glyph));
        info.set_lig_props(0);
        info.set_syllable(0);
    }
}

// GPOS

/// rb_ot_layout_has_positioning:
///
/// @face: #rb_face_t to work upon
///
/// Return value: true if the face has GPOS data, false otherwise
#[no_mangle]
pub extern "C" fn rb_ot_layout_has_positioning(face: *const ffi::rb_face_t) -> ffi::rb_bool_t {
    PosTable::parse(table_data(face, PosTable::TAG)).is_some() as ffi::rb_bool_t
}

/// rb_ot_layout_position_start:
///
/// @font: #rb_font_t to use
/// @buffer: #rb_buffer_t buffer to work upon
///
/// Called before positioning lookups are performed, to ensure that glyph
/// attachment types and glyph-attachment chains are set for the glyphs in the buffer.
#[no_mangle]
pub extern "C" fn rb_ot_layout_position_start(
    font: *const ffi::rb_font_t,
    buffer: *mut ffi::rb_buffer_t,
) {
    let font = Font::from_ptr(font);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    PosTable::position_start(font, &mut buffer);
}

/// rb_ot_layout_position_finish_advances:
///
/// @font: #rb_font_t to use
/// @buffer: #rb_buffer_t buffer to work upon
///
/// Called after positioning lookups are performed, to finish glyph advances.
#[no_mangle]
pub extern "C" fn rb_ot_layout_position_finish_advances(
    font: *const ffi::rb_font_t,
    buffer: *mut ffi::rb_buffer_t,
) {
    let font = Font::from_ptr(font);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    PosTable::position_finish_advances(font, &mut buffer);
}

/// rb_ot_layout_position_finish_offsets:
///
/// @font: #rb_font_t to use
/// @buffer: #rb_buffer_t buffer to work upon
///
/// Called after positioning lookups are performed, to finish glyph offsets.
#[no_mangle]
pub extern "C" fn rb_ot_layout_position_finish_offsets(
    font: *const ffi::rb_font_t,
    buffer: *mut ffi::rb_buffer_t,
) {
    let font = Font::from_ptr(font);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    PosTable::position_finish_offsets(font, &mut buffer);
}

// General

#[no_mangle]
pub extern "C" fn rb_ot_layout_substitute(
    map: *const ffi::rb_ot_map_t,
    plan: *const ffi::rb_ot_shape_plan_t,
    font: *mut ffi::rb_font_t,
    buffer: *mut ffi::rb_buffer_t,
) {
    let map = Map::from_ptr(map);
    let plan = ShapePlan::from_ptr(plan);
    let font = Font::from_ptr(font);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    let table = SubstTable::parse(table_data(font.face_ptr(), SubstTable::TAG));
    apply(&map, &plan, font, &mut buffer, table);
}

#[no_mangle]
pub extern "C" fn rb_ot_layout_position(
    map: *const ffi::rb_ot_map_t,
    plan: *const ffi::rb_ot_shape_plan_t,
    font: *mut ffi::rb_font_t,
    buffer: *mut ffi::rb_buffer_t,
) {
    let map = Map::from_ptr(map);
    let plan = ShapePlan::from_ptr(plan);
    let font = Font::from_ptr(font);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    let table = PosTable::parse(table_data(font.face_ptr(), PosTable::TAG));
    apply(&map, &plan, font, &mut buffer, table);
}

fn apply<T: LayoutTable>(
    map: &Map,
    plan: &ShapePlan,
    font: &Font,
    buffer: &mut Buffer,
    table: Option<T>,
) {
    let mut ctx = ApplyContext::new(T::INDEX, font, buffer);

    for (stage_index, stage) in map.collect_stages(T::INDEX).into_iter().enumerate() {
        for lookup in map.collect_stage_lookups(T::INDEX, stage_index) {
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
            unsafe { func(plan.as_ptr(), font.as_ptr() as *mut _, ctx.buffer.as_ptr()); }
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

fn table_data(face: *const ffi::rb_face_t, table_tag: Tag) -> &'static [u8] {
    unsafe {
        let mut len = 0;
        let data = ffi::rb_face_get_table_data(face, table_tag, &mut len);
        slice::from_raw_parts(data, usize::num_from(len))
    }
}
