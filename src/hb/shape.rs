use super::buffer::{
    glyph_flag, hb_buffer_t, hb_glyph_info_t, BufferClusterLevel, BufferFlags, BufferScratchFlags,
    GlyphPropsFlags,
};
use super::ot_layout::*;
use super::ot_shape_complex::ZeroWidthMarksMode;
use super::shape_plan::hb_ot_shape_plan_t;
use super::unicode::{hb_unicode_general_category_t, CharExt, GeneralCategoryExt};
use super::{
    aat_layout, hb_font_t, ot_shape_fallback, ot_shape_normalize, script, Direction, Feature,
    GlyphBuffer, UnicodeBuffer,
};

/// Shapes the buffer content using provided font and features.
///
/// Consumes the buffer. You can then run [`GlyphBuffer::clear`] to get the [`UnicodeBuffer`] back
/// without allocating a new one.
///
/// If you plan to shape multiple strings using the same [`Face`] prefer [`shape_with_plan`].
/// This is because [`ShapePlan`] initialization is pretty slow and should preferably be called
/// once for each [`Face`].
pub fn shape(face: &hb_font_t, features: &[Feature], mut buffer: UnicodeBuffer) -> GlyphBuffer {
    buffer.0.guess_segment_properties();
    let plan = hb_ot_shape_plan_t::new(
        face,
        buffer.0.direction,
        buffer.0.script,
        buffer.0.language.as_ref(),
        features,
    );
    shape_with_plan(face, &plan, buffer)
}

/// Shapes the buffer content using the provided font and plan.
///
/// Consumes the buffer. You can then run [`GlyphBuffer::clear`] to get the [`UnicodeBuffer`] back
/// without allocating a new one.
///
/// It is up to the caller to ensure that the shape plan matches the properties of the provided
/// buffer, otherwise the shaping result will likely be incorrect.
///
/// # Panics
///
/// Will panic when debugging assertions are enabled if the buffer and plan have mismatched
/// properties.
pub fn shape_with_plan(
    face: &hb_font_t,
    plan: &hb_ot_shape_plan_t,
    buffer: UnicodeBuffer,
) -> GlyphBuffer {
    let mut buffer = buffer.0;
    buffer.guess_segment_properties();

    debug_assert_eq!(buffer.direction, plan.direction);
    debug_assert_eq!(
        buffer.script.unwrap_or(script::UNKNOWN),
        plan.script.unwrap_or(script::UNKNOWN)
    );

    if buffer.len > 0 {
        // Save the original direction, we use it later.
        let target_direction = buffer.direction;
        shape_internal(&mut ShapeContext {
            plan,
            face,
            buffer: &mut buffer,
            target_direction,
        });
    }

    GlyphBuffer(buffer)
}

struct ShapeContext<'a> {
    plan: &'a hb_ot_shape_plan_t,
    face: &'a hb_font_t<'a>,
    buffer: &'a mut hb_buffer_t,
    // Transient stuff
    target_direction: Direction,
}

// Pull it all together!
fn shape_internal(ctx: &mut ShapeContext) {
    ctx.buffer.enter();

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
    ctx.buffer.leave();
}

fn substitute_pre(ctx: &mut ShapeContext) {
    substitute_default(ctx);
    substitute_complex(ctx);
}

fn substitute_post(ctx: &mut ShapeContext) {
    hide_default_ignorables(ctx.buffer, ctx.face);

    if ctx.plan.apply_morx {
        aat_layout::hb_aat_layout_remove_deleted_glyphs(ctx.buffer);
    }

    if let Some(func) = ctx.plan.shaper.postprocess_glyphs {
        func(ctx.plan, ctx.face, ctx.buffer);
    }
}

fn substitute_default(ctx: &mut ShapeContext) {
    rotate_chars(ctx);

    ot_shape_normalize::_hb_ot_shape_normalize(ctx.plan, ctx.buffer, ctx.face);

    setup_masks(ctx);

    // This is unfortunate to go here, but necessary...
    if ctx.plan.fallback_mark_positioning {
        ot_shape_fallback::_hb_ot_shape_fallback_mark_position_recategorize_marks(
            ctx.plan, ctx.face, ctx.buffer,
        );
    }

    map_glyphs_fast(ctx.buffer);
}

fn substitute_complex(ctx: &mut ShapeContext) {
    super::ot_layout_gsub_table::substitute_start(ctx.face, ctx.buffer);

    if ctx.plan.fallback_glyph_classes {
        synthesize_glyph_classes(ctx.buffer);
    }

    substitute_by_plan(ctx.plan, ctx.face, ctx.buffer);
}

fn substitute_by_plan(plan: &hb_ot_shape_plan_t, face: &hb_font_t, buffer: &mut hb_buffer_t) {
    if plan.apply_morx {
        aat_layout::hb_aat_layout_substitute(plan, face, buffer);
    } else {
        super::ot_layout_gsub_table::substitute(plan, face, buffer);
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
        for (info, pos) in ctx.buffer.info[..len]
            .iter()
            .zip(&mut ctx.buffer.pos[..len])
        {
            pos.x_advance = ctx.face.glyph_h_advance(info.as_glyph());
        }
    } else {
        for (info, pos) in ctx.buffer.info[..len]
            .iter()
            .zip(&mut ctx.buffer.pos[..len])
        {
            let glyph = info.as_glyph();
            pos.y_advance = ctx.face.glyph_v_advance(glyph);
            pos.x_offset -= ctx.face.glyph_h_origin(glyph);
            pos.y_offset -= ctx.face.glyph_v_origin(glyph);
        }
    }

    if ctx
        .buffer
        .scratch_flags
        .contains(BufferScratchFlags::HAS_SPACE_FALLBACK)
    {
        ot_shape_fallback::_hb_ot_shape_fallback_spaces(ctx.plan, ctx.face, ctx.buffer);
    }
}

fn position_complex(ctx: &mut ShapeContext) {
    // If the font has no GPOS and direction is forward, then when
    // zeroing mark widths, we shift the mark with it, such that the
    // mark is positioned hanging over the previous glyph.  When
    // direction is backward we don't shift and it will end up
    // hanging over the next glyph after the final reordering.
    //
    // Note: If fallback positioning happens, we don't care about
    // this as it will be overridden.
    let adjust_offsets_when_zeroing =
        ctx.plan.adjust_mark_positioning_when_zeroing && ctx.buffer.direction.is_forward();

    // We change glyph origin to what GPOS expects (horizontal), apply GPOS, change it back.

    super::ot_layout_gpos_table::position_start(ctx.face, ctx.buffer);

    if ctx.plan.zero_marks
        && ctx.plan.shaper.zero_width_marks == Some(ZeroWidthMarksMode::ByGdefEarly)
    {
        zero_mark_widths_by_gdef(ctx.buffer, adjust_offsets_when_zeroing);
    }

    position_by_plan(ctx.plan, ctx.face, ctx.buffer);

    if ctx.plan.zero_marks
        && ctx.plan.shaper.zero_width_marks == Some(ZeroWidthMarksMode::ByGdefLate)
    {
        zero_mark_widths_by_gdef(ctx.buffer, adjust_offsets_when_zeroing);
    }

    // Finish off.  Has to follow a certain order.
    super::ot_layout_gpos_table::position_finish_advances(ctx.face, ctx.buffer);
    zero_width_default_ignorables(ctx.buffer);

    if ctx.plan.apply_morx {
        aat_layout::hb_aat_layout_zero_width_deleted_glyphs(ctx.buffer);
    }

    super::ot_layout_gpos_table::position_finish_offsets(ctx.face, ctx.buffer);

    if ctx.plan.fallback_mark_positioning {
        ot_shape_fallback::position_marks(
            ctx.plan,
            ctx.face,
            ctx.buffer,
            adjust_offsets_when_zeroing,
        );
    }
}

fn position_by_plan(plan: &hb_ot_shape_plan_t, face: &hb_font_t, buffer: &mut hb_buffer_t) {
    if plan.apply_gpos {
        super::ot_layout_gpos_table::position(plan, face, buffer);
    } else if plan.apply_kerx {
        aat_layout::hb_aat_layout_position(plan, face, buffer);
    }
    if plan.apply_kern {
        super::kerning::kern(plan, face, buffer);
    } else if plan.apply_fallback_kern {
        ot_shape_fallback::_hb_ot_shape_fallback_kern(plan, face, buffer);
    }

    if plan.apply_trak {
        aat_layout::hb_aat_layout_track(plan, face, buffer);
    }
}

fn initialize_masks(ctx: &mut ShapeContext) {
    let global_mask = ctx.plan.ot_map.get_global_mask();
    ctx.buffer.reset_masks(global_mask);
}

fn setup_masks(ctx: &mut ShapeContext) {
    setup_masks_fraction(ctx);

    if let Some(func) = ctx.plan.shaper.setup_masks {
        func(ctx.plan, ctx.face, ctx.buffer);
    }

    for feature in &ctx.plan.user_features {
        if !feature.is_global() {
            let (mask, shift) = ctx.plan.ot_map.get_mask(feature.tag);
            ctx.buffer
                .set_masks(feature.value << shift, mask, feature.start, feature.end);
        }
    }
}

fn setup_masks_fraction(ctx: &mut ShapeContext) {
    let buffer = &mut ctx.buffer;
    if !buffer
        .scratch_flags
        .contains(BufferScratchFlags::HAS_NON_ASCII)
        || !ctx.plan.has_frac
    {
        return;
    }

    let (pre_mask, post_mask) = if buffer.direction.is_forward() {
        (
            ctx.plan.numr_mask | ctx.plan.frac_mask,
            ctx.plan.frac_mask | ctx.plan.dnom_mask,
        )
    } else {
        (
            ctx.plan.frac_mask | ctx.plan.dnom_mask,
            ctx.plan.numr_mask | ctx.plan.frac_mask,
        )
    };

    let len = buffer.len;
    let mut i = 0;
    while i < len {
        // FRACTION SLASH
        if buffer.info[i].glyph_id == 0x2044 {
            let mut start = i;
            while start > 0
                && _hb_glyph_info_get_general_category(&buffer.info[start - 1])
                    == hb_unicode_general_category_t::DecimalNumber
            {
                start -= 1;
            }

            let mut end = i + 1;
            while end < len
                && _hb_glyph_info_get_general_category(&buffer.info[end])
                    == hb_unicode_general_category_t::DecimalNumber
            {
                end += 1;
            }

            buffer.unsafe_to_break(Some(start), Some(end));

            for info in &mut buffer.info[start..i] {
                info.mask |= pre_mask;
            }

            buffer.info[i].mask |= ctx.plan.frac_mask;

            for info in &mut buffer.info[i + 1..end] {
                info.mask |= post_mask;
            }

            i = end;
        } else {
            i += 1;
        }
    }
}

fn set_unicode_props(buffer: &mut hb_buffer_t) {
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
        // Mutably borrow buffer.info[i] and immutably borrow
        // buffer.info[i - 1] (if present) in a way that the borrow
        // checker can understand.
        let (prior, later) = buffer.info.split_at_mut(i);
        let info = &mut later[0];
        info.init_unicode_props(&mut buffer.scratch_flags);

        // Marks are already set as continuation by the above line.
        // Handle Emoji_Modifier and ZWJ-continuation.
        if _hb_glyph_info_get_general_category(info)
            == hb_unicode_general_category_t::ModifierSymbol
            && matches!(info.glyph_id, 0x1F3FB..=0x1F3FF)
        {
            _hb_glyph_info_set_continuation(info);
        } else if i != 0 && matches!(info.glyph_id, 0x1F1E6..=0x1F1FF) {
            // Should never fail because we checked for i > 0.
            // TODO: use let chains when they become stable
            let prev = prior.last().unwrap();
            if matches!(prev.glyph_id, 0x1F1E6..=0x1F1FF) && !_hb_glyph_info_is_continuation(prev) {
                _hb_glyph_info_set_continuation(info);
            }
        } else if _hb_glyph_info_is_zwj(info) {
            _hb_glyph_info_set_continuation(info);
            if let Some(next) = buffer.info[..len].get_mut(i + 1) {
                if next.as_char().is_emoji_extended_pictographic() {
                    next.init_unicode_props(&mut buffer.scratch_flags);
                    _hb_glyph_info_set_continuation(next);
                    i += 1;
                }
            }
        } else if matches!(info.glyph_id, 0xE0020..=0xE007F) {
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
            _hb_glyph_info_set_continuation(info);
        }

        i += 1;
    }
}

fn insert_dotted_circle(buffer: &mut hb_buffer_t, face: &hb_font_t) {
    if !buffer
        .flags
        .contains(BufferFlags::DO_NOT_INSERT_DOTTED_CIRCLE)
        && buffer.flags.contains(BufferFlags::BEGINNING_OF_TEXT)
        && buffer.context_len[0] == 0
        && _hb_glyph_info_is_unicode_mark(&buffer.info[0])
        && face.has_glyph(0x25CC)
    {
        let mut info = hb_glyph_info_t {
            glyph_id: 0x25CC,
            mask: buffer.cur(0).mask,
            cluster: buffer.cur(0).cluster,
            ..hb_glyph_info_t::default()
        };

        info.init_unicode_props(&mut buffer.scratch_flags);
        buffer.clear_output();
        buffer.output_info(info);
        buffer.sync();
    }
}

fn form_clusters(buffer: &mut hb_buffer_t) {
    if buffer
        .scratch_flags
        .contains(BufferScratchFlags::HAS_NON_ASCII)
    {
        if buffer.cluster_level == BufferClusterLevel::MonotoneGraphemes {
            foreach_grapheme!(buffer, start, end, { buffer.merge_clusters(start, end) });
        } else {
            foreach_grapheme!(buffer, start, end, {
                buffer.unsafe_to_break(Some(start), Some(end));
            });
        }
    }
}

fn ensure_native_direction(buffer: &mut hb_buffer_t) {
    let dir = buffer.direction;
    let mut hor = buffer
        .script
        .and_then(Direction::from_script)
        .unwrap_or_default();

    // Numeric runs in natively-RTL scripts are actually native-LTR, so we reset
    // the horiz_dir if the run contains at least one decimal-number char, and no
    // letter chars (ideally we should be checking for chars with strong
    // directionality but hb-unicode currently lacks bidi categories).
    //
    // This allows digit sequences in Arabic etc to be shaped in "native"
    // direction, so that features like ligatures will work as intended.
    //
    // https://github.com/harfbuzz/harfbuzz/issues/501

    if hor == Direction::RightToLeft && dir == Direction::LeftToRight {
        let mut found_number = false;
        let mut found_letter = false;
        for info in &buffer.info {
            let gc = _hb_glyph_info_get_general_category(info);
            if gc == hb_unicode_general_category_t::DecimalNumber {
                found_number = true;
            } else if gc.is_letter() {
                found_letter = true;
                break;
            }
        }
        if found_number && !found_letter {
            hor = Direction::LeftToRight;
        }
    }

    // TODO vertical:
    // The only BTT vertical script is Ogham, but it's not clear to me whether OpenType
    // Ogham fonts are supposed to be implemented BTT or not.  Need to research that
    // first.
    if (dir.is_horizontal() && dir != hor && hor != Direction::Invalid)
        || (dir.is_vertical() && dir != Direction::TopToBottom)
    {
        _hb_ot_layout_reverse_graphemes(buffer);
        buffer.direction = buffer.direction.reverse();
    }
}

fn rotate_chars(ctx: &mut ShapeContext) {
    let len = ctx.buffer.len;

    if ctx.target_direction.is_backward() {
        let rtlm_mask = ctx.plan.rtlm_mask;

        for info in &mut ctx.buffer.info[..len] {
            if let Some(c) = info.as_char().mirrored().map(u32::from) {
                if ctx.face.has_glyph(c) {
                    info.glyph_id = c;
                    continue;
                }
            }
            info.mask |= rtlm_mask;
        }
    }

    if ctx.target_direction.is_vertical() && !ctx.plan.has_vert {
        for info in &mut ctx.buffer.info[..len] {
            if let Some(c) = info.as_char().vertical().map(u32::from) {
                if ctx.face.has_glyph(c) {
                    info.glyph_id = c;
                }
            }
        }
    }
}

fn map_glyphs_fast(buffer: &mut hb_buffer_t) {
    // Normalization process sets up glyph_index(), we just copy it.
    let len = buffer.len;
    for info in &mut buffer.info[..len] {
        info.glyph_id = info.glyph_index();
    }
}

fn synthesize_glyph_classes(buffer: &mut hb_buffer_t) {
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
        let class = if _hb_glyph_info_get_general_category(info)
            != hb_unicode_general_category_t::NonspacingMark
            || _hb_glyph_info_is_default_ignorable(info)
        {
            GlyphPropsFlags::BASE_GLYPH
        } else {
            GlyphPropsFlags::MARK
        };

        info.set_glyph_props(class.bits());
    }
}

fn zero_width_default_ignorables(buffer: &mut hb_buffer_t) {
    if buffer
        .scratch_flags
        .contains(BufferScratchFlags::HAS_DEFAULT_IGNORABLES)
        && !buffer
            .flags
            .contains(BufferFlags::PRESERVE_DEFAULT_IGNORABLES)
        && !buffer
            .flags
            .contains(BufferFlags::REMOVE_DEFAULT_IGNORABLES)
    {
        let len = buffer.len;
        for (info, pos) in buffer.info[..len].iter().zip(&mut buffer.pos[..len]) {
            if _hb_glyph_info_is_default_ignorable(info) {
                pos.x_advance = 0;
                pos.y_advance = 0;
                pos.x_offset = 0;
                pos.y_offset = 0;
            }
        }
    }
}

fn zero_mark_widths_by_gdef(buffer: &mut hb_buffer_t, adjust_offsets: bool) {
    let len = buffer.len;
    for (info, pos) in buffer.info[..len].iter().zip(&mut buffer.pos[..len]) {
        if _hb_glyph_info_is_mark(info) {
            if adjust_offsets {
                pos.x_offset -= pos.x_advance;
                pos.y_offset -= pos.y_advance;
            }

            pos.x_advance = 0;
            pos.y_advance = 0;
        }
    }
}

fn hide_default_ignorables(buffer: &mut hb_buffer_t, face: &hb_font_t) {
    if buffer
        .scratch_flags
        .contains(BufferScratchFlags::HAS_DEFAULT_IGNORABLES)
        && !buffer
            .flags
            .contains(BufferFlags::PRESERVE_DEFAULT_IGNORABLES)
    {
        if !buffer
            .flags
            .contains(BufferFlags::REMOVE_DEFAULT_IGNORABLES)
        {
            if let Some(invisible) = buffer
                .invisible
                .or_else(|| face.get_nominal_glyph(u32::from(' ')))
            {
                let len = buffer.len;
                for info in &mut buffer.info[..len] {
                    if _hb_glyph_info_is_default_ignorable(info) {
                        info.glyph_id = u32::from(invisible.0);
                    }
                }
                return;
            }
        }

        buffer.delete_glyphs_inplace(_hb_glyph_info_is_default_ignorable);
    }
}

fn propagate_flags(buffer: &mut hb_buffer_t) {
    // Propagate cluster-level glyph flags to be the same on all cluster glyphs.
    // Simplifies using them.
    if buffer
        .scratch_flags
        .contains(BufferScratchFlags::HAS_GLYPH_FLAGS)
    {
        foreach_cluster!(buffer, start, end, {
            let mut mask = 0;
            for info in &buffer.info[start..end] {
                mask |= info.mask * glyph_flag::DEFINED;
            }

            if mask != 0 {
                for info in &mut buffer.info[start..end] {
                    info.mask |= mask;
                }
            }
        });
    }
}