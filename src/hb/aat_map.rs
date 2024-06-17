use alloc::vec::Vec;

use super::aat_layout::*;
use super::{hb_font_t, hb_mask_t, hb_tag_t};

#[derive(Default)]
pub struct hb_aat_map_t {
    pub chain_flags: Vec<hb_mask_t>,
}

#[derive(Copy, Clone)]
pub struct feature_info_t {
    pub kind: u16,
    pub setting: u16,
    pub is_exclusive: bool,
}

#[derive(Default)]
pub struct hb_aat_map_builder_t {
    pub features: Vec<feature_info_t>,
}

impl hb_aat_map_builder_t {
    pub fn add_feature(&mut self, face: &hb_font_t, tag: hb_tag_t, value: u32) -> Option<()> {
        const FEATURE_TYPE_CHARACTER_ALTERNATIVES: u16 = 17;

        let feat = face.tables().feat?;

        if tag == hb_tag_t::from_bytes(b"aalt") {
            let exposes_feature = feat
                .names
                .find(FEATURE_TYPE_CHARACTER_ALTERNATIVES)
                .map(|f| f.setting_names.len() != 0)
                .unwrap_or(false);

            if !exposes_feature {
                return Some(());
            }

            self.features.push(feature_info_t {
                kind: FEATURE_TYPE_CHARACTER_ALTERNATIVES,
                setting: value as u16,
                is_exclusive: true,
            });
        }

        let idx = feature_mappings
            .binary_search_by(|map| map.ot_feature_tag.cmp(&tag))
            .ok()?;
        let mapping = &feature_mappings[idx];

        let mut feature = feat.names.find(mapping.aat_feature_type as u16);

        match feature {
            Some(feature) if feature.setting_names.len() != 0 => {}
            _ => {
                // Special case: Chain::compile_flags will fall back to the deprecated version of
                // small-caps if necessary, so we need to check for that possibility.
                // https://github.com/harfbuzz/harfbuzz/issues/2307
                if mapping.aat_feature_type == HB_AAT_LAYOUT_FEATURE_TYPE_LOWER_CASE
                    && mapping.selector_to_enable
                        == HB_AAT_LAYOUT_FEATURE_SELECTOR_LOWER_CASE_SMALL_CAPS
                {
                    feature = feat
                        .names
                        .find(HB_AAT_LAYOUT_FEATURE_TYPE_LETTER_CASE as u16);
                }
            }
        }

        match feature {
            Some(feature) if feature.setting_names.len() != 0 => {
                let setting = if value != 0 {
                    mapping.selector_to_enable
                } else {
                    mapping.selector_to_disable
                } as u16;

                self.features.push(feature_info_t {
                    kind: mapping.aat_feature_type as u16,
                    setting,
                    is_exclusive: feature.exclusive,
                });
            }
            _ => {}
        }

        Some(())
    }

    pub fn compile(&mut self, face: &hb_font_t) -> hb_aat_map_t {
        // Sort features and merge duplicates.
        self.features.sort_by(|a, b| {
            if a.kind != b.kind {
                a.kind.cmp(&b.kind)
            } else if !a.is_exclusive && (a.setting & !1) != (b.setting & !1) {
                a.setting.cmp(&b.setting)
            } else {
                core::cmp::Ordering::Equal
            }
        });

        let mut j = 0;
        for i in 0..self.features.len() {
            // Nonexclusive feature selectors come in even/odd pairs to turn a setting on/off
            // respectively, so we mask out the low-order bit when checking for "duplicates"
            // (selectors referring to the same feature setting) here.
            let non_exclusive = !self.features[i].is_exclusive
                && (self.features[i].setting & !1) != (self.features[j].setting & !1);

            if self.features[i].kind != self.features[j].kind || non_exclusive {
                j += 1;
                self.features[j] = self.features[i];
            }
        }
        self.features.truncate(j + 1);

        super::aat_layout_morx_table::compile_flags(face, self).unwrap_or_default()
    }
}
