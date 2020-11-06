//! OpenType layout.

#![allow(dead_code)]

mod apply;
mod common;
mod context_lookups;
mod dyn_array;
mod gpos;
mod gsub;
mod matching;

use crate::{ffi, Tag};

use common::{SubstPosTable, FeatureIndex, Feature, VariationIndex};

pub const MAX_NESTING_LEVEL: usize = 6;
pub const MAX_CONTEXT_LENGTH: usize = 64;

pub const FEATURE_VARIATION_NOT_FOUND_INDEX: u32 = 0xFFFFFFFF;
pub const FEATURE_NOT_FOUND_INDEX: u32 = 0xFFFF;

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
    tag: Tag,
    coords: *const i32,
    num_coords: u32,
    variations_index: *mut u32,
) -> ffi::rb_bool_t {
    unsafe { *variations_index = FEATURE_VARIATION_NOT_FOUND_INDEX; }

    let data = unsafe { get_table_data(face, tag) };
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
    tag: Tag,
    feature_index: u32,
    var_index: u32,
    start: u32,
    lookup_count: *mut u32,
    lookup_indices: *mut u32,
) {
    let data = unsafe { get_table_data(face, tag) };
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

unsafe fn get_table_data(face: *const ffi::rb_face_t, tag: Tag) -> &'static [u8] {
    let data = ffi::rb_face_get_table_data(face, tag);
    let len = ffi::rb_face_get_table_len(face, tag);
    std::slice::from_raw_parts(data, len as usize)
}
