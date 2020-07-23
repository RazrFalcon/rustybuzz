use crate::{ffi, Tag, Font, GlyphInfo};
use crate::buffer::{Buffer, BufferFlags};
use crate::ot::*;
use super::indic::{Category, Position};

const MYANMAR_FEATURES: &[Tag] = &[
    // Basic features.
    // These features are applied in order, one at a time, after reordering.
    feature::REPH_FORMS,
    feature::PRE_BASE_FORMS,
    feature::BELOW_BASE_FORMS,
    feature::POST_BASE_FORMS,
    // Other features.
    // These features are applied all at once after clearing syllables.
    feature::PRE_BASE_SUBSTITUTIONS,
    feature::ABOVE_BASE_SUBSTITUTIONS,
    feature::BELOW_BASE_SUBSTITUTIONS,
    feature::POST_BASE_SUBSTITUTIONS,
];

impl GlyphInfo {
    fn set_myanmar_properties(&mut self) {
        let u = self.codepoint;
        let (mut cat, mut pos) = super::indic::get_category_and_position(u);

        // Myanmar
        // https://docs.microsoft.com/en-us/typography/script-development/myanmar#analyze

        if (0xFE00..=0xFE0F).contains(&u) {
            cat = Category::VS;
        }

        match u {
            // The spec says C, IndicSyllableCategory doesn't have.
            0x104E => cat = Category::C,

            0x002D |
            0x00A0 |
            0x00D7 |
            0x2012 |
            0x2013 |
            0x2014 |
            0x2015 |
            0x2022 |
            0x25CC |
            0x25FB |
            0x25FC |
            0x25FD |
            0x25FE => cat = Category::Placeholder,

            0x1004 |
            0x101B |
            0x105A => cat = Category::Ra,

            0x1032 |
            0x1036 => cat = Category::A,

            0x1039 => cat = Category::H,

            0x103A => cat = Category::Symbol,

            0x1041 |
            0x1042 |
            0x1043 |
            0x1044 |
            0x1045 |
            0x1046 |
            0x1047 |
            0x1048 |
            0x1049 |
            0x1090 |
            0x1091 |
            0x1092 |
            0x1093 |
            0x1094 |
            0x1095 |
            0x1096 |
            0x1097 |
            0x1098 |
            0x1099 => cat = Category::D,

            // XXX The spec says D0, but Uniscribe doesn't seem to do.
            0x1040 => cat = Category::D,

            0x103E |
            0x1060 => cat = Category::Xgroup,

            0x103C => cat = Category::Ygroup,

            0x103D |
            0x1082 => cat = Category::MW,

            0x103B |
            0x105E |
            0x105F => cat = Category::MY,

            0x1063 |
            0x1064 |
            0x1069 |
            0x106A |
            0x106B |
            0x106C |
            0x106D |
            0xAA7B => cat = Category::PT,

            0x1038 |
            0x1087 |
            0x1088 |
            0x1089 |
            0x108A |
            0x108B |
            0x108C |
            0x108D |
            0x108F |
            0x109A |
            0x109B |
            0x109C => cat = Category::SM,

            0x104A |
            0x104B => cat = Category::P,

            // https://github.com/harfbuzz/harfbuzz/issues/218
            0xAA74 |
            0xAA75 |
            0xAA76 => cat = Category::C,

            _ => {}
        }

        // Re-assign position.

        if cat == Category::M {
            match pos {
                Position::PreC => {
                    cat = Category::VPre;
                    pos = Position::PreM;
                }
                Position::BelowC => cat = Category::VBlw,
                Position::AboveC => cat = Category::VAbv,
                Position::PostC => cat = Category::VPst,
                _ => {}
            }
        }

        self.set_indic_category(cat);
        self.set_indic_position(pos);
    }
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_collect_features_myanmar(planner: *mut ffi::hb_ot_shape_planner_t) {
    let mut planner = ShapePlanner::from_ptr_mut(planner);
    collect_features(&mut planner)
}

fn collect_features(planner: &mut ShapePlanner) {
    // Do this before any lookups have been applied.
    planner.ot_map.add_gsub_pause(Some(setup_syllables_raw));

    planner.ot_map.enable_feature(feature::LOCALIZED_FORMS, FeatureFlags::NONE, 1);
    // The Indic specs do not require ccmp, but we apply it here since if
    // there is a use of it, it's typically at the beginning.
    planner.ot_map.enable_feature(feature::GLYPH_COMPOSITION_DECOMPOSITION, FeatureFlags::NONE, 1);

    planner.ot_map.add_gsub_pause(Some(reorder_raw));

    for feature in MYANMAR_FEATURES.iter().take(4) {
        planner.ot_map.enable_feature(*feature, FeatureFlags::MANUAL_ZWJ, 1);
        planner.ot_map.add_gsub_pause(None);
    }

    planner.ot_map.add_gsub_pause(Some(ffi::hb_layout_clear_syllables));

    for feature in MYANMAR_FEATURES.iter().skip(4) {
        planner.ot_map.enable_feature(*feature, FeatureFlags::MANUAL_ZWJ, 1);
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
    super::myanmar_machine::find_syllables_myanmar(buffer);

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

fn reorder(_: &ShapePlan, font: &Font, buffer: &mut Buffer) {
    insert_dotted_circles(font, buffer);

    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len() {
        reorder_syllable(start, end, buffer);
        start = end;
        end = buffer.next_syllable(start);
    }
}

fn insert_dotted_circles(font: &Font, buffer: &mut Buffer) {
    use super::myanmar_machine::SyllableType;

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
    dottedcircle.set_myanmar_properties();
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

            buffer.output_info(ginfo);
        } else {
            buffer.next_glyph();
        }
    }

    buffer.swap_buffers();
}

fn reorder_syllable(start: usize, end: usize, buffer: &mut Buffer) {
    use super::myanmar_machine::SyllableType;

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
fn initial_reordering_consonant_syllable(start: usize, end: usize, buffer: &mut Buffer) {
    let mut base = end;
    let mut has_reph = false;

    {
        let mut limit = start;
        if start + 3 <= end &&
            buffer.info[start + 0].indic_category() == Category::Ra &&
            buffer.info[start + 1].indic_category() == Category::Symbol &&
            buffer.info[start + 2].indic_category() == Category::H
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
            buffer.info[i].set_indic_position(Position::AfterMain);
            i += 1;
        }

        while i < base {
            buffer.info[i].set_indic_position(Position::PreC);
            i += 1;
        }

        if i < end {
            buffer.info[i].set_indic_position(Position::BaseC);
            i += 1;
        }

        let mut pos = Position::AfterMain;
        // The following loop may be ugly, but it implements all of
        // Myanmar reordering!
        for i in i..end {
            // Pre-base reordering
            if buffer.info[i].indic_category() == Category::Ygroup {
                buffer.info[i].set_indic_position(Position::PreC);
                continue;
            }

            // Left matra
            if buffer.info[i].indic_position() < Position::BaseC {
                continue;
            }

            if buffer.info[i].indic_category() == Category::VS {
                let t = buffer.info[i - 1].indic_position();
                buffer.info[i].set_indic_position(t);
                continue;
            }

            if pos == Position::AfterMain && buffer.info[i].indic_category() == Category::VBlw {
                pos = Position::BelowC;
                buffer.info[i].set_indic_position(pos);
                continue;
            }

            if pos == Position::BelowC && buffer.info[i].indic_category() == Category::A {
                buffer.info[i].set_indic_position(Position::BeforeSub);
                continue;
            }

            if pos == Position::BelowC && buffer.info[i].indic_category() == Category::VBlw {
                buffer.info[i].set_indic_position(pos);
                continue;
            }

            if pos == Position::BelowC && buffer.info[i].indic_category() != Category::A {
                pos = Position::AfterSub;
                buffer.info[i].set_indic_position(pos);
                continue;
            }

            buffer.info[i].set_indic_position(pos);
        }
    }

    buffer.sort(start, end, |a, b| a.indic_position().cmp(&b.indic_position()) == std::cmp::Ordering::Greater);
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_override_features_myanmar(planner: *mut ffi::hb_ot_shape_planner_t) {
    let mut planner = ShapePlanner::from_ptr_mut(planner);
    override_features(&mut planner)
}

fn override_features(planner: &mut ShapePlanner) {
    planner.ot_map.disable_feature(feature::STANDARD_LIGATURES);
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_setup_masks_myanmar(
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
        info.set_myanmar_properties();
    }
}
