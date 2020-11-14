use std::ffi::c_void;

use crate::{aat, feature, ffi, Direction, Face, Feature, Language, Mask, Tag, Script};
use crate::complex::{complex_categorize, ComplexShaper, DEFAULT_SHAPER, DUMBER_SHAPER};
use crate::ot::{self, FeatureFlags, Map, TableIndex};
use crate::tables::gsubgpos::VariationIndex;

pub struct ShapePlan {
    pub direction: Direction,
    pub script: Option<Script>,
    pub shaper: &'static ComplexShaper,
    pub ot_map: ot::Map,
    pub aat_map: aat::Map,
    pub data: *mut c_void,

    pub frac_mask: Mask,
    pub numr_mask: Mask,
    pub dnom_mask: Mask,
    pub rtlm_mask: Mask,
    pub kern_mask: Mask,
    pub trak_mask: Mask,

    pub requested_kerning: bool,
    pub requested_tracking: bool,
    pub has_frac: bool,
    pub has_vert: bool,
    pub has_gpos_mark: bool,
    pub zero_marks: bool,
    pub fallback_glyph_classes: bool,
    pub fallback_mark_positioning: bool,
    pub adjust_mark_positioning_when_zeroing: bool,

    pub apply_gpos: bool,
    pub apply_kern: bool,
    pub apply_kerx: bool,
    pub apply_morx: bool,
    pub apply_trak: bool,
}

impl ShapePlan {
    pub fn new(
        face: &Face,
        direction: Direction,
        script: Option<Script>,
        language: Option<&Language>,
        user_features: &[Feature],
    ) -> Option<Self> {
        assert_ne!(direction, Direction::Invalid);

        let variation_indices = TableIndex::array(|index| {
            let coords = face.ttfp_face.variation_coordinates();
            face.layout_table(index)
                .and_then(|table| table.find_variation_index(coords))
        });

        let mut plan = Self {
            direction,
            script,
            shaper: &DEFAULT_SHAPER, // FIXME
            ot_map: Map::new(),
            aat_map: aat::Map::new(),
            data: std::ptr::null_mut(),
            frac_mask: 0,
            numr_mask: 0,
            dnom_mask: 0,
            rtlm_mask: 0,
            kern_mask: 0,
            trak_mask: 0,
            requested_kerning: false,
            requested_tracking: false,
            has_frac: false,
            has_vert: false,
            has_gpos_mark: false,
            zero_marks: false,
            fallback_glyph_classes: false,
            fallback_mark_positioning: false,
            adjust_mark_positioning_when_zeroing: false,
            apply_gpos: false,
            apply_kern: false,
            apply_kerx: false,
            apply_morx: false,
            apply_trak: false,
        };

        let mut planner = ShapePlanner::new(face, direction, script, language);
        collect_features(&mut planner, user_features);
        planner.compile(&mut plan, variation_indices);

        if let Some(func) = plan.shaper.data_create {
            let ptr = func(&plan);
            if ptr.is_null() {
                return None;
            }
            plan.data = ptr;
        }

        Some(plan)
    }

    #[inline]
    pub fn from_ptr(plan: *const ffi::rb_shape_plan_t) -> &'static Self {
        unsafe { &*(plan as *const Self) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const ffi::rb_shape_plan_t {
        self as *const _ as *const ffi::rb_shape_plan_t
    }
}

impl Drop for ShapePlan {
    fn drop(&mut self) {
        if let Some(func) = self.shaper.data_destroy {
            func(self.data);
        }
    }
}

// TODO: Tuple and use predefined feature constants.
const COMMON_FEATURES: &[(Tag, FeatureFlags)] = &[
    (feature::ABOVE_BASE_MARK_POSITIONING, FeatureFlags::GLOBAL),
    (feature::BELOW_BASE_MARK_POSITIONING, FeatureFlags::GLOBAL),
    (feature::GLYPH_COMPOSITION_DECOMPOSITION, FeatureFlags::GLOBAL),
    (feature::LOCALIZED_FORMS, FeatureFlags::GLOBAL),
    (feature::MARK_POSITIONING, FeatureFlags::GLOBAL_MANUAL_JOINERS),
    (feature::MARK_TO_MARK_POSITIONING, FeatureFlags::GLOBAL_MANUAL_JOINERS),
    (feature::REQUIRED_LIGATURES, FeatureFlags::GLOBAL),
];

const HORIZONTAL_FEATURES: &[(Tag, FeatureFlags)] = &[
    (feature::CONTEXTUAL_ALTERNATES, FeatureFlags::GLOBAL),
    (feature::CONTEXTUAL_LIGATURES, FeatureFlags::GLOBAL),
    (feature::CURSIVE_POSITIONING, FeatureFlags::GLOBAL),
    (feature::DISTANCES, FeatureFlags::GLOBAL),
    (feature::KERNING, FeatureFlags::GLOBAL_HAS_FALLBACK),
    (feature::STANDARD_LIGATURES, FeatureFlags::GLOBAL),
    (feature::REQUIRED_CONTEXTUAL_ALTERNATES, FeatureFlags::GLOBAL),
];

fn collect_features(planner: &mut ShapePlanner, user_features: &[Feature]) {
    planner.ot_map.enable_feature(feature::REQUIRED_VARIATION_ALTERNATES, FeatureFlags::empty(), 1);
    planner.ot_map.add_gsub_pause(None);

    match planner.direction {
        Direction::LeftToRight => {
            planner.ot_map.enable_feature(feature::LEFT_TO_RIGHT_ALTERNATES, FeatureFlags::empty(), 1);
            planner.ot_map.enable_feature(feature::LEFT_TO_RIGHT_MIRRORED_FORMS, FeatureFlags::empty(), 1);
        }
        Direction::RightToLeft => {
            planner.ot_map.enable_feature(feature::RIGHT_TO_LEFT_ALTERNATES, FeatureFlags::empty(), 1);
            planner.ot_map.add_feature(feature::RIGHT_TO_LEFT_MIRRORED_FORMS, FeatureFlags::empty(), 1);
        }
        _ => {}
    }

    // Automatic fractions.
    planner.ot_map.add_feature(feature::FRACTIONS, FeatureFlags::empty(), 1);
    planner.ot_map.add_feature(feature::NUMERATORS, FeatureFlags::empty(), 1);
    planner.ot_map.add_feature(feature::DENOMINATORS, FeatureFlags::empty(), 1);

    // Random!
    planner.ot_map.enable_feature(feature::RANDOMIZE, FeatureFlags::RANDOM, ot::Map::MAX_VALUE);

    // Tracking.  We enable dummy feature here just to allow disabling
    // AAT 'trak' table using features.
    // https://github.com/harfbuzz/harfbuzz/issues/1303
    planner.ot_map.enable_feature(Tag::from_bytes(b"trak"), FeatureFlags::HAS_FALLBACK, 1);

    planner.ot_map.enable_feature(Tag::from_bytes(b"HARF"), FeatureFlags::empty(), 1);

    if let Some(func) = planner.shaper.collect_features {
        func(planner);
    }

    planner.ot_map.enable_feature(Tag::from_bytes(b"BUZZ"), FeatureFlags::empty(), 1);

    for &(tag, flags) in COMMON_FEATURES {
        planner.ot_map.add_feature(tag, flags, 1);
    }

    if planner.direction.is_horizontal() {
        for &(tag, flags) in HORIZONTAL_FEATURES {
            planner.ot_map.add_feature(tag, flags, 1);
        }
    } else {
        // We really want to find a 'vert' feature if there's any in the font, no
        // matter which script/langsys it is listed (or not) under.
        // See various bugs referenced from:
        // https://github.com/harfbuzz/harfbuzz/issues/63
        planner.ot_map.enable_feature(feature::VERTICAL_WRITING, FeatureFlags::GLOBAL_SEARCH, 1);
    }

    for feature in user_features {
        let mut flags = FeatureFlags::empty();
        if feature.start == 0 && feature.end == u32::MAX {
            flags |= FeatureFlags::GLOBAL;
        }
        planner.ot_map.add_feature(feature.tag, flags, feature.value);
    }

    if planner.apply_morx {
        for feature in user_features {
            planner.aat_map.add_feature(feature.tag, feature.value);
        }
    }

    if let Some(func) = planner.shaper.override_features {
        func(planner);
    }
}

pub struct ShapePlanner<'a> {
    pub face: &'a Face<'a>,
    pub direction: Direction,
    pub script: Option<Script>,
    pub ot_map: ot::MapBuilder<'a>,
    pub aat_map: aat::MapBuilder<'a>,
    pub apply_morx: bool,
    pub script_zero_marks: bool,
    pub script_fallback_mark_positioning: bool,
    pub shaper: &'static ComplexShaper,
}

impl<'a> ShapePlanner<'a> {
    pub fn new(
        face: &'a Face<'a>,
        direction: Direction,
        script: Option<Script>,
        language: Option<&Language>,
    ) -> Self {
        // https://github.com/harfbuzz/harfbuzz/issues/2124
        let apply_morx = unsafe { ffi::rb_aat_layout_has_substitution(face.as_ptr()) != 0 }
            && (direction.is_horizontal() || !face.gsub.is_some());

        let ot_map = ot::MapBuilder::new(face, script, language);
        let aat_map = aat::MapBuilder::new(face);

        let mut shaper = match script {
            Some(script) => complex_categorize(
                script,
                direction,
                ot_map.chosen_script(TableIndex::GSUB),
            ),
            None => &DEFAULT_SHAPER,
        };

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
            script_zero_marks: shaper.zero_width_marks.is_some(),
            script_fallback_mark_positioning: shaper.fallback_position,
            shaper,
        }
    }

    pub fn compile(&mut self, plan: &mut ShapePlan, variation_indices: [Option<VariationIndex>; 2]) {
        plan.direction = self.direction;
        plan.script = self.script;
        plan.shaper = self.shaper;

        self.ot_map.compile(&mut plan.ot_map, variation_indices);
        if self.apply_morx {
            self.aat_map.compile(&mut plan.aat_map);
        }

        plan.frac_mask = plan.ot_map.one_mask(feature::FRACTIONS);
        plan.numr_mask = plan.ot_map.one_mask(feature::NUMERATORS);
        plan.dnom_mask = plan.ot_map.one_mask(feature::DENOMINATORS);
        plan.has_frac = plan.frac_mask != 0 || (plan.numr_mask != 0 && plan.dnom_mask != 0);

        plan.rtlm_mask = plan.ot_map.one_mask(feature::RIGHT_TO_LEFT_MIRRORED_FORMS);
        plan.has_vert = plan.ot_map.one_mask(feature::VERTICAL_WRITING) != 0;

        let horizontal = self.direction.is_horizontal();
        let kern_tag = if horizontal { feature::KERNING } else { feature::VERTICAL_KERNING };
        plan.kern_mask = plan.ot_map.mask(kern_tag).0;
        plan.requested_kerning = plan.kern_mask != 0;
        plan.trak_mask = plan.ot_map.mask(Tag::from_bytes(b"trak")).0;
        plan.requested_tracking = plan.trak_mask != 0;

        let has_gpos_kern = plan.ot_map.feature_index(TableIndex::GPOS, kern_tag).is_some();
        let disable_gpos = plan.shaper.gpos_tag.is_some()
            && plan.shaper.gpos_tag != plan.ot_map.chosen_script(TableIndex::GPOS);

        // Decide who provides glyph classes. GDEF or Unicode.
        if !self.face.ttfp_face.has_glyph_classes() {
            plan.fallback_glyph_classes = true;
        }

        // Decide who does substitutions. GSUB, morx, or fallback.
        plan.apply_morx = self.apply_morx;

        // Decide who does positioning. GPOS, kerx, kern, or fallback.
        if unsafe { ffi::rb_aat_layout_has_positioning(self.face.as_ptr()) != 0 } {
            plan.apply_kerx = true;
        } else if !plan.apply_morx && !disable_gpos && self.face.gpos.is_some() {
            plan.apply_gpos = true;
        }

        if !plan.apply_kerx && (!has_gpos_kern || !plan.apply_gpos) {
            // Apparently Apple applies kerx if GPOS kern was not applied.
            if unsafe { ffi::rb_aat_layout_has_positioning(self.face.as_ptr()) != 0 } {
                plan.apply_kerx = true;
            } else if unsafe { ffi::rb_ot_layout_has_kerning(self.face.as_ptr()) != 0 } {
                plan.apply_kern = true;
            }
        }

        plan.zero_marks =
            self.script_zero_marks
            && !plan.apply_kerx
            && (!plan.apply_kern || unsafe { ffi::rb_ot_layout_has_machine_kerning(self.face.as_ptr()) == 0 });

        plan.has_gpos_mark = plan.ot_map.one_mask(feature::MARK_POSITIONING) != 0;

        plan.adjust_mark_positioning_when_zeroing =
            !plan.apply_gpos
            && !plan.apply_kerx
            && (!plan.apply_kern || unsafe { ffi::rb_ot_layout_has_cross_kerning(self.face.as_ptr()) == 0 });

        plan.fallback_mark_positioning =
            plan.adjust_mark_positioning_when_zeroing
            && self.script_fallback_mark_positioning;

        // Currently we always apply trak.
        plan.apply_trak =
            plan.requested_tracking
            && unsafe { ffi::rb_aat_layout_has_tracking(self.face.as_ptr()) != 0 };
    }
}

#[no_mangle]
pub extern "C" fn rb_shape_plan_aat_map(plan: *const ffi::rb_shape_plan_t) -> *const ffi::rb_aat_map_t {
    ShapePlan::from_ptr(plan).aat_map.as_ptr()
}

#[no_mangle]
pub extern "C" fn rb_shape_plan_kern_mask(plan: *const ffi::rb_shape_plan_t) -> ffi::rb_mask_t {
    ShapePlan::from_ptr(plan).kern_mask
}

#[no_mangle]
pub extern "C" fn rb_shape_plan_trak_mask(plan: *const ffi::rb_shape_plan_t) -> ffi::rb_mask_t {
    ShapePlan::from_ptr(plan).trak_mask
}

#[no_mangle]
pub extern "C" fn rb_shape_plan_requested_kerning(plan: *const ffi::rb_shape_plan_t) -> ffi::rb_bool_t {
    ShapePlan::from_ptr(plan).requested_kerning as ffi::rb_bool_t
}
