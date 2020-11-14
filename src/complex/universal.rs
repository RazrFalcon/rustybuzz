use std::os::raw::c_void;

use crate::{feature, ffi, script, Tag, Face, GlyphInfo, Mask, Script};
use crate::buffer::{Buffer, BufferFlags};
use crate::ot::FeatureFlags;
use crate::plan::{ShapePlan, ShapePlanner};
use crate::unicode::{CharExt, GeneralCategoryExt};
use super::*;
use super::arabic::ArabicShapePlan;


pub const UNIVERSAL_SHAPER: ComplexShaper = ComplexShaper {
    collect_features: Some(collect_features),
    override_features: None,
    data_create: Some(data_create),
    data_destroy: Some(data_destroy),
    preprocess_text: Some(preprocess_text),
    postprocess_glyphs: None,
    normalization_mode: Some(ShapeNormalizationMode::ComposedDiacriticsNoShortCircuit),
    decompose: None,
    compose: Some(compose),
    setup_masks: Some(setup_masks),
    gpos_tag: None,
    reorder_marks: None,
    zero_width_marks: Some(ZeroWidthMarksMode::ByGdefEarly),
    fallback_position: false,
};


#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum Category {
    O       = 0,    // OTHER

    B       = 1,    // BASE
    IND     = 3,    // BASE_IND
    N       = 4,    // BASE_NUM
    GB      = 5,    // BASE_OTHER
    CGJ     = 6,    // CGJ
//    F       = 7,    // CONS_FINAL
    FM      = 8,    // CONS_FINAL_MOD
//    M       = 9,    // CONS_MED
//    CM      = 10,   // CONS_MOD
    SUB     = 11,   // CONS_SUB
    H       = 12,   // HALANT

    HN      = 13,   // HALANT_NUM
    ZWNJ    = 14,   // Zero width non-joiner
    ZWJ     = 15,   // Zero width joiner
    WJ      = 16,   // Word joiner
    Rsv     = 17,   // Reserved characters
    R       = 18,   // REPHA
    S       = 19,   // SYM
//    SM      = 20,   // SYM_MOD
    VS      = 21,   // VARIATION_SELECTOR
//    V       = 36,   // VOWEL
//    VM      = 40,   // VOWEL_MOD
    CS      = 43,   // CONS_WITH_STACKER

    // https://github.com/harfbuzz/harfbuzz/issues/1102
    HVM     = 44,   // HALANT_OR_VOWEL_MODIFIER

    Sk      = 48,   // SAKOT

    FAbv    = 24,   // CONS_FINAL_ABOVE
    FBlw    = 25,   // CONS_FINAL_BELOW
    FPst    = 26,   // CONS_FINAL_POST
    MAbv    = 27,   // CONS_MED_ABOVE
    MBlw    = 28,   // CONS_MED_BELOW
    MPst    = 29,   // CONS_MED_POST
    MPre    = 30,   // CONS_MED_PRE
    CMAbv   = 31,   // CONS_MOD_ABOVE
    CMBlw   = 32,   // CONS_MOD_BELOW
    VAbv    = 33,   // VOWEL_ABOVE / VOWEL_ABOVE_BELOW / VOWEL_ABOVE_BELOW_POST / VOWEL_ABOVE_POST
    VBlw    = 34,   // VOWEL_BELOW / VOWEL_BELOW_POST
    VPst    = 35,   // VOWEL_POST UIPC = Right
    VPre    = 22,   // VOWEL_PRE / VOWEL_PRE_ABOVE / VOWEL_PRE_ABOVE_POST / VOWEL_PRE_POST
    VMAbv   = 37,   // VOWEL_MOD_ABOVE
    VMBlw   = 38,   // VOWEL_MOD_BELOW
    VMPst   = 39,   // VOWEL_MOD_POST
    VMPre   = 23,   // VOWEL_MOD_PRE
    SMAbv   = 41,   // SYM_MOD_ABOVE
    SMBlw   = 42,   // SYM_MOD_BELOW
    FMAbv   = 45,   // CONS_FINAL_MOD UIPC = Top
    FMBlw   = 46,   // CONS_FINAL_MOD UIPC = Bottom
    FMPst   = 47,   // CONS_FINAL_MOD UIPC = Not_Applicable
}

// These features are applied all at once, before reordering.
const BASIC_FEATURES: &[Tag] = &[
    feature::RAKAR_FORMS,
    feature::ABOVE_BASE_FORMS,
    feature::BELOW_BASE_FORMS,
    feature::HALF_FORMS,
    feature::POST_BASE_FORMS,
    feature::VATTU_VARIANTS,
    feature::CONJUNCT_FORMS,
];

const TOPOGRAPHICAL_FEATURES: &[Tag] = &[
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
const OTHER_FEATURES: &[Tag] = &[
    feature::ABOVE_BASE_SUBSTITUTIONS,
    feature::BELOW_BASE_SUBSTITUTIONS,
    feature::HALANT_FORMS,
    feature::PRE_BASE_SUBSTITUTIONS,
    feature::POST_BASE_SUBSTITUTIONS,
];

impl GlyphInfo {
    fn use_category(&self) -> Category {
        unsafe {
            let v: &ffi::rb_var_int_t = std::mem::transmute(&self.var2);
            std::mem::transmute(v.var_u8[2])
        }
    }

    fn set_use_category(&mut self, c: Category) {
        unsafe {
            let v: &mut ffi::rb_var_int_t = std::mem::transmute(&mut self.var2);
            v.var_u8[2] = c as u8;
        }
    }

    fn is_halant_use(&self) -> bool {
        matches!(self.use_category(), Category::H | Category::HVM) && !self.is_ligated()
    }
}

struct UniversalShapePlan {
    rphf_mask: Mask,
    arabic_plan: Option<ArabicShapePlan>,
}

impl UniversalShapePlan {
    fn from_ptr(plan: *const c_void) -> &'static UniversalShapePlan {
        unsafe { &*(plan as *const UniversalShapePlan) }
    }
}

fn collect_features(planner: &mut ShapePlanner) {
    // Do this before any lookups have been applied.
    planner.ot_map.add_gsub_pause(Some(setup_syllables));

    // Default glyph pre-processing group
    planner.ot_map.enable_feature(feature::LOCALIZED_FORMS, FeatureFlags::NONE, 1);
    planner.ot_map.enable_feature(feature::GLYPH_COMPOSITION_DECOMPOSITION, FeatureFlags::NONE, 1);
    planner.ot_map.enable_feature(feature::NUKTA_FORMS, FeatureFlags::NONE, 1);
    planner.ot_map.enable_feature(feature::AKHANDS, FeatureFlags::MANUAL_ZWJ, 1);

    // Reordering group
    planner.ot_map.add_gsub_pause(Some(crate::ot::clear_substitution_flags));
    planner.ot_map.add_feature(feature::REPH_FORMS, FeatureFlags::MANUAL_ZWJ, 1);
    planner.ot_map.add_gsub_pause(Some(record_rphf));
    planner.ot_map.add_gsub_pause(Some(crate::ot::clear_substitution_flags));
    planner.ot_map.enable_feature(feature::PRE_BASE_FORMS, FeatureFlags::MANUAL_ZWJ, 1);
    planner.ot_map.add_gsub_pause(Some(record_pref));

    // Orthographic unit shaping group
    for feature in BASIC_FEATURES {
        planner.ot_map.enable_feature(*feature, FeatureFlags::MANUAL_ZWJ, 1);
    }

    planner.ot_map.add_gsub_pause(Some(reorder));
    planner.ot_map.add_gsub_pause(Some(crate::ot::clear_syllables));

    // Topographical features
    for feature in TOPOGRAPHICAL_FEATURES {
        planner.ot_map.add_feature(*feature, FeatureFlags::NONE, 1);
    }
    planner.ot_map.add_gsub_pause(None);

    // Standard typographic presentation
    for feature in OTHER_FEATURES {
        planner.ot_map.enable_feature(*feature, FeatureFlags::NONE, 1);
    }
}

fn setup_syllables(plan: &ShapePlan, _: &Face, buffer: &mut Buffer) {
    super::universal_machine::find_syllables(buffer);

    foreach_syllable!(buffer, start, end, {
        buffer.unsafe_to_break(start, end);
    });

    setup_rphf_mask(plan, buffer);
    setup_topographical_masks(plan, buffer);
}

fn setup_rphf_mask(plan: &ShapePlan, buffer: &mut Buffer) {
    let universal_plan = UniversalShapePlan::from_ptr(plan.data as _);

    let mask = universal_plan.rphf_mask;
    if mask == 0 {
        return;
    }

    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len {
        let limit = if buffer.info[start].use_category() == Category::R {
            1
        } else {
            std::cmp::min(3, end - start)
        };

        for i in start..start+limit {
            buffer.info[i].mask |= mask;
        }

        start = end;
        end = buffer.next_syllable(start);
    }
}

fn setup_topographical_masks(plan: &ShapePlan, buffer: &mut Buffer) {
    use super::universal_machine::SyllableType;

    let mut masks = [0; 4];
    let mut all_masks = 0;
    for i in 0..4 {
        masks[i] = plan.ot_map._1_mask(TOPOGRAPHICAL_FEATURES[i]);
        if masks[i] == plan.ot_map.global_mask() {
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
        if syllable == SyllableType::IndependentCluster as u8 ||
            syllable == SyllableType::SymbolCluster as u8 ||
            syllable == SyllableType::NonCluster as u8
        {
            last_form = None;
        } else {
            let join = last_form == Some(JoiningForm::Terminal) || last_form == Some(JoiningForm::Isolated);

            if join {
                // Fixup previous syllable's form.
                let form = if last_form == Some(JoiningForm::Terminal) {
                    JoiningForm::Medial
                } else {
                    JoiningForm::Initial
                };

                for i in last_start..start {
                    buffer.info[i].mask = (buffer.info[i].mask & other_masks) | masks[form as usize];
                }
            }

            // Form for this syllable.
            let form = if join { JoiningForm::Terminal } else { JoiningForm::Isolated };
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

fn record_rphf(plan: &ShapePlan, _: &Face, buffer: &mut Buffer) {
    let universal_plan = UniversalShapePlan::from_ptr(plan.data as _);

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

            if buffer.info[i].is_substituted() {
                buffer.info[i].set_use_category(Category::R);
                break;
            }
        }

        start = end;
        end = buffer.next_syllable(start);
    }
}

fn reorder(_: &ShapePlan, face: &Face, buffer: &mut Buffer) {
    insert_dotted_circles(face, buffer);

    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len {
        reorder_syllable(start, end, buffer);
        start = end;
        end = buffer.next_syllable(start);
    }
}

fn insert_dotted_circles(face: &Face, buffer: &mut Buffer) {
    use super::universal_machine::SyllableType;

    if buffer.flags.contains(BufferFlags::DO_NOT_INSERT_DOTTED_CIRCLE) {
        return;
    }

    // Note: This loop is extra overhead, but should not be measurable.
    // TODO Use a buffer scratch flag to remove the loop.
    let has_broken_syllables = buffer.info_slice().iter()
        .any(|info| info.syllable() & 0x0F == SyllableType::BrokenCluster as u8);

    if !has_broken_syllables {
        return;
    }

    let dottedcircle_glyph = match face.glyph_index(0x25CC) {
        Some(g) => g.0 as u32,
        None => return,
    };

    let mut dottedcircle = GlyphInfo {
        codepoint: dottedcircle_glyph,
        ..GlyphInfo::default()
    };
    dottedcircle.set_use_category(super::universal_table::get_category(0x25CC));

    buffer.clear_output();

    buffer.idx = 0;
    let mut last_syllable = 0;
    while buffer.idx < buffer.len {
        let syllable = buffer.cur(0).syllable();
        let syllable_type = syllable & 0x0F;
        if last_syllable != syllable && syllable_type == SyllableType::BrokenCluster as u8 {
            last_syllable = syllable;

            let mut ginfo = dottedcircle;
            ginfo.cluster = buffer.cur(0).cluster;
            ginfo.mask = buffer.cur(0).mask;
            ginfo.set_syllable(buffer.cur(0).syllable());

            // Insert dottedcircle after possible Repha.
            while buffer.idx < buffer.len &&
                last_syllable == buffer.cur(0).syllable() &&
                buffer.cur(0).use_category() == Category::R
            {
                buffer.next_glyph();
            }

            buffer.output_info(ginfo);
        } else {
            buffer.next_glyph();
        }
    }

    buffer.swap_buffers();
}

const fn category_flag(c: Category) -> u32 {
    rb_flag(c as u32)
}

const fn category_flag64(c: Category) -> u64 {
    rb_flag64(c as u32)
}

const BASE_FLAGS: u64 =
    category_flag64(Category::FM) |
    category_flag64(Category::FAbv) |
    category_flag64(Category::FBlw) |
    category_flag64(Category::FPst) |
    category_flag64(Category::MAbv) |
    category_flag64(Category::MBlw) |
    category_flag64(Category::MPst) |
    category_flag64(Category::MPre) |
    category_flag64(Category::VAbv) |
    category_flag64(Category::VBlw) |
    category_flag64(Category::VPst) |
    category_flag64(Category::VPre) |
    category_flag64(Category::VMAbv) |
    category_flag64(Category::VMBlw) |
    category_flag64(Category::VMPst) |
    category_flag64(Category::VMPre);

fn reorder_syllable(start: usize, end: usize, buffer: &mut Buffer) {
    use super::universal_machine::SyllableType;

    let syllable_type = (buffer.info[start].syllable() & 0x0F) as u32;
    // Only a few syllable types need reordering.
    if (rb_flag_unsafe(syllable_type) &
        (rb_flag(SyllableType::ViramaTerminatedCluster as u32) |
         rb_flag(SyllableType::SakotTerminatedCluster as u32) |
         rb_flag(SyllableType::StandardCluster as u32) |
         rb_flag(SyllableType::BrokenCluster as u32) |
            0)) == 0
    {
        return;
    }

    // Move things forward.
    if buffer.info[start].use_category() == Category::R && end - start > 1 {
        // Got a repha.  Reorder it towards the end, but before the first post-base glyph.
        for i in start+1..end {
            let is_post_base_glyph =
                (rb_flag64_unsafe(buffer.info[i].use_category() as u32) & BASE_FLAGS) != 0 ||
                    buffer.info[i].is_halant_use();

            if is_post_base_glyph || i == end - 1 {
                // If we hit a post-base glyph, move before it; otherwise move to the
                // end. Shift things in between backward.

                let mut i = i;
                if is_post_base_glyph {
                    i -= 1;
                }

                buffer.merge_clusters(start, i + 1);
                let t = buffer.info[start];
                for k in 0..i-start {
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
        } else if (flag & (category_flag(Category::VPre) | category_flag(Category::VMPre))) != 0 &&
            buffer.info[i].lig_comp() == 0 && j < i
        {
            // Only move the first component of a MultipleSubst.
            buffer.merge_clusters(j, i + 1);
            let t = buffer.info[i];
            for k in (0..i-j).rev() {
                buffer.info[k + j + 1] = buffer.info[k + j];
            }
            buffer.info[j] = t;
        }
    }
}

fn record_pref(_: &ShapePlan, _: &Face, buffer: &mut Buffer) {
    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len {
        // Mark a substituted pref as VPre, as they behave the same way.
        for i in start..end {
            if buffer.info[i].is_substituted() {
                buffer.info[i].set_use_category(Category::VPre);
                break;
            }
        }

        start = end;
        end = buffer.next_syllable(start);
    }
}

fn data_create(plan: &ShapePlan) -> *mut c_void {
    let mut arabic_plan = None;

    if has_arabic_joining(plan.script) {
        arabic_plan = Some(super::arabic::data_create_inner(plan));
    }

    let universal_plan = UniversalShapePlan {
        rphf_mask: plan.ot_map._1_mask(feature::REPH_FORMS),
        arabic_plan,
    };

    Box::into_raw(Box::new(universal_plan)) as _
}

fn has_arabic_joining(script: Script) -> bool {
    // List of scripts that have data in arabic-table.
    match script {
        // Unicode-1.1 additions.
        script::ARABIC |

        // Unicode-3.0 additions.
        script::MONGOLIAN |
        script::SYRIAC |

        // Unicode-5.0 additions.
        script::NKO |
        script::PHAGS_PA |

        // Unicode-6.0 additions.
        script::MANDAIC |

        // Unicode-7.0 additions.
        script::MANICHAEAN |
        script::PSALTER_PAHLAVI |

        // Unicode-9.0 additions.
        script::ADLAM => true,

        _ => false,
    }
}

fn data_destroy(data: *mut c_void) {
    unsafe { Box::from_raw(data as *mut UniversalShapePlan) };
}

fn preprocess_text(_: &ShapePlan, _: &Face, buffer: &mut Buffer) {
    super::vowel_constraints::preprocess_text_vowel_constraints(buffer);
}

fn compose(_: &ShapeNormalizeContext, a: char, b: char) -> Option<char> {
    // Avoid recomposing split matras.
    if a.general_category().is_mark() {
        return None;
    }

    crate::unicode::compose(a, b)
}

fn setup_masks(plan: &ShapePlan, _: &Face, buffer: &mut Buffer) {
    let universal_plan = UniversalShapePlan::from_ptr(plan.data as _);

    // Do this before allocating use_category().
    if let Some(ref arabic_plan) = universal_plan.arabic_plan {
        super::arabic::setup_masks_inner(arabic_plan, plan.script, buffer);
    }

    // We cannot setup masks here. We save information about characters
    // and setup masks later on in a pause-callback.
    for info in buffer.info_slice_mut() {
        info.set_use_category(super::universal_table::get_category(info.codepoint));
    }
}
