use crate::{tag, Face, Language, Mask, Tag, Script};
use crate::buffer::{glyph_flag, Buffer};
use crate::plan::ShapePlan;
use crate::tables::gsubgpos::{FeatureIndex, LangIndex, LookupIndex, VariationIndex, ScriptIndex};
use super::TableIndex;

pub type PauseFunc = fn(&ShapePlan, &Face, &mut Buffer);

pub struct Map {
    found_script: [bool; 2],
    chosen_script: [Option<Tag>; 2],
    global_mask: Mask,
    features: Vec<FeatureMap>,
    lookups: [Vec<LookupMap>; 2],
    stages: [Vec<StageMap>; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FeatureMap {
    tag: Tag,
    // GSUB/GPOS
    index: [Option<FeatureIndex>; 2],
    stage: [usize; 2],
    shift: u32,
    mask: Mask,
    // mask for value=1, for quick access
    one_mask: Mask,
    auto_zwnj: bool,
    auto_zwj: bool,
    random: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LookupMap {
    pub index: LookupIndex,
    // TODO: to bitflags
    pub auto_zwnj: bool,
    pub auto_zwj: bool,
    pub random: bool,
    pub mask: Mask,
}

#[derive(Clone, Copy)]
pub struct StageMap {
    // Cumulative
    pub last_lookup: usize,
    pub pause_func: Option<PauseFunc>,
}

impl Map {
    pub const MAX_BITS: u32 = 8;
    pub const MAX_VALUE: u32 = (1 << Self::MAX_BITS) - 1;

    pub fn new() -> Self {
        Self {
            found_script: [false; 2],
            chosen_script: [None; 2],
            global_mask: 0,
            features: vec![],
            lookups: [vec![], vec![]],
            stages: [vec![], vec![]],
        }
    }

    #[inline]
    pub fn found_script(&self, table_index: TableIndex) -> bool {
        self.found_script[table_index]
    }

    #[inline]
    pub fn chosen_script(&self, table_index: TableIndex) -> Option<Tag> {
        self.chosen_script[table_index]
    }

    #[inline]
    pub fn global_mask(&self) -> Mask {
        self.global_mask
    }

    #[inline]
    pub fn mask(&self, feature_tag: Tag) -> (Mask, u32) {
        self.features
            .binary_search_by_key(&feature_tag, |f| f.tag)
            .map_or((0, 0), |idx| (self.features[idx].mask, self.features[idx].shift))
    }

    #[inline]
    pub fn one_mask(&self, feature_tag: Tag) -> Mask {
        self.features
            .binary_search_by_key(&feature_tag, |f| f.tag)
            .map_or(0, |idx| self.features[idx].one_mask)
    }

    #[inline]
    pub fn feature_index(&self, table_index: TableIndex, feature_tag: Tag) -> Option<FeatureIndex> {
        self.features
            .binary_search_by_key(&feature_tag, |f| f.tag)
            .ok()
            .and_then(|idx| self.features[idx].index[table_index])
    }

    #[inline]
    pub fn feature_stage(&self, table_index: TableIndex, feature_tag: Tag) -> Option<usize> {
        self.features
            .binary_search_by_key(&feature_tag, |f| f.tag)
            .map(|idx| self.features[idx].stage[table_index])
            .ok()
    }

    #[inline]
    pub fn stages(&self, table_index: TableIndex) -> &[StageMap] {
        &self.stages[table_index]
    }

    #[inline]
    pub fn stage_lookups(&self, table_index: TableIndex, stage: usize) -> &[LookupMap] {
        let stages = &self.stages[table_index];
        let lookups = &self.lookups[table_index];
        let start = stage.checked_sub(1).map_or(0, |prev| stages[prev].last_lookup);
        let end = stages.get(stage).map_or(lookups.len(), |curr| curr.last_lookup);
        &lookups[start..end]
    }
}

bitflags::bitflags! {
    /// Flags used for serialization with a `BufferSerializer`.
    #[derive(Default)]
    pub struct FeatureFlags: u32 {
        /// Feature applies to all characters; results in no mask allocated for it.
        const GLOBAL = 0x01;
        /// Has fallback implementation, so include mask bit even if feature not found.
        const HAS_FALLBACK = 0x02;
        /// Don't skip over ZWNJ when matching **context**.
        const MANUAL_ZWNJ = 0x04;
        /// Don't skip over ZWJ when matching **input**.
        const MANUAL_ZWJ = 0x08;
        /// If feature not found in LangSys, look for it in global feature list and pick one.
        const GLOBAL_SEARCH = 0x10;
        /// Randomly select a glyph from an AlternateSubstFormat1 subtable.
        const RANDOM = 0x20;

        const MANUAL_JOINERS        = Self::MANUAL_ZWNJ.bits | Self::MANUAL_ZWJ.bits;
        const GLOBAL_MANUAL_JOINERS = Self::GLOBAL.bits | Self::MANUAL_JOINERS.bits;
        const GLOBAL_HAS_FALLBACK   = Self::GLOBAL.bits | Self::HAS_FALLBACK.bits;
    }
}

pub struct MapBuilder<'a> {
    face: &'a Face<'a>,
    found_script: [bool; 2],
    script_index: [Option<ScriptIndex>; 2],
    chosen_script: [Option<Tag>; 2],
    lang_index: [Option<LangIndex>; 2],
    current_stage: [usize; 2],
    feature_infos: Vec<FeatureInfo>,
    stages: [Vec<StageInfo>; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct FeatureInfo {
    tag: Tag,
    // sequence number, used for stable sorting only
    seq: usize,
    max_value: u32,
    flags: FeatureFlags,
    // for non-global features, what should the unset glyphs take
    default_value: u32,
    // GSUB/GPOS
    stage: [usize; 2],
}

#[derive(Clone, Copy)]
struct StageInfo {
    index: usize,
    pause_func: Option<PauseFunc>,
}

impl<'a> MapBuilder<'a> {
    pub fn new(face: &'a Face<'a>, script: Option<Script>, language: Option<&Language>) -> Self {
        use crate::ot::layout::*;

        // Fetch script/language indices for GSUB/GPOS.  We need these later to skip
        // features not available in either table and not waste precious bits for them.

        let (script_tags, lang_tags) = tag::tags_from_script_and_language(script, language);

        let mut found_script = [false; 2];
        let mut script_index = [None; 2];
        let mut chosen_script = [None; 2];
        let mut lang_index = [None; 2];

        for table_index in TableIndex::iter() {
            if let Some(table) = face.layout_table(table_index) {
                if let Some((found, idx, tag)) = table.select_script(&script_tags) {
                    chosen_script[table_index] = Some(tag);
                    found_script[table_index] = found;
                    script_index[table_index] = Some(idx);

                    if let Some(idx) = table.select_script_language(idx, &lang_tags) {
                        lang_index[table_index] = Some(idx);
                    }
                }
            }
        }

        Self {
            face,
            found_script,
            script_index,
            chosen_script,
            lang_index,
            current_stage: [0, 0],
            feature_infos: vec![],
            stages: [vec![], vec![]],
        }
    }

    #[inline]
    pub fn chosen_script(&self, table_index: TableIndex) -> Option<Tag> {
        self.chosen_script[table_index]
    }

    #[inline]
    pub fn add_feature(&mut self, tag: Tag, flags: FeatureFlags, value: u32) {
        if !tag.is_null() {
            let seq = self.feature_infos.len();
            self.feature_infos.push(FeatureInfo {
                tag,
                seq,
                max_value: value,
                flags,
                default_value: if flags.contains(FeatureFlags::GLOBAL) { value } else { 0 },
                stage: self.current_stage,
            });
        }
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
    pub fn add_gsub_pause(&mut self, pause: Option<PauseFunc>) {
        self.add_pause(TableIndex::GSUB, pause);
    }

    #[inline]
    pub fn add_gpos_pause(&mut self, pause: Option<PauseFunc>) {
        self.add_pause(TableIndex::GPOS, pause);
    }

    // TODO: clean up
    fn add_pause(&mut self, table_index: TableIndex, pause: Option<PauseFunc>) {
        self.stages[table_index].push(StageInfo {
            index: self.current_stage[table_index],
            pause_func: pause,
        });

        self.current_stage[table_index] += 1;
    }

    // TODO: clean up
    pub fn compile(&mut self, map: &mut Map, variation_indices: [Option<VariationIndex>; 2]) {
        use crate::ot::layout::*;

        let global_bit_mask = glyph_flag::DEFINED + 1;
        let global_bit_shift = glyph_flag::DEFINED.count_ones();

        map.global_mask = global_bit_mask;

        // We default to applying required feature in stage 0.  If the required
        // feature has a tag that is known to the shaper, we apply required feature
        // in the stage for that tag.
        let mut required_feature_stage = [0; 2];
        let mut required_feature_index = [None; 2];
        let mut required_feature_tag = [None; 2];

        map.chosen_script = self.chosen_script;
        map.found_script = self.found_script;

        for table_index in TableIndex::iter() {
            if let Some(script) = self.script_index[table_index] {
                let lang = self.lang_index[table_index];
                if let Some((idx, tag)) = self.face
                    .layout_table(table_index)
                    .and_then(|table| table.get_required_language_feature(script, lang))
                {
                    required_feature_index[table_index] = Some(idx);
                    required_feature_tag[table_index] = Some(tag);
                }
            }
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
                let num_bits = |v: u32| 8 * std::mem::size_of_val(&v) as u32 - v.leading_zeros();
                bits_needed = Map::MAX_BITS.min(num_bits(info.max_value));
            }

            if info.max_value == 0 || next_bit + bits_needed > 8 * std::mem::size_of::<Mask>() as u32 {
                 // Feature disabled, or not enough bits.
                continue;
            }

            let mut found = false;
            let mut feature_index = [None; 2];
            for table_index in TableIndex::iter() {
                if required_feature_tag[table_index] == Some(info.tag) {
                    required_feature_stage[table_index] = info.stage[table_index];
                }

                if let Some(script) = self.script_index[table_index] {
                    let lang = self.lang_index[table_index];
                    if let Some(idx) = self.face
                        .layout_table(table_index)
                        .and_then(|table| table.find_language_feature(script, lang, info.tag))
                    {
                        feature_index[table_index] = Some(idx);
                        found = true;
                    }
                }
            }

            if !found && info.flags.contains(FeatureFlags::GLOBAL_SEARCH) {
                for table_index in TableIndex::iter() {
                    if let Some(idx) = self.face
                        .layout_table(table_index)
                        .and_then(|table| table.find_feature_index(info.tag))
                    {
                        feature_index[table_index] = Some(idx);
                        found = true;
                    }
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

            map.features.push(FeatureMap {
                tag: info.tag,
                index: feature_index,
                stage: info.stage,
                shift,
                mask,
                one_mask: (1 << shift) & mask,
                auto_zwnj: !info.flags.contains(FeatureFlags::MANUAL_ZWNJ),
                auto_zwj: !info.flags.contains(FeatureFlags::MANUAL_ZWJ),
                random: info.flags.contains(FeatureFlags::RANDOM),
            });
        }

        // Done with these.
        self.feature_infos.clear();

        self.add_gsub_pause(None);
        self.add_gpos_pause(None);

        for table_index in TableIndex::iter() {
            // Collect lookup indices for features.

            let mut stage_index = 0;
            let mut last_num_lookups = 0;

            for stage in 0..self.current_stage[table_index] {
                if required_feature_stage[table_index] == stage {
                    if let Some(feature_index) = required_feature_index[table_index] {
                        self.add_lookups(
                            &mut map.lookups[table_index],
                            table_index,
                            feature_index,
                            variation_indices[table_index],
                            global_bit_mask,
                            true,
                            true,
                            false,
                        );
                    }
                }

                for feature in &map.features {
                    if feature.stage[table_index] == stage {
                        if let Some(feature_index) = feature.index[table_index] {
                            self.add_lookups(
                                &mut map.lookups[table_index],
                                table_index,
                                feature_index,
                                variation_indices[table_index],
                                feature.mask,
                                feature.auto_zwnj,
                                feature.auto_zwj,
                                feature.random,
                            );
                        }
                    }
                }

                // Sort lookups and merge duplicates.
                let lookups = &mut map.lookups[table_index];
                let len = lookups.len();

                if last_num_lookups < len {
                    lookups[last_num_lookups..len].sort();

                    let mut j = last_num_lookups;
                    for i in j+1..len {
                        if lookups[i].index != lookups[j].index {
                            j += 1;
                            lookups[j] = lookups[i];
                        } else {
                            lookups[j].mask |= lookups[i].mask;
                            lookups[j].auto_zwnj &= lookups[i].auto_zwnj;
                            lookups[j].auto_zwj &= lookups[i].auto_zwj;
                        }
                    }

                    lookups.truncate(j + 1);
                }

                last_num_lookups = lookups.len();

                let stages = &self.stages[table_index];
                if stage_index < stages.len() && stages[stage_index].index == stage {
                    map.stages[table_index].push(StageMap {
                        last_lookup: last_num_lookups,
                        pause_func: stages[stage_index].pause_func,
                    });

                    stage_index += 1;
                }
            }
        }
    }

    fn add_lookups(
        &self,
        lookups: &mut Vec<LookupMap>,
        table_index: TableIndex,
        feature_index: FeatureIndex,
        variation_index: Option<VariationIndex>,
        mask: Mask,
        auto_zwnj: bool,
        auto_zwj: bool,
        random: bool,
    ) -> Option<()> {
        let table = self.face.layout_table(table_index)?;
        let lookup_count = table.lookup_count();

        let feature = match variation_index {
            Some(idx) => table.get_variation(feature_index, idx)?,
            None => table.get_feature(feature_index)?,
        };

        for index in feature.lookup_indices {
            if index.0 < lookup_count {
                lookups.push(LookupMap {
                    mask,
                    index,
                    auto_zwnj,
                    auto_zwj,
                    random,
                });
            }
        }

        Some(())
    }
}
