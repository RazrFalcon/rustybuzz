use std::convert::TryFrom;

use ttf_parser::GlyphId;

use crate::{aat, ot, fallback, normalize, Direction, Face, Feature, GlyphBuffer, UnicodeBuffer};
use crate::buffer::{
    glyph_flag, Buffer, BufferClusterLevel, BufferFlags, BufferScratchFlags, GlyphInfo,
    GlyphPropsFlags,
};
use crate::complex::ZeroWidthMarksMode;
use crate::plan::ShapePlan;
use crate::unicode::{CharExt, GeneralCategory};

/// Shapes the buffer content using provided font and features.
///
/// Consumes the buffer. You can then run `GlyphBuffer::clear` to get the `UnicodeBuffer` back
/// without allocating a new one.
pub fn shape(face: &Face, features: &[Feature], buffer: UnicodeBuffer) -> GlyphBuffer {
    let mut buffer = buffer.0;
    buffer.guess_segment_properties();

    if buffer.len > 0 {
        let plan = ShapePlan::new(
            face,
            buffer.direction,
            buffer.script,
            buffer.language.as_ref(),
            features,
        );

        // Save the original direction, we use it later.
        let target_direction = buffer.direction;
        shape_internal(&mut ShapeContext {
            plan: &plan,
            face,
            buffer: &mut buffer,
            user_features: features,
            target_direction,
        });
    }

    GlyphBuffer(buffer)
}

struct ShapeContext<'a> {
    plan: &'a ShapePlan,
    face: &'a Face<'a>,
    buffer: &'a mut Buffer,
    user_features: &'a [Feature],
    // Transient stuff
    target_direction: Direction,
}

// Pull it all together!
fn shape_internal(ctx: &mut ShapeContext) {
    ctx.buffer.scratch_flags = BufferScratchFlags::empty();

    if let Some(len) = ctx.buffer.len.checked_mul(Buffer::MAX_LEN_FACTOR) {
        ctx.buffer.max_len = len.max(Buffer::MAX_LEN_MIN);
    }

    if let Ok(len) = i32::try_from(ctx.buffer.len) {
        if let Some(ops) = len.checked_mul(Buffer::MAX_OPS_FACTOR) {
            ctx.buffer.max_ops = ops.max(Buffer::MAX_OPS_MIN);
        }
    }

    ctx.buffer.clear_output();

    initialize_masks(ctx);
    set_unicode_props(ctx.buffer);
    insert_dotted_circle(ctx.buffer, ctx.face);

    form_clusters(ctx.buffer);

    ensure_native_direction(ctx.buffer);

    if let Some(func) = ctx.plan.shaper.preprocess_text {
        func(ctx.plan, ctx.face, ctx.buffer);
    }

    substitute_pre(ctx);
    position(ctx);
    substitute_post(ctx);

    propagate_flags(ctx.buffer);

    ctx.buffer.direction = ctx.target_direction;
    ctx.buffer.max_len = Buffer::MAX_LEN_DEFAULT;
    ctx.buffer.max_ops = Buffer::MAX_OPS_DEFAULT;
}

fn substitute_pre(ctx: &mut ShapeContext) {
    substitute_default(ctx);
    substitute_complex(ctx);
}

fn substitute_post(ctx: &mut ShapeContext) {
    hide_default_ignorables(ctx.buffer, ctx.face);

    if ctx.plan.apply_morx {
        aat::remove_deleted_glyphs(ctx.buffer);
    }

    if let Some(func) = ctx.plan.shaper.postprocess_glyphs {
        func(&ctx.plan, ctx.face, ctx.buffer);
    }
}

fn substitute_default(ctx: &mut ShapeContext) {
    rotate_chars(ctx);

    normalize::normalize(ctx.plan, ctx.face, ctx.buffer);

    setup_masks(ctx);

    // This is unfortunate to go here, but necessary...
    if ctx.plan.fallback_mark_positioning {
        fallback::recategorize_marks(ctx.plan, ctx.face, ctx.buffer);
    }

    map_glyphs_fast(ctx.buffer);
}

fn substitute_complex(ctx: &mut ShapeContext) {
    ot::substitute_start(ctx.face, ctx.buffer);

    if ctx.plan.fallback_glyph_classes {
        synthesize_glyph_classes(ctx.buffer);
    }

    substitute_by_plan(ctx.plan, ctx.face, ctx.buffer);
}

fn substitute_by_plan(plan: &ShapePlan, face: &Face, buffer: &mut Buffer) {
    if plan.apply_morx {
        aat::substitute(plan, face, buffer);
    } else {
        ot::substitute(plan, face, buffer);
    }
}

fn position(ctx: &mut ShapeContext) {
    ctx.buffer.clear_positions();

    position_default(ctx);

    position_complex(ctx);

    if ctx.buffer.direction.is_backward() {
        ctx.buffer.reverse();
    }
}

fn position_default(ctx: &mut ShapeContext) {
    let len = ctx.buffer.len;

    if ctx.buffer.direction.is_horizontal() {
        for (info, pos) in ctx.buffer.info[..len].iter().zip(&mut ctx.buffer.pos[..len]) {
            let glyph = GlyphId(u16::try_from(info.codepoint).unwrap());
            pos.x_advance = ctx.face.glyph_h_advance(glyph);
        }
    } else {
        for (info, pos) in ctx.buffer.info[..len].iter().zip(&mut ctx.buffer.pos[..len]) {
            let glyph = GlyphId(u16::try_from(info.codepoint).unwrap());
            pos.y_advance = ctx.face.glyph_v_advance(glyph);
            pos.x_offset -= ctx.face.glyph_h_origin(glyph);
            pos.y_offset -= ctx.face.glyph_v_origin(glyph);
        }
    }

    if ctx.buffer.scratch_flags.contains(BufferScratchFlags::HAS_SPACE_FALLBACK) {
        fallback::adjust_spaces(ctx.plan, ctx.face, ctx.buffer);
    }
}

fn position_complex(ctx: &mut ShapeContext) {
    // If the font has no GPOS and direction is forward, then when
    // zeroing mark widths, we shift the mark with it, such that the
    // mark is positioned hanging over the previous glyph.  When
    // direction is backward we don't shift and it will end up
    // hanging over the next glyph after the final reordering.
    //
    // Note: If fallback positinoing happens, we don't care about
    // this as it will be overriden.
    let adjust_offsets_when_zeroing = ctx.plan.adjust_mark_positioning_when_zeroing
        && ctx.buffer.direction.is_forward();

    // We change glyph origin to what GPOS expects (horizontal), apply GPOS, change it back.

    ot::position_start(ctx.face, ctx.buffer);

    if ctx.plan.zero_marks && ctx.plan.shaper.zero_width_marks == Some(ZeroWidthMarksMode::ByGdefEarly) {
        zero_mark_widths_by_gdef(ctx.buffer, adjust_offsets_when_zeroing);
    }

    position_by_plan(ctx.plan, ctx.face, ctx.buffer);

    if ctx.plan.zero_marks && ctx.plan.shaper.zero_width_marks == Some(ZeroWidthMarksMode::ByGdefLate) {
        zero_mark_widths_by_gdef(ctx.buffer, adjust_offsets_when_zeroing);
    }

    // Finish off.  Has to follow a certain order.
    ot::position_finish_advances(ctx.face, ctx.buffer);
    zero_width_default_ignorables(ctx.buffer);

    if ctx.plan.apply_morx {
        aat::zero_width_deleted_glyphs(ctx.buffer);
    }

    ot::position_finish_offsets(ctx.face, ctx.buffer);

    if ctx.plan.fallback_mark_positioning {
        fallback::position_marks(ctx.plan, ctx.face, ctx.buffer, adjust_offsets_when_zeroing);
    }
}

fn position_by_plan(plan: &ShapePlan, face: &Face, buffer: &mut Buffer) {
    if plan.apply_gpos {
        ot::position(plan, face, buffer);
    } else if plan.apply_kerx {
        aat::position(plan, face, buffer);
    } else if plan.apply_kern {
        ot::kern(plan, face, buffer);
    }

    if plan.apply_trak {
        aat::track(plan, face, buffer);
    }
}

fn initialize_masks(ctx: &mut ShapeContext) {
    let global_mask = ctx.plan.ot_map.global_mask();
    ctx.buffer.reset_masks(global_mask);
}

fn setup_masks(ctx: &mut ShapeContext) {
    setup_masks_fraction(ctx);

    if let Some(func) = ctx.plan.shaper.setup_masks {
        func(ctx.plan, ctx.face, ctx.buffer);
    }

    for feature in ctx.user_features {
        if !feature.is_global() {
            let (mask, shift) = ctx.plan.ot_map.mask(feature.tag);
            ctx.buffer.set_masks(feature.value << shift, mask, feature.start, feature.end);
        }
    }
}

fn setup_masks_fraction(ctx: &mut ShapeContext) {
    let buffer = &mut ctx.buffer;
    if !buffer.scratch_flags.contains(BufferScratchFlags::HAS_NON_ASCII) || !ctx.plan.has_frac {
        return;
    }

    let (pre_mask, post_mask) = if buffer.direction.is_forward() {
        (ctx.plan.numr_mask | ctx.plan.frac_mask, ctx.plan.frac_mask | ctx.plan.dnom_mask)
    } else {
        (ctx.plan.frac_mask | ctx.plan.dnom_mask, ctx.plan.numr_mask | ctx.plan.frac_mask)
    };

    let len = buffer.len;
    let mut i = 0;
    while i < len {
        // FRACTION SLASH
        if buffer.info[i].codepoint == 0x2044 {
            let mut start = i;
            while start > 0 && buffer.info[start - 1].general_category() == GeneralCategory::DecimalNumber {
                start -= 1;
            }

            let mut end = i + 1;
            while end < len && buffer.info[end].general_category() == GeneralCategory::DecimalNumber {
                end += 1;
            }

            buffer.unsafe_to_break(start, end);

            for info in &mut buffer.info[start..i] {
                info.mask |= pre_mask;
            }

            buffer.info[i].mask |= ctx.plan.frac_mask;

            for info in &mut buffer.info[i+1..end] {
                info.mask |= post_mask;
            }

            i = end;
        } else {
            i += 1;
        }
    }
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

fn rotate_chars(ctx: &mut ShapeContext) {
    let len = ctx.buffer.len;

    if ctx.target_direction.is_backward() {
        let rtlm_mask = ctx.plan.rtlm_mask;

        for info in &mut ctx.buffer.info[..len] {
            let c = char::try_from(info.codepoint).unwrap().mirrored().map_or(0, u32::from);
            if c != info.codepoint && ctx.face.has_glyph(c) {
                info.codepoint = c;
            } else {
                info.mask |= rtlm_mask;
            }
        }
    }

    if ctx.target_direction.is_vertical() && !ctx.plan.has_vert {
        for info in &mut ctx.buffer.info[..len] {
            let c = vert_char_for(info.codepoint);
            if c != info.codepoint && ctx.face.has_glyph(c) {
                info.codepoint = c;
            }
        }
    }
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

fn map_glyphs_fast(buffer: &mut Buffer) {
    // Normalization process sets up glyph_index(), we just copy it.
    let len = buffer.len;
    for info in &mut buffer.info[..len] {
        info.codepoint = info.glyph_index();
    }
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
