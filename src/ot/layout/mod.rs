//! OpenType layout.

#![allow(dead_code)]

mod apply;
mod common;
mod context_lookups;
mod dyn_array;
mod gpos;
mod gsub;
mod matching;

use crate::common::TagExt;
use crate::{ffi, Tag};

use common::{
    SubstPosTable, FeatureIndex, Feature, LangSys, LangSysIndex, ScriptIndex, VariationIndex,
};

pub const MAX_NESTING_LEVEL: usize = 6;
pub const MAX_CONTEXT_LENGTH: usize = 64;

pub const SCRIPT_NOT_FOUND_INDEX: u32 = 0xFFFF;
pub const LANGUAGE_NOT_FOUND_INDEX: u32 = 0xFFFF;
pub const FEATURE_NOT_FOUND_INDEX: u32 = 0xFFFF;
pub const FEATURE_VARIATION_NOT_FOUND_INDEX: u32 = 0xFFFFFFFF;

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

    let tags = unsafe { std::slice::from_raw_parts(script_tags, script_count as usize) };
    for &tag in tags {
        if let Some(index) = table.find_script_index(tag) {
            unsafe {
                *script_index = index.0 as u32;
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
                *script_index = index.0 as u32;
                *chosen_script = tag;
            }
            return 0;
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

    let languages = unsafe { std::slice::from_raw_parts(language_tags, language_count as usize) };
    for &tag in languages {
        if let Some(index) = script.find_lang_sys_index(tag) {
            unsafe { *language_index = index.0 as u32; }
            return 1;
        }
    }

    /* try finding 'dflt' */
    if let Some(index) = script.find_lang_sys_index(Tag::default_language()) {
        unsafe { *language_index = index.0 as u32; }
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
        *feature_index = sys.required_feature_index.0 as u32;
        if let Some(tag) = table.get_feature_tag(index) {
            *feature_tag = tag;
        }
    }

    return sys.has_required_feature() as ffi::rb_bool_t;
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
                unsafe { *feature_index = index.0 as u32; }
                return 1;
            }
        }
    }

    0
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
                unsafe { *feature_index = i as u32; }
                return 1;
            }
        }
    }

    0
}

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
    let coords = unsafe { std::slice::from_raw_parts(coords, num_coords as usize) };
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
    for index in feature.lookup_indices.into_iter().skip(start as usize) {
        if i == *count {
            break;
        }
        *indices.offset(i as isize) = index.0 as u32;
        i += 1;
    }
    *count = i as u32;
}

unsafe fn get_table_data(face: *const ffi::rb_face_t, table_tag: Tag) -> &'static [u8] {
    let data = ffi::rb_face_get_table_data(face, table_tag);
    let len = ffi::rb_face_get_table_len(face, table_tag);
    std::slice::from_raw_parts(data, len as usize)
}
