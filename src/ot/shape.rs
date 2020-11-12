use std::convert::TryFrom;

use crate::{ffi, Direction, Face};
use crate::buffer::{
    glyph_flag, Buffer, BufferClusterLevel, BufferFlags, BufferScratchFlags, GlyphInfo,
    GlyphPropsFlags,
};
use crate::unicode::{CharExt, GeneralCategory};

#[no_mangle]
pub extern "C" fn rb_set_unicode_props(buffer: *mut ffi::rb_buffer_t) {
    let mut buffer = Buffer::from_ptr_mut(buffer);
    set_unicode_props(&mut buffer);
}

fn set_unicode_props(buffer: &mut Buffer) {
    // Implement enough of Unicode Graphemes here that shaping
    // in reverse-direction wouldn't break graphemes.  Namely,
    // we mark all marks and ZWJ and ZWJ,Extended_Pictographic
    // sequences as continuations.  The foreach_grapheme()
    // macro uses this bit.
    //
    // https://www.unicode.org/reports/tr29/#Regex_Definitions

    let len = buffer.len;
    let mut i = 0;

    while i < len {
        let info = &mut buffer.info[i];
        info.init_unicode_props(&mut buffer.scratch_flags);

        // Marks are already set as continuation by the above line.
        // Handle Emoji_Modifier and ZWJ-continuation.
        if info.general_category() == GeneralCategory::ModifierSymbol
            && matches!(info.codepoint, 0x1F3FB..=0x1F3FF)
        {
            info.set_continuation();
        } else if info.is_zwj() {
            info.set_continuation();
            if let Some(next) = buffer.info[..len].get_mut(i + 1) {
                let c = char::try_from(next.codepoint).unwrap();
                if c.is_emoji_extended_pictographic() {
                    next.init_unicode_props(&mut buffer.scratch_flags);
                    next.set_continuation();
                    i += 1;
                }
            }
        } else if matches!(info.codepoint, 0xE0020..=0xE007F) {
            // Or part of the Other_Grapheme_Extend that is not marks.
            // As of Unicode 11 that is just:
            //
            // 200C          ; Other_Grapheme_Extend # Cf       ZERO WIDTH NON-JOINER
            // FF9E..FF9F    ; Other_Grapheme_Extend # Lm   [2] HALFWIDTH KATAKANA VOICED SOUND MARK..HALFWIDTH KATAKANA
            // SEMI-VOICED SOUND MARK E0020..E007F  ; Other_Grapheme_Extend # Cf  [96] TAG SPACE..CANCEL TAG
            //
            // ZWNJ is special, we don't want to merge it as there's no need, and keeping
            // it separate results in more granular clusters.  Ignore Katakana for now.
            // Tags are used for Emoji sub-region flag sequences:
            // https://github.com/harfbuzz/harfbuzz/issues/1556
            info.set_continuation();
        }

        i += 1;
    }
}

#[no_mangle]
pub extern "C" fn rb_insert_dotted_circle(
    buffer: *mut ffi::rb_buffer_t,
    face: *const ffi::rb_face_t,
) {
    let mut buffer = Buffer::from_ptr_mut(buffer);
    let face = Face::from_ptr(face);
    insert_dotted_circle(&mut buffer, face);
}

fn insert_dotted_circle(buffer: &mut Buffer, face: &Face) {
    if !buffer.flags.contains(BufferFlags::DO_NOT_INSERT_DOTTED_CIRCLE)
        && buffer.flags.contains(BufferFlags::BEGINNING_OF_TEXT)
        && buffer.context_len[0] == 0
        && buffer.info[0].is_unicode_mark()
        && face.has_glyph(0x25CC)
    {
        let mut info = GlyphInfo {
            codepoint: 0x25CC,
            mask: buffer.cur(0).mask,
            cluster: buffer.cur(0).cluster,
            var1: 0,
            var2: 0,
        };

        info.init_unicode_props(&mut buffer.scratch_flags);
        buffer.clear_output();
        buffer.output_info(info);

        while buffer.idx < buffer.len && buffer.successful {
            buffer.next_glyph();
        }

        buffer.swap_buffers();
    }
}

#[no_mangle]
pub extern "C" fn rb_form_clusters(buffer: *mut ffi::rb_buffer_t) {
    let mut buffer = Buffer::from_ptr_mut(buffer);
    form_clusters(&mut buffer);
}

fn form_clusters(buffer: &mut Buffer) {
    if buffer.scratch_flags.contains(BufferScratchFlags::HAS_NON_ASCII) {
        if buffer.cluster_level == BufferClusterLevel::MonotoneGraphemes {
            foreach_grapheme!(buffer, start, end, {
                buffer.merge_clusters(start, end)
            });
        } else {
            foreach_grapheme!(buffer, start, end, {
                buffer.unsafe_to_break(start, end);
            });
        }
    }
}

#[no_mangle]
pub extern "C" fn rb_ensure_native_direction(buffer: *mut ffi::rb_buffer_t) {
    let mut buffer = Buffer::from_ptr_mut(buffer);
    ensure_native_direction(&mut buffer);
}

fn ensure_native_direction(buffer: &mut Buffer) {
    let dir = buffer.direction;
    let hor = buffer.script.and_then(Direction::from_script).unwrap_or_default();

    if (dir.is_horizontal() && dir != hor && hor != Direction::Invalid)
        || (dir.is_vertical() && dir != Direction::TopToBottom)
    {
        if buffer.cluster_level == BufferClusterLevel::MonotoneCharacters {
            foreach_grapheme!(buffer, start, end, {
                buffer.merge_clusters(start, end);
                buffer.reverse_range(start, end);
            });
        } else {
            foreach_grapheme!(buffer, start, end, {
                // form_clusters() merged clusters already, we don't merge.
                buffer.reverse_range(start, end);
            })
        }

        buffer.reverse();
        buffer.direction = buffer.direction.reverse();
    }
}

#[no_mangle]
pub extern "C" fn rb_vert_char_for(u: ffi::rb_codepoint_t) -> ffi::rb_codepoint_t {
    vert_char_for(u)
}

fn vert_char_for(u: u32) -> u32 {
    match u >> 8 {
        0x20 => match u {
            0x2013 => 0xfe32, // EN DASH
            0x2014 => 0xfe31, // EM DASH
            0x2025 => 0xfe30, // TWO DOT LEADER
            0x2026 => 0xfe19, // HORIZONTAL ELLIPSIS
            _ => u,
        },
        0x30 => match u {
            0x3001 => 0xfe11, // IDEOGRAPHIC COMMA
            0x3002 => 0xfe12, // IDEOGRAPHIC FULL STOP
            0x3008 => 0xfe3f, // LEFT ANGLE BRACKET
            0x3009 => 0xfe40, // RIGHT ANGLE BRACKET
            0x300a => 0xfe3d, // LEFT DOUBLE ANGLE BRACKET
            0x300b => 0xfe3e, // RIGHT DOUBLE ANGLE BRACKET
            0x300c => 0xfe41, // LEFT CORNER BRACKET
            0x300d => 0xfe42, // RIGHT CORNER BRACKET
            0x300e => 0xfe43, // LEFT WHITE CORNER BRACKET
            0x300f => 0xfe44, // RIGHT WHITE CORNER BRACKET
            0x3010 => 0xfe3b, // LEFT BLACK LENTICULAR BRACKET
            0x3011 => 0xfe3c, // RIGHT BLACK LENTICULAR BRACKET
            0x3014 => 0xfe39, // LEFT TORTOISE SHELL BRACKET
            0x3015 => 0xfe3a, // RIGHT TORTOISE SHELL BRACKET
            0x3016 => 0xfe17, // LEFT WHITE LENTICULAR BRACKET
            0x3017 => 0xfe18, // RIGHT WHITE LENTICULAR BRACKET
            _ => u,
        },
        0xfe => match u {
            0xfe4f => 0xfe34, // WAVY LOW LINE
            _ => u,
        },
        0xff => match u {
            0xff01 => 0xfe15, // FULLWIDTH EXCLAMATION MARK
            0xff08 => 0xfe35, // FULLWIDTH LEFT PARENTHESIS
            0xff09 => 0xfe36, // FULLWIDTH RIGHT PARENTHESIS
            0xff0c => 0xfe10, // FULLWIDTH COMMA
            0xff1a => 0xfe13, // FULLWIDTH COLON
            0xff1b => 0xfe14, // FULLWIDTH SEMICOLON
            0xff1f => 0xfe16, // FULLWIDTH QUESTION MARK
            0xff3b => 0xfe47, // FULLWIDTH LEFT SQUARE BRACKET
            0xff3d => 0xfe48, // FULLWIDTH RIGHT SQUARE BRACKET
            0xff3f => 0xfe33, // FULLWIDTH LOW LINE
            0xff5b => 0xfe37, // FULLWIDTH LEFT CURLY BRACKET
            0xff5d => 0xfe38, // FULLWIDTH RIGHT CURLY BRACKET
            _ => u,
        }
        _ => u,
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_zero_width_default_ignorables(buffer: *mut ffi::rb_buffer_t) {
    let mut buffer = Buffer::from_ptr_mut(buffer);
    zero_width_default_ignorables(&mut buffer);
}

fn zero_width_default_ignorables(buffer: &mut Buffer) {
    if buffer.scratch_flags.contains(BufferScratchFlags::HAS_DEFAULT_IGNORABLES)
        && !buffer.flags.contains(BufferFlags::PRESERVE_DEFAULT_IGNORABLES)
        && !buffer.flags.contains(BufferFlags::REMOVE_DEFAULT_IGNORABLES)
    {
        let len = buffer.len;
        for (info, pos) in buffer.info[..len].iter().zip(&mut buffer.pos[..len]) {
            if info.is_default_ignorable() {
                pos.x_advance = 0;
                pos.y_advance = 0;
                pos.x_offset = 0;
                pos.y_offset = 0;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_hide_default_ignorables(
    buffer: *mut ffi::rb_buffer_t,
    face: *const ffi::rb_face_t,
) {
    let mut buffer = Buffer::from_ptr_mut(buffer);
    let face = Face::from_ptr(face);
    hide_default_ignorables(&mut buffer, face);
}

fn hide_default_ignorables(buffer: &mut Buffer, face: &Face) {
    if buffer.scratch_flags.contains(BufferScratchFlags::HAS_DEFAULT_IGNORABLES)
        && !buffer.flags.contains(BufferFlags::PRESERVE_DEFAULT_IGNORABLES)
    {
        if !buffer.flags.contains(BufferFlags::REMOVE_DEFAULT_IGNORABLES) {
            if let Some(invisible) = buffer.invisible.or_else(|| face.glyph_index(u32::from(' '))) {
                let len = buffer.len;
                for info in &mut buffer.info[..len] {
                    if info.is_default_ignorable() {
                        info.codepoint = u32::from(invisible.0);
                    }
                }
                return;
            }
        }

        buffer.delete_glyphs_inplace(GlyphInfo::is_default_ignorable);
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_map_glyphs_fast(buffer: *mut ffi::rb_buffer_t) {
    let mut buffer = Buffer::from_ptr_mut(buffer);
    map_glyphs_fast(&mut buffer);
}

fn map_glyphs_fast(buffer: &mut Buffer) {
    // Normalization process sets up glyph_index(), we just copy it.
    let len = buffer.len;
    for info in &mut buffer.info[..len] {
        info.codepoint = info.glyph_index();
    }
}

#[no_mangle]
pub extern "C" fn rb_synthesize_glyph_classes(buffer: *mut ffi::rb_buffer_t) {
    let mut buffer = Buffer::from_ptr_mut(buffer);
    synthesize_glyph_classes(&mut buffer);
}

fn synthesize_glyph_classes(buffer: &mut Buffer) {
    let len = buffer.len;
    for info in &mut buffer.info[..len] {
        // Never mark default-ignorables as marks.
        // They won't get in the way of lookups anyway,
        // but having them as mark will cause them to be skipped
        // over if the lookup-flag says so, but at least for the
        // Mongolian variation selectors, looks like Uniscribe
        // marks them as non-mark.  Some Mongolian fonts without
        // GDEF rely on this.  Another notable character that
        // this applies to is COMBINING GRAPHEME JOINER.
        let class = if info.general_category() != GeneralCategory::NonspacingMark
            || info.is_default_ignorable()
        {
            GlyphPropsFlags::BASE_GLYPH
        } else {
            GlyphPropsFlags::MARK
        };

        info.set_glyph_props(class.bits());
    }
}

#[no_mangle]
pub extern "C" fn rb_zero_mark_widths_by_gdef(
    buffer: *mut ffi::rb_buffer_t,
    adjust_offsets: ffi::rb_bool_t,
) {
    let mut buffer = Buffer::from_ptr_mut(buffer);
    zero_mark_widths_by_gdef(&mut buffer, adjust_offsets != 0);
}

fn zero_mark_widths_by_gdef(buffer: &mut Buffer, adjust_offsets: bool) {
    let len = buffer.len;
    for (info, pos) in buffer.info[..len].iter().zip(&mut buffer.pos[..len]) {
        if info.is_mark() {
            if adjust_offsets {
                pos.x_offset -= pos.x_advance;
                pos.y_offset -= pos.y_advance;
            }

            pos.x_advance = 0;
            pos.y_advance = 0;
        }
    }
}

#[no_mangle]
pub extern "C" fn rb_propagate_flags(buffer: *mut ffi::rb_buffer_t) {
    let mut buffer = Buffer::from_ptr_mut(buffer);
    propagate_flags(&mut buffer);
}

fn propagate_flags(buffer: &mut Buffer) {
    // Propagate cluster-level glyph flags to be the same on all cluster glyphs.
    // Simplifies using them.
    if buffer.scratch_flags.contains(BufferScratchFlags::HAS_UNSAFE_TO_BREAK) {
        foreach_cluster!(buffer, start, end, {
            for info in &buffer.info[start..end] {
                if info.mask & glyph_flag::UNSAFE_TO_BREAK != 0 {
                    for info in &mut buffer.info[start..end] {
                        info.mask |= glyph_flag::UNSAFE_TO_BREAK;
                    }
                    break;
                }
            }
        });
    }
}
