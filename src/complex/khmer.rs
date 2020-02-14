use std::convert::TryFrom;
use std::os::raw::c_void;

use crate::{Tag, Mask, Buffer, GlyphInfo, CodePoint};
use crate::buffer::BufferFlags;
use crate::map::{Map, MapBuilder, FeatureFlags};
use crate::ffi;
use crate::unicode::{CharExt, GeneralCategoryExt};
use super::indic::{Category, Position};

const KHMER_FEATURES: &[(Tag, FeatureFlags)] = &[
    // Basic features.
    // These features are applied in order, one at a time, after reordering.
    (Tag::from_bytes(b"pref"), FeatureFlags::MANUAL_JOINERS),
    (Tag::from_bytes(b"blwf"), FeatureFlags::MANUAL_JOINERS),
    (Tag::from_bytes(b"abvf"), FeatureFlags::MANUAL_JOINERS),
    (Tag::from_bytes(b"pstf"), FeatureFlags::MANUAL_JOINERS),
    (Tag::from_bytes(b"cfar"), FeatureFlags::MANUAL_JOINERS),
    // Other features.
    // These features are applied all at once after clearing syllables.
    (Tag::from_bytes(b"pres"), FeatureFlags::GLOBAL_MANUAL_JOINERS),
    (Tag::from_bytes(b"abvs"), FeatureFlags::GLOBAL_MANUAL_JOINERS),
    (Tag::from_bytes(b"blws"), FeatureFlags::GLOBAL_MANUAL_JOINERS),
    (Tag::from_bytes(b"psts"), FeatureFlags::GLOBAL_MANUAL_JOINERS),
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
    fn new(map: &Map) -> Self {
        let mut mask_array = [0; KHMER_FEATURES.len()];
        for (i, feature) in KHMER_FEATURES.iter().enumerate() {
            mask_array[i] = if feature.1.contains(FeatureFlags::GLOBAL) {
                0
            } else {
                map.get_1_mask(feature.0)
            }
        }

        KhmerShapePlan {
            mask_array,
        }
    }
}

#[no_mangle]
pub extern "C" fn rb_complex_khmer_data_create(map: *const ffi::rb_ot_map_t) -> *mut c_void {
    let plan = KhmerShapePlan::new(Map::from_ptr(map));
    Box::into_raw(Box::new(plan)) as *mut _
}

#[no_mangle]
pub extern "C" fn rb_complex_khmer_data_destroy(data: *mut c_void) {
    unsafe { Box::from_raw(data as *mut KhmerShapePlan) };
}

extern "C" {
    fn hb_complex_khmer_setup_syllables(
        plan: *const ffi::hb_shape_plan_t,
        font: *mut ffi::hb_font_t,
        buffer: *mut ffi::rb_buffer_t,
    );

    fn hb_complex_khmer_reorder(
        plan: *const ffi::hb_shape_plan_t,
        font: *mut ffi::hb_font_t,
        buffer: *mut ffi::rb_buffer_t,
    );
}

#[no_mangle]
pub extern "C" fn rb_complex_khmer_collect_features(builder: *mut ffi::rb_ot_map_builder_t) {
    let builder = MapBuilder::from_ptr_mut(builder);

    // Do this before any lookups have been applied.
    builder.add_gsub_pause(Some(hb_complex_khmer_setup_syllables));
    builder.add_gsub_pause(Some(hb_complex_khmer_reorder));

    // Testing suggests that Uniscribe does NOT pause between basic
    // features.  Test with KhmerUI.ttf and the following three
    // sequences:
    //
    //   U+1789,U+17BC
    //   U+1789,U+17D2,U+1789
    //   U+1789,U+17D2,U+1789,U+17BC
    //
    // https://github.com/harfbuzz/harfbuzz/issues/974
    builder.enable_feature(Tag::from_bytes(b"locl"), FeatureFlags::default(), 1);
    builder.enable_feature(Tag::from_bytes(b"ccmp"), FeatureFlags::default(), 1);

    for feature in KHMER_FEATURES.iter().take(5) {
        builder.add_feature(feature.0, feature.1, 1);
    }

    builder.add_gsub_pause(Some(super::hb_layout_clear_syllables));

    for feature in KHMER_FEATURES.iter().skip(5) {
        builder.add_feature(feature.0, feature.1, 1);
    }
}

#[no_mangle]
pub extern "C" fn rb_complex_khmer_override_features(builder: *mut ffi::rb_ot_map_builder_t) {
    let builder = MapBuilder::from_ptr_mut(builder);

    // Khmer spec has 'clig' as part of required shaping features:
    // "Apply feature 'clig' to form ligatures that are desired for
    // typographical correctness.", hence in overrides...
    builder.enable_feature(Tag::from_bytes(b"clig"), FeatureFlags::default(), 1);

    // Uniscribe does not apply 'kern' in Khmer.
    // TODO: this, `override_features_khmer` doesn't have a pointer to KhmerShapePlan
//    if plan.uniscribe_bug_compatible {
//        builder.disable_feature(Tag::from_bytes(b"kern"));
//    }

    builder.disable_feature(Tag::from_bytes(b"liga"));
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
pub extern "C" fn rb_complex_khmer_decompose(
    ab: CodePoint,
    a: *mut CodePoint,
    b: *mut CodePoint,
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

    crate::unicode::rb_ucd_decompose(ab, a, b) != 0
}

#[no_mangle]
pub extern "C" fn rb_complex_khmer_compose(
    a: CodePoint,
    b: CodePoint,
    ab: *mut CodePoint,
) -> bool {
    // Avoid recomposing split matras.
    if char::try_from(a).unwrap().general_category().is_unicode_mark() {
        return false;
    }

    crate::unicode::rb_ucd_compose(a, b, ab) != 0
}

fn insert_dotted_circles(font: &ttf_parser::Font, buffer: &mut Buffer) {
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

    let dottedcircle_glyph = match font.glyph_index('\u{25CC}') {
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
    plan: &KhmerShapePlan,
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
            reorder_consonant_syllable(plan, start, end, buffer);
        }
        SyllableType::NonKhmerCluster => {}
    }
}

#[no_mangle]
pub extern "C" fn rb_complex_khmer_reorder(
    plan: *const c_void,
    ttf_parser_data: *const ffi::rb_ttf_parser_t,
    buffer: *mut ffi::rb_buffer_t,
) {
    let plan = unsafe { &*(plan as *const KhmerShapePlan) };
    let buffer = Buffer::from_ptr_mut(buffer);

    insert_dotted_circles(
        crate::font::ttf_parser_from_raw(ttf_parser_data),
        buffer,
    );

    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len() {
        reorder_syllable(plan, start, end, buffer);
        start = end;
        end = buffer.next_syllable(start);
    }
}

#[no_mangle]
pub extern "C" fn rb_complex_khmer_setup_masks(
    buffer: *mut ffi::rb_buffer_t,
) {
    // We cannot setup masks here.  We save information about characters
    // and setup masks later on in a pause-callback.
    let buffer = Buffer::from_ptr_mut(buffer);
    for info in buffer.info_slice_mut() {
        info.set_khmer_properties();
    }
}

#[no_mangle]
pub extern "C" fn rb_complex_khmer_setup_syllables(
    buffer: *mut ffi::rb_buffer_t,
) {
    let buffer = Buffer::from_ptr_mut(buffer);

    super::khmer_machine::find_syllables_khmer(buffer);

    let mut start = 0;
    let mut end = buffer.next_syllable(0);
    while start < buffer.len() {
        buffer.unsafe_to_break(start, end);
        start = end;
        end = buffer.next_syllable(start);
    }
}
