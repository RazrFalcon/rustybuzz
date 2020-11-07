//! OpenType layout.

mod apply;
mod common;
mod context_lookups;
mod dyn_array;
mod gpos;
mod gsub;
mod kern;
mod matching;

use std::convert::TryFrom;

use ttf_parser::parser::{NumFrom, Offset, Offset16, Stream};
use ttf_parser::GlyphId;

use crate::buffer::{Buffer, GlyphInfo};
use crate::common::TagExt;
use crate::{ffi, Font, Tag};
use apply::WouldApplyContext;
use common::{
    SubstPosTable, FeatureIndex, Feature, LangSys, LangSysIndex, LookupIndex, ScriptIndex,
    VariationIndex,
};
use gpos::PosTable;
use gsub::SubstTable;

pub const MAX_NESTING_LEVEL: usize = 6;
pub const MAX_CONTEXT_LENGTH: usize = 64;

pub const SCRIPT_NOT_FOUND_INDEX: u32 = 0xFFFF;
pub const LANGUAGE_NOT_FOUND_INDEX: u32 = 0xFFFF;
pub const FEATURE_NOT_FOUND_INDEX: u32 = 0xFFFF;
pub const FEATURE_VARIATION_NOT_FOUND_INDEX: u32 = 0xFFFFFFFF;

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
    let data = unsafe { get_table_data(face, Tag::from_bytes(b"GDEF")) };
    (|| {
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
    const LATIN_SCRIPT: Tag = Tag::from_bytes(b"latn");

    unsafe {
        *script_index = SCRIPT_NOT_FOUND_INDEX;
        *chosen_script = Tag(SCRIPT_NOT_FOUND_INDEX);
    }

    let data = unsafe { get_table_data(face, table_tag) };
    let table = match SubstPosTable::parse(data) {
        Some(table) => table,
        None => return 0,
    };

    let tags = unsafe { std::slice::from_raw_parts(script_tags, usize::num_from(script_count)) };
    for &tag in tags {
        if let Some(index) = table.find_script_index(tag) {
            unsafe {
                *script_index = u32::from(index.0);
                *chosen_script = tag;
            }
            return 1;
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
            unsafe {
                *script_index = u32::from(index.0);
                *chosen_script = tag;
            }
            return 0;
        }
    }

    0
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

    let data = unsafe { get_table_data(face, table_tag) };
    let (table, sys) = match get_table_and_lang_sys(data, script_index, language_index) {
        Some(v) => v,
        None => return 0,
    };

    for i in 0..sys.feature_count() {
        if let Some(index) = sys.get_feature_index(i) {
            if table.get_feature_tag(index) == Some(feature_tag) {
                unsafe { *feature_index = u32::from(index.0); }
                return 1;
            }
        }
    }

    0
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

    let data = unsafe { get_table_data(face, table_tag) };
    let script = match SubstPosTable::parse(data)
        .and_then(|table| table.get_script(ScriptIndex(script_index as u16)))
    {
        Some(script) => script,
        None => return 0,
    };

    let languages = unsafe { std::slice::from_raw_parts(language_tags, usize::num_from(language_count)) };
    for &tag in languages {
        if let Some(index) = script.find_lang_sys_index(tag) {
            unsafe { *language_index = u32::from(index.0); }
            return 1;
        }
    }

    /* try finding 'dflt' */
    if let Some(index) = script.find_lang_sys_index(Tag::default_language()) {
        unsafe { *language_index = u32::from(index.0); }
        return 0;
    }

    0
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

    let data = unsafe { get_table_data(face, table_tag) };
    let (table, sys) = match get_table_and_lang_sys(data, script_index, language_index) {
        Some(v) => v,
        None => return 0,
    };

    let index = sys.required_feature_index;
    unsafe {
        *feature_index = u32::from(sys.required_feature_index.0);
        if let Some(tag) = table.get_feature_tag(index) {
            *feature_tag = tag;
        }
    }

    return sys.has_required_feature() as ffi::rb_bool_t;
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

    let data = unsafe { get_table_data(face, table_tag) };
    if let Some(table) = SubstPosTable::parse(data) {
        for i in 0..table.feature_count() {
            if table.get_feature_tag(FeatureIndex(i)) == Some(feature_tag) {
                unsafe { *feature_index = u32::from(i); }
                return 1;
            }
        }
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
    let data = unsafe { get_table_data(face, table_tag) };
    SubstPosTable::parse(data).map_or(0, |table| u32::from(table.lookup_count()))
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

    let data = unsafe { get_table_data(face, table_tag) };
    let coords = unsafe { std::slice::from_raw_parts(coords, usize::num_from(num_coords)) };
    if let Some(table) = SubstPosTable::parse(data) {
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
    let data = unsafe { get_table_data(face, table_tag) };
    let feature = SubstPosTable::parse(data).and_then(|table| {
        let feature_index = FeatureIndex(feature_index as u16);
        if var_index == FEATURE_VARIATION_NOT_FOUND_INDEX {
            table.get_feature(feature_index)
        } else {
            table.get_feature_variation(feature_index, VariationIndex(var_index))
        }
    });

    if let Some(feature) = feature {
        unsafe { write_lookup_indices(feature, start, lookup_count, lookup_indices); }
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
    let data = unsafe { get_table_data(face, SubstTable::TAG) };
    SubstTable::parse(data).is_some() as ffi::rb_bool_t
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
    let data = unsafe { get_table_data(face, SubstTable::TAG) };
    let glyphs = unsafe { std::slice::from_raw_parts(glyphs, usize::num_from(glyphs_length)) };
    let zero_context = zero_context != 0;
    let ctx = WouldApplyContext { glyphs, zero_context };
    SubstTable::parse(data)
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

#[no_mangle]
pub extern "C" fn rb_ot_layout_delete_glyphs_inplace(
    buffer: *mut ffi::rb_buffer_t,
    filter: unsafe extern "C" fn(info: *const GlyphInfo) -> ffi::rb_bool_t,
) {
    let mut buffer = Buffer::from_ptr_mut(buffer);

    // Merge clusters and delete filtered glyphs.
    // NOTE! We can't use out-buffer as we have positioning data.
    let mut j = 0;
    let len = buffer.len;

    for i in 0..len {
        if unsafe { filter(&buffer.info[i]) != 0 } {
            // Merge clusters.
            // Same logic as buffer.delete_glyph(), but for in-place removal

            let cluster = buffer.info[i].cluster;
            if i + 1 < len && cluster == buffer.info[i + 1].cluster {
                // Cluster survives; do nothing.
                continue;
            }

            if j != 0 {
                // Merge cluster backward.
                if cluster < buffer.info[j - 1].cluster {
                    let mask = buffer.info[i].mask;
                    let old_cluster = buffer.info[j - 1].cluster;

                    let mut k = j;
                    while k > 0 && buffer.info[k - 1].cluster == old_cluster {
                        Buffer::set_cluster(&mut buffer.info[k - 1], cluster, mask);
                        k -= 1;
                    }
                }
                continue;
            }

            if i + 1 < len {
                // Merge cluster forward.
                buffer.merge_clusters(i, i + 2);
            }

            continue;
        }

        if j != i {
            buffer.info[j] = buffer.info[i];
            buffer.pos[j] = buffer.pos[i];
        }

        j += 1;
    }

    buffer.len = j;
}

// GPOS

/// rb_ot_layout_has_positioning:
///
/// @face: #rb_face_t to work upon
///
/// Return value: true if the face has GPOS data, false otherwise
#[no_mangle]
pub extern "C" fn rb_ot_layout_has_positioning(face: *const ffi::rb_face_t) -> ffi::rb_bool_t {
    let data = unsafe { get_table_data(face, PosTable::TAG) };
    PosTable::parse(data).is_some() as ffi::rb_bool_t
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

// Helpers

fn get_table_and_lang_sys<'a>(
    data: &'a [u8],
    script_index: u32,
    language_index: u32,
) -> Option<(SubstPosTable<'a>, LangSys<'a>)> {
    let table = SubstPosTable::parse(data)?;
    let script = table.get_script(ScriptIndex(script_index as u16))?;
    let lang_sys = if language_index == LANGUAGE_NOT_FOUND_INDEX {
        script.default_lang_sys()?
    } else {
        script.get_lang_sys(LangSysIndex(language_index as u16))?
    };
    Some((table, lang_sys))
}

unsafe fn write_lookup_indices(
    feature: Feature,
    start: u32,
    count: *mut u32,
    indices: *mut u32,
) {
    let mut i = 0;
    for index in feature.lookup_indices.into_iter().skip(usize::num_from(start)) {
        if i == *count {
            break;
        }
        *indices.offset(i as isize) = u32::from(index.0);
        i += 1;
    }
    *count = i;
}

unsafe fn get_table_data(face: *const ffi::rb_face_t, table_tag: Tag) -> &'static [u8] {
    let mut len = 0;
    let data = ffi::rb_face_get_table_data(face, table_tag, &mut len);
    std::slice::from_raw_parts(data, usize::num_from(len))
}
