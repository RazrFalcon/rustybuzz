use alloc::boxed::Box;
use alloc::vec::Vec;
use core::any::Any;

use super::feature;
use super::ot_layout::TableIndex;
use super::ot_map::*;
use super::ot_shape_complex::*;
use super::{aat_map, hb_font_t, hb_mask_t, hb_tag_t, Direction, Feature, Language, Script};

/// A reusable plan for shaping a text buffer.
pub struct hb_ot_shape_plan_t {
    pub(crate) direction: Direction,
    pub(crate) script: Option<Script>,
    pub(crate) shaper: &'static ComplexShaper,
    pub(crate) ot_map: hb_ot_map_t,
    pub(crate) aat_map: aat_map::hb_aat_map_t,
    data: Option<Box<dyn Any + Send + Sync>>,

    pub(crate) frac_mask: hb_mask_t,
    pub(crate) numr_mask: hb_mask_t,
    pub(crate) dnom_mask: hb_mask_t,
    pub(crate) rtlm_mask: hb_mask_t,
    pub(crate) kern_mask: hb_mask_t,
    pub(crate) trak_mask: hb_mask_t,

    pub(crate) requested_kerning: bool,
    pub(crate) has_frac: bool,
    pub(crate) has_vert: bool,
    pub(crate) has_gpos_mark: bool,
    pub(crate) zero_marks: bool,
    pub(crate) fallback_glyph_classes: bool,
    pub(crate) fallback_mark_positioning: bool,
    pub(crate) adjust_mark_positioning_when_zeroing: bool,

    pub(crate) apply_gpos: bool,
    pub(crate) apply_fallback_kern: bool,
    pub(crate) apply_kern: bool,
    pub(crate) apply_kerx: bool,
    pub(crate) apply_morx: bool,
    pub(crate) apply_trak: bool,

    pub(crate) user_features: Vec<Feature>,
}

impl hb_ot_shape_plan_t {
    /// Returns a plan that can be used for shaping any buffer with the
    /// provided properties.
    pub fn new(
        face: &hb_font_t,
        direction: Direction,
        script: Option<Script>,
        language: Option<&Language>,
        user_features: &[Feature],
    ) -> Self {
        assert_ne!(direction, Direction::Invalid);
        let mut planner = ShapePlanner::new(face, direction, script, language);
        planner.collect_features(user_features);
        planner.compile(user_features)
    }

    pub(crate) fn data<T: 'static>(&self) -> &T {
        self.data.as_ref().unwrap().downcast_ref().unwrap()
    }
}

pub struct ShapePlanner<'a> {
    pub face: &'a hb_font_t<'a>,
    pub direction: Direction,
    pub script: Option<Script>,
    pub ot_map: hb_ot_map_builder_t<'a>,
    pub aat_map: aat_map::hb_aat_map_builder_t,
    pub apply_morx: bool,
    pub script_zero_marks: bool,
    pub script_fallback_mark_positioning: bool,
    pub shaper: &'static ComplexShaper,
}

impl<'a> ShapePlanner<'a> {
    fn new(
        face: &'a hb_font_t<'a>,
        direction: Direction,
        script: Option<Script>,
        language: Option<&Language>,
    ) -> Self {
        let ot_map = hb_ot_map_builder_t::new(face, script, language);
        let aat_map = aat_map::hb_aat_map_builder_t::default();

        let mut shaper = match script {
            Some(script) => {
                complex_categorize(script, direction, ot_map.chosen_script(TableIndex::GSUB))
            }
            None => &DEFAULT_SHAPER,
        };

        let script_zero_marks = shaper.zero_width_marks.is_some();
        let script_fallback_mark_positioning = shaper.fallback_position;

        // https://github.com/harfbuzz/harfbuzz/issues/2124
        let apply_morx =
            face.tables().morx.is_some() && (direction.is_horizontal() || face.gsub.is_none());

        // https://github.com/harfbuzz/harfbuzz/issues/1528
        if apply_morx && shaper as *const _ != &DEFAULT_SHAPER as *const _ {
            shaper = &DUMBER_SHAPER;
        }

        ShapePlanner {
            face,
            direction,
            script,
            ot_map,
            aat_map,
            apply_morx,
            script_zero_marks,
            script_fallback_mark_positioning,
            shaper,
        }
    }

    fn collect_features(&mut self, user_features: &[Feature]) {
        const COMMON_FEATURES: &[(hb_tag_t, FeatureFlags)] = &[
            (feature::ABOVE_BASE_MARK_POSITIONING, FeatureFlags::GLOBAL),
            (feature::BELOW_BASE_MARK_POSITIONING, FeatureFlags::GLOBAL),
            (
                feature::GLYPH_COMPOSITION_DECOMPOSITION,
                FeatureFlags::GLOBAL,
            ),
            (feature::LOCALIZED_FORMS, FeatureFlags::GLOBAL),
            (
                feature::MARK_POSITIONING,
                FeatureFlags::GLOBAL_MANUAL_JOINERS,
            ),
            (
                feature::MARK_TO_MARK_POSITIONING,
                FeatureFlags::GLOBAL_MANUAL_JOINERS,
            ),
            (feature::REQUIRED_LIGATURES, FeatureFlags::GLOBAL),
        ];

        const HORIZONTAL_FEATURES: &[(hb_tag_t, FeatureFlags)] = &[
            (feature::CONTEXTUAL_ALTERNATES, FeatureFlags::GLOBAL),
            (feature::CONTEXTUAL_LIGATURES, FeatureFlags::GLOBAL),
            (feature::CURSIVE_POSITIONING, FeatureFlags::GLOBAL),
            (feature::DISTANCES, FeatureFlags::GLOBAL),
            (feature::KERNING, FeatureFlags::GLOBAL_HAS_FALLBACK),
            (feature::STANDARD_LIGATURES, FeatureFlags::GLOBAL),
            (
                feature::REQUIRED_CONTEXTUAL_ALTERNATES,
                FeatureFlags::GLOBAL,
            ),
        ];

        let empty = FeatureFlags::empty();

        self.ot_map
            .enable_feature(feature::REQUIRED_VARIATION_ALTERNATES, empty, 1);
        self.ot_map.add_gsub_pause(None);

        match self.direction {
            Direction::LeftToRight => {
                self.ot_map
                    .enable_feature(feature::LEFT_TO_RIGHT_ALTERNATES, empty, 1);
                self.ot_map
                    .enable_feature(feature::LEFT_TO_RIGHT_MIRRORED_FORMS, empty, 1);
            }
            Direction::RightToLeft => {
                self.ot_map
                    .enable_feature(feature::RIGHT_TO_LEFT_ALTERNATES, empty, 1);
                self.ot_map
                    .add_feature(feature::RIGHT_TO_LEFT_MIRRORED_FORMS, empty, 1);
            }
            _ => {}
        }

        // Automatic fractions.
        self.ot_map.add_feature(feature::FRACTIONS, empty, 1);
        self.ot_map.add_feature(feature::NUMERATORS, empty, 1);
        self.ot_map.add_feature(feature::DENOMINATORS, empty, 1);

        // Random!
        self.ot_map.enable_feature(
            feature::RANDOMIZE,
            FeatureFlags::RANDOM,
            hb_ot_map_t::MAX_VALUE,
        );

        // Tracking.  We enable dummy feature here just to allow disabling
        // AAT 'trak' table using features.
        // https://github.com/harfbuzz/harfbuzz/issues/1303
        self.ot_map
            .enable_feature(hb_tag_t::from_bytes(b"trak"), FeatureFlags::HAS_FALLBACK, 1);

        self.ot_map
            .enable_feature(hb_tag_t::from_bytes(b"Harf"), empty, 1); // Considered required.
        self.ot_map
            .enable_feature(hb_tag_t::from_bytes(b"HARF"), empty, 1); // Considered discretionary.

        if let Some(func) = self.shaper.collect_features {
            func(self);
        }

        self.ot_map
            .enable_feature(hb_tag_t::from_bytes(b"Buzz"), empty, 1); // Considered required.
        self.ot_map
            .enable_feature(hb_tag_t::from_bytes(b"BUZZ"), empty, 1); // Considered discretionary.

        for &(tag, flags) in COMMON_FEATURES {
            self.ot_map.add_feature(tag, flags, 1);
        }

        if self.direction.is_horizontal() {
            for &(tag, flags) in HORIZONTAL_FEATURES {
                self.ot_map.add_feature(tag, flags, 1);
            }
        } else {
            // We only apply `vert` feature. See:
            // https://github.com/harfbuzz/harfbuzz/commit/d71c0df2d17f4590d5611239577a6cb532c26528
            // https://lists.freedesktop.org/archives/harfbuzz/2013-August/003490.html

            // We really want to find a 'vert' feature if there's any in the font, no
            // matter which script/langsys it is listed (or not) under.
            // See various bugs referenced from:
            // https://github.com/harfbuzz/harfbuzz/issues/63
            self.ot_map
                .enable_feature(feature::VERTICAL_WRITING, FeatureFlags::GLOBAL_SEARCH, 1);
        }

        for feature in user_features {
            let flags = if feature.is_global() {
                FeatureFlags::GLOBAL
            } else {
                empty
            };
            self.ot_map.add_feature(feature.tag, flags, feature.value);
        }

        if self.apply_morx {
            for feature in user_features {
                self.aat_map
                    .add_feature(self.face, feature.tag, feature.value);
            }
        }

        if let Some(func) = self.shaper.override_features {
            func(self);
        }
    }

    fn compile(mut self, user_features: &[Feature]) -> hb_ot_shape_plan_t {
        let ot_map = self.ot_map.compile();

        let aat_map = if self.apply_morx {
            self.aat_map.compile(self.face)
        } else {
            aat_map::hb_aat_map_t::default()
        };

        let frac_mask = ot_map.get_1_mask(feature::FRACTIONS);
        let numr_mask = ot_map.get_1_mask(feature::NUMERATORS);
        let dnom_mask = ot_map.get_1_mask(feature::DENOMINATORS);
        let has_frac = frac_mask != 0 || (numr_mask != 0 && dnom_mask != 0);

        let rtlm_mask = ot_map.get_1_mask(feature::RIGHT_TO_LEFT_MIRRORED_FORMS);
        let has_vert = ot_map.get_1_mask(feature::VERTICAL_WRITING) != 0;

        let horizontal = self.direction.is_horizontal();
        let kern_tag = if horizontal {
            feature::KERNING
        } else {
            feature::VERTICAL_KERNING
        };
        let kern_mask = ot_map.get_mask(kern_tag).0;
        let requested_kerning = kern_mask != 0;
        let trak_mask = ot_map.get_mask(hb_tag_t::from_bytes(b"trak")).0;
        let requested_tracking = trak_mask != 0;

        let has_gpos_kern = ot_map
            .get_feature_index(TableIndex::GPOS, kern_tag)
            .is_some();
        let disable_gpos = self.shaper.gpos_tag.is_some()
            && self.shaper.gpos_tag != ot_map.chosen_script(TableIndex::GPOS);

        // Decide who provides glyph classes. GDEF or Unicode.
        let fallback_glyph_classes = !self
            .face
            .tables()
            .gdef
            .map_or(false, |table| table.has_glyph_classes());

        // Decide who does substitutions. GSUB, morx, or fallback.
        let apply_morx = self.apply_morx;

        let mut apply_gpos = false;
        let mut apply_kerx = false;
        let mut apply_kern = false;

        // Decide who does positioning. GPOS, kerx, kern, or fallback.
        let has_kerx = self.face.tables().kerx.is_some();
        let has_gsub = self.face.tables().gsub.is_some();
        let has_gpos = !disable_gpos && self.face.tables().gpos.is_some();

        // Prefer GPOS over kerx if GSUB is present;
        // https://github.com/harfbuzz/harfbuzz/issues/3008
        if has_kerx && !(has_gsub && has_gpos) {
            apply_kerx = true;
        } else if has_gpos {
            apply_gpos = true;
        }

        if !apply_kerx && (!has_gpos_kern || !apply_gpos) {
            if has_kerx {
                apply_kerx = true;
            } else if super::kerning::has_kerning(self.face) {
                apply_kern = true;
            }
        }

        let apply_fallback_kern = !(apply_gpos || apply_kerx || apply_kern);
        let zero_marks = self.script_zero_marks
            && !apply_kerx
            && (!apply_kern || !super::kerning::has_machine_kerning(self.face));

        let has_gpos_mark = ot_map.get_1_mask(feature::MARK_POSITIONING) != 0;

        let mut adjust_mark_positioning_when_zeroing = !apply_gpos
            && !apply_kerx
            && (!apply_kern || !super::kerning::has_cross_kerning(self.face));

        let fallback_mark_positioning =
            adjust_mark_positioning_when_zeroing && self.script_fallback_mark_positioning;

        // If we're using morx shaping, we cancel mark position adjustment because
        // Apple Color Emoji assumes this will NOT be done when forming emoji sequences;
        // https://github.com/harfbuzz/harfbuzz/issues/2967.
        if apply_morx {
            adjust_mark_positioning_when_zeroing = false;
        }

        // Currently we always apply trak.
        let apply_trak = requested_tracking && self.face.tables().trak.is_some();

        let mut plan = hb_ot_shape_plan_t {
            direction: self.direction,
            script: self.script,
            shaper: self.shaper,
            ot_map,
            aat_map,
            data: None,
            frac_mask,
            numr_mask,
            dnom_mask,
            rtlm_mask,
            kern_mask,
            trak_mask,
            requested_kerning,
            has_frac,
            has_vert,
            has_gpos_mark,
            zero_marks,
            fallback_glyph_classes,
            fallback_mark_positioning,
            adjust_mark_positioning_when_zeroing,
            apply_gpos,
            apply_kern,
            apply_fallback_kern,
            apply_kerx,
            apply_morx,
            apply_trak,
            user_features: user_features.to_vec(),
        };

        if let Some(func) = self.shaper.create_data {
            plan.data = Some(func(&plan));
        }

        plan
    }
}

#[cfg(test)]
mod tests {
    use super::hb_ot_shape_plan_t;

    #[test]
    fn test_shape_plan_is_send_and_sync() {
        fn ensure_send_and_sync<T: Send + Sync>() {}
        ensure_send_and_sync::<hb_ot_shape_plan_t>();
    }
}
