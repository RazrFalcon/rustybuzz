use alloc::boxed::Box;

use super::buffer::hb_buffer_t;
use super::ot_map::*;
use super::ot_shape::*;
use super::ot_shape_complex::*;
use super::ot_shape_complex_indic::{category, position};
use super::ot_shape_normalize::*;
use super::shape_plan::hb_ot_shape_plan_t;
use super::unicode::{CharExt, GeneralCategoryExt};
use super::{hb_font_t, hb_glyph_info_t, hb_mask_t, hb_tag_t};

pub const KHMER_SHAPER: hb_ot_complex_shaper_t = hb_ot_complex_shaper_t {
    collect_features: Some(collect_features),
    override_features: Some(override_features),
    create_data: Some(|plan| Box::new(KhmerShapePlan::new(plan))),
    preprocess_text: None,
    postprocess_glyphs: None,
    normalization_preference: HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT,
    decompose: Some(decompose),
    compose: Some(compose),
    setup_masks: Some(setup_masks),
    gpos_tag: None,
    reorder_marks: None,
    zero_width_marks: HB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    fallback_position: false,
};

const KHMER_FEATURES: &[(hb_tag_t, hb_ot_map_feature_flags_t)] = &[
    // Basic features.
    // These features are applied all at once, before reordering, constrained
    // to the syllable.
    (hb_tag_t::from_bytes(b"pref"), F_MANUAL_JOINERS),
    (hb_tag_t::from_bytes(b"blwf"), F_MANUAL_JOINERS),
    (hb_tag_t::from_bytes(b"abvf"), F_MANUAL_JOINERS),
    (hb_tag_t::from_bytes(b"pstf"), F_MANUAL_JOINERS),
    (hb_tag_t::from_bytes(b"cfar"), F_MANUAL_JOINERS),
    // Other features.
    // These features are applied all at once after clearing syllables.
    (hb_tag_t::from_bytes(b"pres"), F_GLOBAL_MANUAL_JOINERS),
    (hb_tag_t::from_bytes(b"abvs"), F_GLOBAL_MANUAL_JOINERS),
    (hb_tag_t::from_bytes(b"blws"), F_GLOBAL_MANUAL_JOINERS),
    (hb_tag_t::from_bytes(b"psts"), F_GLOBAL_MANUAL_JOINERS),
];

// Must be in the same order as the KHMER_FEATURES array.
mod khmer_feature {
    pub const PREF: usize = 0;
    pub const BLWF: usize = 1;
    pub const ABVF: usize = 2;
    pub const PSTF: usize = 3;
    pub const CFAR: usize = 4;
}

impl hb_glyph_info_t {
    fn set_khmer_properties(&mut self) {
        let u = self.glyph_id;
        let (mut cat, pos) = crate::hb::ot_shape_complex_indic::get_category_and_position(u);

        // Re-assign category

        // These categories are experimentally extracted from what Uniscribe allows.

        match u {
            0x179A => cat = category::RA,
            0x17CC | 0x17C9 | 0x17CA => cat = category::ROBATIC,
            0x17C6 | 0x17CB | 0x17CD | 0x17CE | 0x17CF | 0x17D0 | 0x17D1 => cat = category::X_GROUP,
            // Just guessing. Uniscribe doesn't categorize it.
            0x17C7 | 0x17C8 | 0x17DD | 0x17D3 => cat = category::Y_GROUP,
            _ => {}
        }

        // Re-assign position.

        if cat == category::M {
            match pos {
                position::PRE_C => cat = category::V_PRE,
                position::BELOW_C => cat = category::V_BLW,
                position::ABOVE_C => cat = category::V_AVB,
                position::POST_C => cat = category::V_PST,
                _ => {}
            }
        }

        self.set_indic_category(cat);
    }
}

struct KhmerShapePlan {
    mask_array: [hb_mask_t; KHMER_FEATURES.len()],
}

impl KhmerShapePlan {
    fn new(plan: &hb_ot_shape_plan_t) -> Self {
        let mut mask_array = [0; KHMER_FEATURES.len()];
        for (i, feature) in KHMER_FEATURES.iter().enumerate() {
            mask_array[i] = if feature.1 & F_GLOBAL != 0 {
                0
            } else {
                plan.ot_map.get_1_mask(feature.0)
            }
        }

        KhmerShapePlan { mask_array }
    }
}

fn collect_features(planner: &mut hb_ot_shape_planner_t) {
    // Do this before any lookups have been applied.
    planner.ot_map.add_gsub_pause(Some(setup_syllables));
    planner.ot_map.add_gsub_pause(Some(reorder));

    // Testing suggests that Uniscribe does NOT pause between basic
    // features.  Test with KhmerUI.ttf and the following three
    // sequences:
    //
    //   U+1789,U+17BC
    //   U+1789,U+17D2,U+1789
    //   U+1789,U+17D2,U+1789,U+17BC
    //
    // https://github.com/harfbuzz/harfbuzz/issues/974
    planner
        .ot_map
        .enable_feature(hb_tag_t::from_bytes(b"locl"), F_NONE, 1);
    planner
        .ot_map
        .enable_feature(hb_tag_t::from_bytes(b"ccmp"), F_NONE, 1);

    for feature in KHMER_FEATURES.iter().take(5) {
        planner.ot_map.add_feature(feature.0, feature.1, 1);
    }

    planner
        .ot_map
        .add_gsub_pause(Some(crate::hb::ot_layout::_hb_clear_syllables));

    for feature in KHMER_FEATURES.iter().skip(5) {
        planner.ot_map.add_feature(feature.0, feature.1, 1);
    }
}

fn setup_syllables(_: &hb_ot_shape_plan_t, _: &hb_font_t, buffer: &mut hb_buffer_t) {
    super::ot_shape_complex_khmer_machine::find_syllables_khmer(buffer);

    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len {
        buffer.unsafe_to_break(Some(start), Some(end));
        start = end;
        end = buffer.next_syllable(start);
    }
}

fn reorder(plan: &hb_ot_shape_plan_t, face: &hb_font_t, buffer: &mut hb_buffer_t) {
    use super::ot_shape_complex_khmer_machine::SyllableType;

    super::ot_shape_complex_syllabic::insert_dotted_circles(
        face,
        buffer,
        SyllableType::BrokenCluster as u8,
        category::DOTTED_CIRCLE,
        Some(category::REPHA),
        None,
    );

    let khmer_plan = plan.data::<KhmerShapePlan>();

    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len {
        reorder_syllable(khmer_plan, start, end, buffer);
        start = end;
        end = buffer.next_syllable(start);
    }
}

fn reorder_syllable(
    khmer_plan: &KhmerShapePlan,
    start: usize,
    end: usize,
    buffer: &mut hb_buffer_t,
) {
    use super::ot_shape_complex_khmer_machine::SyllableType;

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
    buffer: &mut hb_buffer_t,
) {
    // Setup masks.
    {
        // Post-base
        let mask = plan.mask_array[khmer_feature::BLWF]
            | plan.mask_array[khmer_feature::ABVF]
            | plan.mask_array[khmer_feature::PSTF];
        for info in &mut buffer.info[start + 1..end] {
            info.mask |= mask;
        }
    }

    let mut num_coengs = 0;
    for i in start + 1..end {
        // When a COENG + (Cons | IndV) combination are found (and subscript count
        // is less than two) the character combination is handled according to the
        // subscript type of the character following the COENG.
        //
        // ...
        //
        // Subscript Type 2 - The COENG + RO characters are reordered to immediately
        // before the base glyph. Then the COENG + RO characters are assigned to have
        // the 'pref' OpenType feature applied to them.
        if buffer.info[i].indic_category() == category::COENG && num_coengs <= 2 && i + 1 < end {
            num_coengs += 1;

            if buffer.info[i + 1].indic_category() == category::RA {
                for j in 0..2 {
                    buffer.info[i + j].mask |= plan.mask_array[khmer_feature::PREF];
                }

                // Move the Coeng,Ro sequence to the start.
                buffer.merge_clusters(start, i + 2);
                let t0 = buffer.info[i];
                let t1 = buffer.info[i + 1];
                for k in (0..i - start).rev() {
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
                    for j in i + 2..end {
                        buffer.info[j].mask |= plan.mask_array[khmer_feature::CFAR];
                    }
                }

                num_coengs = 2; // Done.
            }
        } else if buffer.info[i].indic_category() == category::V_PRE {
            // Reorder left matra piece.

            // Move to the start.
            buffer.merge_clusters(start, i + 1);
            let t = buffer.info[i];
            for k in (0..i - start).rev() {
                buffer.info[k + start + 1] = buffer.info[k + start];
            }
            buffer.info[start] = t;
        }
    }
}

fn override_features(planner: &mut hb_ot_shape_planner_t) {
    // Khmer spec has 'clig' as part of required shaping features:
    // "Apply feature 'clig' to form ligatures that are desired for
    // typographical correctness.", hence in overrides...
    planner
        .ot_map
        .enable_feature(hb_tag_t::from_bytes(b"clig"), F_NONE, 1);

    planner
        .ot_map
        .disable_feature(hb_tag_t::from_bytes(b"liga"));
}

fn decompose(_: &hb_ot_shape_normalize_context_t, ab: char) -> Option<(char, char)> {
    // Decompose split matras that don't have Unicode decompositions.
    match ab {
        '\u{17BE}' | '\u{17BF}' | '\u{17C0}' | '\u{17C4}' | '\u{17C5}' => Some(('\u{17C1}', ab)),
        _ => crate::hb::unicode::decompose(ab),
    }
}

fn compose(_: &hb_ot_shape_normalize_context_t, a: char, b: char) -> Option<char> {
    // Avoid recomposing split matras.
    if a.general_category().is_mark() {
        return None;
    }

    crate::hb::unicode::compose(a, b)
}

fn setup_masks(_: &hb_ot_shape_plan_t, _: &hb_font_t, buffer: &mut hb_buffer_t) {
    // We cannot setup masks here.  We save information about characters
    // and setup masks later on in a pause-callback.
    for info in buffer.info_slice_mut() {
        info.set_khmer_properties();
    }
}
