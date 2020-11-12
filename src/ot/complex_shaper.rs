use std::ffi::c_void;

use super::{ShapeNormalizationMode, ShapePlanner};
use crate::buffer::Buffer;
use crate::common::TagExt;
use crate::ot::*;
use crate::{complex, ffi, script, Face, Tag};

pub const MAX_COMBINING_MARKS: usize = 32;

pub const DEFAULT_SHAPER: ComplexShaper = ComplexShaper {
    collect_features: None,
    override_features: None,
    data_create: None,
    data_destroy: None,
    preprocess_text: None,
    postprocess_glyphs: None,
    normalization_mode: Some(ShapeNormalizationMode::Auto),
    decompose: None,
    compose: None,
    setup_masks: None,
    gpos_tag: None,
    reorder_marks: None,
    zero_width_marks: Some(ZeroWidthMarksMode::ByGdefLate),
    fallback_position: true,
};

// Same as default but no mark advance zeroing / fallback positioning.
// Dumbest shaper ever, basically.
pub const DUMBER_SHAPER: ComplexShaper = ComplexShaper {
    collect_features: None,
    override_features: None,
    data_create: None,
    data_destroy: None,
    preprocess_text: None,
    postprocess_glyphs: None,
    normalization_mode: Some(ShapeNormalizationMode::Auto),
    decompose: None,
    compose: None,
    setup_masks: None,
    gpos_tag: None,
    reorder_marks: None,
    zero_width_marks: None,
    fallback_position: false,
};

pub struct ComplexShaper {
    /// Called during `shape_plan()`.
    /// Shapers should use plan.map to add their features and callbacks.
    pub collect_features: Option<fn(&mut ShapePlanner)>,

    /// Called during `shape_plan()`.
    /// Shapers should use plan.map to override features and add callbacks after
    /// common features are added.
    pub override_features: Option<fn(&mut ShapePlanner)>,

    /// Called at the end of `shape_plan()`.
    /// Whatever shapers return will be accessible through plan.data later.
    pub data_create: Option<fn(&ShapePlan) -> *mut c_void>,

    /// Called when the shape plan is being destroyed.
    /// plan.data is passed here for destruction.
    /// If nullptr is returned, means a plan failure.
    pub data_destroy: Option<fn(*mut c_void)>,

    /// Called during `shape()`.
    /// Shapers can use to modify text before shaping starts.
    pub preprocess_text: Option<fn(&ShapePlan, &Face, &mut Buffer)>,

    /// Called during `shape()`.
    /// Shapers can use to modify text before shaping starts.
    pub postprocess_glyphs: Option<fn(&ShapePlan, &Face, &mut Buffer)>,

    /// How to normalize.
    pub normalization_mode: Option<ShapeNormalizationMode>,

    /// Called during `shape()`'s normalization.
    pub decompose: Option<fn(&ShapeNormalizeContext, char) -> Option<(char, char)>>,

    /// Called during `shape()`'s normalization.
    pub compose: Option<fn(&ShapeNormalizeContext, char, char) -> Option<char>>,

    /// Called during `shape()`.
    /// Shapers should use map to get feature masks and set on buffer.
    /// Shapers may NOT modify characters.
    pub setup_masks: Option<fn(&ShapePlan, &Face, &mut Buffer)>,

    /// If not `Tag(0)`, then must match found GPOS script tag for
    /// GPOS to be applied.  Otherwise, fallback positioning will be used.
    pub gpos_tag: Option<Tag>,

    /// Called during `shape()`.
    /// Shapers can use to modify ordering of combining marks.
    pub reorder_marks: Option<fn(&ShapePlan, &mut Buffer, usize, usize)>,

    /// If and when to zero-width marks.
    pub zero_width_marks: Option<ZeroWidthMarksMode>,

    /// Whether to use fallback mark positioning.
    pub fallback_position: bool,
}

impl ComplexShaper {
    #[inline]
    pub fn from_ptr(shaper: *const ffi::rb_ot_complex_shaper_t) -> &'static ComplexShaper {
        unsafe { &*(shaper as *const ComplexShaper) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const ffi::rb_ot_complex_shaper_t {
        self as *const _ as *const ffi::rb_ot_complex_shaper_t
    }
}

pub enum ZeroWidthMarksMode {
    ByGdefEarly,
    ByGdefLate,
}

#[no_mangle]
pub extern "C" fn rb_ot_shape_complex_categorize(
    planner: *const ffi::rb_ot_shape_planner_t
) -> *const ffi::rb_ot_complex_shaper_t {
    let planner = ShapePlanner::from_ptr(planner);
    complex_categorize(&planner).as_ptr()
}

fn complex_categorize(planner: &ShapePlanner) -> &'static ComplexShaper {
    let script = planner.script();
    let chosen = planner.map.chosen_script[TableIndex::GSUB as usize];
    match script {
        // Unicode-1.1 additions
        script::ARABIC

        // Unicode-3.0 additions
        | script::MONGOLIAN
        | script::SYRIAC

        // Unicode-5.0 additions
        | script::NKO
        | script::PHAGS_PA

        // Unicode-6.0 additions
        | script::MANDAIC

        // Unicode-7.0 additions
        | script::MANICHAEAN
        | script::PSALTER_PAHLAVI

        // Unicode-9.0 additions
        | script::ADLAM

        // Unicode-11.0 additions
        | script::HANIFI_ROHINGYA
        | script::SOGDIAN => {
            // For Arabic script, use the Arabic shaper even if no OT script tag was found.
            // This is because we do fallback shaping for Arabic script (and not others).
            // But note that Arabic shaping is applicable only to horizontal layout; for
            // vertical text, just use the generic shaper instead.
            //
            // TODO: Does this still apply? Arabic fallback shaping was removed.
            if (chosen != Tag::default_script() || script == script::ARABIC)
                && planner.direction().is_horizontal()
            {
                &complex::arabic::ARABIC_SHAPER
            } else {
                &DEFAULT_SHAPER
            }
        }

        // Unicode-1.1 additions
        script::THAI
        | script::LAO => &complex::thai::THAI_SHAPER,

        // Unicode-1.1 additions
        script::HANGUL => &complex::hangul::HANGUL_SHAPER,

        // Unicode-1.1 additions
        script::HEBREW => &complex::hebrew::HEBREW_SHAPER,

        // Unicode-1.1 additions
        script::BENGALI
        | script::DEVANAGARI
        | script::GUJARATI
        | script::GURMUKHI
        | script::KANNADA
        | script::MALAYALAM
        | script::ORIYA
        | script::TAMIL
        | script::TELUGU

        // Unicode-3.0 additions
        | script::SINHALA => {
            // If the designer designed the font for the 'DFLT' script,
            // (or we ended up arbitrarily pick 'latn'), use the default shaper.
            // Otherwise, use the specific shaper.
            //
            // If it's indy3 tag, send to USE.
            if chosen == Tag::default_script() || chosen == Tag::from_bytes(b"latn") {
                &DEFAULT_SHAPER
            } else if chosen.0 as u8 == b'3' {
                &complex::universal::UNIVERSAL_SHAPER
            } else {
                &complex::indic::INDIC_SHAPER
            }
        }

        script::KHMER => &complex::khmer::KHMER_SHAPER,

        script::MYANMAR => {
            // If the designer designed the font for the 'DFLT' script,
            // (or we ended up arbitrarily pick 'latn'), use the default shaper.
            // Otherwise, use the specific shaper.
            //
            // If designer designed for 'mymr' tag, also send to default
            // shaper.  That's tag used from before Myanmar shaping spec
            // was developed.  The shaping spec uses 'mym2' tag.
            if chosen == Tag::default_script()
                || chosen == Tag::from_bytes(b"latn")
                || chosen == Tag::from_bytes(b"mymr")
            {
                &DEFAULT_SHAPER
            } else {
                &complex::myanmar::MYANMAR_SHAPER
            }
        }

        // https://github.com/harfbuzz/harfbuzz/issues/1162
        script::MYANMAR_ZAWGYI => &complex::myanmar::MYANMAR_ZAWGYI_SHAPER,

        // Unicode-2.0 additions
        script::TIBETAN

        // Unicode-3.0 additions
        // | script::MONGOLIAN
        // | script::SINHALA

        // Unicode-3.2 additions
        | script::BUHID
        | script::HANUNOO
        | script::TAGALOG
        | script::TAGBANWA

        // Unicode-4.0 additions
        | script::LIMBU
        | script::TAI_LE

        // Unicode-4.1 additions
        | script::BUGINESE
        | script::KHAROSHTHI
        | script::SYLOTI_NAGRI
        | script::TIFINAGH

        // Unicode-5.0 additions
        | script::BALINESE
        // | script::NKO
        // | script::PHAGS_PA

        // Unicode-5.1 additions
        | script::CHAM
        | script::KAYAH_LI
        | script::LEPCHA
        | script::REJANG
        | script::SAURASHTRA
        | script::SUNDANESE

        // Unicode-5.2 additions
        | script::EGYPTIAN_HIEROGLYPHS
        | script::JAVANESE
        | script::KAITHI
        | script::MEETEI_MAYEK
        | script::TAI_THAM
        | script::TAI_VIET

        // Unicode-6.0 additions
        | script::BATAK
        | script::BRAHMI
        // | script::MANDAIC

        // Unicode-6.1 additions
        | script::CHAKMA
        | script::SHARADA
        | script::TAKRI

        // Unicode-7.0 additions
        | script::DUPLOYAN
        | script::GRANTHA
        | script::KHOJKI
        | script::KHUDAWADI
        | script::MAHAJANI
        // | script::MANICHAEAN
        | script::MODI
        | script::PAHAWH_HMONG
        // | script::PSALTER_PAHLAVI
        | script::SIDDHAM
        | script::TIRHUTA

        // Unicode-8.0 additions
        | script::AHOM

        // Unicode-9.0 additions
        // | script::ADLAM
        | script::BHAIKSUKI
        | script::MARCHEN
        | script::NEWA

        // Unicode-10.0 additions
        | script::MASARAM_GONDI
        | script::SOYOMBO
        | script::ZANABAZAR_SQUARE

        // Unicode-11.0 additions
        | script::DOGRA
        | script::GUNJALA_GONDI
        // | script::HANIFI_ROHINGYA
        | script::MAKASAR
        // | script::SOGDIAN

        // Unicode-12.0 additions
        | script::NANDINAGARI

        // Unicode-13.0 additions
        | script::CHORASMIAN
        | script::DIVES_AKURU => {
            // If the designer designed the font for the 'DFLT' script,
            // (or we ended up arbitrarily pick 'latn'), use the default shaper.
            // Otherwise, use the specific shaper.
            // Note that for some simple scripts, there may not be *any*
            // GSUB/GPOS needed, so there may be no scripts found!
            if chosen == Tag::default_script() || chosen == Tag::from_bytes(b"latn") {
                &DEFAULT_SHAPER
            } else {
                &complex::universal::UNIVERSAL_SHAPER
            }
        }

        _ => &DEFAULT_SHAPER
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_complex_shaper_reconsider_shaper_if_applying_morx(
    shaper: *const ffi::rb_ot_complex_shaper_t,
) -> *const ffi::rb_ot_complex_shaper_t {
    if shaper == DEFAULT_SHAPER.as_ptr() {
        shaper
    } else {
        DUMBER_SHAPER.as_ptr()
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_complex_shaper_collect_features(
    shaper: *const ffi::rb_ot_complex_shaper_t,
    planner: *mut ffi::rb_ot_shape_planner_t,
) {
    let shaper = ComplexShaper::from_ptr(shaper);
    let mut planner = ShapePlanner::from_ptr_mut(planner);
    if let Some(func) = shaper.collect_features {
        func(&mut planner);
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_complex_shaper_override_features(
    shaper: *const ffi::rb_ot_complex_shaper_t,
    planner: *mut ffi::rb_ot_shape_planner_t,
) {
    let shaper = ComplexShaper::from_ptr(shaper);
    let mut planner = ShapePlanner::from_ptr_mut(planner);
    if let Some(func) = shaper.override_features {
        func(&mut planner);
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_complex_shaper_data_create(
    shaper: *const ffi::rb_ot_complex_shaper_t,
    plan: *const ffi::rb_ot_shape_plan_t,
    data: *mut *mut c_void,
) -> ffi::rb_bool_t {
    let shaper = ComplexShaper::from_ptr(shaper);
    let plan = ShapePlan::from_ptr(plan);
    if let Some(func) = shaper.data_create {
        let ptr = func(&plan);
        if ptr.is_null() {
            return 0;
        }
        unsafe { *data = ptr; }
    }
    1
}

#[no_mangle]
pub extern "C" fn rb_ot_complex_shaper_data_destroy(
    shaper: *const ffi::rb_ot_complex_shaper_t,
    data: *mut c_void,
) {
    let shaper = ComplexShaper::from_ptr(shaper);
    if let Some(func) = shaper.data_destroy {
        func(data);
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_complex_shaper_preprocess_text(
    shaper: *const ffi::rb_ot_complex_shaper_t,
    plan: *const ffi::rb_ot_shape_plan_t,
    buffer: *mut ffi::rb_buffer_t,
    face: *const ffi::rb_face_t,
) {
    let shaper = ComplexShaper::from_ptr(shaper);
    let plan = ShapePlan::from_ptr(plan);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    let face = Face::from_ptr(face);
    if let Some(func) = shaper.preprocess_text {
        func(&plan, face, &mut buffer);
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_complex_shaper_postprocess_glyphs(
    shaper: *const ffi::rb_ot_complex_shaper_t,
    plan: *const ffi::rb_ot_shape_plan_t,
    buffer: *mut ffi::rb_buffer_t,
    face: *const ffi::rb_face_t,
) {
    let shaper = ComplexShaper::from_ptr(shaper);
    let plan = ShapePlan::from_ptr(plan);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    let face = Face::from_ptr(face);
    if let Some(func) = shaper.postprocess_glyphs {
        func(&plan, face, &mut buffer);
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_complex_shaper_setup_masks(
    shaper: *const ffi::rb_ot_complex_shaper_t,
    plan: *const ffi::rb_ot_shape_plan_t,
    buffer: *mut ffi::rb_buffer_t,
    face: *const ffi::rb_face_t,
) {
    let shaper = ComplexShaper::from_ptr(shaper);
    let plan = ShapePlan::from_ptr(plan);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    let face = Face::from_ptr(face);
    if let Some(func) = shaper.setup_masks {
        func(&plan, face, &mut buffer);
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_complex_shaper_get_gpos_tag(shaper: *const ffi::rb_ot_complex_shaper_t) -> Tag {
    ComplexShaper::from_ptr(shaper).gpos_tag.unwrap_or(Tag(0))
}

#[no_mangle]
pub extern "C" fn rb_ot_complex_shaper_reorder_marks(
    shaper: *const ffi::rb_ot_complex_shaper_t,
    plan: *const ffi::rb_ot_shape_plan_t,
    buffer: *mut ffi::rb_buffer_t,
    start: u32,
    end: u32,
) {
    let shaper = ComplexShaper::from_ptr(shaper);
    let plan = ShapePlan::from_ptr(plan);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    if let Some(func) = shaper.reorder_marks {
        func(&plan, &mut buffer, start as usize, end as usize);
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_complex_shaper_get_zero_width_marks_mode(
    shaper: *const ffi::rb_ot_complex_shaper_t,
) -> ffi::rb_ot_shape_zero_width_marks_mode_t {
    let shaper = ComplexShaper::from_ptr(shaper);
    match shaper.zero_width_marks {
        Some(ZeroWidthMarksMode::ByGdefEarly) => ffi::RB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_EARLY,
        Some(ZeroWidthMarksMode::ByGdefLate) => ffi::RB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE,
        None => ffi::RB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_complex_shaper_get_fallback_position(
    shaper: *const ffi::rb_ot_complex_shaper_t,
) -> ffi::rb_bool_t {
    ComplexShaper::from_ptr(shaper).fallback_position as ffi::rb_bool_t
}
