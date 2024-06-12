use super::buffer::hb_buffer_t;
use super::ot_map::*;
use super::ot_shape::*;
use super::ot_shape_normalize::*;
use super::ot_shape_plan::hb_ot_shape_plan_t;
use super::ot_shaper::*;
use super::ot_shaper_indic::{indic_category_t, position};
use super::{hb_font_t, hb_glyph_info_t, hb_tag_t};
use crate::hb::ot_shaper_khmer::khmer_category_t;

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

pub mod myanmar_category_t {
    use crate::hb::ot_shaper_indic::indic_category_t::{N, PLACEHOLDER};

    pub const As: u8 = 18; /* Asat */
    #[allow(dead_code)]
    pub const D0: u8 = 20; /* Digit zero */
    #[allow(dead_code)]
    pub const DB: u8 = N; /* Dot below */
    pub const GB: u8 = PLACEHOLDER;
    pub const MH: u8 = 21; /* Various consonant medial types */
    pub const MR: u8 = 22; /* Various consonant medial types */
    pub const MW: u8 = 23; /* Various consonant medial types */
    pub const MY: u8 = 24; /* Various consonant medial types */
    pub const PT: u8 = 25; /* Pwo and other tones */
    //pub const VAbv: u8 = 26;
    //pub const VBlw: u8 = 27;
    //pub const VPre: u8 = 28;
    //pub const VPst: u8 = 29;
    pub const VS: u8 = 30; /* Variation selectors */
    pub const P: u8 = 31; /* Punctuation */
    pub const D: u8 = GB; /* Digits except zero */
    pub const ML: u8 = 32; /* Various consonant medial types */
}

impl hb_glyph_info_t {
    fn set_myanmar_properties(&mut self) {
        let u = self.glyph_id;
        let (mut cat, mut pos) = crate::hb::ot_shaper_indic::get_category_and_position(u);

        // Myanmar
        // https://docs.microsoft.com/en-us/typography/script-development/myanmar#analyze

        if (0xFE00..=0xFE0F).contains(&u) {
            cat = myanmar_category_t::VS;
        }

        match u {
            // The spec says C, IndicSyllableCategory doesn't have.
            0x104E => cat = indic_category_t::C,

            0x002D | 0x00A0 | 0x00D7 | 0x2012 | 0x2013 | 0x2014 | 0x2015 | 0x2022 | 0x25CC
            | 0x25FB | 0x25FC | 0x25FD | 0x25FE => cat = myanmar_category_t::GB,

            0x1004 | 0x101B | 0x105A => cat = indic_category_t::RA,

            0x1032 | 0x1036 => cat = indic_category_t::A,

            0x1039 => cat = indic_category_t::H,

            0x103A => cat = myanmar_category_t::As,

            0x1041 | 0x1042 | 0x1043 | 0x1044 | 0x1045 | 0x1046 | 0x1047 | 0x1048 | 0x1049
            | 0x1090 | 0x1091 | 0x1092 | 0x1093 | 0x1094 | 0x1095 | 0x1096 | 0x1097 | 0x1098
            | 0x1099 => cat = myanmar_category_t::D,

            // XXX The spec says D0, but Uniscribe doesn't seem to do.
            0x1040 => cat = myanmar_category_t::D,

            0x103E => cat = myanmar_category_t::MH,

            0x1060 => cat = myanmar_category_t::ML,

            0x103C => cat = myanmar_category_t::MR,

            0x103D | 0x1082 => cat = myanmar_category_t::MW,

            0x103B | 0x105E | 0x105F => cat = myanmar_category_t::MY,

            0x1063 | 0x1064 | 0x1069 | 0x106A | 0x106B | 0x106C | 0x106D | 0xAA7B => {
                cat = myanmar_category_t::PT
            }

            0x1038 | 0x1087 | 0x1088 | 0x1089 | 0x108A | 0x108B | 0x108C | 0x108D | 0x108F
            | 0x109A | 0x109B | 0x109C => cat = indic_category_t::SM,

            0x104A | 0x104B => cat = myanmar_category_t::P,

            // https://github.com/harfbuzz/harfbuzz/issues/218
            0xAA74 | 0xAA75 | 0xAA76 => cat = indic_category_t::C,

            _ => {}
        }

        // Re-assign position.

        if cat == indic_category_t::M {
            match pos {
                position::PRE_C => {
                    cat = indic_category_t::V_PRE;
                    pos = position::PRE_M;
                }
                position::BELOW_C => cat = indic_category_t::V_BLW,
                position::ABOVE_C => cat = indic_category_t::V_AVB,
                position::POST_C => cat = indic_category_t::V_PST,
                _ => {}
            }
        }

        self.set_indic_category(cat);
        self.set_indic_position(pos);
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
        indic_category_t::PLACEHOLDER,
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
            && buffer.info[start + 0].indic_category() == indic_category_t::RA
            && buffer.info[start + 1].indic_category() == myanmar_category_t::As
            && buffer.info[start + 2].indic_category() == indic_category_t::H
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
            buffer.info[i].set_indic_position(position::AFTER_MAIN);
            i += 1;
        }

        while i < base {
            buffer.info[i].set_indic_position(position::PRE_C);
            i += 1;
        }

        if i < end {
            buffer.info[i].set_indic_position(position::BASE_C);
            i += 1;
        }

        let mut pos = position::AFTER_MAIN;
        // The following loop may be ugly, but it implements all of
        // Myanmar reordering!
        for i in i..end {
            // Pre-base reordering
            if buffer.info[i].indic_category() == khmer_category_t::Y_GROUP {
                buffer.info[i].set_indic_position(position::PRE_C);
                continue;
            }

            // Left matra
            if buffer.info[i].indic_position() < position::BASE_C {
                continue;
            }

            if buffer.info[i].indic_category() == myanmar_category_t::VS {
                let t = buffer.info[i - 1].indic_position();
                buffer.info[i].set_indic_position(t);
                continue;
            }

            if pos == position::AFTER_MAIN
                && buffer.info[i].indic_category() == indic_category_t::V_BLW
            {
                pos = position::BELOW_C;
                buffer.info[i].set_indic_position(pos);
                continue;
            }

            if pos == position::BELOW_C && buffer.info[i].indic_category() == indic_category_t::A {
                buffer.info[i].set_indic_position(position::BEFORE_SUB);
                continue;
            }

            if pos == position::BELOW_C
                && buffer.info[i].indic_category() == indic_category_t::V_BLW
            {
                buffer.info[i].set_indic_position(pos);
                continue;
            }

            if pos == position::BELOW_C && buffer.info[i].indic_category() != indic_category_t::A {
                pos = position::AFTER_SUB;
                buffer.info[i].set_indic_position(pos);
                continue;
            }

            buffer.info[i].set_indic_position(pos);
        }
    }

    buffer.sort(start, end, |a, b| {
        a.indic_position().cmp(&b.indic_position()) == core::cmp::Ordering::Greater
    });
}

fn setup_masks(_: &hb_ot_shape_plan_t, _: &hb_font_t, buffer: &mut hb_buffer_t) {
    // No masks, we just save information about characters.
    for info in buffer.info_slice_mut() {
        info.set_myanmar_properties();
    }
}
