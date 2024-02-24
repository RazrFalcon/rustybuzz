use super::aat_map;
use super::ot_layout::*;
use super::ot_map::hb_ot_map_builder_t;
use super::ot_map::*;
use super::ot_shape_complex::*;
use super::shape_plan::hb_ot_shape_plan_t;
use super::{hb_font_t, hb_tag_t};
use crate::{Direction, Feature, Language, Script};

pub struct hb_ot_shape_planner_t<'a> {
    pub face: &'a hb_font_t<'a>,
    pub direction: Direction,
    pub script: Option<Script>,
    pub ot_map: hb_ot_map_builder_t<'a>,
    pub aat_map: aat_map::hb_aat_map_builder_t,
    pub apply_morx: bool,
    pub script_zero_marks: bool,
    pub script_fallback_mark_positioning: bool,
    pub shaper: &'static hb_ot_complex_shaper_t,
}

impl<'a> hb_ot_shape_planner_t<'a> {
    pub fn new(
        face: &'a hb_font_t<'a>,
        direction: Direction,
        script: Option<Script>,
        language: Option<&Language>,
    ) -> Self {
        let ot_map = hb_ot_map_builder_t::new(face, script, language);
        let aat_map = aat_map::hb_aat_map_builder_t::default();

        let mut shaper = match script {
            Some(script) => hb_ot_shape_complex_categorize(
                script,
                direction,
                ot_map.chosen_script(TableIndex::GSUB),
            ),
            None => &DEFAULT_SHAPER,
        };

        let script_zero_marks = shaper.zero_width_marks != HB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE;
        let script_fallback_mark_positioning = shaper.fallback_position;

        // https://github.com/harfbuzz/harfbuzz/issues/2124
        let apply_morx =
            face.tables().morx.is_some() && (direction.is_horizontal() || face.gsub.is_none());

        // https://github.com/harfbuzz/harfbuzz/issues/1528
        if apply_morx && shaper as *const _ != &DEFAULT_SHAPER as *const _ {
            shaper = &DUMBER_SHAPER;
        }

        hb_ot_shape_planner_t {
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

    pub fn collect_features(&mut self, user_features: &[Feature]) {
        const COMMON_FEATURES: &[(hb_tag_t, hb_ot_map_feature_flags_t)] = &[
            (hb_tag_t::from_bytes(b"abvm"), F_GLOBAL),
            (hb_tag_t::from_bytes(b"blwm"), F_GLOBAL),
            (hb_tag_t::from_bytes(b"ccmp"), F_GLOBAL),
            (hb_tag_t::from_bytes(b"locl"), F_GLOBAL),
            (hb_tag_t::from_bytes(b"mark"), F_GLOBAL_MANUAL_JOINERS),
            (hb_tag_t::from_bytes(b"mkmk"), F_GLOBAL_MANUAL_JOINERS),
            (hb_tag_t::from_bytes(b"rlig"), F_GLOBAL),
        ];

        const HORIZONTAL_FEATURES: &[(hb_tag_t, hb_ot_map_feature_flags_t)] = &[
            (hb_tag_t::from_bytes(b"calt"), F_GLOBAL),
            (hb_tag_t::from_bytes(b"clig"), F_GLOBAL),
            (hb_tag_t::from_bytes(b"curs"), F_GLOBAL),
            (hb_tag_t::from_bytes(b"dist"), F_GLOBAL),
            (hb_tag_t::from_bytes(b"kern"), F_GLOBAL_HAS_FALLBACK),
            (hb_tag_t::from_bytes(b"liga"), F_GLOBAL),
            (hb_tag_t::from_bytes(b"rclt"), F_GLOBAL),
        ];

        let empty = F_NONE;

        self.ot_map
            .enable_feature(hb_tag_t::from_bytes(b"rvrn"), empty, 1);
        self.ot_map.add_gsub_pause(None);

        match self.direction {
            Direction::LeftToRight => {
                self.ot_map
                    .enable_feature(hb_tag_t::from_bytes(b"ltra"), empty, 1);
                self.ot_map
                    .enable_feature(hb_tag_t::from_bytes(b"ltrm"), empty, 1);
            }
            Direction::RightToLeft => {
                self.ot_map
                    .enable_feature(hb_tag_t::from_bytes(b"rtla"), empty, 1);
                self.ot_map
                    .add_feature(hb_tag_t::from_bytes(b"rtlm"), empty, 1);
            }
            _ => {}
        }

        // Automatic fractions.
        self.ot_map
            .add_feature(hb_tag_t::from_bytes(b"frac"), empty, 1);
        self.ot_map
            .add_feature(hb_tag_t::from_bytes(b"numr"), empty, 1);
        self.ot_map
            .add_feature(hb_tag_t::from_bytes(b"dnom"), empty, 1);

        // Random!
        self.ot_map.enable_feature(
            hb_tag_t::from_bytes(b"rand"),
            F_RANDOM,
            hb_ot_map_t::MAX_VALUE,
        );

        // Tracking.  We enable dummy feature here just to allow disabling
        // AAT 'trak' table using features.
        // https://github.com/harfbuzz/harfbuzz/issues/1303
        self.ot_map
            .enable_feature(hb_tag_t::from_bytes(b"trak"), F_HAS_FALLBACK, 1);

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
                .enable_feature(hb_tag_t::from_bytes(b"vert"), F_GLOBAL_SEARCH, 1);
        }

        for feature in user_features {
            let flags = if feature.is_global() { F_GLOBAL } else { empty };
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

    pub fn compile(mut self, user_features: &[Feature]) -> hb_ot_shape_plan_t {
        let ot_map = self.ot_map.compile();

        let aat_map = if self.apply_morx {
            self.aat_map.compile(self.face)
        } else {
            aat_map::hb_aat_map_t::default()
        };

        let frac_mask = ot_map.get_1_mask(hb_tag_t::from_bytes(b"frac"));
        let numr_mask = ot_map.get_1_mask(hb_tag_t::from_bytes(b"numr"));
        let dnom_mask = ot_map.get_1_mask(hb_tag_t::from_bytes(b"dnom"));
        let has_frac = frac_mask != 0 || (numr_mask != 0 && dnom_mask != 0);

        let rtlm_mask = ot_map.get_1_mask(hb_tag_t::from_bytes(b"rtlm"));
        let has_vert = ot_map.get_1_mask(hb_tag_t::from_bytes(b"vert")) != 0;

        let horizontal = self.direction.is_horizontal();
        let kern_tag = if horizontal {
            hb_tag_t::from_bytes(b"kern")
        } else {
            hb_tag_t::from_bytes(b"vkrn")
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
        let fallback_glyph_classes = !hb_ot_layout_has_glyph_classes(self.face);

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
            } else if hb_ot_layout_has_kerning(self.face) {
                apply_kern = true;
            }
        }

        let apply_fallback_kern = !(apply_gpos || apply_kerx || apply_kern);
        let zero_marks = self.script_zero_marks
            && !apply_kerx
            && (!apply_kern || !hb_ot_layout_has_machine_kerning(self.face));

        let has_gpos_mark = ot_map.get_1_mask(hb_tag_t::from_bytes(b"mark")) != 0;

        let mut adjust_mark_positioning_when_zeroing = !apply_gpos
            && !apply_kerx
            && (!apply_kern || !hb_ot_layout_has_cross_kerning(self.face));

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
