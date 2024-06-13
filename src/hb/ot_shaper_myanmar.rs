use super::buffer::hb_buffer_t;
use super::ot_map::*;
use super::ot_shape::*;
use super::ot_shape_normalize::*;
use super::ot_shape_plan::hb_ot_shape_plan_t;
use super::ot_shaper::*;
use super::ot_shaper_indic::{indic_category_t, indic_position_t};
use super::{hb_font_t, hb_glyph_info_t, hb_tag_t};
use crate::hb::ot_shaper_indic::indic_category_t::OT_VPre;

pub const MYANMAR_SHAPER: hb_ot_shaper_t = hb_ot_shaper_t {
    collect_features: Some(collect_features),
    override_features: None,
    create_data: None,
    preprocess_text: None,
    postprocess_glyphs: None,
    normalization_preference: HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT,
    decompose: None,
    compose: None,
    setup_masks: Some(setup_masks),
    gpos_tag: None,
    reorder_marks: None,
    zero_width_marks: HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_EARLY,
    fallback_position: false,
};

// Ugly Zawgyi encoding.
// Disable all auto processing.
// https://github.com/harfbuzz/harfbuzz/issues/1162
pub const MYANMAR_ZAWGYI_SHAPER: hb_ot_shaper_t = hb_ot_shaper_t {
    collect_features: None,
    override_features: None,
    create_data: None,
    preprocess_text: None,
    postprocess_glyphs: None,
    normalization_preference: HB_OT_SHAPE_NORMALIZATION_MODE_NONE,
    decompose: None,
    compose: None,
    setup_masks: None,
    gpos_tag: None,
    reorder_marks: None,
    zero_width_marks: HB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    fallback_position: false,
};

const MYANMAR_FEATURES: &[hb_tag_t] = &[
    // Basic features.
    // These features are applied in order, one at a time, after reordering,
    // constrained to the syllable.
    hb_tag_t::from_bytes(b"rphf"),
    hb_tag_t::from_bytes(b"pref"),
    hb_tag_t::from_bytes(b"blwf"),
    hb_tag_t::from_bytes(b"pstf"),
    // Other features.
    // These features are applied all at once after clearing syllables.
    hb_tag_t::from_bytes(b"pres"),
    hb_tag_t::from_bytes(b"abvs"),
    hb_tag_t::from_bytes(b"blws"),
    hb_tag_t::from_bytes(b"psts"),
];

impl hb_glyph_info_t {
    fn set_myanmar_properties(&mut self) {
        let u = self.glyph_id;
        let (cat, _) = crate::hb::ot_shaper_indic_table::get_categories(u);

        self.set_myanmar_category(cat);
    }
}

fn collect_features(planner: &mut hb_ot_shape_planner_t) {
    // Do this before any lookups have been applied.
    planner.ot_map.add_gsub_pause(Some(setup_syllables));

    planner
        .ot_map
        .enable_feature(hb_tag_t::from_bytes(b"locl"), F_PER_SYLLABLE, 1);
    // The Indic specs do not require ccmp, but we apply it here since if
    // there is a use of it, it's typically at the beginning.
    planner
        .ot_map
        .enable_feature(hb_tag_t::from_bytes(b"ccmp"), F_PER_SYLLABLE, 1);

    planner.ot_map.add_gsub_pause(Some(reorder));

    for feature in MYANMAR_FEATURES.iter().take(4) {
        planner.ot_map.enable_feature(*feature, F_MANUAL_ZWJ, 1);
        planner.ot_map.add_gsub_pause(None);
    }

    for feature in MYANMAR_FEATURES.iter().skip(4) {
        planner
            .ot_map
            .enable_feature(*feature, F_MANUAL_ZWJ | F_PER_SYLLABLE, 1);
    }
}

fn setup_syllables(_: &hb_ot_shape_plan_t, _: &hb_font_t, buffer: &mut hb_buffer_t) {
    super::ot_shaper_myanmar_machine::find_syllables_myanmar(buffer);

    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len {
        buffer.unsafe_to_break(Some(start), Some(end));
        start = end;
        end = buffer.next_syllable(start);
    }
}

fn reorder(_: &hb_ot_shape_plan_t, face: &hb_font_t, buffer: &mut hb_buffer_t) {
    use super::ot_shaper_myanmar_machine::SyllableType;

    super::ot_shaper_syllabic::insert_dotted_circles(
        face,
        buffer,
        SyllableType::BrokenCluster as u8,
        indic_category_t::OT_DOTTEDCIRCLE,
        None,
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

fn reorder_syllable(start: usize, end: usize, buffer: &mut hb_buffer_t) {
    use super::ot_shaper_myanmar_machine::SyllableType;

    let syllable_type = match buffer.info[start].syllable() & 0x0F {
        0 => SyllableType::ConsonantSyllable,
        1 => SyllableType::PunctuationCluster,
        2 => SyllableType::BrokenCluster,
        3 => SyllableType::NonMyanmarCluster,
        _ => unreachable!(),
    };

    match syllable_type {
        // We already inserted dotted-circles, so just call the consonant_syllable.
        SyllableType::ConsonantSyllable | SyllableType::BrokenCluster => {
            initial_reordering_consonant_syllable(start, end, buffer);
        }
        SyllableType::PunctuationCluster | SyllableType::NonMyanmarCluster => {}
    }
}

// Rules from:
// https://docs.microsoft.com/en-us/typography/script-development/myanmar
fn initial_reordering_consonant_syllable(start: usize, end: usize, buffer: &mut hb_buffer_t) {
    let mut base = end;
    let mut has_reph = false;

    {
        let mut limit = start;
        if start + 3 <= end
            && buffer.info[start + 0].myanmar_category() == indic_category_t::OT_Ra
            && buffer.info[start + 1].myanmar_category() == indic_category_t::OT_As
            && buffer.info[start + 2].myanmar_category() == indic_category_t::OT_H
        {
            limit += 3;
            base = start;
            has_reph = true;
        }

        {
            if !has_reph {
                base = limit;
            }

            for i in limit..end {
                if buffer.info[i].is_consonant() {
                    base = i;
                    break;
                }
            }
        }
    }

    // Reorder!
    {
        let mut i = start;
        while i < start + if has_reph { 3 } else { 0 } {
            buffer.info[i].set_myanmar_position(indic_position_t::POS_AFTER_MAIN);
            i += 1;
        }

        while i < base {
            buffer.info[i].set_myanmar_position(indic_position_t::POS_PRE_C);
            i += 1;
        }

        if i < end {
            buffer.info[i].set_myanmar_position(indic_position_t::POS_BASE_C);
            i += 1;
        }

        let mut pos = indic_position_t::POS_AFTER_MAIN;
        // The following loop may be ugly, but it implements all of
        // Myanmar reordering!
        for i in i..end {
            // Pre-base reordering
            if buffer.info[i].myanmar_category() == indic_category_t::OT_MR {
                buffer.info[i].set_myanmar_position(indic_position_t::POS_PRE_C);
                continue;
            }

            // Left matra
            if buffer.info[i].myanmar_category() == OT_VPre {
                buffer.info[i].set_myanmar_position(indic_position_t::POS_PRE_M);
                continue;
            }

            if buffer.info[i].myanmar_category() == indic_category_t::OT_VS {
                let t = buffer.info[i - 1].myanmar_position();
                buffer.info[i].set_myanmar_position(t);
                continue;
            }

            if pos == indic_position_t::POS_AFTER_MAIN
                && buffer.info[i].myanmar_category() == indic_category_t::OT_VBlw
            {
                pos = indic_position_t::POS_BELOW_C;
                buffer.info[i].set_myanmar_position(pos);
                continue;
            }

            if pos == indic_position_t::POS_BELOW_C
                && buffer.info[i].myanmar_category() == indic_category_t::OT_A
            {
                buffer.info[i].set_myanmar_position(indic_position_t::POS_BEFORE_SUB);
                continue;
            }

            if pos == indic_position_t::POS_BELOW_C
                && buffer.info[i].myanmar_category() == indic_category_t::OT_VBlw
            {
                buffer.info[i].set_myanmar_position(pos);
                continue;
            }

            if pos == indic_position_t::POS_BELOW_C
                && buffer.info[i].myanmar_category() != indic_category_t::OT_A
            {
                pos = indic_position_t::POS_AFTER_SUB;
                buffer.info[i].set_myanmar_position(pos);
                continue;
            }

            buffer.info[i].set_myanmar_position(pos);
        }
    }

    buffer.sort(start, end, |a, b| {
        a.myanmar_position().cmp(&b.myanmar_position()) == core::cmp::Ordering::Greater
    });
}

fn setup_masks(_: &hb_ot_shape_plan_t, _: &hb_font_t, buffer: &mut hb_buffer_t) {
    // No masks, we just save information about characters.
    for info in buffer.info_slice_mut() {
        info.set_myanmar_properties();
    }
}
