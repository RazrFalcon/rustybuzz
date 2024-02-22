use alloc::boxed::Box;

use super::ot_shape_normalize::hb_ot_shape_normalize_context_t;
use crate::hb::buffer::hb_buffer_t;
use crate::hb::feature;
use crate::hb::ot_layout::*;
use crate::hb::ot_map::FeatureFlags;
use crate::hb::ot_shape_complex::*;
use crate::hb::ot_shape_complex_arabic::ArabicShapePlan;
use crate::hb::ot_shape_normalize::HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT;
use crate::hb::shape_plan::{hb_ot_shape_plan_t, ShapePlanner};
use crate::hb::unicode::{CharExt, GeneralCategoryExt};
use crate::hb::{hb_font_t, hb_glyph_info_t, hb_mask_t, hb_tag_t, script, Script};

pub const UNIVERSAL_SHAPER: ComplexShaper = ComplexShaper {
    collect_features: Some(collect_features),
    override_features: None,
    create_data: Some(|plan| Box::new(UniversalShapePlan::new(plan))),
    preprocess_text: Some(preprocess_text),
    postprocess_glyphs: None,
    normalization_preference: HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT,
    decompose: None,
    compose: Some(compose),
    setup_masks: Some(setup_masks),
    gpos_tag: None,
    reorder_marks: None,
    zero_width_marks: Some(ZeroWidthMarksMode::ByGdefEarly),
    fallback_position: false,
};

pub type Category = u8;
#[allow(dead_code)]
pub mod category {
    pub const O: u8 = 0; // OTHER

    pub const B: u8 = 1; // BASE

    // pub const IND: u8     = 3;    // BASE_IND

    pub const N: u8 = 4; // BASE_NUM
    pub const GB: u8 = 5; // BASE_OTHER
    pub const CGJ: u8 = 6;

    // pub const CGJ: u8     = 6;    // CGJ
    // pub const F: u8       = 7;    // CONS_FINAL
    // pub const FM: u8 = 8;         // CONS_FINAL_MOD
    // pub const M: u8       = 9;    // CONS_MED
    // pub const CM: u8      = 10;   // CONS_MOD

    pub const SUB: u8 = 11; // CONS_SUB
    pub const H: u8 = 12; // HALANT

    pub const HN: u8 = 13; // HALANT_NUM
    pub const ZWNJ: u8 = 14; // Zero width non-joiner

    // pub const ZWJ: u8     = 15;   // Zero width joiner
    pub const WJ: u8 = 16; // Word joiner

    pub const RSV: u8 = 17; // Reserved characters
    pub const R: u8 = 18; // REPHA
    pub const S: u8 = 19; // SYM

    // pub const SM: u8      = 20;   // SYM_MOD
    // pub const VS: u8      = 21;   // VARIATION_SELECTOR
    // pub const V: u8       = 36;   // VOWEL
    // pub const VM: u8      = 40;   // VOWEL_MOD

    pub const CS: u8 = 43; // CONS_WITH_STACKER

    // https://github.com/harfbuzz/harfbuzz/issues/1102
    pub const IS: u8 = 44; // HALANT_OR_VOWEL_MODIFIER

    pub const SK: u8 = 48; // SAKOT

    pub const FABV: u8 = 24; // CONS_FINAL_ABOVE
    pub const FBLW: u8 = 25; // CONS_FINAL_BELOW
    pub const FPST: u8 = 26; // CONS_FINAL_POST
    pub const MABV: u8 = 27; // CONS_MED_ABOVE
    pub const MBLW: u8 = 28; // CONS_MED_BELOW
    pub const MPST: u8 = 29; // CONS_MED_POST
    pub const MPRE: u8 = 30; // CONS_MED_PRE
    pub const CMABV: u8 = 31; // CONS_MOD_ABOVE
    pub const CMBLW: u8 = 32; // CONS_MOD_BELOW
    pub const VABV: u8 = 33; // VOWEL_ABOVE / VOWEL_ABOVE_BELOW / VOWEL_ABOVE_BELOW_POST / VOWEL_ABOVE_POST
    pub const VBLW: u8 = 34; // VOWEL_BELOW / VOWEL_BELOW_POST
    pub const VPST: u8 = 35; // VOWEL_POST UIPC = Right
    pub const VPRE: u8 = 22; // VOWEL_PRE / VOWEL_PRE_ABOVE / VOWEL_PRE_ABOVE_POST / VOWEL_PRE_POST
    pub const VMABV: u8 = 37; // VOWEL_MOD_ABOVE
    pub const VMBLW: u8 = 38; // VOWEL_MOD_BELOW
    pub const VMPST: u8 = 39; // VOWEL_MOD_POST
    pub const VMPRE: u8 = 23; // VOWEL_MOD_PRE
    pub const SMABV: u8 = 41; // SYM_MOD_ABOVE
    pub const SMBLW: u8 = 42; // SYM_MOD_BELOW
    pub const FMABV: u8 = 45; // CONS_FINAL_MOD UIPC = Top
    pub const FMBLW: u8 = 46; // CONS_FINAL_MOD UIPC = Bottom
    pub const FMPST: u8 = 47; // CONS_FINAL_MOD UIPC = Not_Applicable
    pub const G: u8 = 49; // HIEROGLYPH
    pub const J: u8 = 50; // HIEROGLYPH_JOINER
    pub const SB: u8 = 51; // HIEROGLYPH_SEGMENT_BEGIN
    pub const SE: u8 = 52; // HIEROGLYPH_SEGMENT_END
}

// These features are applied all at once, before reordering,
// constrained to the syllable.
const BASIC_FEATURES: &[hb_tag_t] = &[
    feature::RAKAR_FORMS,
    feature::ABOVE_BASE_FORMS,
    feature::BELOW_BASE_FORMS,
    feature::HALF_FORMS,
    feature::POST_BASE_FORMS,
    feature::VATTU_VARIANTS,
    feature::CONJUNCT_FORMS,
];

const TOPOGRAPHICAL_FEATURES: &[hb_tag_t] = &[
    feature::ISOLATED_FORMS,
    feature::INITIAL_FORMS,
    feature::MEDIAL_FORMS_1,
    feature::TERMINAL_FORMS_1,
];

// Same order as use_topographical_features.
#[derive(Clone, Copy, PartialEq)]
enum JoiningForm {
    Isolated = 0,
    Initial,
    Medial,
    Terminal,
}

// These features are applied all at once, after reordering and clearing syllables.
const OTHER_FEATURES: &[hb_tag_t] = &[
    feature::ABOVE_BASE_SUBSTITUTIONS,
    feature::BELOW_BASE_SUBSTITUTIONS,
    feature::HALANT_FORMS,
    feature::PRE_BASE_SUBSTITUTIONS,
    feature::POST_BASE_SUBSTITUTIONS,
];

impl hb_glyph_info_t {
    pub(crate) fn use_category(&self) -> Category {
        self.complex_var_u8_category()
    }

    fn set_use_category(&mut self, c: Category) {
        self.set_complex_var_u8_category(c)
    }

    fn is_halant_use(&self) -> bool {
        matches!(self.use_category(), category::H | category::IS) && !_hb_glyph_info_ligated(self)
    }
}

struct UniversalShapePlan {
    rphf_mask: hb_mask_t,
    arabic_plan: Option<ArabicShapePlan>,
}

impl UniversalShapePlan {
    fn new(plan: &hb_ot_shape_plan_t) -> UniversalShapePlan {
        let mut arabic_plan = None;

        if plan.script.map_or(false, has_arabic_joining) {
            arabic_plan = Some(crate::hb::ot_shape_complex_arabic::ArabicShapePlan::new(
                plan,
            ));
        }

        UniversalShapePlan {
            rphf_mask: plan.ot_map.get_1_mask(feature::REPH_FORMS),
            arabic_plan,
        }
    }
}

fn collect_features(planner: &mut ShapePlanner) {
    // Do this before any lookups have been applied.
    planner.ot_map.add_gsub_pause(Some(setup_syllables));

    // Default glyph pre-processing group
    planner
        .ot_map
        .enable_feature(feature::LOCALIZED_FORMS, FeatureFlags::empty(), 1);
    planner.ot_map.enable_feature(
        feature::GLYPH_COMPOSITION_DECOMPOSITION,
        FeatureFlags::empty(),
        1,
    );
    planner
        .ot_map
        .enable_feature(feature::NUKTA_FORMS, FeatureFlags::empty(), 1);
    planner
        .ot_map
        .enable_feature(feature::AKHANDS, FeatureFlags::MANUAL_ZWJ, 1);

    // Reordering group
    planner
        .ot_map
        .add_gsub_pause(Some(crate::hb::ot_layout::_hb_clear_substitution_flags));
    planner
        .ot_map
        .add_feature(feature::REPH_FORMS, FeatureFlags::MANUAL_ZWJ, 1);
    planner.ot_map.add_gsub_pause(Some(record_rphf));
    planner
        .ot_map
        .add_gsub_pause(Some(crate::hb::ot_layout::_hb_clear_substitution_flags));
    planner
        .ot_map
        .enable_feature(feature::PRE_BASE_FORMS, FeatureFlags::MANUAL_ZWJ, 1);
    planner.ot_map.add_gsub_pause(Some(record_pref));

    // Orthographic unit shaping group
    for feature in BASIC_FEATURES {
        planner
            .ot_map
            .enable_feature(*feature, FeatureFlags::MANUAL_ZWJ, 1);
    }

    planner.ot_map.add_gsub_pause(Some(reorder));
    planner
        .ot_map
        .add_gsub_pause(Some(crate::hb::ot_layout::_hb_clear_syllables));

    // Topographical features
    for feature in TOPOGRAPHICAL_FEATURES {
        planner
            .ot_map
            .add_feature(*feature, FeatureFlags::empty(), 1);
    }
    planner.ot_map.add_gsub_pause(None);

    // Standard typographic presentation
    for feature in OTHER_FEATURES {
        planner
            .ot_map
            .enable_feature(*feature, FeatureFlags::empty(), 1);
    }
}

fn setup_syllables(plan: &hb_ot_shape_plan_t, _: &hb_font_t, buffer: &mut hb_buffer_t) {
    super::ot_shape_complex_use_machine::find_syllables(buffer);

    foreach_syllable!(buffer, start, end, {
        buffer.unsafe_to_break(Some(start), Some(end));
    });

    setup_rphf_mask(plan, buffer);
    setup_topographical_masks(plan, buffer);
}

fn setup_rphf_mask(plan: &hb_ot_shape_plan_t, buffer: &mut hb_buffer_t) {
    let universal_plan = plan.data::<UniversalShapePlan>();

    let mask = universal_plan.rphf_mask;
    if mask == 0 {
        return;
    }

    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len {
        let limit = if buffer.info[start].use_category() == category::R {
            1
        } else {
            core::cmp::min(3, end - start)
        };

        for i in start..start + limit {
            buffer.info[i].mask |= mask;
        }

        start = end;
        end = buffer.next_syllable(start);
    }
}

fn setup_topographical_masks(plan: &hb_ot_shape_plan_t, buffer: &mut hb_buffer_t) {
    use super::ot_shape_complex_use_machine::SyllableType;

    if plan.data::<UniversalShapePlan>().arabic_plan.is_some() {
        return;
    }

    let mut masks = [0; 4];
    let mut all_masks = 0;
    for i in 0..4 {
        masks[i] = plan.ot_map.get_1_mask(TOPOGRAPHICAL_FEATURES[i]);
        if masks[i] == plan.ot_map.get_global_mask() {
            masks[i] = 0;
        }

        all_masks |= masks[i];
    }

    if all_masks == 0 {
        return;
    }

    let other_masks = !all_masks;

    let mut last_start = 0;
    let mut last_form = None;
    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len {
        let syllable = buffer.info[start].syllable() & 0x0F;
        if syllable == SyllableType::HieroglyphCluster as u8
            || syllable == SyllableType::NonCluster as u8
        {
            last_form = None;
        } else {
            let join = last_form == Some(JoiningForm::Terminal)
                || last_form == Some(JoiningForm::Isolated);

            if join {
                // Fixup previous syllable's form.
                let form = if last_form == Some(JoiningForm::Terminal) {
                    JoiningForm::Medial
                } else {
                    JoiningForm::Initial
                };

                for i in last_start..start {
                    buffer.info[i].mask =
                        (buffer.info[i].mask & other_masks) | masks[form as usize];
                }
            }

            // Form for this syllable.
            let form = if join {
                JoiningForm::Terminal
            } else {
                JoiningForm::Isolated
            };
            last_form = Some(form);
            for i in start..end {
                buffer.info[i].mask = (buffer.info[i].mask & other_masks) | masks[form as usize];
            }
        }

        last_start = start;
        start = end;
        end = buffer.next_syllable(start);
    }
}

fn record_rphf(plan: &hb_ot_shape_plan_t, _: &hb_font_t, buffer: &mut hb_buffer_t) {
    let universal_plan = plan.data::<UniversalShapePlan>();

    let mask = universal_plan.rphf_mask;
    if mask == 0 {
        return;
    }

    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len {
        // Mark a substituted repha as USE_R.
        for i in start..end {
            if buffer.info[i].mask & mask == 0 {
                break;
            }

            if _hb_glyph_info_substituted(&buffer.info[i]) {
                buffer.info[i].set_use_category(category::R);
                break;
            }
        }

        start = end;
        end = buffer.next_syllable(start);
    }
}

fn reorder(_: &hb_ot_shape_plan_t, face: &hb_font_t, buffer: &mut hb_buffer_t) {
    use super::ot_shape_complex_use_machine::SyllableType;

    crate::hb::ot_shape_complex_syllabic::insert_dotted_circles(
        face,
        buffer,
        SyllableType::BrokenCluster as u8,
        category::B,
        Some(category::R),
        None,
    );

    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len {
        reorder_syllable(start, end, buffer);
        start = end;
        end = buffer.next_syllable(start);
    }
}

const fn category_flag(c: Category) -> u32 {
    rb_flag(c as u32)
}

const fn category_flag64(c: Category) -> u64 {
    rb_flag64(c as u32)
}

const BASE_FLAGS: u64 = category_flag64(category::FABV)
    | category_flag64(category::FBLW)
    | category_flag64(category::FPST)
    | category_flag64(category::MABV)
    | category_flag64(category::MBLW)
    | category_flag64(category::MPST)
    | category_flag64(category::MPRE)
    | category_flag64(category::VABV)
    | category_flag64(category::VBLW)
    | category_flag64(category::VPST)
    | category_flag64(category::VPRE)
    | category_flag64(category::VMABV)
    | category_flag64(category::VMBLW)
    | category_flag64(category::VMPST)
    | category_flag64(category::VMPRE);

fn reorder_syllable(start: usize, end: usize, buffer: &mut hb_buffer_t) {
    use super::ot_shape_complex_use_machine::SyllableType;

    let syllable_type = (buffer.info[start].syllable() & 0x0F) as u32;
    // Only a few syllable types need reordering.
    if (rb_flag_unsafe(syllable_type)
        & (rb_flag(SyllableType::ViramaTerminatedCluster as u32)
            | rb_flag(SyllableType::SakotTerminatedCluster as u32)
            | rb_flag(SyllableType::StandardCluster as u32)
            | rb_flag(SyllableType::BrokenCluster as u32)
            | 0))
        == 0
    {
        return;
    }

    // Move things forward.
    if buffer.info[start].use_category() == category::R && end - start > 1 {
        // Got a repha.  Reorder it towards the end, but before the first post-base glyph.
        for i in start + 1..end {
            let is_post_base_glyph =
                (rb_flag64_unsafe(buffer.info[i].use_category() as u32) & BASE_FLAGS) != 0
                    || buffer.info[i].is_halant_use();

            if is_post_base_glyph || i == end - 1 {
                // If we hit a post-base glyph, move before it; otherwise move to the
                // end. Shift things in between backward.

                let mut i = i;
                if is_post_base_glyph {
                    i -= 1;
                }

                buffer.merge_clusters(start, i + 1);
                let t = buffer.info[start];
                for k in 0..i - start {
                    buffer.info[k + start] = buffer.info[k + start + 1];
                }
                buffer.info[i] = t;

                break;
            }
        }
    }

    // Move things back.
    let mut j = start;
    for i in start..end {
        let flag = rb_flag_unsafe(buffer.info[i].use_category() as u32);
        if buffer.info[i].is_halant_use() {
            // If we hit a halant, move after it; otherwise move to the beginning, and
            // shift things in between forward.
            j = i + 1;
        } else if (flag & (category_flag(category::VPRE) | category_flag(category::VMPRE))) != 0
            && _hb_glyph_info_get_lig_comp(&buffer.info[i]) == 0
            && j < i
        {
            // Only move the first component of a MultipleSubst.
            buffer.merge_clusters(j, i + 1);
            let t = buffer.info[i];
            for k in (0..i - j).rev() {
                buffer.info[k + j + 1] = buffer.info[k + j];
            }
            buffer.info[j] = t;
        }
    }
}

fn record_pref(_: &hb_ot_shape_plan_t, _: &hb_font_t, buffer: &mut hb_buffer_t) {
    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len {
        // Mark a substituted pref as VPre, as they behave the same way.
        for i in start..end {
            if _hb_glyph_info_substituted(&buffer.info[i]) {
                buffer.info[i].set_use_category(category::VPRE);
                break;
            }
        }

        start = end;
        end = buffer.next_syllable(start);
    }
}

fn has_arabic_joining(script: Script) -> bool {
    // List of scripts that have data in arabic-table.
    matches!(
        script,
        script::ADLAM
            | script::ARABIC
            | script::CHORASMIAN
            | script::HANIFI_ROHINGYA
            | script::MANDAIC
            | script::MANICHAEAN
            | script::MONGOLIAN
            | script::NKO
            | script::OLD_UYGHUR
            | script::PHAGS_PA
            | script::PSALTER_PAHLAVI
            | script::SOGDIAN
            | script::SYRIAC
    )
}

fn preprocess_text(_: &hb_ot_shape_plan_t, _: &hb_font_t, buffer: &mut hb_buffer_t) {
    super::ot_shape_complex_vowel_constraints::preprocess_text_vowel_constraints(buffer);
}

fn compose(_: &hb_ot_shape_normalize_context_t, a: char, b: char) -> Option<char> {
    // Avoid recomposing split matras.
    if a.general_category().is_mark() {
        return None;
    }

    crate::hb::unicode::compose(a, b)
}

fn setup_masks(plan: &hb_ot_shape_plan_t, _: &hb_font_t, buffer: &mut hb_buffer_t) {
    let universal_plan = plan.data::<UniversalShapePlan>();

    // Do this before allocating use_category().
    if let Some(ref arabic_plan) = universal_plan.arabic_plan {
        crate::hb::ot_shape_complex_arabic::setup_masks_inner(arabic_plan, plan.script, buffer);
    }

    // We cannot setup masks here. We save information about characters
    // and setup masks later on in a pause-callback.
    for info in buffer.info_slice_mut() {
        info.set_use_category(super::ot_shape_complex_use_table::get_category(info));
    }
}
