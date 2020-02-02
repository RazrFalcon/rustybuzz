use std::cmp;
use std::os::raw::c_void;

use crate::ffi;

use ttf_parser::{
    GlyphPosSubTable, ScriptIndex, LanguageIndex, FeatureIndex, FeatureVariationIndex,
    LookupIndex,
};

use crate::{Tag, Mask, SegmentProperties, Language};

const TAG_GSUB: Tag = Tag::from_bytes(b"GSUB");
const TAG_GPOS: Tag = Tag::from_bytes(b"GPOS");
const TABLE_TAGS: [Tag; 2] = [TAG_GSUB, TAG_GPOS];
const TABLES_COUNT: usize = 2;


fn with_table<T: Default, F>(font: &ttf_parser::Font, tag: Tag, f: F) -> T
    where F: FnOnce(&dyn GlyphPosSubTable) -> T
{
    match tag {
        TAG_GSUB if font.has_table(ttf_parser::TableName::GlyphSubstitution) =>
            f(&font.substitution_table().unwrap()),
        TAG_GPOS if font.has_table(ttf_parser::TableName::GlyphPositioning) =>
            f(&font.positioning_table().unwrap()),
        _ => T::default()
    }
}


#[derive(Debug)]
pub struct Map {
    chosen_script: [Tag; 2],
    found_script: [bool; 2],
    global_mask: Mask,
    features: Vec<MapFeature>,
    lookups: [Vec<MapLookup>; 2], // GSUB/GPOS
    stages: [Vec<StageMap>; 2], // GSUB/GPOS
}

impl Map {
    fn new() -> Self {
        Map {
            chosen_script: [Tag(0xffff); 2],
            found_script: [false; 2],
            global_mask: 0,
            features: Vec::new(),
            lookups: [Vec::new(), Vec::new()],
            stages: [Vec::new(), Vec::new()],
        }
    }

    fn from_ptr(map: *const ffi::hb_ot_map_t) -> &'static Map {
        unsafe { &*(map as *const Map) }
    }
}


#[derive(Clone, Copy, Debug)]
struct MapFeature {
    tag: Tag,
    index: [FeatureIndex; 2], // GSUB/GPOS
    stage: [u32; 2], // GSUB/GPOS
    shift: u32,
    mask: Mask,
    mask1: Mask, // mask for value=1, for quick access
    needs_fallback: bool,
    auto_zwnj: bool,
    auto_zwj: bool,
    random: bool,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MapLookup {
    index: LookupIndex,
    // TODO: to bitflags
    auto_zwnj: bool,
    auto_zwj: bool,
    random: bool,
    mask: Mask,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct StageMap {
    last_lookup: u32, // Cumulative
    pause: ffi::pause_func_t,
}

impl std::fmt::Debug for StageMap {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("StageMap")
            .field("last_lookup", &self.last_lookup)
            .field("pause", if self.pause.is_some() { &"Some(fn)" } else { &"None" })
            .finish()
    }
}


#[derive(Clone, Copy, Debug)]
pub struct FeatureFlags(u8);

impl FeatureFlags {
    /// Feature applies to all characters; results in no mask allocated for it.
    pub const GLOBAL: Self                = Self(0x0001);

    /// Has fallback implementation, so include mask bit even if feature not found.
    pub const HAS_FALLBACK: Self          = Self(0x0002);

    /// Don't skip over ZWNJ when matching **context**.
    pub const MANUAL_ZWNJ: Self           = Self(0x0004);

    /// Don't skip over ZWJ when matching **input**.
    pub const MANUAL_ZWJ: Self            = Self(0x0008);

    /// If feature not found in LangSys, look for it in global feature list and pick one.
    pub const GLOBAL_SEARCH: Self         = Self(0x0010);

    /// Randomly select a glyph from an AlternateSubstFormat1 subtable.
    pub const RANDOM: Self                = Self(0x0020);

    #[inline] pub fn contains(&self, other: Self) -> bool { (self.0 & other.0) == other.0 }
    #[inline] pub fn remove(&mut self, other: Self) { self.0 &= !other.0; }
}

impl_bit_ops!(FeatureFlags);


#[derive(Debug)]
pub struct MapBuilder {
    chosen_script: [Tag; 2],
    found_script: [bool; 2],
    script_index: [ScriptIndex; 2],
    language_index: [LanguageIndex; 2],
    current_stage: [u32; 2],
    feature_infos: Vec<FeatureInfo>,
    stages: [Vec<StageInfo>; 2],
}

impl MapBuilder {
    pub fn new(font: &ttf_parser::Font, props: &SegmentProperties) -> Self {
        let mut builder = MapBuilder {
            chosen_script: [Tag(0xFFFF); 2],
            found_script: [false; 2],
            script_index: [ScriptIndex(0xFFFF); 2],
            language_index: [LanguageIndex(0xFFFF); 2],
            current_stage: [0; 2],
            feature_infos: Vec::new(),
            stages: [Vec::new(), Vec::new()],
        };

        // Fetch script/language indices for GSUB/GPOS.  We need these later to skip
        // features not available in either table and not waste precious bits for them.

        let (scripts, languages) = crate::tag::tags_from_script_and_language(
            props.script, props.language.as_ref(),
        );

        for i in 0..TABLES_COUNT {
            with_table(font, TABLE_TAGS[i], |table| {
                if let Some(script_info) = select_script(table, scripts.as_slice()) {
                    builder.chosen_script[i] = script_info.script;
                    builder.script_index[i] = script_info.index;
                    builder.found_script[i] = script_info.from_plan;
                }

                if let Some(lang_idx) = select_language(table, builder.script_index[i], languages.as_slice()) {
                    builder.language_index[i] = lang_idx;
                }
            });
        }

        builder
    }

    fn add_feature(&mut self, tag: Tag, flags: FeatureFlags, value: u32) {
        if tag.is_null() {
            return;
        }

        self.feature_infos.push(FeatureInfo {
            tag,
            max_value: value,
            flags,
            default_value: if flags.contains(FeatureFlags::GLOBAL) { value } else { 0 },
            stage: self.current_stage,
        })
    }

    fn add_lookups(
        &mut self,
        font: &ttf_parser::Font,
        table_index: usize,
        feature_index: FeatureIndex,
        variations_index: FeatureVariationIndex,
        mask: Mask,
        auto_zwnj: bool,
        auto_zwj: bool,
        random: bool,
        map: &mut Map,
    ) {
        with_table(font, TABLE_TAGS[table_index], |table| {
            let mut process = || -> Result<(), ttf_parser::Error> {
                let table_lookup_count = table.lookups()?.count() as u16;

                let mut feature = None;
                if let Some(variation) = table.feature_variation_at(variations_index)? {
                    if let Some(substitution) = variation.substitutions()?.nth(feature_index.0 as usize) {
                        if substitution.index() == feature_index {
                            feature = Some(substitution.feature()?);
                        }
                    }
                }

                if feature.is_none() {
                    feature = table.feature_at(feature_index)?;
                }

                if let Some(feature) = feature {
                    for index in feature.lookup_indices {
                        if index.0 < table_lookup_count {
                            map.lookups[table_index].push(MapLookup {
                                index,
                                auto_zwnj,
                                auto_zwj,
                                random,
                                mask,
                            });
                        }
                    }
                }

                Ok(())
            };

            process().unwrap();
        });
    }

    fn add_pause(&mut self, table_index: usize, pause: ffi::pause_func_t) {
        self.stages[table_index].push(StageInfo {
            index: self.current_stage[table_index],
            pause,
        });

        self.current_stage[table_index] += 1;
    }

    fn add_gsub_pause(&mut self, pause: ffi::pause_func_t) {
        self.add_pause(0, pause);
    }

    fn add_gpos_pause(&mut self, pause: ffi::pause_func_t) {
        self.add_pause(1, pause);
    }

    fn compile(
        &mut self,
        font: &ttf_parser::Font,
        variations_index: &[FeatureVariationIndex],
        map: &mut Map,
    ) -> Result<(), ttf_parser::Error> {
        let global_bit_mask = crate::buffer::glyph_flag::DEFINED + 1;
        let global_bit_shift = crate::buffer::glyph_flag::DEFINED.count_ones();

        map.global_mask = global_bit_mask;

        let mut required_feature_index = [FeatureIndex(0xFFFF), FeatureIndex(0xFFFF)];
        let mut required_feature_tag = [Tag(0), Tag(0)];
        // We default to applying required feature in stage 0.  If the required
        // feature has a tag that is known to the shaper, we apply required feature
        // in the stage for that tag.
        let mut required_feature_stage = [0, 0];

        map.chosen_script = self.chosen_script;
        map.found_script = self.found_script;
        for table_index in 0..TABLES_COUNT {
            with_table(font, TABLE_TAGS[table_index], |table| {
                if let Ok(Some(script)) = table.script_at(self.script_index[table_index]) {
                    if let Some(lang) = script.language_at(self.language_index[table_index]).or_else(|| script.default_language()) {
                        if let Some(idx) = lang.required_feature_index {
                            required_feature_index[table_index] = idx;
                            required_feature_tag[table_index] = lang.tag;
                        }
                    }
                }
            });
        }

        // Sort features and merge duplicates.
        if !self.feature_infos.is_empty() {
            self.feature_infos.sort_by_key(|v| v.tag);
            let mut j = 0;
            for i in 0..self.feature_infos.len() {
                if self.feature_infos[i].tag != self.feature_infos[j].tag {
                    j += 1;
                    self.feature_infos[j] = self.feature_infos[i];
                } else {
                    if self.feature_infos[i].flags.contains(FeatureFlags::GLOBAL) {
                        self.feature_infos[j].flags |= FeatureFlags::GLOBAL;
                        self.feature_infos[j].max_value = self.feature_infos[i].max_value;
                        self.feature_infos[j].default_value = self.feature_infos[i].default_value;
                    } else {
                        if self.feature_infos[j].flags.contains(FeatureFlags::GLOBAL) {
                            self.feature_infos[j].flags.remove(FeatureFlags::GLOBAL);
                        }

                        self.feature_infos[j].max_value = cmp::max(self.feature_infos[j].max_value,
                                                                   self.feature_infos[i].max_value)
                        // Inherit default_value from j
                    }

                    let new_flags = self.feature_infos[i].flags & FeatureFlags::HAS_FALLBACK;
                    self.feature_infos[j].flags |= new_flags;
                    self.feature_infos[j].stage[0] = cmp::min(self.feature_infos[j].stage[0],
                                                              self.feature_infos[i].stage[0]);
                    self.feature_infos[j].stage[1] = cmp::min(self.feature_infos[j].stage[1],
                                                              self.feature_infos[i].stage[1]);
                }
            }

            while self.feature_infos.len() != j + 1 {
                self.feature_infos.pop();
            }
        }

        // Allocate bits now.
        let mut next_bit = global_bit_shift + 1;

        for info in &self.feature_infos {
            let bits_needed = if info.flags.contains(FeatureFlags::GLOBAL) && info.max_value == 1 {
                // Uses the global bit
                0
            } else {
                // Limit bits per feature.
                32 - info.max_value.leading_zeros()
            };

            if info.max_value == 0 || next_bit + bits_needed > 8 * std::mem::size_of::<Mask>() as u32 {
                // Feature disabled, or not enough bits.
                continue;
            }

            let mut found = false;
            let mut feature_index = [FeatureIndex(0xFFFF), FeatureIndex(0xFFFF)];
            for table_index in 0..TABLES_COUNT {
                if required_feature_tag[table_index] == info.tag {
                    required_feature_stage[table_index] = info.stage[table_index];
                }

                let res = language_find_feature(
                    font,
                    TABLE_TAGS[table_index],
                    self.script_index[table_index],
                    self.language_index[table_index],
                    info.tag,
                );

                if let Some(idx) = res {
                    feature_index[table_index] = idx;
                    found |= true;
                }
            }

            if !found && info.flags.contains(FeatureFlags::GLOBAL_SEARCH) {
                for table_index in 0..TABLES_COUNT {
                    with_table(font, TABLE_TAGS[table_index], |table| {
                        for (idx, feature) in table.features().unwrap().enumerate() {
                            if feature.unwrap().tag == info.tag {
                                feature_index[0] = FeatureIndex(idx as u16);
                                found |= true;
                                break;
                            }
                        }
                    });
                }
            }

            if !found && !info.flags.contains(FeatureFlags::HAS_FALLBACK) {
                continue;
            }

            let (shift, mask) = if info.flags.contains(FeatureFlags::GLOBAL) && info.max_value == 1 {
                // Uses the global bit.
                (global_bit_shift, global_bit_mask)
            } else {
                let shift = next_bit;
                let mask = (1 << (next_bit + bits_needed)) - (1 << next_bit);
                next_bit += bits_needed;
                map.global_mask |= (info.default_value << shift) & mask;
                (shift, mask)
            };

            map.features.push(MapFeature {
                tag: info.tag,
                index: feature_index,
                stage: info.stage,
                shift,
                mask,
                mask1: (1 << shift) & mask,
                needs_fallback: !found,
                auto_zwnj: !info.flags.contains(FeatureFlags::MANUAL_ZWNJ),
                auto_zwj: !info.flags.contains(FeatureFlags::MANUAL_ZWJ),
                random: info.flags.contains(FeatureFlags::RANDOM),
            });
        }

        self.feature_infos.clear();

        self.add_gsub_pause(None);
        self.add_gpos_pause(None);

        for table_index in 0..TABLES_COUNT {
            let mut stage_index = 0;
            let mut last_num_lookups = 0;
            for stage in 0..self.current_stage[table_index] {
                if required_feature_index[table_index] != FeatureIndex(0xFFFF) &&
                   required_feature_stage[table_index] == stage
                {
                    self.add_lookups(
                        font,
                        table_index,
                        required_feature_index[table_index],
                        variations_index[table_index],
                        global_bit_mask,
                        true,
                        true,
                        false,
                        map,
                    );
                }

                for i in 0..map.features.len() {
                    if map.features[i].stage[table_index] == stage {
                        self.add_lookups(
                            font,
                            table_index,
                            map.features[i].index[table_index],
                            variations_index[table_index],
                            map.features[i].mask,
                            map.features[i].auto_zwnj,
                            map.features[i].auto_zwj,
                            map.features[i].random,
                            map,
                        );
                    }
                }

                // Sort lookups and merge duplicates.
                if last_num_lookups < map.lookups[table_index].len() {
                    let map_len = map.lookups[table_index].len();
                    map.lookups[table_index][last_num_lookups..map_len]
                        .sort_by_key(|v| v.index);

                    let mut j = last_num_lookups;
                    if j+1 < map.lookups[table_index].len() {
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

                        while map.lookups[table_index].len() > j + 1 {
                            map.lookups[table_index].pop();
                        }
                    }
                }

                last_num_lookups = map.lookups[table_index].len();

                if stage_index < self.stages[table_index].len() && self.stages[table_index][stage_index].index == stage {
                    map.stages[table_index].push(StageMap {
                        last_lookup: last_num_lookups as u32,
                        pause: self.stages[table_index][stage_index].pause.clone(),
                    });

                    stage_index += 1;
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct TableScriptInfo {
    script: Tag,
    index: ScriptIndex,
    from_plan: bool, // Indicates that a `script` was in the original plan.
}

fn select_script(
    table: &dyn GlyphPosSubTable,
    scripts: &[Tag],
) -> Option<TableScriptInfo> {
    let mut font_scripts = Vec::new();
    for script in table.scripts().ok()? {
        if let Ok(script) = script {
            font_scripts.push(script.tag());
        } else {
            font_scripts.push(Tag(0));
        }
    }

    for script in scripts.iter().cloned() {
        if let Some(idx) = font_scripts.iter().position(|s| *s == script) {
            return Some(TableScriptInfo { script, index: ScriptIndex(idx as u16), from_plan: true });
        }
    }

    // Try finding 'dflt'.
    if let Some(idx) = font_scripts.iter().position(|s| *s == Tag::default_script()) {
        return Some(TableScriptInfo { script: Tag::default_script(), index: ScriptIndex(idx as u16), from_plan: false });
    }

    // Try with 'latn'; some old fonts put their features there even though
    // they're really trying to support Thai, for example :(
    if let Some(idx) = font_scripts.iter().position(|s| *s == Tag::from_bytes(b"latn")) {
        return Some(TableScriptInfo { script: Tag::from_bytes(b"latn"), index: ScriptIndex(idx as u16), from_plan: false });
    }

    None
}

fn select_language(
    table: &dyn GlyphPosSubTable,
    script_index: ScriptIndex,
    languages: &[Tag],
) -> Option<LanguageIndex> {
    if let Ok(Some(script)) = table.script_at(script_index) {
        for language in languages {
            if let Some((idx, _)) = script.language_by_tag(*language) {
                return Some(idx);
            }
        }

        // Try finding 'dflt'.
        if let Some((idx, _)) = script.language_by_tag(Tag::default_language()) {
            return Some(idx);
        }
    }

    None
}


#[derive(Clone, Copy, Debug)]
struct FeatureInfo {
    tag: Tag,
    max_value: u32,
    flags: FeatureFlags,
    default_value: u32, // for non-global features, what should the unset glyphs take
    stage: [u32; 2], // GSUB/GPOS
}

#[derive(Clone, Copy)]
struct StageInfo {
    index: u32,
    pause: ffi::pause_func_t,
}

impl std::fmt::Debug for StageInfo {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("StageInfo")
            .field("index", &self.index)
            .field("pause", if self.pause.is_some() { &"Some(fn)" } else { &"None" })
            .finish()
    }
}

fn language_find_feature(
    font: &ttf_parser::Font,
    table_tag: Tag,
    script_index: ScriptIndex,
    language_index: LanguageIndex,
    feature_tag: Tag,
) -> Option<FeatureIndex> {
    with_table(font, table_tag, |table| {
        let script = table.script_at(script_index).ok()??;
        let lang = if language_index.0 != 0xFFFF {
            script.language_at(language_index)?
        } else {
            script.default_language()?
        };

        for idx in lang.feature_indices {
            if let Ok(Some(feature)) = table.feature_at(idx) {
                if feature.tag == feature_tag {
                    return Some(idx);
                }
            }
        }

        None
    })
}

// Map

#[no_mangle]
pub extern "C" fn rb_ot_map_init() -> *mut ffi::hb_ot_map_t {
    Box::into_raw(Box::new(Map::new())) as *mut _
}

#[no_mangle]
pub extern "C" fn rb_ot_map_fini(map: *mut ffi::hb_ot_map_t) {
    unsafe { Box::from_raw(map as *mut Map) };
}

#[no_mangle]
pub extern "C" fn rb_ot_map_get_global_mask(map: *const ffi::hb_ot_map_t) -> Mask {
    Map::from_ptr(map).global_mask
}

#[no_mangle]
pub extern "C" fn rb_ot_map_get_mask(
    map: *const ffi::hb_ot_map_t,
    feature_tag: Tag,
    shift: *mut u32,
) -> Mask {
    let map = Map::from_ptr(map);
    if let Ok(idx) = map.features.binary_search_by(|v| v.tag.cmp(&feature_tag)) {
        if shift != std::ptr::null_mut() {
            unsafe { *shift = map.features[idx].shift; }
        }

        map.features[idx].mask
    } else {
        if shift != std::ptr::null_mut() {
            unsafe { *shift = 0; }
        }

        0
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_map_needs_fallback(map: *const ffi::hb_ot_map_t, feature_tag: Tag) -> bool {
    let map = Map::from_ptr(map);
    if let Ok(idx) = map.features.binary_search_by(|v| v.tag.cmp(&feature_tag)) {
        map.features[idx].needs_fallback
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_map_get_1_mask(map: *const ffi::hb_ot_map_t, feature_tag: Tag) -> Mask {
    let map = Map::from_ptr(map);
    if let Ok(idx) = map.features.binary_search_by(|v| v.tag.cmp(&feature_tag)) {
        map.features[idx].mask1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_map_get_feature_index(
    map: *const ffi::hb_ot_map_t,
    table_index: u32,
    feature_tag: Tag,
) -> u32 {
    let map = Map::from_ptr(map);
    if let Ok(idx) = map.features.binary_search_by(|v| v.tag.cmp(&feature_tag)) {
        map.features[idx].index[table_index as usize].0 as u32
    } else {
        std::u32::MAX
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_map_get_feature_stage(
    map: *const ffi::hb_ot_map_t,
    table_index: u32,
    feature_tag: Tag,
) -> u32 {
    let map = Map::from_ptr(map);
    if let Ok(idx) = map.features.binary_search_by(|v| v.tag.cmp(&feature_tag)) {
        map.features[idx].stage[table_index as usize]
    } else {
        std::u32::MAX
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_map_get_chosen_script(map: *const ffi::hb_ot_map_t, table_index: u32) -> Tag {
    Map::from_ptr(map).chosen_script[table_index as usize]
}

#[no_mangle]
pub extern "C" fn rb_ot_map_has_found_script(map: *const ffi::hb_ot_map_t, table_index: u32) -> bool {
    Map::from_ptr(map).found_script[table_index as usize]
}

#[no_mangle]
pub extern "C" fn rb_ot_map_get_lookup(map: *const ffi::hb_ot_map_t, table_index: u32, i: u32) -> *const MapLookup {
    &Map::from_ptr(map).lookups[table_index as usize][i as usize] as *const _
}

#[no_mangle]
pub extern "C" fn rb_ot_map_get_stages_length(map: *const ffi::hb_ot_map_t, table_index: u32) -> u32 {
    Map::from_ptr(map).stages[table_index as usize].len() as u32
}

#[no_mangle]
pub extern "C" fn rb_ot_map_get_stage(map: *const ffi::hb_ot_map_t, table_index: u32, i: u32) -> *const StageMap {
    &Map::from_ptr(map).stages[table_index as usize][i as usize] as *const _
}

#[no_mangle]
pub extern "C" fn rb_ot_map_get_stage_lookups(
    map: *const ffi::hb_ot_map_t,
    table_index: u32,
    stage: u32,
    plookups: *mut *const MapLookup,
    lookup_count: *mut u32,
) {
    if stage == std::u32::MAX {
        unsafe {
            *plookups = std::ptr::null_mut();
            *lookup_count = 0;
            return;
        }
    }

    let map = Map::from_ptr(map);
    let table_index = table_index as usize;
    let stage = stage as usize;

    let start = if stage != 0 {
        map.stages[table_index][stage - 1].last_lookup as usize
    } else {
        0
    };


    let end = if stage < map.stages[table_index].len() {
        map.stages[table_index][stage].last_lookup as usize
    } else {
        map.lookups[table_index].len()
    };

    unsafe {
        if end == start {
            *plookups = std::ptr::null_mut();
        } else {
            *plookups = map.lookups[table_index][start..].as_ptr();
        }

        *lookup_count = (end - start) as u32;
    }
}

// Builder

#[no_mangle]
pub extern "C" fn rb_ot_map_builder_init(
    font_data: *const c_void,
    props: *const ffi::hb_segment_properties_t,
) -> *mut ffi::hb_ot_map_builder_t {
    use std::str::FromStr;

    let font = unsafe { &*(font_data as *const ttf_parser::Font) };

    let props = unsafe {
        let lang = if (*props).language != std::ptr::null() {
            let s = std::ffi::CStr::from_ptr((*props).language);
            let s = s.to_str().expect("locale must be ASCII");
            Language::from_str(s).ok()
        } else {
            Language::from_str("en_US.UTF-8").ok()
        };

        let script = if (*props).script != 0 {
            Some(crate::Script(Tag((*props).script)))
        } else {
            None
        };

        SegmentProperties {
            direction: crate::Direction::from_raw((*props).direction),
            script,
            language: lang,
        }
    };

    Box::into_raw(Box::new(MapBuilder::new(font, &props))) as *mut _
}

#[no_mangle]
pub extern "C" fn rb_ot_map_builder_fini(builder: *mut ffi::hb_ot_map_builder_t) {
    unsafe { Box::from_raw(builder as *mut MapBuilder) };
}

#[no_mangle]
pub extern "C" fn rb_ot_map_builder_compile(
    builder: *mut ffi::hb_ot_map_builder_t,
    map: *mut ffi::hb_ot_map_t,
    font_data: *const c_void,
    variations_index: *const FeatureVariationIndex,
) {
    let font = unsafe { &*(font_data as *const ttf_parser::Font) };
    let builder = unsafe { &mut *(builder as *mut MapBuilder) };
    let map = unsafe { &mut *(map as *mut Map) };
    let variations = unsafe { std::slice::from_raw_parts(variations_index as *const _, 2) };
    builder.compile(font, variations, map).unwrap();
}

#[no_mangle]
pub extern "C" fn rb_ot_map_builder_add_feature(
    builder: *mut ffi::hb_ot_map_builder_t,
    tag: Tag,
    flags: u32,
    value: u32,
) {
    let builder = unsafe { &mut *(builder as *mut MapBuilder) };
    builder.add_feature(tag, FeatureFlags(flags as u8), value);
}

#[no_mangle]
pub extern "C" fn rb_ot_map_builder_enable_feature(
    builder: *mut ffi::hb_ot_map_builder_t,
    tag: Tag,
    flags: u32,
    value: u32,
) {
    let builder = unsafe { &mut *(builder as *mut MapBuilder) };
    builder.add_feature(tag, FeatureFlags(flags as u8) | FeatureFlags::GLOBAL, value);
}

#[no_mangle]
pub extern "C" fn rb_ot_map_builder_disable_feature(
    builder: *mut ffi::hb_ot_map_builder_t,
    tag: Tag,
) {
    let builder = unsafe { &mut *(builder as *mut MapBuilder) };
    builder.add_feature(tag, FeatureFlags::GLOBAL, 0);
}

#[no_mangle]
pub extern "C" fn rb_ot_map_builder_add_gsub_pause(
    builder: *mut ffi::hb_ot_map_builder_t,
    pause_func: ffi::pause_func_t,
) {
    let builder = unsafe { &mut *(builder as *mut MapBuilder) };
    builder.add_gsub_pause(pause_func);
}

#[no_mangle]
pub extern "C" fn rb_ot_map_builder_chosen_script(
    builder: *mut ffi::hb_ot_map_builder_t,
    table_index: u32,
) -> Tag {
    let builder = unsafe { &mut *(builder as *mut MapBuilder) };
    builder.chosen_script[table_index as usize]
}
