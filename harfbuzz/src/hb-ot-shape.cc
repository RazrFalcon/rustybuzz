/*
 * Copyright © 2009,2010  Red Hat, Inc.
 * Copyright © 2010,2011,2012  Google, Inc.
 *
 *  This is part of HarfBuzz, a text shaping library.
 *
 * Permission is hereby granted, without written agreement and without
 * license or royalty fees, to use, copy, modify, and distribute this
 * software and its documentation for any purpose, provided that the
 * above copyright notice and the following two paragraphs appear in
 * all copies of this software.
 *
 * IN NO EVENT SHALL THE COPYRIGHT HOLDER BE LIABLE TO ANY PARTY FOR
 * DIRECT, INDIRECT, SPECIAL, INCIDENTAL, OR CONSEQUENTIAL DAMAGES
 * ARISING OUT OF THE USE OF THIS SOFTWARE AND ITS DOCUMENTATION, EVEN
 * IF THE COPYRIGHT HOLDER HAS BEEN ADVISED OF THE POSSIBILITY OF SUCH
 * DAMAGE.
 *
 * THE COPYRIGHT HOLDER SPECIFICALLY DISCLAIMS ANY WARRANTIES, INCLUDING,
 * BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
 * FITNESS FOR A PARTICULAR PURPOSE.  THE SOFTWARE PROVIDED HEREUNDER IS
 * ON AN "AS IS" BASIS, AND THE COPYRIGHT HOLDER HAS NO OBLIGATION TO
 * PROVIDE MAINTENANCE, SUPPORT, UPDATES, ENHANCEMENTS, OR MODIFICATIONS.
 *
 * Red Hat Author(s): Behdad Esfahbod
 * Google Author(s): Behdad Esfahbod
 */

#include "hb.hh"

#include "hb-ot-shape.hh"
#include "hb-ot-shape-complex.hh"
#include "hb-ot-shape-fallback.hh"
#include "hb-ot-shape-normalize.hh"

#include "hb-ot-face.hh"

#include "hb-set.hh"

#include "hb-aat-layout.hh"

const rb_ot_map_t *rb_ot_shape_plan_get_ot_map(const rb_ot_shape_plan_t *plan)
{
    return &plan->map;
}

const void *rb_ot_shape_plan_get_data(const rb_ot_shape_plan_t *plan)
{
    return plan->data;
}

rb_script_t rb_ot_shape_plan_get_script(const rb_ot_shape_plan_t *plan)
{
    return plan->props.script;
}

rb_direction_t rb_ot_shape_plan_get_direction(const rb_ot_shape_plan_t *plan)
{
    return plan->props.direction;
}

bool rb_ot_shape_plan_has_gpos_mark(const rb_ot_shape_plan_t *plan)
{
    return plan->has_gpos_mark;
}

rb_ot_map_builder_t *rb_ot_shape_planner_get_ot_map(rb_ot_shape_planner_t *planner)
{
    return &planner->map;
}

RB_EXTERN rb_script_t rb_ot_shape_planner_get_script(rb_ot_shape_planner_t *planner)
{
    return planner->props.script;
}

static inline bool _rb_apply_morx(rb_face_t *face, const rb_segment_properties_t *props)
{
    /* https://github.com/harfbuzz/harfbuzz/issues/2124 */
    return rb_aat_layout_has_substitution(face) &&
           (RB_DIRECTION_IS_HORIZONTAL(props->direction) || !rb_ot_layout_has_substitution(face));
}

/**
 * SECTION:hb-ot-shape
 * @title: hb-ot-shape
 * @short_description: OpenType shaping support
 * @include: hb-ot.h
 *
 * Support functions for OpenType shaping related queries.
 **/

static void rb_ot_shape_collect_features(rb_ot_shape_planner_t *planner,
                                         const rb_feature_t *user_features,
                                         unsigned int num_user_features);

rb_ot_shape_planner_t::rb_ot_shape_planner_t(rb_face_t *face, const rb_segment_properties_t *props)
    : face(face)
    , props(*props)
    , map(face, props)
    , aat_map(face, props)
    , apply_morx(_rb_apply_morx(face, props))
{
    shaper = rb_ot_shape_complex_categorize(this);

    script_zero_marks = shaper->zero_width_marks != RB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE;
    script_fallback_mark_positioning = shaper->fallback_position;

    /* https://github.com/harfbuzz/harfbuzz/issues/1528 */
    if (apply_morx && shaper != &_rb_ot_complex_shaper_default)
        shaper = &_rb_ot_complex_shaper_dumber;
}

void rb_ot_shape_planner_t::compile(rb_ot_shape_plan_t &plan, unsigned int *variations_index)
{
    plan.props = props;
    plan.shaper = shaper;
    map.compile(plan.map, variations_index);
    if (apply_morx)
        aat_map.compile(plan.aat_map);

    plan.frac_mask = plan.map.get_1_mask(RB_TAG('f', 'r', 'a', 'c'));
    plan.numr_mask = plan.map.get_1_mask(RB_TAG('n', 'u', 'm', 'r'));
    plan.dnom_mask = plan.map.get_1_mask(RB_TAG('d', 'n', 'o', 'm'));
    plan.has_frac = plan.frac_mask || (plan.numr_mask && plan.dnom_mask);

    plan.rtlm_mask = plan.map.get_1_mask(RB_TAG('r', 't', 'l', 'm'));
    plan.has_vert = !!plan.map.get_1_mask(RB_TAG('v', 'e', 'r', 't'));

    rb_tag_t kern_tag =
        RB_DIRECTION_IS_HORIZONTAL(props.direction) ? RB_TAG('k', 'e', 'r', 'n') : RB_TAG('v', 'k', 'r', 'n');
    plan.kern_mask = plan.map.get_mask(kern_tag);
    plan.requested_kerning = !!plan.kern_mask;
    plan.trak_mask = plan.map.get_mask(RB_TAG('t', 'r', 'a', 'k'));
    plan.requested_tracking = !!plan.trak_mask;

    bool has_gpos_kern = plan.map.get_feature_index(1, kern_tag) != RB_OT_LAYOUT_NO_FEATURE_INDEX;
    bool disable_gpos = plan.shaper->gpos_tag && plan.shaper->gpos_tag != plan.map.chosen_script[1];

    /*
     * Decide who provides glyph classes. GDEF or Unicode.
     */

    if (!rb_ot_layout_has_glyph_classes(face))
        plan.fallback_glyph_classes = true;

    /*
     * Decide who does substitutions. GSUB, morx, or fallback.
     */

    plan.apply_morx = apply_morx;

    /*
     * Decide who does positioning. GPOS, kerx, kern, or fallback.
     */

    if (0)
        ;
    else if (rb_aat_layout_has_positioning(face))
        plan.apply_kerx = true;
    else if (!apply_morx && !disable_gpos && rb_ot_layout_has_positioning(face))
        plan.apply_gpos = true;

    if (!plan.apply_kerx && (!has_gpos_kern || !plan.apply_gpos)) {
        /* Apparently Apple applies kerx if GPOS kern was not applied. */
        if (rb_aat_layout_has_positioning(face))
            plan.apply_kerx = true;
        else if (rb_ot_layout_has_kerning(face))
            plan.apply_kern = true;
    }

    plan.zero_marks =
        script_zero_marks && !plan.apply_kerx && (!plan.apply_kern || !rb_ot_layout_has_machine_kerning(face));
    plan.has_gpos_mark = !!plan.map.get_1_mask(RB_TAG('m', 'a', 'r', 'k'));

    plan.adjust_mark_positioning_when_zeroing =
        !plan.apply_gpos && !plan.apply_kerx && (!plan.apply_kern || !rb_ot_layout_has_cross_kerning(face));

    plan.fallback_mark_positioning = plan.adjust_mark_positioning_when_zeroing && script_fallback_mark_positioning;

    /* Currently we always apply trak. */
    plan.apply_trak = plan.requested_tracking && rb_aat_layout_has_tracking(face);
}

bool rb_ot_shape_plan_t::init0(rb_face_t *face,
                               const rb_segment_properties_t *props,
                               const rb_feature_t *user_features,
                               unsigned int num_user_features,
                               unsigned int *variations_index)
{
    map.init();
    aat_map.init();

    rb_ot_shape_planner_t planner(face, props);

    rb_ot_shape_collect_features(&planner, user_features, num_user_features);

    planner.compile(*this, variations_index);

    if (shaper->data_create) {
        data = shaper->data_create(this);
        if (unlikely(!data)) {
            map.fini();
            aat_map.fini();
            return false;
        }
    }

    return true;
}

void rb_ot_shape_plan_t::fini()
{
    if (shaper->data_destroy)
        shaper->data_destroy(const_cast<void *>(data));

    map.fini();
    aat_map.fini();
}

void rb_ot_shape_plan_t::substitute(rb_font_t *font, rb_buffer_t *buffer) const
{
    if (unlikely(apply_morx))
        rb_aat_layout_substitute(this, font, buffer);
    else
        map.substitute(this, font, buffer);
}

void rb_ot_shape_plan_t::position(rb_font_t *font, rb_buffer_t *buffer) const
{
    if (this->apply_gpos)
        map.position(this, font, buffer);
    else if (this->apply_kerx)
        rb_aat_layout_position(this, font, buffer);
    else if (this->apply_kern)
        rb_ot_layout_kern(this, font, buffer);
    else
        _rb_ot_shape_fallback_kern(this, font, buffer);

    if (this->apply_trak)
        rb_aat_layout_track(this, font, buffer);
}

static const rb_ot_map_feature_t common_features[] = {
    {RB_TAG('a', 'b', 'v', 'm'), F_GLOBAL},
    {RB_TAG('b', 'l', 'w', 'm'), F_GLOBAL},
    {RB_TAG('c', 'c', 'm', 'p'), F_GLOBAL},
    {RB_TAG('l', 'o', 'c', 'l'), F_GLOBAL},
    {RB_TAG('m', 'a', 'r', 'k'), F_GLOBAL_MANUAL_JOINERS},
    {RB_TAG('m', 'k', 'm', 'k'), F_GLOBAL_MANUAL_JOINERS},
    {RB_TAG('r', 'l', 'i', 'g'), F_GLOBAL},
};

static const rb_ot_map_feature_t horizontal_features[] = {
    {RB_TAG('c', 'a', 'l', 't'), F_GLOBAL},
    {RB_TAG('c', 'l', 'i', 'g'), F_GLOBAL},
    {RB_TAG('c', 'u', 'r', 's'), F_GLOBAL},
    {RB_TAG('d', 'i', 's', 't'), F_GLOBAL},
    {RB_TAG('k', 'e', 'r', 'n'), F_GLOBAL_HAS_FALLBACK},
    {RB_TAG('l', 'i', 'g', 'a'), F_GLOBAL},
    {RB_TAG('r', 'c', 'l', 't'), F_GLOBAL},
};

static void rb_ot_shape_collect_features(rb_ot_shape_planner_t *planner,
                                         const rb_feature_t *user_features,
                                         unsigned int num_user_features)
{
    rb_ot_map_builder_t *map = &planner->map;

    map->enable_feature(RB_TAG('r', 'v', 'r', 'n'));
    map->add_gsub_pause(nullptr);

    switch (planner->props.direction) {
    case RB_DIRECTION_LTR:
        map->enable_feature(RB_TAG('l', 't', 'r', 'a'));
        map->enable_feature(RB_TAG('l', 't', 'r', 'm'));
        break;
    case RB_DIRECTION_RTL:
        map->enable_feature(RB_TAG('r', 't', 'l', 'a'));
        map->add_feature(RB_TAG('r', 't', 'l', 'm'));
        break;
    case RB_DIRECTION_TTB:
    case RB_DIRECTION_BTT:
    case RB_DIRECTION_INVALID:
    default:
        break;
    }

    /* Automatic fractions. */
    map->add_feature(RB_TAG('f', 'r', 'a', 'c'));
    map->add_feature(RB_TAG('n', 'u', 'm', 'r'));
    map->add_feature(RB_TAG('d', 'n', 'o', 'm'));

    /* Random! */
    map->enable_feature(RB_TAG('r', 'a', 'n', 'd'), F_RANDOM, RB_OT_MAP_MAX_VALUE);

    /* Tracking.  We enable dummy feature here just to allow disabling
     * AAT 'trak' table using features.
     * https://github.com/harfbuzz/harfbuzz/issues/1303 */
    map->enable_feature(RB_TAG('t', 'r', 'a', 'k'), F_HAS_FALLBACK);

    map->enable_feature(RB_TAG('H', 'A', 'R', 'F'));

    if (planner->shaper->collect_features)
        planner->shaper->collect_features(planner);

    map->enable_feature(RB_TAG('B', 'U', 'Z', 'Z'));

    for (unsigned int i = 0; i < ARRAY_LENGTH(common_features); i++)
        map->add_feature(common_features[i]);

    if (RB_DIRECTION_IS_HORIZONTAL(planner->props.direction))
        for (unsigned int i = 0; i < ARRAY_LENGTH(horizontal_features); i++)
            map->add_feature(horizontal_features[i]);
    else {
        /* We really want to find a 'vert' feature if there's any in the font, no
         * matter which script/langsys it is listed (or not) under.
         * See various bugs referenced from:
         * https://github.com/harfbuzz/harfbuzz/issues/63 */
        map->enable_feature(RB_TAG('v', 'e', 'r', 't'), F_GLOBAL_SEARCH);
    }

    for (unsigned int i = 0; i < num_user_features; i++) {
        const rb_feature_t *feature = &user_features[i];
        map->add_feature(feature->tag,
                         (feature->start == RB_FEATURE_GLOBAL_START && feature->end == RB_FEATURE_GLOBAL_END) ? F_GLOBAL
                                                                                                              : F_NONE,
                         feature->value);
    }

    if (planner->apply_morx) {
        rb_aat_map_builder_t *aat_map = &planner->aat_map;
        for (unsigned int i = 0; i < num_user_features; i++) {
            const rb_feature_t *feature = &user_features[i];
            aat_map->add_feature(feature->tag, feature->value);
        }
    }

    if (planner->shaper->override_features)
        planner->shaper->override_features(planner);
}

/*
 * shaper
 */

struct rb_ot_shape_context_t
{
    rb_ot_shape_plan_t *plan;
    rb_font_t *font;
    rb_face_t *face;
    rb_buffer_t *buffer;
    const rb_feature_t *user_features;
    unsigned int num_user_features;

    /* Transient stuff */
    rb_direction_t target_direction;
};

/* Main shaper */

/* Prepare */

static void rb_set_unicode_props(rb_buffer_t *buffer)
{
    /* Implement enough of Unicode Graphemes here that shaping
     * in reverse-direction wouldn't break graphemes.  Namely,
     * we mark all marks and ZWJ and ZWJ,Extended_Pictographic
     * sequences as continuations.  The foreach_grapheme()
     * macro uses this bit.
     *
     * https://www.unicode.org/reports/tr29/#Regex_Definitions
     */
    unsigned int count = rb_buffer_get_length(buffer);
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);
    for (unsigned int i = 0; i < count; i++) {
        _rb_glyph_info_set_unicode_props(&info[i], buffer);

        /* Marks are already set as continuation by the above line.
         * Handle Emoji_Modifier and ZWJ-continuation. */
        if (unlikely(_rb_glyph_info_get_general_category(&info[i]) == RB_UNICODE_GENERAL_CATEGORY_MODIFIER_SYMBOL &&
                     rb_in_range<rb_codepoint_t>(info[i].codepoint, 0x1F3FBu, 0x1F3FFu))) {
            _rb_glyph_info_set_continuation(&info[i]);
        }
#ifndef RB_NO_EMOJI_SEQUENCES
        else if (unlikely(_rb_glyph_info_is_zwj(&info[i]))) {
            _rb_glyph_info_set_continuation(&info[i]);
            if (i + 1 < count && rb_ucd_is_emoji_extended_pictographic(info[i + 1].codepoint)) {
                i++;
                _rb_glyph_info_set_unicode_props(&info[i], buffer);
                _rb_glyph_info_set_continuation(&info[i]);
            }
        }
#endif
        /* Or part of the Other_Grapheme_Extend that is not marks.
         * As of Unicode 11 that is just:
         *
         * 200C          ; Other_Grapheme_Extend # Cf       ZERO WIDTH NON-JOINER
         * FF9E..FF9F    ; Other_Grapheme_Extend # Lm   [2] HALFWIDTH KATAKANA VOICED SOUND MARK..HALFWIDTH KATAKANA
         * SEMI-VOICED SOUND MARK E0020..E007F  ; Other_Grapheme_Extend # Cf  [96] TAG SPACE..CANCEL TAG
         *
         * ZWNJ is special, we don't want to merge it as there's no need, and keeping
         * it separate results in more granular clusters.  Ignore Katakana for now.
         * Tags are used for Emoji sub-region flag sequences:
         * https://github.com/harfbuzz/harfbuzz/issues/1556
         */
        else if (unlikely(rb_in_range<rb_codepoint_t>(info[i].codepoint, 0xE0020u, 0xE007Fu)))
            _rb_glyph_info_set_continuation(&info[i]);
    }
}

static void rb_insert_dotted_circle(rb_buffer_t *buffer, rb_font_t *font)
{
    if (unlikely(rb_buffer_get_flags(buffer) & RB_BUFFER_FLAG_DO_NOT_INSERT_DOTTED_CIRCLE))
        return;

    if (!(rb_buffer_get_flags(buffer) & RB_BUFFER_FLAG_BOT) || rb_buffer_get_context_len(buffer, 0) ||
        !_rb_glyph_info_is_unicode_mark(&rb_buffer_get_glyph_infos(buffer)[0]))
        return;

    if (!rb_font_has_glyph(font, 0x25CCu))
        return;

    rb_glyph_info_t dottedcircle = {0};
    dottedcircle.codepoint = 0x25CCu;
    _rb_glyph_info_set_unicode_props(&dottedcircle, buffer);

    rb_buffer_clear_output(buffer);

    rb_buffer_set_index(buffer, 0);
    rb_glyph_info_t info = dottedcircle;
    info.cluster = rb_buffer_get_cur(buffer, 0)->cluster;
    info.mask = rb_buffer_get_cur(buffer, 0)->mask;
    rb_buffer_output_info(buffer, info);
    while (rb_buffer_get_index(buffer) < rb_buffer_get_length(buffer) && rb_buffer_is_allocation_successful(buffer))
        rb_buffer_next_glyph(buffer);
    rb_buffer_swap_buffers(buffer);
}

static void rb_form_clusters(rb_buffer_t *buffer)
{
    if (!(rb_buffer_get_scratch_flags(buffer) & RB_BUFFER_SCRATCH_FLAG_HAS_NON_ASCII))
        return;

    if (rb_buffer_get_cluster_level(buffer) == RB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES)
        foreach_grapheme(buffer, start, end) rb_buffer_merge_clusters(buffer, start, end);
    else
        foreach_grapheme(buffer, start, end) rb_buffer_unsafe_to_break(buffer, start, end);
}

static void rb_ensure_native_direction(rb_buffer_t *buffer)
{
    rb_direction_t direction = rb_buffer_get_direction(buffer);
    rb_direction_t horiz_dir = rb_script_get_horizontal_direction(rb_buffer_get_script(buffer));

    /* TODO vertical:
     * The only BTT vertical script is Ogham, but it's not clear to me whether OpenType
     * Ogham fonts are supposed to be implemented BTT or not.  Need to research that
     * first. */
    if ((RB_DIRECTION_IS_HORIZONTAL(direction) && direction != horiz_dir && horiz_dir != RB_DIRECTION_INVALID) ||
        (RB_DIRECTION_IS_VERTICAL(direction) && direction != RB_DIRECTION_TTB)) {

        if (rb_buffer_get_cluster_level(buffer) == RB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS)
            foreach_grapheme(buffer, start, end)
            {
                rb_buffer_merge_clusters(buffer, start, end);
                rb_buffer_reverse_range(buffer, start, end);
            }
        else
            foreach_grapheme(buffer, start, end)
                /* form_clusters() merged clusters already, we don't merge. */
                rb_buffer_reverse_range(buffer, start, end);

        rb_buffer_reverse(buffer);

        rb_buffer_set_direction(buffer, RB_DIRECTION_REVERSE(rb_buffer_get_direction(buffer)));
    }
}

/*
 * Substitute
 */

static rb_codepoint_t rb_vert_char_for(rb_codepoint_t u)
{
    switch (u >> 8) {
    case 0x20:
        switch (u) {
        case 0x2013u:
            return 0xfe32u; // EN DASH
        case 0x2014u:
            return 0xfe31u; // EM DASH
        case 0x2025u:
            return 0xfe30u; // TWO DOT LEADER
        case 0x2026u:
            return 0xfe19u; // HORIZONTAL ELLIPSIS
        }
        break;
    case 0x30:
        switch (u) {
        case 0x3001u:
            return 0xfe11u; // IDEOGRAPHIC COMMA
        case 0x3002u:
            return 0xfe12u; // IDEOGRAPHIC FULL STOP
        case 0x3008u:
            return 0xfe3fu; // LEFT ANGLE BRACKET
        case 0x3009u:
            return 0xfe40u; // RIGHT ANGLE BRACKET
        case 0x300au:
            return 0xfe3du; // LEFT DOUBLE ANGLE BRACKET
        case 0x300bu:
            return 0xfe3eu; // RIGHT DOUBLE ANGLE BRACKET
        case 0x300cu:
            return 0xfe41u; // LEFT CORNER BRACKET
        case 0x300du:
            return 0xfe42u; // RIGHT CORNER BRACKET
        case 0x300eu:
            return 0xfe43u; // LEFT WHITE CORNER BRACKET
        case 0x300fu:
            return 0xfe44u; // RIGHT WHITE CORNER BRACKET
        case 0x3010u:
            return 0xfe3bu; // LEFT BLACK LENTICULAR BRACKET
        case 0x3011u:
            return 0xfe3cu; // RIGHT BLACK LENTICULAR BRACKET
        case 0x3014u:
            return 0xfe39u; // LEFT TORTOISE SHELL BRACKET
        case 0x3015u:
            return 0xfe3au; // RIGHT TORTOISE SHELL BRACKET
        case 0x3016u:
            return 0xfe17u; // LEFT WHITE LENTICULAR BRACKET
        case 0x3017u:
            return 0xfe18u; // RIGHT WHITE LENTICULAR BRACKET
        }
        break;
    case 0xfe:
        switch (u) {
        case 0xfe4fu:
            return 0xfe34u; // WAVY LOW LINE
        }
        break;
    case 0xff:
        switch (u) {
        case 0xff01u:
            return 0xfe15u; // FULLWIDTH EXCLAMATION MARK
        case 0xff08u:
            return 0xfe35u; // FULLWIDTH LEFT PARENTHESIS
        case 0xff09u:
            return 0xfe36u; // FULLWIDTH RIGHT PARENTHESIS
        case 0xff0cu:
            return 0xfe10u; // FULLWIDTH COMMA
        case 0xff1au:
            return 0xfe13u; // FULLWIDTH COLON
        case 0xff1bu:
            return 0xfe14u; // FULLWIDTH SEMICOLON
        case 0xff1fu:
            return 0xfe16u; // FULLWIDTH QUESTION MARK
        case 0xff3bu:
            return 0xfe47u; // FULLWIDTH LEFT SQUARE BRACKET
        case 0xff3du:
            return 0xfe48u; // FULLWIDTH RIGHT SQUARE BRACKET
        case 0xff3fu:
            return 0xfe33u; // FULLWIDTH LOW LINE
        case 0xff5bu:
            return 0xfe37u; // FULLWIDTH LEFT CURLY BRACKET
        case 0xff5du:
            return 0xfe38u; // FULLWIDTH RIGHT CURLY BRACKET
        }
        break;
    }

    return u;
}

static inline void rb_ot_rotate_chars(const rb_ot_shape_context_t *c)
{
    rb_buffer_t *buffer = c->buffer;
    unsigned int count = rb_buffer_get_length(buffer);
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);

    if (RB_DIRECTION_IS_BACKWARD(c->target_direction)) {
        rb_mask_t rtlm_mask = c->plan->rtlm_mask;

        for (unsigned int i = 0; i < count; i++) {
            rb_codepoint_t codepoint = rb_ucd_mirroring(info[i].codepoint);
            if (unlikely(codepoint != info[i].codepoint && rb_font_has_glyph(c->font, codepoint)))
                info[i].codepoint = codepoint;
            else
                info[i].mask |= rtlm_mask;
        }
    }

    if (RB_DIRECTION_IS_VERTICAL(c->target_direction) && !c->plan->has_vert) {
        for (unsigned int i = 0; i < count; i++) {
            rb_codepoint_t codepoint = rb_vert_char_for(info[i].codepoint);
            if (unlikely(codepoint != info[i].codepoint && rb_font_has_glyph(c->font, codepoint)))
                info[i].codepoint = codepoint;
        }
    }
}

static inline void rb_ot_shape_setup_masks_fraction(const rb_ot_shape_context_t *c)
{
    if (!(rb_buffer_get_scratch_flags(c->buffer) & RB_BUFFER_SCRATCH_FLAG_HAS_NON_ASCII) || !c->plan->has_frac)
        return;

    rb_buffer_t *buffer = c->buffer;

    rb_mask_t pre_mask, post_mask;
    if (RB_DIRECTION_IS_FORWARD(rb_buffer_get_direction(c->buffer))) {
        pre_mask = c->plan->numr_mask | c->plan->frac_mask;
        post_mask = c->plan->frac_mask | c->plan->dnom_mask;
    } else {
        pre_mask = c->plan->frac_mask | c->plan->dnom_mask;
        post_mask = c->plan->numr_mask | c->plan->frac_mask;
    }

    unsigned int count = rb_buffer_get_length(buffer);
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);
    for (unsigned int i = 0; i < count; i++) {
        if (info[i].codepoint == 0x2044u) /* FRACTION SLASH */
        {
            unsigned int start = i, end = i + 1;
            while (start &&
                   _rb_glyph_info_get_general_category(&info[start - 1]) == RB_UNICODE_GENERAL_CATEGORY_DECIMAL_NUMBER)
                start--;
            while (end < count &&
                   _rb_glyph_info_get_general_category(&info[end]) == RB_UNICODE_GENERAL_CATEGORY_DECIMAL_NUMBER)
                end++;

            rb_buffer_unsafe_to_break(buffer, start, end);

            for (unsigned int j = start; j < i; j++)
                info[j].mask |= pre_mask;
            info[i].mask |= c->plan->frac_mask;
            for (unsigned int j = i + 1; j < end; j++)
                info[j].mask |= post_mask;

            i = end - 1;
        }
    }
}

static inline void rb_ot_shape_initialize_masks(const rb_ot_shape_context_t *c)
{
    rb_ot_map_t *map = &c->plan->map;

    rb_mask_t global_mask = map->get_global_mask();
    rb_buffer_reset_masks(c->buffer, global_mask);
}

static inline void rb_ot_shape_setup_masks(const rb_ot_shape_context_t *c)
{
    rb_ot_map_t *map = &c->plan->map;
    rb_buffer_t *buffer = c->buffer;

    rb_ot_shape_setup_masks_fraction(c);

    if (c->plan->shaper->setup_masks)
        c->plan->shaper->setup_masks(c->plan, buffer, c->font);

    for (unsigned int i = 0; i < c->num_user_features; i++) {
        const rb_feature_t *feature = &c->user_features[i];
        if (!(feature->start == RB_FEATURE_GLOBAL_START && feature->end == RB_FEATURE_GLOBAL_END)) {
            unsigned int shift;
            rb_mask_t mask = map->get_mask(feature->tag, &shift);
            rb_buffer_set_masks(buffer, feature->value << shift, mask, feature->start, feature->end);
        }
    }
}

static void rb_ot_zero_width_default_ignorables(const rb_buffer_t *buffer)
{
    if (!(rb_buffer_get_scratch_flags(buffer) & RB_BUFFER_SCRATCH_FLAG_HAS_DEFAULT_IGNORABLES) ||
        (rb_buffer_get_flags(buffer) & RB_BUFFER_FLAG_PRESERVE_DEFAULT_IGNORABLES) ||
        (rb_buffer_get_flags(buffer) & RB_BUFFER_FLAG_REMOVE_DEFAULT_IGNORABLES))
        return;

    unsigned int count = rb_buffer_get_length(buffer);
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos((rb_buffer_t *)buffer);
    rb_glyph_position_t *pos = rb_buffer_get_glyph_positions((rb_buffer_t *)buffer);
    unsigned int i = 0;
    for (i = 0; i < count; i++)
        if (unlikely(_rb_glyph_info_is_default_ignorable(&info[i])))
            pos[i].x_advance = pos[i].y_advance = pos[i].x_offset = pos[i].y_offset = 0;
}

static void rb_ot_hide_default_ignorables(rb_buffer_t *buffer, rb_font_t *font)
{
    if (!(rb_buffer_get_scratch_flags(buffer) & RB_BUFFER_SCRATCH_FLAG_HAS_DEFAULT_IGNORABLES) ||
        (rb_buffer_get_flags(buffer) & RB_BUFFER_FLAG_PRESERVE_DEFAULT_IGNORABLES))
        return;

    unsigned int count = rb_buffer_get_length(buffer);
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);

    rb_codepoint_t invisible = rb_buffer_get_invisible_glyph(buffer);
    if (!(rb_buffer_get_flags(buffer) & RB_BUFFER_FLAG_REMOVE_DEFAULT_IGNORABLES) &&
        (invisible || rb_font_get_nominal_glyph(font, ' ', &invisible))) {
        /* Replace default-ignorables with a zero-advance invisible glyph. */
        for (unsigned int i = 0; i < count; i++) {
            if (_rb_glyph_info_is_default_ignorable(&info[i]))
                info[i].codepoint = invisible;
        }
    } else
        rb_ot_layout_delete_glyphs_inplace(buffer, _rb_glyph_info_is_default_ignorable);
}

static inline void rb_ot_map_glyphs_fast(rb_buffer_t *buffer)
{
    /* Normalization process sets up glyph_index(), we just copy it. */
    unsigned int count = rb_buffer_get_length(buffer);
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);
    for (unsigned int i = 0; i < count; i++)
        info[i].codepoint = info[i].glyph_index();
}

static inline void rb_synthesize_glyph_classes(rb_buffer_t *buffer)
{
    unsigned int count = rb_buffer_get_length(buffer);
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);
    for (unsigned int i = 0; i < count; i++) {
        rb_ot_layout_glyph_props_flags_t klass;

        /* Never mark default-ignorables as marks.
         * They won't get in the way of lookups anyway,
         * but having them as mark will cause them to be skipped
         * over if the lookup-flag says so, but at least for the
         * Mongolian variation selectors, looks like Uniscribe
         * marks them as non-mark.  Some Mongolian fonts without
         * GDEF rely on this.  Another notable character that
         * this applies to is COMBINING GRAPHEME JOINER. */
        klass = (_rb_glyph_info_get_general_category(&info[i]) != RB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK ||
                 _rb_glyph_info_is_default_ignorable(&info[i]))
                    ? RB_OT_LAYOUT_GLYPH_PROPS_BASE_GLYPH
                    : RB_OT_LAYOUT_GLYPH_PROPS_MARK;
        _rb_glyph_info_set_glyph_props(&info[i], klass);
    }
}

static inline void rb_ot_substitute_default(const rb_ot_shape_context_t *c)
{
    rb_buffer_t *buffer = c->buffer;

    rb_ot_rotate_chars(c);

    _rb_ot_shape_normalize(c->plan, buffer, c->font);

    rb_ot_shape_setup_masks(c);

    /* This is unfortunate to go here, but necessary... */
    if (c->plan->fallback_mark_positioning)
        _rb_ot_shape_fallback_mark_position_recategorize_marks(c->plan, c->font, buffer);

    rb_ot_map_glyphs_fast(buffer);
}

static inline void rb_ot_substitute_complex(const rb_ot_shape_context_t *c)
{
    rb_buffer_t *buffer = c->buffer;

    rb_ot_layout_substitute_start(c->font, buffer);

    if (c->plan->fallback_glyph_classes)
        rb_synthesize_glyph_classes(c->buffer);

    c->plan->substitute(c->font, buffer);
}

static inline void rb_ot_substitute_pre(const rb_ot_shape_context_t *c)
{
    rb_ot_substitute_default(c);
    rb_ot_substitute_complex(c);
}

static inline void rb_ot_substitute_post(const rb_ot_shape_context_t *c)
{
    rb_ot_hide_default_ignorables(c->buffer, c->font);
    if (c->plan->apply_morx)
        rb_aat_layout_remove_deleted_glyphs(c->buffer);

    if (c->plan->shaper->postprocess_glyphs)
        c->plan->shaper->postprocess_glyphs(c->plan, c->buffer, c->font);
}

/*
 * Position
 */

static inline void adjust_mark_offsets(rb_glyph_position_t *pos)
{
    pos->x_offset -= pos->x_advance;
    pos->y_offset -= pos->y_advance;
}

static inline void zero_mark_width(rb_glyph_position_t *pos)
{
    pos->x_advance = 0;
    pos->y_advance = 0;
}

static inline void zero_mark_widths_by_gdef(rb_buffer_t *buffer, bool adjust_offsets)
{
    unsigned int count = rb_buffer_get_length(buffer);
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);
    for (unsigned int i = 0; i < count; i++)
        if (_rb_glyph_info_is_mark(&info[i])) {
            if (adjust_offsets)
                adjust_mark_offsets(&rb_buffer_get_glyph_positions(buffer)[i]);
            zero_mark_width(&rb_buffer_get_glyph_positions(buffer)[i]);
        }
}

static inline void rb_ot_position_default(const rb_ot_shape_context_t *c)
{
    rb_direction_t direction = rb_buffer_get_direction(c->buffer);
    unsigned int count = rb_buffer_get_length(c->buffer);
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(c->buffer);
    rb_glyph_position_t *pos = rb_buffer_get_glyph_positions(c->buffer);

    if (RB_DIRECTION_IS_HORIZONTAL(direction)) {
        rb_font_get_glyph_h_advances(
            c->font, count, &info[0].codepoint, sizeof(info[0]), &pos[0].x_advance, sizeof(pos[0]));
    } else {
        rb_font_get_glyph_v_advances(
            c->font, count, &info[0].codepoint, sizeof(info[0]), &pos[0].y_advance, sizeof(pos[0]));
        for (unsigned int i = 0; i < count; i++) {
            rb_font_subtract_glyph_v_origin(c->font, info[i].codepoint, &pos[i].x_offset, &pos[i].y_offset);
        }
    }
    if (rb_buffer_get_scratch_flags(c->buffer) & RB_BUFFER_SCRATCH_FLAG_HAS_SPACE_FALLBACK)
        _rb_ot_shape_fallback_spaces(c->plan, c->font, c->buffer);
}

static inline void rb_ot_position_complex(const rb_ot_shape_context_t *c)
{
    /* If the font has no GPOS and direction is forward, then when
     * zeroing mark widths, we shift the mark with it, such that the
     * mark is positioned hanging over the previous glyph.  When
     * direction is backward we don't shift and it will end up
     * hanging over the next glyph after the final reordering.
     *
     * Note: If fallback positinoing happens, we don't care about
     * this as it will be overriden.
     */
    bool adjust_offsets_when_zeroing =
        c->plan->adjust_mark_positioning_when_zeroing && RB_DIRECTION_IS_FORWARD(rb_buffer_get_direction(c->buffer));

    /* We change glyph origin to what GPOS expects (horizontal), apply GPOS, change it back. */

    rb_ot_layout_position_start(c->font, c->buffer);

    if (c->plan->zero_marks)
        switch (c->plan->shaper->zero_width_marks) {
        case RB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_EARLY:
            zero_mark_widths_by_gdef(c->buffer, adjust_offsets_when_zeroing);
            break;

        default:
        case RB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE:
        case RB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE:
            break;
        }

    c->plan->position(c->font, c->buffer);

    if (c->plan->zero_marks)
        switch (c->plan->shaper->zero_width_marks) {
        case RB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE:
            zero_mark_widths_by_gdef(c->buffer, adjust_offsets_when_zeroing);
            break;

        default:
        case RB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE:
        case RB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_EARLY:
            break;
        }

    /* Finish off.  Has to follow a certain order. */
    rb_ot_layout_position_finish_advances(c->font, c->buffer);
    rb_ot_zero_width_default_ignorables(c->buffer);
    if (c->plan->apply_morx)
        rb_aat_layout_zero_width_deleted_glyphs(c->buffer);
    rb_ot_layout_position_finish_offsets(c->font, c->buffer);

    if (c->plan->fallback_mark_positioning)
        _rb_ot_shape_fallback_mark_position(c->plan, c->font, c->buffer, adjust_offsets_when_zeroing);
}

static inline void rb_ot_position(const rb_ot_shape_context_t *c)
{
    rb_buffer_clear_positions(c->buffer);

    rb_ot_position_default(c);

    rb_ot_position_complex(c);

    if (RB_DIRECTION_IS_BACKWARD(rb_buffer_get_direction(c->buffer)))
        rb_buffer_reverse(c->buffer);
}

static inline void rb_propagate_flags(rb_buffer_t *buffer)
{
    /* Propagate cluster-level glyph flags to be the same on all cluster glyphs.
     * Simplifies using them. */

    if (!(rb_buffer_get_scratch_flags(buffer) & RB_BUFFER_SCRATCH_FLAG_HAS_UNSAFE_TO_BREAK))
        return;

    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);

    foreach_cluster(buffer, start, end)
    {
        unsigned int mask = 0;
        for (unsigned int i = start; i < end; i++)
            if (info[i].mask & RB_GLYPH_FLAG_UNSAFE_TO_BREAK) {
                mask = RB_GLYPH_FLAG_UNSAFE_TO_BREAK;
                break;
            }
        if (mask)
            for (unsigned int i = start; i < end; i++)
                info[i].mask |= mask;
    }
}

/* Pull it all together! */

static void rb_ot_shape_internal(rb_ot_shape_context_t *c)
{
    rb_buffer_set_scratch_flags(c->buffer, RB_BUFFER_SCRATCH_FLAG_DEFAULT);
    if (likely(!rb_unsigned_mul_overflows(rb_buffer_get_length(c->buffer), RB_BUFFER_MAX_LEN_FACTOR))) {
        rb_buffer_set_max_len(
            c->buffer,
            rb_max(rb_buffer_get_length(c->buffer) * RB_BUFFER_MAX_LEN_FACTOR, (unsigned)RB_BUFFER_MAX_LEN_MIN));
    }
    if (likely(!rb_unsigned_mul_overflows(rb_buffer_get_length(c->buffer), RB_BUFFER_MAX_OPS_FACTOR))) {
        rb_buffer_set_max_ops(
            c->buffer,
            rb_max(rb_buffer_get_length(c->buffer) * RB_BUFFER_MAX_OPS_FACTOR, (unsigned)RB_BUFFER_MAX_OPS_MIN));
    }

    /* Save the original direction, we use it later. */
    c->target_direction = rb_buffer_get_direction(c->buffer);

    rb_buffer_clear_output(c->buffer);

    rb_ot_shape_initialize_masks(c);
    rb_set_unicode_props(c->buffer);
    rb_insert_dotted_circle(c->buffer, c->font);

    rb_form_clusters(c->buffer);

    rb_ensure_native_direction(c->buffer);

    if (c->plan->shaper->preprocess_text)
        c->plan->shaper->preprocess_text(c->plan, c->buffer, c->font);

    rb_ot_substitute_pre(c);
    rb_ot_position(c);
    rb_ot_substitute_post(c);

    rb_propagate_flags(c->buffer);

    rb_buffer_set_direction(c->buffer, c->target_direction);

    rb_buffer_set_max_len(c->buffer, RB_BUFFER_MAX_LEN_DEFAULT);
    rb_buffer_set_max_ops(c->buffer, RB_BUFFER_MAX_OPS_DEFAULT);
}

void _rb_ot_shape(rb_shape_plan_t *shape_plan,
                  rb_font_t *font,
                  rb_buffer_t *buffer,
                  const rb_feature_t *features,
                  unsigned int num_features)
{
    rb_ot_shape_context_t c = {
        &shape_plan->ot, (rb_font_t *)font, rb_font_get_face(font), buffer, features, num_features};
    rb_ot_shape_internal(&c);
}

/**
 * rb_ot_shape_plan_collect_lookups:
 *
 * Since: 0.9.7
 **/
void rb_ot_shape_plan_collect_lookups(rb_shape_plan_t *shape_plan,
                                      rb_tag_t table_tag,
                                      rb_set_t *lookup_indexes /* OUT */)
{
    shape_plan->ot.collect_lookups(table_tag, lookup_indexes);
}
