use std::convert::TryFrom;
use std::os::raw::c_void;

use crate::{ffi, Tag, Mask, Font, GlyphInfo};
use crate::buffer::{Buffer, BufferFlags};
use crate::unicode::{CharExt, GeneralCategoryExt};
use crate::ot::*;
use super::indic::{Category, Position};

const KHMER_FEATURES: &[(Tag, FeatureFlags)] = &[
    // Basic features.
    // These features are applied in order, one at a time, after reordering.
    (feature::PRE_BASE_FORMS, FeatureFlags::MANUAL_JOINERS),
    (feature::BELOW_BASE_FORMS, FeatureFlags::MANUAL_JOINERS),
    (feature::ABOVE_BASE_FORMS, FeatureFlags::MANUAL_JOINERS),
    (feature::POST_BASE_FORMS, FeatureFlags::MANUAL_JOINERS),
    (feature::CONJUNCT_FORM_AFTER_RO, FeatureFlags::MANUAL_JOINERS),
    // Other features.
    // These features are applied all at once after clearing syllables.
    (feature::PRE_BASE_SUBSTITUTIONS, FeatureFlags::GLOBAL_MANUAL_JOINERS),
    (feature::ABOVE_BASE_SUBSTITUTIONS, FeatureFlags::GLOBAL_MANUAL_JOINERS),
    (feature::BELOW_BASE_SUBSTITUTIONS, FeatureFlags::GLOBAL_MANUAL_JOINERS),
    (feature::POST_BASE_SUBSTITUTIONS, FeatureFlags::GLOBAL_MANUAL_JOINERS),
];

// Must be in the same order as the KHMER_FEATURES array.
mod khmer_feature {
    pub const PREF: usize = 0;
    pub const BLWF: usize = 1;
    pub const ABVF: usize = 2;
    pub const PSTF: usize = 3;
    pub const CFAR: usize = 4;
}

impl GlyphInfo {
    fn set_khmer_properties(&mut self) {
        let u = self.codepoint;
        let (mut cat, pos) = super::indic::get_category_and_position(u);

        // Re-assign category

        // These categories are experimentally extracted from what Uniscribe allows.

        match u {
            0x179A => cat = Category::Ra,
            0x17CC | 0x17C9 | 0x17CA => cat = Category::Robatic,
            0x17C6 | 0x17CB | 0x17CD | 0x17CE | 0x17CF | 0x17D0 | 0x17D1 => cat = Category::Xgroup,
            // Just guessing. Uniscribe doesn't categorize it.
            0x17C7 | 0x17C8 | 0x17DD | 0x17D3 => cat = Category::Ygroup,
            _ => {}
        }

        // Re-assign position.

        if cat == Category::M {
            match pos {
                Position::PreC => cat = Category::VPre,
                Position::BelowC => cat = Category::VBlw,
                Position::AboveC => cat = Category::VAbv,
                Position::PostC => cat = Category::VPst,
                _ => {}
            }
        }

        self.set_indic_category(cat);
    }
}

struct KhmerShapePlan {
    mask_array: [Mask; KHMER_FEATURES.len()],
}

impl KhmerShapePlan {
    fn new(plan: &ShapePlan) -> Self {
        let mut mask_array = [0; KHMER_FEATURES.len()];
        for (i, feature) in KHMER_FEATURES.iter().enumerate() {
            mask_array[i] = if feature.1.contains(FeatureFlags::GLOBAL) {
                0
            } else {
                plan.ot_map.get_1_mask(feature.0)
            }
        }

        KhmerShapePlan {
            mask_array,
        }
    }

    fn from_ptr(plan: *const c_void) -> &'static KhmerShapePlan {
        unsafe { &*(plan as *const KhmerShapePlan) }
    }
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_collect_features_khmer(planner: *mut ffi::hb_ot_shape_planner_t) {
    let mut planner = ShapePlanner::from_ptr_mut(planner);
    collect_features(&mut planner)
}

fn collect_features(planner: &mut ShapePlanner) {
    // Do this before any lookups have been applied.
    planner.ot_map.add_gsub_pause(Some(setup_syllables_raw));
    planner.ot_map.add_gsub_pause(Some(reorder_raw));

    // Testing suggests that Uniscribe does NOT pause between basic
    // features.  Test with KhmerUI.ttf and the following three
    // sequences:
    //
    //   U+1789,U+17BC
    //   U+1789,U+17D2,U+1789
    //   U+1789,U+17D2,U+1789,U+17BC
    //
    // https://github.com/harfbuzz/harfbuzz/issues/974
    planner.ot_map.enable_feature(feature::LOCALIZED_FORMS, FeatureFlags::NONE, 1);
    planner.ot_map.enable_feature(feature::GLYPH_COMPOSITION_DECOMPOSITION, FeatureFlags::NONE, 1);

    for feature in KHMER_FEATURES.iter().take(5) {
        planner.ot_map.add_feature(feature.0, feature.1, 1);
    }

    planner.ot_map.add_gsub_pause(Some(ffi::hb_layout_clear_syllables));

    for feature in KHMER_FEATURES.iter().skip(5) {
        planner.ot_map.add_feature(feature.0, feature.1, 1);
    }
}

extern "C" fn setup_syllables_raw(
    plan: *const ffi::hb_ot_shape_plan_t,
    font: *mut ffi::hb_font_t,
    buffer: *mut ffi::hb_buffer_t,
) {
    let plan = ShapePlan::from_ptr(plan);
    let font = Font::from_ptr(font);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    setup_syllables(&plan, font, &mut buffer);
}

fn setup_syllables(_: &ShapePlan, _: &Font, buffer: &mut Buffer) {
    super::khmer_machine::find_syllables_khmer(buffer);

    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len() {
        buffer.unsafe_to_break(start, end);
        start = end;
        end = buffer.next_syllable(start);
    }
}

extern "C" fn reorder_raw(
    plan: *const ffi::hb_ot_shape_plan_t,
    font: *mut ffi::hb_font_t,
    buffer: *mut ffi::hb_buffer_t,
) {
    let plan = ShapePlan::from_ptr(plan);
    let font = Font::from_ptr(font);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    reorder(&plan, font, &mut buffer);
}

fn reorder(plan: &ShapePlan, font: &Font, buffer: &mut Buffer) {
    insert_dotted_circles(font, buffer);

    let khmer_plan = KhmerShapePlan::from_ptr(plan.data() as _);

    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len() {
        reorder_syllable(khmer_plan, start, end, buffer);
        start = end;
        end = buffer.next_syllable(start);
    }
}

fn insert_dotted_circles(font: &Font, buffer: &mut Buffer) {
    use super::khmer_machine::SyllableType;

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

    let dottedcircle_glyph = match font.glyph_index(0x25CC) {
        Some(g) => g.0 as u32,
        None => return,
    };

    let mut dottedcircle = GlyphInfo {
        codepoint: 0x25CC,
        ..GlyphInfo::default()
    };
    dottedcircle.set_khmer_properties();
    dottedcircle.codepoint = dottedcircle_glyph;

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
                buffer.cur(0).indic_category() == Category::Repha
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

fn reorder_syllable(
    khmer_plan: &KhmerShapePlan,
    start: usize,
    end: usize,
    buffer: &mut Buffer,
) {
    use super::khmer_machine::SyllableType;

    let syllable_type = match buffer.info[start].syllable() & 0x0F {
        0 => SyllableType::ConsonantSyllable,
        1 => SyllableType::BrokenCluster,
        2 => SyllableType::NonKhmerCluster,
        _ => unreachable!(),
    };

    match syllable_type {
        SyllableType::ConsonantSyllable | SyllableType::BrokenCluster => {
            reorder_consonant_syllable(khmer_plan, start, end, buffer);
        }
        SyllableType::NonKhmerCluster => {}
    }
}

// Rules from:
// https://docs.microsoft.com/en-us/typography/script-development/devanagari
fn reorder_consonant_syllable(
    plan: &KhmerShapePlan,
    start: usize,
    end: usize,
    buffer: &mut Buffer,
) {
    // Setup masks.
    {
        // Post-base
        let mask = plan.mask_array[khmer_feature::BLWF] |
            plan.mask_array[khmer_feature::ABVF] |
            plan.mask_array[khmer_feature::PSTF];
        for info in &mut buffer.info[start+1..end] {
            info.mask |= mask;
        }
    }

    let mut num_coengs = 0;
    for i in start+1..end {
        // When a COENG + (Cons | IndV) combination are found (and subscript count
        // is less than two) the character combination is handled according to the
        // subscript type of the character following the COENG.
        //
        // ...
        //
        // Subscript Type 2 - The COENG + RO characters are reordered to immediately
        // before the base glyph. Then the COENG + RO characters are assigned to have
        // the 'pref' OpenType feature applied to them.
        if buffer.info[i].indic_category() == Category::Coeng && num_coengs <= 2 && i + 1 < end {
            num_coengs += 1;

            if buffer.info[i + 1].indic_category() == Category::Ra {
                for j in 0..2 {
                    buffer.info[i + j].mask |= plan.mask_array[khmer_feature::PREF];
                }

                // Move the Coeng,Ro sequence to the start.
                buffer.merge_clusters(start, i + 2);
                let t0 = buffer.info[i];
                let t1 = buffer.info[i + 1];
                for k in (0..i-start).rev() {
                    buffer.info[k + start + 2] = buffer.info[k + start];
                }

                buffer.info[start] = t0;
                buffer.info[start + 1] = t1;

                // Mark the subsequent stuff with 'cfar'.  Used in Khmer.
                // Read the feature spec.
                // This allows distinguishing the following cases with MS Khmer fonts:
                // U+1784,U+17D2,U+179A,U+17D2,U+1782
                // U+1784,U+17D2,U+1782,U+17D2,U+179A
                if plan.mask_array[khmer_feature::CFAR] != 0 {
                    for j in i+2..end {
                        buffer.info[j].mask |= plan.mask_array[khmer_feature::CFAR];
                    }
                }

                num_coengs = 2; // Done.
            }
        } else if buffer.info[i].indic_category() == Category::VPre {
            // Reorder left matra piece.

            // Move to the start.
            buffer.merge_clusters(start, i + 1);
            let t = buffer.info[i];
            for k in (0..i-start).rev() {
                buffer.info[k + start + 1] = buffer.info[k + start];
            }
            buffer.info[start] = t;
        }
    }
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_override_features_khmer(planner: *mut ffi::hb_ot_shape_planner_t) {
    let mut planner = ShapePlanner::from_ptr_mut(planner);
    override_features(&mut planner)
}

fn override_features(planner: &mut ShapePlanner) {
    // Khmer spec has 'clig' as part of required shaping features:
    // "Apply feature 'clig' to form ligatures that are desired for
    // typographical correctness.", hence in overrides...
    planner.ot_map.enable_feature(feature::CONTEXTUAL_LIGATURES, FeatureFlags::NONE, 1);

    planner.ot_map.disable_feature(feature::STANDARD_LIGATURES);
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_data_create_khmer(
    plan: *const ffi::hb_ot_shape_plan_t,
) -> *mut c_void {
    let plan = ShapePlan::from_ptr(plan);
    let indic_plan = KhmerShapePlan::new(&plan);
    Box::into_raw(Box::new(indic_plan)) as _
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_data_destroy_khmer(data: *mut c_void) {
    unsafe { Box::from_raw(data) };
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_decompose_khmer(
    _: *const ffi::hb_ot_shape_normalize_context_t,
    ab: ffi::hb_codepoint_t,
    a: *mut ffi::hb_codepoint_t,
    b: *mut ffi::hb_codepoint_t,
) -> bool {
    // Decompose split matras that don't have Unicode decompositions.

    match ab {
        0x17BE => {
            unsafe { *a = 0x17C1; }
            unsafe { *b = 0x17BE; }
            return true;
        }
        0x17BF => {
            unsafe { *a = 0x17C1; }
            unsafe { *b = 0x17BF; }
            return true;
        }
        0x17C0 => {
            unsafe { *a = 0x17C1; }
            unsafe { *b = 0x17C0; }
            return true;
        }
        0x17C4 => {
            unsafe { *a = 0x17C1; }
            unsafe { *b = 0x17C4; }
            return true;
        }
        0x17C5 => {
            unsafe { *a = 0x17C1; }
            unsafe { *b = 0x17C5; }
            return true;
        }
        _ => {}
    }

    crate::unicode::hb_ucd_decompose(ab, a, b) != 0
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_compose_khmer(
    ctx: *const ffi::hb_ot_shape_normalize_context_t,
    a: u32,
    b: u32,
    ab: *mut u32,
) -> bool {
    let ctx = ShapeNormalizeContext::from_ptr(ctx);
    let a = char::try_from(a).unwrap();
    let b = char::try_from(b).unwrap();
    match compose(&ctx, a, b) {
        Some(c) => unsafe {
            *ab = c as u32;
            true
        }
        None => false,
    }
}

fn compose(_: &ShapeNormalizeContext, a: char, b: char) -> Option<char> {
    // Avoid recomposing split matras.
    if a.general_category().is_mark() {
        return None;
    }

    crate::unicode::compose(a, b)
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_setup_masks_khmer(
    plan: *const ffi::hb_ot_shape_plan_t,
    buffer: *mut ffi::hb_buffer_t,
    font: *mut ffi::hb_font_t,
) {
    let plan = ShapePlan::from_ptr(plan);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    let font = Font::from_ptr(font);
    setup_masks(&plan, font, &mut buffer);
}

fn setup_masks(_: &ShapePlan, _: &Font, buffer: &mut Buffer) {
    // We cannot setup masks here.  We save information about characters
    // and setup masks later on in a pause-callback.
    for info in buffer.info_slice_mut() {
        info.set_khmer_properties();
    }
}
