//! Universal Shaping Engine.
//! https://docs.microsoft.com/en-us/typography/script-development/use

use std::convert::TryFrom;
use std::os::raw::c_void;

use crate::{ffi, CodePoint, Tag, Buffer, GlyphInfo};
use crate::buffer::BufferFlags;
use crate::map::{Map, MapBuilder, FeatureFlags};
use crate::unicode::{CharExt, GeneralCategoryExt};
use super::{hb_flag64, hb_flag64_unsafe, hb_flag, hb_flag_unsafe};

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

#[no_mangle]
pub extern "C" fn rb_complex_universal_get_category(u: CodePoint) -> u8 {
    super::universal_table::get_category(u) as u8
}

extern "C" {
    fn hb_complex_universal_setup_syllables(
        plan: *const ffi::hb_shape_plan_t,
        font: *mut ffi::hb_font_t,
        buffer: *mut ffi::rb_buffer_t,
    );

    fn hb_complex_universal_record_rphf(
        plan: *const ffi::hb_shape_plan_t,
        font: *mut ffi::hb_font_t,
        buffer: *mut ffi::rb_buffer_t,
    );
}

// These features are applied all at once, before reordering.
const BASIC_FEATURES: &[Tag] = &[
    Tag::from_bytes(b"rkrf"),
    Tag::from_bytes(b"abvf"),
    Tag::from_bytes(b"blwf"),
    Tag::from_bytes(b"half"),
    Tag::from_bytes(b"pstf"),
    Tag::from_bytes(b"vatu"),
    Tag::from_bytes(b"cjct"),
];

const TOPOGRAPHICAL_FEATURES: &[Tag] = &[
    Tag::from_bytes(b"isol"),
    Tag::from_bytes(b"init"),
    Tag::from_bytes(b"medi"),
    Tag::from_bytes(b"fina"),
];

// Same order as use_topographical_features.
#[derive(Clone, Copy, PartialEq)]
enum JoiningForm {
    ISOL = 0,
    INIT,
    MEDI,
    FINA,
}

// These features are applied all at once, after reordering and clearing syllables.
const OTHER_FEATURES: &[Tag] = &[
    Tag::from_bytes(b"abvs"),
    Tag::from_bytes(b"blws"),
    Tag::from_bytes(b"haln"),
    Tag::from_bytes(b"pres"),
    Tag::from_bytes(b"psts"),
];

impl GlyphInfo {
    fn use_category(&self) -> Category {
        unsafe {
            let v: &ffi::hb_var_int_t = std::mem::transmute(&self.var2);
            std::mem::transmute(v.var_u8[2])
        }
    }

    fn set_use_category(&mut self, c: Category) {
        unsafe {
            let v: &mut ffi::hb_var_int_t = std::mem::transmute(&mut self.var2);
            v.var_u8[2] = c as u8;
        }
    }

    fn is_halant_use(&self) -> bool {
        matches!(self.use_category(), Category::H | Category::HVM) && !self.is_ligated()
    }
}

#[no_mangle]
pub extern "C" fn rb_complex_universal_collect_features(planner: *mut ffi::hb_ot_shape_planner_t) {
    let builder = {
        let map = unsafe { ffi::hb_ot_shape_planner_map(planner) };
        MapBuilder::from_ptr_mut(map)
    };

    // Do this before any lookups have been applied.
    builder.add_gsub_pause(Some(hb_complex_universal_setup_syllables));

    // Default glyph pre-processing group
    builder.enable_feature(Tag::from_bytes(b"locl"), FeatureFlags::default(), 1);
    builder.enable_feature(Tag::from_bytes(b"ccmp"), FeatureFlags::default(), 1);
    builder.enable_feature(Tag::from_bytes(b"nukt"), FeatureFlags::default(), 1);
    builder.enable_feature(Tag::from_bytes(b"akhn"), FeatureFlags::MANUAL_ZWJ, 1);

    // Reordering group
    builder.add_gsub_pause(Some(rb_complex_universal_clear_substitution_flags));
    builder.add_feature(Tag::from_bytes(b"rphf"), FeatureFlags::MANUAL_ZWJ, 1);
    builder.add_gsub_pause(Some(hb_complex_universal_record_rphf));
    builder.add_gsub_pause(Some(rb_complex_universal_clear_substitution_flags));
    builder.enable_feature(Tag::from_bytes(b"pref"), FeatureFlags::MANUAL_ZWJ, 1);
    builder.add_gsub_pause(Some(rb_complex_universal_record_pref));

    // Orthographic unit shaping group
    for feature in BASIC_FEATURES {
        builder.enable_feature(*feature, FeatureFlags::MANUAL_ZWJ, 1);
    }

    builder.add_gsub_pause(Some(rb_complex_universal_reorder));
    builder.add_gsub_pause(Some(super::hb_layout_clear_syllables));

    // Topographical features
    for feature in TOPOGRAPHICAL_FEATURES {
        builder.add_feature(*feature, FeatureFlags::default(), 1);
    }
    builder.add_gsub_pause(None);

    // Standard typographic presentation
    for feature in OTHER_FEATURES {
        builder.enable_feature(*feature, FeatureFlags::default(), 1);
    }
}

fn insert_dotted_circles(font: &ttf_parser::Font, buffer: &mut Buffer) {
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

    let dottedcircle_glyph = match font.glyph_index('\u{25CC}') {
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
    while buffer.idx < buffer.len() {
        let syllable = buffer.cur(0).syllable();
        let syllable_type = syllable & 0x0F;
        if last_syllable != syllable && syllable_type == SyllableType::BrokenCluster as u8 {
            last_syllable = syllable;

            let mut ginfo = dottedcircle;
            ginfo.cluster = buffer.cur(0).cluster;
            ginfo.mask = buffer.cur(0).mask;
            ginfo.set_syllable(buffer.cur(0).syllable());

            // Insert dottedcircle after possible Repha.
            while buffer.idx < buffer.len() &&
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

#[no_mangle]
pub extern "C" fn rb_complex_universal_insert_dotted_circles(
    ttf_parser_data: *const ffi::rb_ttf_parser_t,
    buffer: *mut ffi::rb_buffer_t,
) {
    insert_dotted_circles(
        crate::font::ttf_parser_from_raw(ttf_parser_data),
        Buffer::from_ptr_mut(buffer),
    )
}

#[no_mangle]
pub extern "C" fn rb_complex_universal_find_syllables(buffer: *mut ffi::rb_buffer_t) {
    super::universal_machine::find_syllables(Buffer::from_ptr_mut(buffer))
}

fn setup_topographical_masks(map: &Map, buffer: &mut Buffer) {
    use super::universal_machine::SyllableType;

    let mut masks = [0; 4];
    let mut all_masks = 0;
    for i in 0..4 {
        masks[i] = map.get_1_mask(TOPOGRAPHICAL_FEATURES[i]);
        if masks[i] == map.global_mask {
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
    while start < buffer.len() {
        let syllable = buffer.info[start].syllable() & 0x0F;
        if syllable == SyllableType::IndependentCluster as u8 ||
            syllable == SyllableType::SymbolCluster as u8 ||
            syllable == SyllableType::NonCluster as u8
        {
            last_form = None;
        } else {
            let join = last_form == Some(JoiningForm::FINA) || last_form == Some(JoiningForm::ISOL);

            if join {
                // Fixup previous syllable's form.
                let form = if last_form == Some(JoiningForm::FINA) {
                    JoiningForm::MEDI
                } else {
                    JoiningForm::INIT
                };

                for i in last_start..start {
                    buffer.info[i].mask = (buffer.info[i].mask & other_masks) | masks[form as usize];
                }
            }

            // Form for this syllable.
            let form = if join { JoiningForm::FINA } else { JoiningForm::ISOL };
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

#[no_mangle]
pub extern "C" fn rb_complex_universal_setup_topographical_masks(
    map: *const ffi::rb_ot_map_t,
    buffer: *mut ffi::rb_buffer_t,
) {
    setup_topographical_masks(
        Map::from_ptr(map),
        Buffer::from_ptr_mut(buffer),
    )
}

const fn category_flag(c: Category) -> u32 {
    hb_flag(c as u32)
}

const fn category_flag64(c: Category) -> u64 {
    hb_flag64(c as u32)
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
    if (hb_flag_unsafe(syllable_type) &
        (hb_flag(SyllableType::ViramaTerminatedCluster as u32) |
         hb_flag(SyllableType::SakotTerminatedCluster as u32) |
         hb_flag(SyllableType::StandardCluster as u32) |
         hb_flag(SyllableType::BrokenCluster as u32) |
         0)) == 0
    {
        return;
    }

    // Move things forward.
    if buffer.info[start].use_category() == Category::R && end - start > 1 {
        // Got a repha.  Reorder it towards the end, but before the first post-base glyph.
        for i in start+1..end {
            let is_post_base_glyph =
                (hb_flag64_unsafe(buffer.info[i].use_category() as u32) & BASE_FLAGS) != 0 ||
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
        let flag = hb_flag_unsafe(buffer.info[i].use_category() as u32);
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

#[no_mangle]
pub extern "C" fn rb_complex_universal_reorder_syllable(
    start: u32,
    end: u32,
    buffer: *mut ffi::rb_buffer_t,
) {
    reorder_syllable(
        start as usize,
        end as usize,
        Buffer::from_ptr_mut(buffer),
    )
}

#[no_mangle]
pub extern "C" fn rb_complex_universal_compose(
    _: *const ffi::hb_ot_shape_normalize_context_t,
    a: ffi::hb_codepoint_t,
    b: ffi::hb_codepoint_t,
    ab: *mut ffi::hb_codepoint_t,
) -> bool {
    // Avoid recomposing split matras.
    if char::try_from(a).unwrap().general_category().is_mark() {
        return false;
    }

    crate::unicode::rb_ucd_compose(a, b, ab) != 0
}

#[no_mangle]
pub extern "C" fn rb_complex_universal_clear_substitution_flags(
    _: *const ffi::hb_shape_plan_t,
    _: *mut ffi::hb_font_t,
    buffer: *mut ffi::rb_buffer_t,
) {
    let buffer = Buffer::from_ptr_mut(buffer);
    for info in buffer.info_slice_mut() {
        info.clear_substituted();
    }
}

#[no_mangle]
pub extern "C" fn rb_complex_universal_record_pref(
    _: *const ffi::hb_shape_plan_t,
    _: *mut ffi::hb_font_t,
    buffer: *mut ffi::rb_buffer_t,
) {
    let buffer = Buffer::from_ptr_mut(buffer);

    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len() {
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

#[no_mangle]
pub extern "C" fn rb_complex_universal_reorder(
    plan: *const ffi::hb_shape_plan_t,
    _: *mut ffi::hb_font_t,
    buffer: *mut ffi::rb_buffer_t,
) {
    let buffer = Buffer::from_ptr_mut(buffer);
    let font = unsafe {
        let ttf_parser_data = ffi::hb_shape_plan_ttf_parser(plan);
        crate::font::ttf_parser_from_raw(ttf_parser_data)
    };

    insert_dotted_circles(font, buffer);

    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len() {
        reorder_syllable(start, end, buffer);
        start = end;
        end = buffer.next_syllable(start);
    }
}

extern "C" {
    fn hb_complex_universal_data_create(plan: *const ffi::hb_shape_plan_t) -> *mut c_void;
    fn hb_complex_universal_data_destroy(data: *mut c_void);
    fn hb_complex_universal_preprocess_text(plan: *const ffi::hb_shape_plan_t,
                                            buffer: *mut ffi::rb_buffer_t,
                                            font: *mut ffi::hb_font_t);
    fn hb_complex_universal_setup_masks(plan: *const ffi::hb_shape_plan_t,
                                        buffer: *mut ffi::rb_buffer_t,
                                        font: *mut ffi::hb_font_t);
}

#[no_mangle]
pub extern "C" fn rb_create_use_shaper() -> *const ffi::hb_ot_complex_shaper_t {
    let shaper = Box::new(ffi::hb_ot_complex_shaper_t {
        collect_features: Some(rb_complex_universal_collect_features),
        override_features: None,
        data_create: Some(hb_complex_universal_data_create),
        data_destroy: Some(hb_complex_universal_data_destroy),
        preprocess_text: Some(hb_complex_universal_preprocess_text),
        postprocess_glyphs: None,
        normalization_preference: ffi::HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT,
        decompose: None,
        compose: Some(rb_complex_universal_compose),
        setup_masks: Some(hb_complex_universal_setup_masks),
        gpos_tag: 0,
        reorder_marks: None,
        zero_width_marks: ffi::HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_EARLY,
        fallback_position: false,
    });
    Box::into_raw(shaper)
}
