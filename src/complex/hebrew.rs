use std::convert::TryFrom;

use crate::{ffi, CodePoint, Tag};

const S_DAGESH_FORMS: &[CodePoint] = &[
    0xFB30, // ALEF
    0xFB31, // BET
    0xFB32, // GIMEL
    0xFB33, // DALET
    0xFB34, // HE
    0xFB35, // VAV
    0xFB36, // ZAYIN
    0x0000, // HET
    0xFB38, // TET
    0xFB39, // YOD
    0xFB3A, // FINAL KAF
    0xFB3B, // KAF
    0xFB3C, // LAMED
    0x0000, // FINAL MEM
    0xFB3E, // MEM
    0x0000, // FINAL NUN
    0xFB40, // NUN
    0xFB41, // SAMEKH
    0x0000, // AYIN
    0xFB43, // FINAL PE
    0xFB44, // PE
    0x0000, // FINAL TSADI
    0xFB46, // TSADI
    0xFB47, // QOF
    0xFB48, // RESH
    0xFB49, // SHIN
    0xFB4A, // TAV
];

fn compose(
    has_gpos_mark: bool,
    a: CodePoint,
    b: CodePoint,
) -> Option<CodePoint> {
    // Hebrew presentation-form shaping.
    // https://bugzilla.mozilla.org/show_bug.cgi?id=728866
    // Hebrew presentation forms with dagesh, for characters U+05D0..05EA;
    // Note that some letters do not have a dagesh presForm encoded.

    let a_char = char::try_from(a).unwrap();
    let b_char = char::try_from(b).unwrap();

    let mut ab = None;
    match unicode_normalization::char::compose(a_char, b_char) {
        Some(c) => ab = Some(c as CodePoint),
        None if !has_gpos_mark => {
            // Special-case Hebrew presentation forms that are excluded from
            // standard normalization, but wanted for old fonts.
            match b {
                0x05B4 => { // HIRIQ
                    if a == 0x05D9 { // YOD
                        ab = Some(0xFB1D);
                    }
                }
                0x05B7 => { // patah
                    match a {
                        0x05D9 => ab = Some(0xFB1F), // YIDDISH YOD YOD
                        0x05D0 => ab = Some(0xFB2E), // ALEF
                        _ => {}
                    }
                }
                0x05B8 => { // QAMATS
                    if a == 0x05D0 { // ALEF
                        ab = Some(0xFB2F);
                    }
                }
                0x05B9 => { // HOLAM
                    if a == 0x05D5 { // VAV
                        ab = Some(0xFB4B);
                    }
                }
                0x05BC => { // DAGESH
                    if (0x05D0..=0x05EA).contains(&a) {
                        let c = S_DAGESH_FORMS[a as usize - 0x05D0];
                        if c != 0x0000 {
                            ab = Some(c);
                        }
                    } else if a == 0xFB2A { // SHIN WITH SHIN DOT
                        ab = Some(0xFB2C);
                    } else if a == 0xFB2B { // SHIN WITH SIN DOT
                        ab = Some(0xFB2D);
                    }
                }
                0x05BF => { // RAFE
                    match a {
                        0x05D1 => ab = Some(0xFB4C), // BET
                        0x05DB => ab = Some(0xFB4D), // KAF
                        0x05E4 => ab = Some(0xFB4E), // PE
                        _ => {}
                    }
                }
                0x05C1 => { // SHIN DOT
                    match a {
                        0x05E9 => ab = Some(0xFB2A), // SHIN
                        0xFB49 => ab = Some(0xFB2C), // SHIN WITH DAGESH
                        _ => {}
                    }
                }
                0x05C2 => { // SIN DOT
                    match a {
                        0x05E9 => ab = Some(0xFB2B), // SHIN
                        0xFB49 => ab = Some(0xFB2D), // SHIN WITH DAGESH
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        None => {}
    }

    ab
}

#[no_mangle]
pub extern "C" fn rb_complex_hebrew_compose(
    c: *const ffi::hb_ot_shape_normalize_context_t,
    a: CodePoint,
    b: CodePoint,
    ab: *mut CodePoint,
) -> bool {
    let has_gpos_mark = unsafe { ffi::hb_ot_shape_normalize_context_has_gpos_mark(c) };
    if let Some(new_ab) = compose(has_gpos_mark, a, b) {
        unsafe { *ab = new_ab; }
        true
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn rb_create_hebrew_shaper() -> *const ffi::hb_ot_complex_shaper_t {
    let shaper = Box::new(ffi::hb_ot_complex_shaper_t {
        collect_features: None,
        override_features: None,
        data_create: None,
        data_destroy: None,
        preprocess_text: None,
        postprocess_glyphs: None,
        normalization_preference: ffi::HB_OT_SHAPE_NORMALIZATION_MODE_DEFAULT,
        decompose: None,
        compose: Some(rb_complex_hebrew_compose),
        setup_masks: None,
        // https://github.com/harfbuzz/harfbuzz/issues/347#issuecomment-267838368
        gpos_tag: Tag::from_bytes(b"hebr").0,
        reorder_marks: None,
        zero_width_marks: ffi::HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE,
        fallback_position: true,
    });
    Box::into_raw(shaper)
}
