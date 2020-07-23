use std::os::raw::c_void;

use crate::{ffi, Font, GlyphInfo, Mask};
use crate::buffer::{Buffer, BufferFlags, BufferClusterLevel};
use crate::ot::*;


const L_BASE: u32 = 0x1100;
const V_BASE: u32 = 0x1161;
const T_BASE: u32 = 0x11A7;
const L_COUNT: u32 = 19;
const V_COUNT: u32 = 21;
const T_COUNT: u32 = 28;
const N_COUNT: u32 = V_COUNT * T_COUNT;
const S_COUNT: u32 = L_COUNT * N_COUNT;
const S_BASE: u32 = 0xAC00;

const LJMO: u8 = 1;
const VJMO: u8 = 2;
const TJMO: u8 = 3;


impl GlyphInfo {
    fn hangul_shaping_feature(&self) -> u8 {
        unsafe {
            let v: &ffi::hb_var_int_t = std::mem::transmute(&self.var2);
            v.var_u8[2]
        }
    }

    fn set_hangul_shaping_feature(&mut self, feature: u8) {
        unsafe {
            let v: &mut ffi::hb_var_int_t = std::mem::transmute(&mut self.var2);
            v.var_u8[2] = feature;
        }
    }
}


struct HangulShapePlan {
    mask_array: [Mask; 4],
}

impl HangulShapePlan {
    fn new(map: &Map) -> Self {
        HangulShapePlan {
            mask_array: [
                0,
                map.get_1_mask(feature::LEADING_JAMO_FORMS),
                map.get_1_mask(feature::VOWEL_JAMO_FORMS),
                map.get_1_mask(feature::TRAILING_JAMO_FORMS),
            ]
        }
    }

    fn from_ptr(plan: *const c_void) -> &'static HangulShapePlan {
        unsafe { &*(plan as *const HangulShapePlan) }
    }
}


#[no_mangle]
pub extern "C" fn hb_ot_complex_collect_features_hangul(planner: *mut ffi::hb_ot_shape_planner_t) {
    let mut planner = ShapePlanner::from_ptr_mut(planner);
    collect_features(&mut planner)
}

fn collect_features(planner: &mut ShapePlanner) {
    planner.ot_map.add_feature(feature::LEADING_JAMO_FORMS, FeatureFlags::NONE, 1);
    planner.ot_map.add_feature(feature::VOWEL_JAMO_FORMS, FeatureFlags::NONE, 1);
    planner.ot_map.add_feature(feature::TRAILING_JAMO_FORMS, FeatureFlags::NONE, 1);
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_override_features_hangul(planner: *mut ffi::hb_ot_shape_planner_t) {
    let mut planner = ShapePlanner::from_ptr_mut(planner);
    override_features(&mut planner)
}

fn override_features(planner: &mut ShapePlanner) {
    // Uniscribe does not apply 'calt' for Hangul, and certain fonts
    // (Noto Sans CJK, Source Sans Han, etc) apply all of jamo lookups
    // in calt, which is not desirable.
    planner.ot_map.disable_feature(feature::CONTEXTUAL_ALTERNATES);
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_data_create_hangul(
    plan: *const ffi::hb_ot_shape_plan_t,
) -> *mut c_void {
    let plan = ShapePlan::from_ptr(plan);
    let hangul_plan = HangulShapePlan::new(&plan.ot_map);
    Box::into_raw(Box::new(hangul_plan)) as _
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_data_destroy_hangul(data: *mut c_void) {
    unsafe { Box::from_raw(data) };
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_preprocess_text_hangul(
    plan: *const ffi::hb_ot_shape_plan_t,
    buffer: *mut ffi::hb_buffer_t,
    font: *mut ffi::hb_font_t,
) {
    let plan = ShapePlan::from_ptr(plan);
    let font = Font::from_ptr(font);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    preprocess_text(&plan, font, &mut buffer)
}

fn preprocess_text(_: &ShapePlan, font: &Font, buffer: &mut Buffer) {
    // Hangul syllables come in two shapes: LV, and LVT.  Of those:
    //
    //   - LV can be precomposed, or decomposed.  Lets call those
    //     <LV> and <L,V>,
    //   - LVT can be fully precomposed, partically precomposed, or
    //     fully decomposed.  Ie. <LVT>, <LV,T>, or <L,V,T>.
    //
    // The composition / decomposition is mechanical.  However, not
    // all <L,V> sequences compose, and not all <LV,T> sequences
    // compose.
    //
    // Here are the specifics:
    //
    //   - <L>: U+1100..115F, U+A960..A97F
    //   - <V>: U+1160..11A7, U+D7B0..D7C7
    //   - <T>: U+11A8..11FF, U+D7CB..D7FB
    //
    //   - Only the <L,V> sequences for some of the U+11xx ranges combine.
    //   - Only <LV,T> sequences for some of the Ts in U+11xx range combine.
    //
    // Here is what we want to accomplish in this shaper:
    //
    //   - If the whole syllable can be precomposed, do that,
    //   - Otherwise, fully decompose and apply ljmo/vjmo/tjmo features.
    //   - If a valid syllable is followed by a Hangul tone mark, reorder the tone
    //     mark to precede the whole syllable - unless it is a zero-width glyph, in
    //     which case we leave it untouched, assuming it's designed to overstrike.
    //
    // That is, of the different possible syllables:
    //
    //   <L>
    //   <L,V>
    //   <L,V,T>
    //   <LV>
    //   <LVT>
    //   <LV, T>
    //
    // - <L> needs no work.
    //
    // - <LV> and <LVT> can stay the way they are if the font supports them, otherwise we
    //   should fully decompose them if font supports.
    //
    // - <L,V> and <L,V,T> we should compose if the whole thing can be composed.
    //
    // - <LV,T> we should compose if the whole thing can be composed, otherwise we should
    //   decompose.

    buffer.clear_output();
    // Extent of most recently seen syllable; valid only if start < end
    let mut start = 0;
    let mut end = 0;
    buffer.idx = 0;
    while buffer.idx < buffer.len {
        let u = buffer.cur(0).codepoint;
        let c = buffer.cur(0).as_char();

        if is_hangul_tone(u) {
            // We could cache the width of the tone marks and the existence of dotted-circle,
            // but the use of the Hangul tone mark characters seems to be rare enough that
            // I didn't bother for now.
            if start < end && end == buffer.out_len {
                // Tone mark follows a valid syllable; move it in front, unless it's zero width.
                buffer.unsafe_to_break_from_outbuffer(start, buffer.idx);
                buffer.next_glyph();
                if !is_zero_width_char(font, c) {
                    buffer.merge_out_clusters(start, end + 1);
                    let out_info = buffer.out_info_mut();
                    let tone = out_info[end];
                    for i in (0..end-start).rev() {
                        out_info[i + start + 1] = out_info[i + start];
                    }
                    out_info[start] = tone;
                }
            } else {
                // No valid syllable as base for tone mark; try to insert dotted circle.
                if !buffer.flags.contains(BufferFlags::DO_NOT_INSERT_DOTTED_CIRCLE) && font_has_glyph(font, 0x25CC) {
                    let mut chars = [0; 2];
                    if !is_zero_width_char(font, c) {
                        chars[0] = u;
                        chars[1] = 0x25CC;
                    } else {
                        chars[0] = 0x25CC;
                        chars[1] = u;
                    }

                    buffer.replace_glyphs(1, 2, &chars);
                } else {
                    // No dotted circle available in the font; just leave tone mark untouched.
                    buffer.next_glyph();
                }
            }

            start = buffer.out_len;
            end = buffer.out_len;
            continue;
        }

        // Remember current position as a potential syllable start;
        // will only be used if we set end to a later position.
        start = buffer.out_len;

        if is_l(u) && buffer.idx + 1 < buffer.len {
            let l = u;
            let v = buffer.cur(1).codepoint;
            if is_v(v) {
                // Have <L,V> or <L,V,T>.
                let mut t = 0;
                let mut tindex = 0;
                if buffer.idx + 2 < buffer.len {
                    t = buffer.cur(2).codepoint;
                    if is_t(t) {
                        // Only used if isCombiningT (t); otherwise invalid.
                        tindex = t - T_BASE;
                    } else {
                        // The next character was not a trailing jamo.
                        t = 0;
                    }
                }

                let offset = if t != 0 { 3 } else { 2 };
                buffer.unsafe_to_break(buffer.idx, buffer.idx + offset);

                // We've got a syllable <L,V,T?>; see if it can potentially be composed.
                if is_combining_l(l) && is_combining_v(v) && (t == 0 || is_combining_t(t)) {
                    // Try to compose; if this succeeds, end is set to start+1.
                    let s = S_BASE + (l - L_BASE) * N_COUNT + (v - V_BASE) * T_COUNT + tindex;
                    if font_has_glyph(font, s) {
                        let n = if t != 0 { 3 } else { 2 };
                        buffer.replace_glyphs(n, 1, &[s]);
                        end = start + 1;
                        continue;
                    }
                }

                // We didn't compose, either because it's an Old Hangul syllable without a
                // precomposed character in Unicode, or because the font didn't support the
                // necessary precomposed glyph.
                // Set jamo features on the individual glyphs, and advance past them.
                buffer.cur_mut(0).set_hangul_shaping_feature(LJMO);
                buffer.next_glyph();
                buffer.cur_mut(0).set_hangul_shaping_feature(VJMO);
                buffer.next_glyph();
                if t != 0 {
                    buffer.cur_mut(0).set_hangul_shaping_feature(TJMO);
                    buffer.next_glyph();
                    end = start + 3;
                } else {
                    end = start + 2;
                }

                if buffer.cluster_level == BufferClusterLevel::MonotoneGraphemes {
                    buffer.merge_out_clusters(start, end);
                }

                continue;
            }
        } else if is_combined_s(u) {
            // Have <LV>, <LVT>, or <LV,T>
            let s = u;
            let has_glyph = font_has_glyph(font, s);

            let lindex = (s - S_BASE) / N_COUNT;
            let nindex = (s - S_BASE) % N_COUNT;
            let vindex = nindex / T_COUNT;
            let tindex = nindex % T_COUNT;

            if tindex == 0 && buffer.idx + 1 < buffer.len && is_combining_t(buffer.cur(1).codepoint) {
                // <LV,T>, try to combine.
                let new_tindex = buffer.cur(1).codepoint - T_BASE;
                let new_s = s + new_tindex;

                if font_has_glyph(font, new_s) {
                    buffer.replace_glyphs(2, 1, &[new_s]);
                    end = start + 1;
                    continue;
                } else {
                    // Mark unsafe between LV and T.
                    buffer.unsafe_to_break(buffer.idx, buffer.idx + 2);
                }
            }

            // Otherwise, decompose if font doesn't support <LV> or <LVT>,
            // or if having non-combining <LV,T>.  Note that we already handled
            // combining <LV,T> above.
            if !has_glyph || (tindex == 0 && buffer.idx + 1 < buffer.len && is_t(buffer.cur(1).codepoint)) {
                let decomposed = [L_BASE + lindex, V_BASE + vindex, T_BASE + tindex];
                if font_has_glyph(font, decomposed[0]) && font_has_glyph(font, decomposed[1]) &&
                    (tindex == 0 || font_has_glyph(font, decomposed[2]))
                {
                    let mut s_len = if tindex != 0 { 3 } else { 2 };
                    buffer.replace_glyphs(1, s_len, &decomposed);

                    // If we decomposed an LV because of a non-combining T following,
                    // we want to include this T in the syllable.
                    if has_glyph && tindex == 0 {
                        buffer.next_glyph();
                        s_len += 1;
                    }

                    // We decomposed S: apply jamo features to the individual glyphs
                    // that are now in `buffer.out_info`.
                    end = start + s_len;

                    buffer.out_info_mut()[start + 0].set_hangul_shaping_feature(LJMO);
                    buffer.out_info_mut()[start + 1].set_hangul_shaping_feature(VJMO);
                    if start + 2 < end {
                        buffer.out_info_mut()[start + 2].set_hangul_shaping_feature(TJMO);
                    }

                    if buffer.cluster_level == BufferClusterLevel::MonotoneGraphemes {
                        buffer.merge_out_clusters(start, end);
                    }

                    continue;
                } else if tindex == 0 && buffer.idx + 1 > buffer.len && is_t(buffer.cur(1).codepoint) {
                    // Mark unsafe between LV and T.
                    buffer.unsafe_to_break(buffer.idx, buffer.idx + 2);
                }
            }

            if has_glyph {
                // We didn't decompose the S, so just advance past it.
                end = start + 1;
                buffer.next_glyph();
                continue;
            }
        }

        // Didn't find a recognizable syllable, so we leave end <= start;
        // this will prevent tone-mark reordering happening.
        buffer.next_glyph();
    }

    buffer.swap_buffers();
}

fn is_hangul_tone(u: u32) -> bool {
    (0x302E..=0x302F).contains(&u)
}

fn is_zero_width_char(font: &Font, c: char) -> bool {
    if let Some(glyph) = font.glyph_index(c as u32).map(|gid| gid.0 as u32) {
        font.glyph_h_advance(glyph) == 0
    } else {
        false
    }
}

fn is_l(u: u32) -> bool {
    (0x1100..=0x115F).contains(&u) || (0xA960..=0xA97C).contains(&u)
}

fn is_v(u: u32) -> bool {
    (0x1160..=0x11A7).contains(&u) || (0xD7B0..=0xD7C6).contains(&u)
}

fn is_t(u: u32) -> bool {
    (0x11A8..=0x11FF).contains(&u) || (0xD7CB..=0xD7FB).contains(&u)
}

fn is_combining_l(u: u32) -> bool {
    (L_BASE ..= L_BASE + L_COUNT - 1).contains(&u)
}

fn is_combining_v(u: u32) -> bool {
    (V_BASE ..= V_BASE + V_COUNT - 1).contains(&u)
}

fn is_combining_t(u: u32) -> bool {
    (T_BASE + 1 ..= T_BASE + T_COUNT - 1).contains(&u)
}

fn is_combined_s(u: u32) -> bool {
    (S_BASE ..= S_BASE + S_COUNT - 1).contains(&u)
}

fn font_has_glyph(font: &Font, u: u32) -> bool {
    font.glyph_index(u).is_some()
}

#[no_mangle]
pub extern "C" fn hb_ot_complex_setup_masks_hangul(
    plan: *const ffi::hb_ot_shape_plan_t,
    buffer: *mut ffi::hb_buffer_t,
    font: *mut ffi::hb_font_t,
) {
    let plan = ShapePlan::from_ptr(plan);
    let font = Font::from_ptr(font);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    setup_masks(&plan, font, &mut buffer);
}

fn setup_masks(plan: &ShapePlan, _: &Font, buffer: &mut Buffer) {
    let hangul_plan = HangulShapePlan::from_ptr(plan.data() as _);
    for info in buffer.info_slice_mut() {
        info.mask |= hangul_plan.mask_array[info.hangul_shaping_feature() as usize];
    }
}
