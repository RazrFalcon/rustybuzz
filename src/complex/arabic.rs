use crate::{Buffer, GlyphInfo, Tag, Script, script};
use crate::buffer::BufferScratchFlags;
use crate::map::{MapBuilder, FeatureFlags};
use crate::unicode::{CharExt, GeneralCategory, GeneralCategoryExt, modified_combining_class};
use crate::ffi;
use super::{hb_flag_unsafe, hb_flag};

use super::arabic_table;

extern "C" {
    fn hb_complex_arabic_record_stch(
        plan: *const ffi::hb_shape_plan_t,
        font: *mut ffi::hb_font_t,
        buffer: *mut ffi::rb_buffer_t,
    );

    fn hb_complex_arabic_fallback_shape(
        plan: *const ffi::hb_shape_plan_t,
        font: *mut ffi::hb_font_t,
        buffer: *mut ffi::rb_buffer_t,
    );
}

const ARABIC_FEATURES: &[Tag] = &[
    Tag::from_bytes(b"isol"),
    Tag::from_bytes(b"fina"),
    Tag::from_bytes(b"fin2"),
    Tag::from_bytes(b"fin3"),
    Tag::from_bytes(b"medi"),
    Tag::from_bytes(b"med2"),
    Tag::from_bytes(b"init"),
];

fn feature_is_syriac(tag: Tag) -> bool {
    matches!(tag.to_bytes()[3], b'2' | b'3')
}

impl GlyphInfo {
    fn arabic_shaping_action(&self) -> Action {
        unsafe {
            let v: &ffi::hb_var_int_t = std::mem::transmute(&self.var2);
            std::mem::transmute(v.var_u8[2])
        }
    }

    fn set_arabic_shaping_action(&mut self, action: Action) {
        unsafe {
            let v: &mut ffi::hb_var_int_t = std::mem::transmute(&mut self.var2);
            v.var_u8[2] = action as u8;
        }
    }
}

fn collect_features_arabic(map: &mut MapBuilder, script: Script) {
    // We apply features according to the Arabic spec, with pauses
    // in between most.
    //
    // The pause between init/medi/... and rlig is required.  See eg:
    // https://bugzilla.mozilla.org/show_bug.cgi?id=644184
    //
    // The pauses between init/medi/... themselves are not necessarily
    // needed as only one of those features is applied to any character.
    // The only difference it makes is when fonts have contextual
    // substitutions.  We now follow the order of the spec, which makes
    // for better experience if that's what Uniscribe is doing.
    //
    // At least for Arabic, looks like Uniscribe has a pause between
    // rlig and calt.  Otherwise the IranNastaliq's ALLAH ligature won't
    // work.  However, testing shows that rlig and calt are applied
    // together for Mongolian in Uniscribe.  As such, we only add a
    // pause for Arabic, not other scripts.
    //
    // A pause after calt is required to make KFGQPC Uthmanic Script HAFS
    // work correctly.  See https://github.com/harfbuzz/harfbuzz/issues/505

    map.enable_feature(Tag::from_bytes(b"stch"), FeatureFlags::default(), 1);
    map.add_gsub_pause(Some(hb_complex_arabic_record_stch));

    map.enable_feature(Tag::from_bytes(b"ccmp"), FeatureFlags::default(), 1);
    map.enable_feature(Tag::from_bytes(b"locl"), FeatureFlags::default(), 1);

    map.add_gsub_pause(None);

    for feature in ARABIC_FEATURES.iter().cloned() {
        let has_fallback = script == script::ARABIC && !feature_is_syriac(feature);
        let flags = if has_fallback { FeatureFlags::HAS_FALLBACK } else { FeatureFlags::default() };
        map.add_feature(feature, flags, 1);
        map.add_gsub_pause(None);
    }

    // Normally, Unicode says a ZWNJ means "don't ligate".  In Arabic script
    // however, it says a ZWJ should also mean "don't ligate".  So we run
    // the main ligating features as MANUAL_ZWJ.

    map.enable_feature(Tag::from_bytes(b"rlig"), FeatureFlags::MANUAL_ZWJ | FeatureFlags::HAS_FALLBACK, 1);

    if script == script::ARABIC {
        map.add_gsub_pause(Some(hb_complex_arabic_fallback_shape));
    }

    // No pause after rclt.
    // See https://github.com/harfbuzz/harfbuzz/commit/98460779bae19e4d64d29461ff154b3527bf8420
    map.enable_feature(Tag::from_bytes(b"rclt"), FeatureFlags::MANUAL_ZWJ, 1);
    map.enable_feature(Tag::from_bytes(b"calt"), FeatureFlags::MANUAL_ZWJ, 1);
    map.add_gsub_pause(None);

    // And undo here.

    // The spec includes 'cswh'.  Earlier versions of Windows
    // used to enable this by default, but testing suggests
    // that Windows 8 and later do not enable it by default,
    // and spec now says 'Off by default'.
    // We disabled this in ae23c24c32.
    // Note that IranNastaliq uses this feature extensively
    // to fixup broken glyph sequences.  Oh well...
    // Test case: U+0643,U+0640,U+0631.

    // map->enable_feature (HB_TAG('c','s','w','h'));
    map.enable_feature(Tag::from_bytes(b"mset"), FeatureFlags::default(), 1);
}

#[no_mangle]
pub extern "C" fn rb_complex_arabic_collect_features(map: *mut ffi::rb_ot_map_builder_t, script: Tag) {
    let map = unsafe { &mut *(map as *mut MapBuilder) };
    let script = Script(script);
    collect_features_arabic(map, script);
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum JoiningType {
    U = 0,
    L = 1,
    R = 2,
    D = 3,
    // We don't have C, like harfbuzz, because Rust doesn't allow duplicated enum variants.
    GroupAlaph = 4,
    GroupDalathRish = 5,
    T = 7,
    X = 8, // means: use general-category to choose between U or T.
}

fn get_joining_type(u: char, gc: GeneralCategory) -> JoiningType {
    let j_type = arabic_table::joining_type(u);
    if j_type != JoiningType::X {
        return j_type;
    }

    let ok = hb_flag_unsafe(gc.to_hb()) &
        (hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK) |
         hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_ENCLOSING_MARK) |
         hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_FORMAT));

    if ok != 0 { JoiningType::T } else { JoiningType::U }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug)]
enum Action {
    ISOL = 0,
    FINA = 1,
    FIN2 = 2,
    FIN3 = 3,
    MEDI = 4,
    MED2 = 5,
    INIT = 6,
    NONE = 7,

    // We abuse the same byte for other things...
    StretchingFixed = 8,
    StretchingRepeating = 9,
}

const STATE_TABLE: &[[(Action, Action, u16); 6]] = &[
    // jt_U,          jt_L,          jt_R,
    // jt_D,          jg_ALAPH,      jg_DALATH_RISH

    // State 0: prev was U, not willing to join.
    [
        (Action::NONE, Action::NONE, 0), (Action::NONE, Action::ISOL, 2), (Action::NONE, Action::ISOL, 1),
        (Action::NONE, Action::ISOL, 2), (Action::NONE, Action::ISOL, 1), (Action::NONE, Action::ISOL, 6),
    ],

    // State 1: prev was R or Action::ISOL/ALAPH, not willing to join.
    [
        (Action::NONE, Action::NONE, 0), (Action::NONE, Action::ISOL, 2), (Action::NONE, Action::ISOL, 1),
        (Action::NONE, Action::ISOL, 2), (Action::NONE, Action::FIN2, 5), (Action::NONE, Action::ISOL, 6),
    ],

    // State 2: prev was D/L in Action::ISOL form, willing to join.
    [
        (Action::NONE, Action::NONE, 0), (Action::NONE, Action::ISOL, 2), (Action::INIT, Action::FINA, 1),
        (Action::INIT, Action::FINA, 3), (Action::INIT, Action::FINA, 4), (Action::INIT, Action::FINA, 6),
    ],

    // State 3: prev was D in Action::FINA form, willing to join.
    [
        (Action::NONE, Action::NONE, 0), (Action::NONE, Action::ISOL, 2), (Action::MEDI, Action::FINA, 1),
        (Action::MEDI, Action::FINA, 3), (Action::MEDI, Action::FINA, 4), (Action::MEDI, Action::FINA, 6),
    ],

    // State 4: prev was Action::FINA ALAPH, not willing to join.
    [
        (Action::NONE, Action::NONE, 0), (Action::NONE, Action::ISOL, 2), (Action::MED2, Action::ISOL, 1),
        (Action::MED2, Action::ISOL, 2), (Action::MED2, Action::FIN2, 5), (Action::MED2, Action::ISOL, 6),
    ],

    // State 5: prev was FIN2/FIN3 ALAPH, not willing to join.
    [
        (Action::NONE, Action::NONE, 0), (Action::NONE, Action::ISOL, 2), (Action::ISOL, Action::ISOL, 1),
        (Action::ISOL, Action::ISOL, 2), (Action::ISOL, Action::FIN2, 5), (Action::ISOL, Action::ISOL, 6),
    ],

    // State 6: prev was DALATH/RISH, not willing to join.
    [
        (Action::NONE, Action::NONE, 0), (Action::NONE, Action::ISOL, 2), (Action::NONE, Action::ISOL, 1),
        (Action::NONE, Action::ISOL, 2), (Action::NONE, Action::FIN3, 5), (Action::NONE, Action::ISOL, 6),
    ]
];

fn arabic_joining(buffer: &mut Buffer) {
    let mut prev: Option<usize> = None;
    let mut state = 0;

    // Check pre-context.
    for i in 0..buffer.context_len[0] {
        let c = buffer.context[0][i];
        let this_type = get_joining_type(c, c.general_category());
        if this_type == JoiningType::T {
            continue;
        }

        state = STATE_TABLE[state][this_type as usize].2 as usize;
        break;
    }

    for i in 0..buffer.len() {
        let this_type = get_joining_type(buffer.info[i].as_char(), buffer.info[i].general_category());
        if this_type == JoiningType::T {
            buffer.info[i].set_arabic_shaping_action(Action::NONE);
            continue;
        }

        let entry = &STATE_TABLE[state][this_type as usize];
        if entry.0 != Action::NONE && prev.is_some() {
            if let Some(prev) = prev {
                buffer.info[prev].set_arabic_shaping_action(entry.0);
                buffer.unsafe_to_break(prev, i + 1);
            }
        }

        buffer.info[i].set_arabic_shaping_action(entry.1);

        prev = Some(i);
        state = entry.2 as usize;
    }

    for i in 0..buffer.context_len[1] {
        let c = buffer.context[1][i];
        let this_type = get_joining_type(c, c.general_category());
        if this_type == JoiningType::T {
            continue;
        }

        let entry = &STATE_TABLE[state][this_type as usize];
        if entry.0 != Action::NONE && prev.is_some() {
            if let Some(prev) = prev {
                buffer.info[prev].set_arabic_shaping_action(entry.0);
            }
        }

        break;
    }
}

#[no_mangle]
pub extern "C" fn rb_complex_arabic_joining(buffer: *mut ffi::rb_buffer_t) {
    arabic_joining(Buffer::from_ptr_mut(buffer));
}

fn apply_stch(font: *mut ffi::hb_font_t, buffer: &mut Buffer) {
    if !buffer.scratch_flags.contains(BufferScratchFlags::COMPLEX0) {
        return;
    }

    // The Arabic shaper currently always processes in RTL mode, so we should
    // stretch / position the stretched pieces to the left / preceding glyphs.

    // We do a two pass implementation:
    // First pass calculates the exact number of extra glyphs we need,
    // We then enlarge buffer to have that much room,
    // Second pass applies the stretch, copying things to the end of buffer.

    let sign = {
        let mut scale = (0i32, 0i32);
        unsafe { ffi::hb_font_get_scale(font, &mut scale.0, &mut scale.1) };
        scale.0.signum()
    };

    let mut extra_glyphs_needed: usize = 0; // Set during MEASURE, used during CUT
    const MEASURE: usize = 0;
    const CUT: usize = 1;

    for step in 0..2 {
        let new_len = buffer.len() + extra_glyphs_needed; // write head during CUT
        let mut i = buffer.len();
        let mut j = new_len;
        while i != 0 {
            if !is_stch_action(buffer.info[i - 1].arabic_shaping_action()) {
                if step == CUT {
                    j -= 1;
                    buffer.info[j] = buffer.info[i - 1];
                    buffer.pos[j] = buffer.pos[i - 1];
                }

                i -= 1;
                continue;
            }

            // Yay, justification!

            let mut w_total = 0;     // Total to be filled
            let mut w_fixed = 0;     // Sum of fixed tiles
            let mut w_repeating = 0; // Sum of repeating tiles
            let mut n_repeating: i32 = 0;

            let end = i;
            while i != 0 && is_stch_action(buffer.info[i - 1].arabic_shaping_action()) {
                i -= 1;
                let width = unsafe { ffi::hb_font_get_glyph_h_advance_default(
                    font, buffer.info[i].codepoint,
                )};

                if buffer.info[i].arabic_shaping_action() == Action::StretchingFixed {
                    w_fixed += width;
                } else {
                    w_repeating += width;
                    n_repeating += 1;
                }
            }

            let start = i;
            let mut context = i;
            while context != 0 &&
                !is_stch_action(buffer.info[context - 1].arabic_shaping_action()) &&
                (buffer.info[context - 1].is_default_ignorable() ||
                    is_word_category(buffer.info[context - 1].general_category()))
            {
                context -= 1;
                w_total += buffer.pos[context].x_advance;
            }

            i += 1; // Don't touch i again.

            // Number of additional times to repeat each repeating tile.
            let mut n_copies: i32 = 0;

            let w_remaining = w_total - w_fixed;
            if sign * w_remaining > sign * w_repeating && sign * w_repeating > 0 {
                n_copies = (sign * w_remaining) / (sign * w_repeating) - 1;
            }

            // See if we can improve the fit by adding an extra repeat and squeezing them together a bit.
            let mut extra_repeat_overlap = 0;
            let shortfall = sign * w_remaining - sign * w_repeating * (n_copies + 1);
            if shortfall > 0 && n_repeating > 0 {
                n_copies += 1;
                let excess = (n_copies + 1) * sign * w_repeating - sign * w_remaining;
                if excess > 0 {
                    extra_repeat_overlap = excess / (n_copies * n_repeating);
                }
            }

            if step == MEASURE {
                extra_glyphs_needed += (n_copies * n_repeating) as usize;
            } else {
                buffer.unsafe_to_break(context, end);
                let mut x_offset = 0;
                for k in (start+1..=end).rev() {
                    let width = unsafe { ffi::hb_font_get_glyph_h_advance_default(
                        font, buffer.info[k - 1].codepoint,
                    )};

                    let mut repeat = 1;
                    if buffer.info[k - 1].arabic_shaping_action() == Action::StretchingRepeating {
                        repeat += n_copies;
                    }

                    for n in 0..repeat {
                        x_offset -= width;
                        if n > 0 {
                            x_offset += extra_repeat_overlap;
                        }

                        buffer.pos[k - 1].x_offset = x_offset;

                        // Append copy.
                        j -= 1;
                        buffer.info[j] = buffer.info[k - 1];
                        buffer.pos[j] = buffer.pos[k - 1];
                    }
                }
            }

            i -= 1;
        }

        if step == MEASURE {
            buffer.ensure(buffer.len() + extra_glyphs_needed);
        } else {
            debug_assert_eq!(j, 0);
            buffer.set_len(new_len);
        }
    }
}

// See:
// https://github.com/harfbuzz/harfbuzz/commit/6e6f82b6f3dde0fc6c3c7d991d9ec6cfff57823d#commitcomment-14248516
fn is_word_category(gc: GeneralCategory) -> bool {
    (hb_flag_unsafe(gc.to_hb()) &
        (   hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_UNASSIGNED) |
            hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_PRIVATE_USE) |
            hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_MODIFIER_LETTER) |
            hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_OTHER_LETTER) |
            hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_SPACING_MARK) |
            hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_ENCLOSING_MARK) |
            hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK) |
            hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_DECIMAL_NUMBER) |
            hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_LETTER_NUMBER) |
            hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_OTHER_NUMBER) |
            hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_CURRENCY_SYMBOL) |
            hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_MODIFIER_SYMBOL) |
            hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_MATH_SYMBOL) |
            hb_flag(ffi::HB_UNICODE_GENERAL_CATEGORY_OTHER_SYMBOL)
        )) != 0
}

fn is_stch_action(action: Action) -> bool {
    matches!(action, Action::StretchingFixed | Action::StretchingRepeating)
}

#[no_mangle]
pub extern "C" fn rb_complex_arabic_apply_stch(buffer: *mut ffi::rb_buffer_t, font: *mut ffi::hb_font_t) {
    apply_stch(font, Buffer::from_ptr_mut(buffer));
}

// http://www.unicode.org/reports/tr53/
const MODIFIER_COMBINING_MARKS: &[u32] = &[
    0x0654, // ARABIC HAMZA ABOVE
    0x0655, // ARABIC HAMZA BELOW
    0x0658, // ARABIC MARK NOON GHUNNA
    0x06DC, // ARABIC SMALL HIGH SEEN
    0x06E3, // ARABIC SMALL LOW SEEN
    0x06E7, // ARABIC SMALL HIGH YEH
    0x06E8, // ARABIC SMALL HIGH NOON
    0x08D3, // ARABIC SMALL LOW WAW
    0x08F3, // ARABIC SMALL HIGH WAW
];

const MAX_COMBINING_MARKS: usize = 32;

fn reorder_marks_arabic(mut start: usize, end: usize, buffer: &mut Buffer) {
    let mut i = start;
    for cc in [220u8, 230].iter().cloned() {
        while i < end && buffer.info[i].modified_combining_class() < cc {
            i += 1;
        }

        if i == end {
            break;
        }

        if buffer.info[i].modified_combining_class() > cc {
            continue;
        }

        let mut j = i;
        while j < end &&
            buffer.info[j].modified_combining_class() == cc &&
            MODIFIER_COMBINING_MARKS.contains(&buffer.info[j].codepoint)
        {
            j += 1;
        }

        if i == j {
            continue;
        }

        // Shift it!
        let mut temp = [GlyphInfo::default(); MAX_COMBINING_MARKS];
        debug_assert!(j - i <= MAX_COMBINING_MARKS);
        buffer.merge_clusters(start, j);

        for k in 0..j-i {
            temp[k] = buffer.info[k + i];
        }

        for k in (0..i-start).rev() {
            buffer.info[k + start + j - i] = buffer.info[k + start];
        }

        for k in 0..j-i {
            buffer.info[k + start] = temp[k];
        }

        // Renumber CC such that the reordered sequence is still sorted.
        // 22 and 26 are chosen because they are smaller than all Arabic categories,
        // and are folded back to 220/230 respectively during fallback mark positioning.
        //
        // We do this because the CGJ-handling logic in the normalizer relies on
        // mark sequences having an increasing order even after this reordering.
        // https://github.com/harfbuzz/harfbuzz/issues/554
        // This, however, does break some obscure sequences, where the normalizer
        // might compose a sequence that it should not.  For example, in the seequence
        // ALEF, HAMZAH, MADDAH, we should NOT try to compose ALEF+MADDAH, but with this
        // renumbering, we will.
        let new_start = start + j - i;
        let new_cc = if cc == 220 {
            modified_combining_class::CCC22
        } else {
            modified_combining_class::CCC26
        };

        while start < new_start {
            buffer.info[start].set_modified_combining_class(new_cc);
            start += 1;
        }

        i = j;
    }
}

#[no_mangle]
pub extern "C" fn rb_complex_arabic_reorder_marks(buffer: *mut ffi::rb_buffer_t, start: u32, end: u32) {
    reorder_marks_arabic(start as usize, end as usize, Buffer::from_ptr_mut(buffer));
}
