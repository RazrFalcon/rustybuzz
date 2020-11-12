use std::os::raw::c_char;

use crate::buffer::glyph_flag;
use crate::{ffi, Face, Mask, Tag, Script};
use super::layout::TableIndex;

const TABLE_TAGS: [Tag; 2] = [Tag::from_bytes(b"GSUB"), Tag::from_bytes(b"GPOS")];

pub struct Map {
    pub chosen_script: [Tag; 2],
    pub found_script: [bool; 2],
    global_mask: Mask,
    features: Vec<FeatureMap>,
    lookups: [Vec<LookupMap>; 2],
    stages: [Vec<StageMap>; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FeatureMap {
    tag: Tag,
    // GSUB/GPOS
    index: [u32; 2],
    stage: [u32; 2],
    shift: u32,
    mask: Mask,
    // mask for value=1, for quick access
    _1_mask: Mask,
    auto_zwnj: bool,
    auto_zwj: bool,
    random: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LookupMap {
    pub index: u16,
    // TODO: to bitflags
    pub auto_zwnj: bool,
    pub auto_zwj: bool,
    pub random: bool,
    pub mask: Mask,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct StageMap {
    // Cumulative
    pub last_lookup: u32,
    pub pause_func: ffi::rb_ot_pause_func_t,
}

impl Map {
    pub const MAX_BITS: u32 = 8;
    pub const MAX_VALUE: u32 = (1 << Self::MAX_BITS) - 1;

    fn new() -> Self {
        Self {
            chosen_script: [Tag(0); 2],
            found_script: [false; 2],
            global_mask: 0,
            features: vec![],
            lookups: [vec![], vec![]],
            stages: [vec![], vec![]],
        }
    }

    #[inline]
    pub fn from_ptr(map: *const ffi::rb_ot_map_t) -> &'static Map {
        unsafe { &*(map as *const Map) }
    }

    #[inline]
    pub fn from_ptr_mut(map: *mut ffi::rb_ot_map_t) -> &'static mut Map {
        unsafe { &mut *(map as *mut Map) }
    }

    #[inline]
    pub fn global_mask(&self) -> Mask {
        self.global_mask
    }

    #[inline]
    pub fn get_mask(&self, feature_tag: Tag) -> (Mask, u32) {
        self.features
            .binary_search_by_key(&feature_tag, |f| f.tag)
            .map_or((0, 0), |idx| (self.features[idx].mask, self.features[idx].shift))
    }

    #[inline]
    pub fn get_1_mask(&self, feature_tag: Tag) -> Mask {
        self.features
            .binary_search_by_key(&feature_tag, |f| f.tag)
            .map_or(0, |idx| self.features[idx]._1_mask)
    }

    #[inline]
    pub fn get_feature_index(&self, table_index: TableIndex, feature_tag: Tag) -> u32 {
        use crate::ot::layout::*;
        self.features
            .binary_search_by_key(&feature_tag, |f| f.tag)
            .map_or(FEATURE_NOT_FOUND_INDEX, |idx| self.features[idx].index[table_index as usize])
    }

    #[inline]
    pub fn get_feature_stage(&self, table_index: TableIndex, feature_tag: Tag) -> usize {
        self.features
            .binary_search_by_key(&feature_tag, |f| f.tag)
            .map_or(usize::MAX, |idx| self.features[idx].stage[table_index as usize] as usize)
    }

    #[inline]
    pub fn get_stages(&self, table_index: TableIndex) -> &[StageMap] {
        &self.stages[table_index as usize]
    }

    #[inline]
    pub fn get_stage_lookups(&self, table_index: TableIndex, stage: usize) -> &[LookupMap] {
        if stage == usize::MAX {
            return &[];
        }

        // TODO
        // let stages = self.stages[table_index as usize];
        // let start = stage.checked_sub(1).map_or(0, |prev| stages[prev]);

        let start = if stage != 0 {
            self.stages[table_index as usize][stage - 1].last_lookup as usize
        } else {
            0
        };

        let end = if stage < self.stages[table_index as usize].len() {
            self.stages[table_index as usize][stage].last_lookup as usize
        } else {
            self.lookups[table_index as usize].len()
        };

        &self.lookups[table_index as usize][start..end]
    }
}

bitflags::bitflags! {
    /// Flags used for serialization with a `BufferSerializer`.
    #[derive(Default)]
    pub struct FeatureFlags: u32 {
        const NONE = 0x00;

        /// Feature applies to all characters; results in no mask allocated for it.
        const GLOBAL = 0x01;

        /// Has fallback implementation, so include mask bit even if feature not found.
        const HAS_FALLBACK = 0x02;

        /// Don't skip over ZWNJ when matching **context**.
        const MANUAL_ZWNJ = 0x04;

        /// Don't skip over ZWJ when matching **input**.
        const MANUAL_ZWJ = 0x08;

        const MANUAL_JOINERS        = Self::MANUAL_ZWNJ.bits | Self::MANUAL_ZWJ.bits;
        const GLOBAL_MANUAL_JOINERS = Self::GLOBAL.bits | Self::MANUAL_JOINERS.bits;

        /// If feature not found in LangSys, look for it in global feature list and pick one.
        const GLOBAL_SEARCH = 0x10;

        /// Randomly select a glyph from an AlternateSubstFormat1 subtable.
        const RANDOM = 0x20;
    }
}

pub struct MapBuilder {
    pub face: &'static Face<'static>,
    pub chosen_script: [Tag; 2],
    pub found_script: [bool; 2],
    pub script_index: [u32; 2],
    pub language_index: [u32; 2],
    current_stage: [u32; 2],
    feature_infos: Vec<FeatureInfo>,
    stages: [Vec<StageInfo>; 2],
}

#[allow(dead_code)]
pub struct MapFeature {
    pub tag: Tag,
    pub flags: FeatureFlags,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct FeatureInfo {
    tag: Tag,
    // sequence#, used for stable sorting only
    seq: u32,
    max_value: u32,
    flags: FeatureFlags,
    // for non-global features, what should the unset glyphs take
    default_value: u32,
    // GSUB/GPOS
    stage: [u32; 2],
}

struct StageInfo {
    index: u32,
    pause_func: ffi::rb_ot_pause_func_t,
}

impl MapBuilder {
    fn new(face: &'static Face<'static>, script: Script, language: *const c_char) -> Self {
        use crate::ot::layout::*;

        // Fetch script/language indices for GSUB/GPOS.  We need these later to skip
        // features not available in either table and not waste precious bits for them.

        let mut script_count = MAX_TAGS_PER_SCRIPT as u32;
        let mut language_count = MAX_TAGS_PER_LANGUAGE as u32;
        let mut script_tags = [Tag(0); MAX_TAGS_PER_SCRIPT];
        let mut language_tags = [Tag(0); MAX_TAGS_PER_LANGUAGE];

        crate::tag::rb_ot_tags_from_script_and_language(
            script.0.0,
            language,
            (&mut script_count) as *mut _,
            script_tags.as_mut_ptr(),
            (&mut language_count) as *mut _,
            language_tags.as_mut_ptr(),
        );

        let mut chosen_script = [Tag(0); 2];
        let mut found_script = [false; 2];
        let mut script_index = [0u32; 2];
        let mut language_index = [0u32; 2];

        for table_index in 0..2 {
            let table_tag = TABLE_TAGS[table_index];

            found_script[table_index] = 0 != rb_ot_layout_table_select_script(
                face.as_ptr(),
                table_tag,
                script_count,
                script_tags.as_ptr(),
                (&mut script_index[table_index]) as *mut _,
                (&mut chosen_script[table_index]) as *mut _,
            );

            rb_ot_layout_script_select_language(
                face.as_ptr(),
                table_tag,
                script_index[table_index],
                language_count,
                language_tags.as_ptr(),
                (&mut language_index[table_index]) as *mut _,
            );
        }

        Self {
            face,
            chosen_script,
            found_script,
            script_index,
            language_index,
            current_stage: [0, 0],
            feature_infos: vec![],
            stages: [vec![], vec![]],
        }
    }

    #[inline]
    pub fn from_ptr_mut(builder: *mut ffi::rb_ot_map_builder_t) -> &'static mut MapBuilder {
        unsafe { &mut *(builder as *mut MapBuilder) }
    }

    pub fn add_feature(&mut self, tag: Tag, flags: FeatureFlags, value: u32) {
        // TODO: Flip into if.
        if tag == Tag(0) {
            return;
        }

        let seq = self.feature_infos.len() as u32 + 1;
        self.feature_infos.push(FeatureInfo {
            tag,
            // TODO: make usize
            seq,
            max_value: value,
            flags,
            default_value: if flags.contains(FeatureFlags::GLOBAL) { value } else { 0 },
            stage: self.current_stage,
        });
    }

    #[inline]
    pub fn enable_feature(&mut self, tag: Tag, flags: FeatureFlags, value: u32) {
        self.add_feature(tag, flags | FeatureFlags::GLOBAL, value);
    }

    #[inline]
    pub fn disable_feature(&mut self, tag: Tag) {
        self.add_feature(tag, FeatureFlags::GLOBAL, 0);
    }

    #[inline]
    pub fn add_gsub_pause(&mut self, pause: ffi::rb_ot_pause_func_t) {
        self.add_pause(TableIndex::GSUB, pause);
    }

    #[inline]
    pub fn add_gpos_pause(&mut self, pause: ffi::rb_ot_pause_func_t) {
        self.add_pause(TableIndex::GPOS, pause);
    }

    // TODO: clean up
    fn add_pause(&mut self, table_index: TableIndex, pause: ffi::rb_ot_pause_func_t) {
        self.stages[table_index as usize].push(StageInfo {
            index: self.current_stage[table_index as usize],
            pause_func: pause,
        });

        self.current_stage[table_index as usize] += 1;
    }

    // TODO: clean up
    pub fn compile(&mut self, map: &mut Map, variation_index: &[u32]) {
        use crate::ot::layout::*;

        let global_bit_mask = glyph_flag::DEFINED + 1;
        let global_bit_shift = glyph_flag::DEFINED.count_ones();

        map.global_mask = global_bit_mask;

        // We default to applying required feature in stage 0.  If the required
        // feature has a tag that is known to the shaper, we apply required feature
        // in the stage for that tag.
        let mut required_feature_stage = [0u32; 2];
        let mut required_feature_index = [0u32; 2];
        let mut required_feature_tag = [Tag(0); 2];

        for table_index in 0..2 {
            map.chosen_script[table_index] = self.chosen_script[table_index];
            map.found_script[table_index] = self.found_script[table_index];

            rb_ot_layout_language_get_required_feature(
                self.face.as_ptr(),
                TABLE_TAGS[table_index],
                self.script_index[table_index],
                self.language_index[table_index],
                (&mut required_feature_index[table_index]) as *mut _,
                (&mut required_feature_tag[table_index]) as *mut _,
            );
        }

        // Sort features and merge duplicates.
        if !self.feature_infos.is_empty() {
            let feature_infos = &mut self.feature_infos;
            feature_infos.sort();

            let mut j = 0;
            for i in 1..feature_infos.len() {
                if feature_infos[i].tag != feature_infos[j].tag {
                    j += 1;
                    feature_infos[j] = feature_infos[i];
                } else {
                    if feature_infos[i].flags.contains(FeatureFlags::GLOBAL) {
                        feature_infos[j].flags |= FeatureFlags::GLOBAL;
                        feature_infos[j].max_value = feature_infos[i].max_value;
                        feature_infos[j].default_value = feature_infos[i].default_value;
                    } else {
                        if feature_infos[j].flags.contains(FeatureFlags::GLOBAL) {
                            feature_infos[j].flags ^= FeatureFlags::GLOBAL;
                        }
                        feature_infos[j].max_value = feature_infos[j].max_value.max(feature_infos[i].max_value);
                        // Inherit default_value from j
                    }
                    let f = feature_infos[i].flags & FeatureFlags::HAS_FALLBACK;
                    feature_infos[j].flags |= f;
                    feature_infos[j].stage[0] = feature_infos[j].stage[0].min(feature_infos[i].stage[0]);
                    feature_infos[j].stage[1] = feature_infos[j].stage[1].min(feature_infos[i].stage[1]);
                }
            }

            feature_infos.truncate(j + 1);
        }

        // Allocate bits now.
        let mut next_bit = global_bit_shift + 1;

        for i in 0..self.feature_infos.len() {
            let info = &self.feature_infos[i];

            let bits_needed;
            if info.flags.contains(FeatureFlags::GLOBAL) && info.max_value == 1 {
                // Uses the global bit.
                bits_needed = 0;
            } else {
                // Limit bits per feature.
                let bit_storage = |v: u32| 8 * std::mem::size_of_val(&v) as u32 - v.leading_zeros();
                bits_needed = Map::MAX_BITS.min(bit_storage(info.max_value));
            }

            if info.max_value == 0 || next_bit + bits_needed > 8 * std::mem::size_of::<Mask>() as u32 {
                 // Feature disabled, or not enough bits.
                continue;
            }

            let mut found = false;
            let mut feature_index = [0u32; 2];
            for table_index in 0..2 {
                if required_feature_tag[table_index] == info.tag {
                    required_feature_stage[table_index] = info.stage[table_index];
                }

                found |= 0 != rb_ot_layout_language_find_feature(
                    self.face.as_ptr(),
                    TABLE_TAGS[table_index],
                    self.script_index[table_index],
                    self.language_index[table_index],
                    info.tag,
                    &mut feature_index[table_index],
                );
            }

            if !found && info.flags.contains(FeatureFlags::GLOBAL_SEARCH) {
                for table_index in 0..2 {
                    found |= 0 != rb_ot_layout_table_find_feature(
                        self.face.as_ptr(),
                        TABLE_TAGS[table_index],
                        info.tag,
                        &mut feature_index[table_index],
                    );
                }
            }

            if !found && !info.flags.contains(FeatureFlags::HAS_FALLBACK) {
                continue;
            }

            let (shift, mask) = if info.flags.contains(FeatureFlags::GLOBAL) && info.max_value == 1 {
                // Uses the global bit
                (global_bit_shift, global_bit_mask)
            } else {
                let shift = next_bit;
                let mask = (1 << (next_bit + bits_needed)) - (1 << next_bit);
                next_bit += bits_needed;
                map.global_mask |= (info.default_value << shift) & mask;
                (shift, mask)
            };

            // TODO: order
            map.features.push(FeatureMap {
                tag: info.tag,
                index: feature_index,
                stage: info.stage,
                auto_zwnj: !info.flags.contains(FeatureFlags::MANUAL_ZWNJ),
                auto_zwj: !info.flags.contains(FeatureFlags::MANUAL_ZWJ),
                random: info.flags.contains(FeatureFlags::RANDOM),
                shift,
                mask,
                _1_mask: (1 << shift) & mask,
            });
        }

        // Done with these.
        self.feature_infos.clear();

        self.add_gsub_pause(None);
        self.add_gpos_pause(None);

        for table_index in 0..2 {
            // Collect lookup indices for features.

            let mut stage_index = 0;
            let mut last_num_lookups = 0;

            for stage in 0..self.current_stage[table_index] {
                if required_feature_index[table_index] != FEATURE_NOT_FOUND_INDEX
                    && required_feature_stage[table_index] == stage
                {
                    self.add_lookups(
                        map,
                        table_index,
                        required_feature_index[table_index],
                        variation_index[table_index],
                        global_bit_mask,
                        true,
                        true,
                        false,
                    );
                }

                for i in 0..map.features.len() {
                    if map.features[i].stage[table_index] == stage {
                        self.add_lookups(
                            map,
                            table_index,
                            map.features[i].index[table_index],
                            variation_index[table_index],
                            map.features[i].mask,
                            map.features[i].auto_zwnj,
                            map.features[i].auto_zwj,
                            map.features[i].random,
                        );
                    }
                }

                // Sort lookups and merge duplicates.
                if last_num_lookups < map.lookups[table_index].len() {
                    let len = map.lookups[table_index].len();
                    map.lookups[table_index][last_num_lookups..len].sort();

                    let mut j = last_num_lookups;
                    for i in j+1..map.lookups[table_index].len() {
                        if map.lookups[table_index][i].index != map.lookups[table_index][j].index {
                            j += 1;
                            map.lookups[table_index][j] = map.lookups[table_index][i];
                        } else {
                            map.lookups[table_index][j].mask |= map.lookups[table_index][i].mask;
                            map.lookups[table_index][j].auto_zwnj &= map.lookups[table_index][i].auto_zwnj;
                            map.lookups[table_index][j].auto_zwj &= map.lookups[table_index][i].auto_zwj;
                        }
                    }

                    map.lookups[table_index].truncate(j + 1);
                }

                last_num_lookups = map.lookups[table_index].len();

                if stage_index < self.stages[table_index].len()
                    && self.stages[table_index][stage_index].index == stage
                {
                    map.stages[table_index].push(StageMap {
                        last_lookup: last_num_lookups as u32,
                        pause_func: self.stages[table_index][stage_index].pause_func,
                    });

                    stage_index += 1;
                }
            }
        }
    }

    // TODO: clean up
    fn add_lookups(
        &self,
        map: &mut Map,
        table_index: usize, // TODO: enum
        feature_index: u32,
        variations_index: u32,
        mask: Mask,
        auto_zwnj: bool,
        auto_zwj: bool,
        random: bool,
    ) {
        use crate::ot::layout::*;

        let mut lookup_indices = [0u32; 32];
        let mut offset = 0;
        let mut len;

        let table_lookup_count = rb_ot_layout_table_get_lookup_count(
            self.face.as_ptr(),
            TABLE_TAGS[table_index],
        );

        loop {
            len = lookup_indices.len() as u32;
            rb_ot_layout_feature_with_variations_get_lookups(
                self.face.as_ptr(),
                TABLE_TAGS[table_index],
                feature_index,
                variations_index,
                offset,
                (&mut len) as *mut _,
                lookup_indices.as_mut_ptr(),
            );

            for i in 0..len {
                if lookup_indices[i as usize] >= table_lookup_count {
                    continue;
                }

                map.lookups[table_index].push(LookupMap {
                    mask,
                    index: lookup_indices[i as usize] as u16,
                    auto_zwnj,
                    auto_zwj,
                    random,
                });
            }

            offset += len;

            if len != lookup_indices.len() as u32 {
                break;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_map_create() -> *mut ffi::rb_ot_map_t {
    Box::into_raw(Box::new(Map::new())) as _
}

#[no_mangle]
pub extern "C" fn rb_ot_map_destroy(map: *mut ffi::rb_ot_map_t) {
    unsafe { Box::from_raw(map as *mut Map); }
}

#[no_mangle]
pub extern "C" fn rb_ot_map_get_global_mask(map: *mut ffi::rb_ot_map_t) -> ffi::rb_mask_t {
    Map::from_ptr(map).global_mask()
}

#[no_mangle]
pub extern "C" fn rb_ot_map_get_mask(
    map: *mut ffi::rb_ot_map_t,
    feature_tag: Tag,
    pshift: *mut u32,
) -> ffi::rb_mask_t {
    let (mask, shift) = Map::from_ptr(map).get_mask(feature_tag);
    unsafe { *pshift = shift; }
    mask
}

#[no_mangle]
pub extern "C" fn rb_ot_map_get_1_mask(
    map: *mut ffi::rb_ot_map_t,
    feature_tag: Tag,
) -> ffi::rb_mask_t {
    Map::from_ptr(map).get_1_mask(feature_tag)
}

#[no_mangle]
pub extern "C" fn rb_ot_map_get_feature_index(
    map: *mut ffi::rb_ot_map_t,
    table_index: u32,
    feature_tag: Tag,
) -> u32 {
    let table_index = unsafe { std::mem::transmute(table_index as u8) };
    Map::from_ptr(map).get_feature_index(table_index, feature_tag)
}

#[no_mangle]
pub extern "C" fn rb_ot_map_get_chosen_script(
    map: *mut ffi::rb_ot_map_t,
    table_index: u32,
) -> Tag {
    Map::from_ptr(map).chosen_script[table_index as usize]
}

#[no_mangle]
pub extern "C" fn rb_ot_map_builder_create(
    face: *const ffi::rb_face_t,
    props: *const ffi::rb_segment_properties_t,
) -> *mut ffi::rb_ot_map_builder_t {
    let face = Face::from_ptr(face);
    let builder = unsafe {
        let script = Script::from_raw((*props).script);
        let language = (*props).language;
        MapBuilder::new(face, script, language)
    };
    Box::into_raw(Box::new(builder)) as _
}

#[no_mangle]
pub extern "C" fn rb_ot_map_builder_destroy(builder: *mut ffi::rb_ot_map_builder_t) {
    unsafe { Box::from_raw(builder as *mut MapBuilder); }
}

#[no_mangle]
pub extern "C" fn rb_ot_map_builder_compile(
    builder: *mut ffi::rb_ot_map_builder_t,
    map: *mut ffi::rb_ot_map_t,
    variation_index: *const u32,
) {
    let builder = MapBuilder::from_ptr_mut(builder);
    let mut map = Map::from_ptr_mut(map);
    let variation_index = unsafe { std::slice::from_raw_parts(variation_index, 2) };
    builder.compile(&mut map, variation_index)
}

#[no_mangle]
pub extern "C" fn rb_ot_map_builder_add_feature(
    builder: *mut ffi::rb_ot_map_builder_t,
    tag: Tag,
    flags: ffi::rb_ot_map_feature_flags_t,
    value: u32,
) {
    let builder = MapBuilder::from_ptr_mut(builder);
    let flags = unsafe { FeatureFlags::from_bits_unchecked(flags) };
    builder.add_feature(tag, flags, value);
}

#[no_mangle]
pub extern "C" fn rb_ot_map_builder_enable_feature(
    builder: *mut ffi::rb_ot_map_builder_t,
    tag: Tag,
    flags: ffi::rb_ot_map_feature_flags_t,
    value: u32,
) {
    let builder = MapBuilder::from_ptr_mut(builder);
    let flags = unsafe { FeatureFlags::from_bits_unchecked(flags) };
    builder.enable_feature(tag, flags, value);
}

#[no_mangle]
pub extern "C" fn rb_ot_map_builder_add_gsub_pause(
    builder: *mut ffi::rb_ot_map_builder_t,
    pause: ffi::rb_ot_pause_func_t,
) {
    MapBuilder::from_ptr_mut(builder).add_gsub_pause(pause);
}
