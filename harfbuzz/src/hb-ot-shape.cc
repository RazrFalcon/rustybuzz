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

#ifndef HB_NO_OT_SHAPE

#ifdef HB_NO_OT_LAYOUT
#error "Cannot compile 'ot' shaper with HB_NO_OT_LAYOUT."
#endif

#include "hb-shaper-impl.hh"

#include "hb-ot-shape-complex.hh"
#include "hb-ot-shape-fallback.hh"
#include "hb-ot-shape-normalize.hh"
#include "hb-ot-shape.hh"

#include "hb-ot-face.hh"

#include "hb-set.hh"

#include "hb-aat-layout.hh"

#include "hb-ot-map.h"

/**
 * SECTION:hb-ot-shape
 * @title: hb-ot-shape
 * @short_description: OpenType shaping support
 * @include: hb-ot.h
 *
 * Support functions for OpenType shaping related queries.
 **/

static void hb_ot_shape_collect_features(hb_ot_shape_planner_t *planner,
                                         const hb_feature_t *user_features,
                                         unsigned int num_user_features);

static inline bool _hb_apply_morx(hb_face_t *face)
{
    if (hb_options().aat && hb_aat_layout_has_substitution(face))
        return true;

    /* Ignore empty GSUB tables. */
    return (!rb_ot_layout_has_substitution(face->rust_data) ||
            !rb_ot_layout_table_get_script_count(face->rust_data, HB_OT_TAG_GSUB)) &&
           hb_aat_layout_has_substitution(face);
}

hb_ot_shape_planner_t::hb_ot_shape_planner_t(hb_face_t *face, const hb_segment_properties_t *props)
    : face(face)
    , props(*props)
    , map(rb_ot_map_builder_init(face->rust_data, props))
    , aat_map(face, props)
    , apply_morx(_hb_apply_morx(face))
{
    shaper = hb_ot_shape_complex_categorize(this);

    script_zero_marks = shaper->zero_width_marks != HB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE;
    script_fallback_mark_positioning = shaper->fallback_position;

    if (apply_morx)
        shaper = &_hb_ot_complex_shaper_default;
}

hb_ot_shape_planner_t::~hb_ot_shape_planner_t()
{
    rb_ot_map_builder_fini(map);
}

void hb_ot_shape_planner_t::compile(hb_shape_plan_t &plan, unsigned int *variations_index)
{
    plan.props = props;
    plan.shaper = shaper;
    rb_ot_map_builder_compile(map, plan.map, face->rust_data, variations_index);
    if (apply_morx)
        aat_map.compile(plan.aat_map);

    plan.frac_mask = rb_ot_map_get_1_mask(plan.map, HB_TAG('f', 'r', 'a', 'c'));
    plan.numr_mask = rb_ot_map_get_1_mask(plan.map, HB_TAG('n', 'u', 'm', 'r'));
    plan.dnom_mask = rb_ot_map_get_1_mask(plan.map, HB_TAG('d', 'n', 'o', 'm'));
    plan.has_frac = plan.frac_mask || (plan.numr_mask && plan.dnom_mask);

    plan.rtlm_mask = rb_ot_map_get_1_mask(plan.map, HB_TAG('r', 't', 'l', 'm'));
    hb_tag_t kern_tag =
        HB_DIRECTION_IS_HORIZONTAL(props.direction) ? HB_TAG('k', 'e', 'r', 'n') : HB_TAG('v', 'k', 'r', 'n');
    plan.kern_mask = rb_ot_map_get_mask(plan.map, kern_tag, nullptr);
    plan.requested_kerning = !!plan.kern_mask;
    plan.trak_mask = rb_ot_map_get_mask(plan.map, HB_TAG('t', 'r', 'a', 'k'), nullptr);
    plan.requested_tracking = !!plan.trak_mask;

    bool has_gpos_kern = rb_ot_map_get_feature_index(plan.map, 1, kern_tag) != HB_OT_LAYOUT_NO_FEATURE_INDEX;
    bool disable_gpos = plan.shaper->gpos_tag && plan.shaper->gpos_tag != rb_ot_map_get_chosen_script(plan.map, 1);

    /*
     * Decide who provides glyph classes. GDEF or Unicode.
     */

    if (!rb_ot_layout_has_glyph_classes(face->rust_data))
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
    else if (hb_options().aat && hb_aat_layout_has_positioning(face))
        plan.apply_kerx = true;
    else if (!apply_morx && !disable_gpos && rb_ot_layout_has_positioning(face->rust_data))
        plan.apply_gpos = true;
    else if (hb_aat_layout_has_positioning(face))
        plan.apply_kerx = true;

    if (!plan.apply_kerx && !has_gpos_kern) {
        /* Apparently Apple applies kerx if GPOS kern was not applied. */
        if (hb_aat_layout_has_positioning(face))
            plan.apply_kerx = true;
        else if (hb_ot_layout_has_kerning(face))
            plan.apply_kern = true;
    }

    plan.zero_marks =
        script_zero_marks && !plan.apply_kerx && (!plan.apply_kern || !hb_ot_layout_has_machine_kerning(face));
    plan.has_gpos_mark = !!rb_ot_map_get_1_mask(plan.map, HB_TAG('m', 'a', 'r', 'k'));

    plan.adjust_mark_positioning_when_zeroing =
        !plan.apply_gpos && !plan.apply_kerx && (!plan.apply_kern || !hb_ot_layout_has_cross_kerning(face));

    plan.fallback_mark_positioning = plan.adjust_mark_positioning_when_zeroing && script_fallback_mark_positioning;

    /* Currently we always apply trak. */
    plan.apply_trak = plan.requested_tracking && hb_aat_layout_has_tracking(face);
}

bool hb_shape_plan_t::init0(hb_face_t *face,
                            unsigned int *variations_index,
                            const hb_segment_properties_t *props,
                            const hb_feature_t *user_features,
                            unsigned int num_user_features,
                            const int *coords,
                            unsigned int num_coords)
{
    map = rb_ot_map_init();
    aat_map.init();

    hb_ot_shape_planner_t planner(face, props);

    hb_ot_shape_collect_features(&planner, user_features, num_user_features);

    planner.compile(*this, variations_index);

    if (shaper->data_create) {
        data = shaper->data_create(this);
        if (unlikely(!data)) {
            return false;
        }
    }

    return true;
}

void hb_shape_plan_t::fini()
{
    if (shaper->data_destroy)
        shaper->data_destroy(const_cast<void *>(data));

    rb_ot_map_fini(map);
    aat_map.fini();
}

void hb_shape_plan_t::substitute(hb_font_t *font, rb_buffer_t *buffer) const
{
    if (unlikely(apply_morx))
        hb_aat_layout_substitute(this, font, buffer);
    else
        hb_ot_layout_substitute(map, this, font, buffer);
}

void hb_shape_plan_t::position(hb_font_t *font, rb_buffer_t *buffer) const
{
    if (this->apply_gpos)
        hb_ot_layout_position(map, this, font, buffer);
    else if (this->apply_kerx)
        hb_aat_layout_position(this, font, buffer);
    else if (this->apply_kern)
        hb_ot_layout_kern(this, font, buffer);

    if (this->apply_trak)
        hb_aat_layout_track(this, font, buffer);
}

static const hb_ot_map_feature_t common_features[] = {
    {HB_TAG('a', 'b', 'v', 'm'), F_GLOBAL},
    {HB_TAG('b', 'l', 'w', 'm'), F_GLOBAL},
    {HB_TAG('c', 'c', 'm', 'p'), F_GLOBAL},
    {HB_TAG('l', 'o', 'c', 'l'), F_GLOBAL},
    {HB_TAG('m', 'a', 'r', 'k'), F_GLOBAL_MANUAL_JOINERS},
    {HB_TAG('m', 'k', 'm', 'k'), F_GLOBAL_MANUAL_JOINERS},
    {HB_TAG('r', 'l', 'i', 'g'), F_GLOBAL},
};

static const hb_ot_map_feature_t horizontal_features[] = {
    {HB_TAG('c', 'a', 'l', 't'), F_GLOBAL},
    {HB_TAG('c', 'l', 'i', 'g'), F_GLOBAL},
    {HB_TAG('c', 'u', 'r', 's'), F_GLOBAL},
    {HB_TAG('d', 'i', 's', 't'), F_GLOBAL},
    {HB_TAG('k', 'e', 'r', 'n'), F_GLOBAL_HAS_FALLBACK},
    {HB_TAG('l', 'i', 'g', 'a'), F_GLOBAL},
    {HB_TAG('r', 'c', 'l', 't'), F_GLOBAL},
};

static void hb_ot_shape_collect_features(hb_ot_shape_planner_t *planner,
                                         const hb_feature_t *user_features,
                                         unsigned int num_user_features)
{
    rb_ot_map_builder_t *map = planner->map;

    rb_ot_map_builder_enable_feature(map, HB_TAG('r', 'v', 'r', 'n'), F_NONE, 1);
    rb_ot_map_builder_add_gsub_pause(map, nullptr);

    switch (planner->props.direction) {
    case HB_DIRECTION_LTR:
        rb_ot_map_builder_enable_feature(map, HB_TAG('l', 't', 'r', 'a'), F_NONE, 1);
        rb_ot_map_builder_enable_feature(map, HB_TAG('l', 't', 'r', 'm'), F_NONE, 1);
        break;
    case HB_DIRECTION_RTL:
        rb_ot_map_builder_enable_feature(map, HB_TAG('r', 't', 'l', 'a'), F_NONE, 1);
        rb_ot_map_builder_add_feature(map, HB_TAG('r', 't', 'l', 'm'), F_NONE, 1);
        break;
    case HB_DIRECTION_TTB:
    case HB_DIRECTION_BTT:
    case HB_DIRECTION_INVALID:
    default:
        break;
    }

    /* Automatic fractions. */
    rb_ot_map_builder_add_feature(map, HB_TAG('f', 'r', 'a', 'c'), F_NONE, 1);
    rb_ot_map_builder_add_feature(map, HB_TAG('n', 'u', 'm', 'r'), F_NONE, 1);
    rb_ot_map_builder_add_feature(map, HB_TAG('d', 'n', 'o', 'm'), F_NONE, 1);

    /* Random! */
    rb_ot_map_builder_enable_feature(map, HB_TAG('r', 'a', 'n', 'd'), F_RANDOM, HB_OT_MAP_MAX_VALUE);

    /* Tracking.  We enable dummy feature here just to allow disabling
     * AAT 'trak' table using features.
     * https://github.com/harfbuzz/harfbuzz/issues/1303 */
    rb_ot_map_builder_enable_feature(map, HB_TAG('t', 'r', 'a', 'k'), F_HAS_FALLBACK, 1);

    rb_ot_map_builder_enable_feature(map, HB_TAG('H', 'A', 'R', 'F'), F_NONE, 1);

    if (planner->shaper->collect_features)
        planner->shaper->collect_features(planner);

    rb_ot_map_builder_enable_feature(map, HB_TAG('B', 'U', 'Z', 'Z'), F_NONE, 1);

    for (unsigned int i = 0; i < ARRAY_LENGTH(common_features); i++)
        rb_ot_map_builder_add_feature(map, common_features[i].tag, common_features[i].flags, 1);

    if (HB_DIRECTION_IS_HORIZONTAL(planner->props.direction))
        for (unsigned int i = 0; i < ARRAY_LENGTH(horizontal_features); i++)
            rb_ot_map_builder_add_feature(map, horizontal_features[i].tag, horizontal_features[i].flags, 1);
    else {
        /* We really want to find a 'vert' feature if there's any in the font, no
         * matter which script/langsys it is listed (or not) under.
         * See various bugs referenced from:
         * https://github.com/harfbuzz/harfbuzz/issues/63 */
        rb_ot_map_builder_enable_feature(map, HB_TAG('v', 'e', 'r', 't'), F_GLOBAL_SEARCH, 1);
    }

    for (unsigned int i = 0; i < num_user_features; i++) {
        const hb_feature_t *feature = &user_features[i];
        rb_ot_map_builder_add_feature(
            map,
            feature->tag,
            (feature->start == HB_FEATURE_GLOBAL_START && feature->end == HB_FEATURE_GLOBAL_END) ? F_GLOBAL : F_NONE,
            feature->value);
    }

    if (planner->apply_morx) {
        hb_aat_map_builder_t *aat_map = &planner->aat_map;
        for (unsigned int i = 0; i < num_user_features; i++) {
            const hb_feature_t *feature = &user_features[i];
            aat_map->add_feature(feature->tag, feature->value);
        }
    }

    if (planner->shaper->override_features)
        planner->shaper->override_features(planner);
}

/*
 * shaper
 */

struct hb_ot_shape_context_t
{
    hb_shape_plan_t *plan;
    hb_font_t *font;
    hb_face_t *face;
    rb_buffer_t *buffer;
    const hb_feature_t *user_features;
    unsigned int num_user_features;

    /* Transient stuff */
    hb_direction_t target_direction;
};

/* Main shaper */

/* Prepare */

static void hb_set_unicode_props(rb_buffer_t *buffer)
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
    hb_glyph_info_t *info = rb_buffer_get_info(buffer);
    for (unsigned int i = 0; i < count; i++) {
        _hb_glyph_info_set_unicode_props(&info[i], buffer);

        /* Marks are already set as continuation by the above line.
         * Handle Emoji_Modifier and ZWJ-continuation. */
        if (unlikely(_hb_glyph_info_get_general_category(&info[i]) == HB_UNICODE_GENERAL_CATEGORY_MODIFIER_SYMBOL &&
                     hb_in_range<hb_codepoint_t>(info[i].codepoint, 0x1F3FBu, 0x1F3FFu))) {
            _hb_glyph_info_set_continuation(&info[i]);
        }
#ifndef HB_NO_EMOJI_SEQUENCES
        else if (unlikely(_hb_glyph_info_is_zwj(&info[i]))) {
            _hb_glyph_info_set_continuation(&info[i]);
            if (i + 1 < count && rb_unicode_is_emoji_extended_pictographic(info[i + 1].codepoint)) {
                i++;
                _hb_glyph_info_set_unicode_props(&info[i], buffer);
                _hb_glyph_info_set_continuation(&info[i]);
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
        else if (unlikely(hb_in_range<hb_codepoint_t>(info[i].codepoint, 0xE0020u, 0xE007Fu)))
            _hb_glyph_info_set_continuation(&info[i]);
    }
}

static void hb_insert_dotted_circle(rb_buffer_t *buffer, hb_font_t *font)
{
    if (unlikely(rb_buffer_get_flags(buffer) & HB_BUFFER_FLAG_DO_NOT_INSERT_DOTTED_CIRCLE))
        return;

    if (!(rb_buffer_get_flags(buffer) & HB_BUFFER_FLAG_BOT) || rb_buffer_get_context_len(buffer, 0) ||
        !_hb_glyph_info_is_unicode_mark(&rb_buffer_get_info(buffer)[0]))
        return;

    if (!font->has_glyph(0x25CCu))
        return;

    hb_glyph_info_t dottedcircle = {0};
    dottedcircle.codepoint = 0x25CCu;
    _hb_glyph_info_set_unicode_props(&dottedcircle, buffer);

    rb_buffer_clear_output(buffer);

    rb_buffer_set_idx(buffer, 0);
    hb_glyph_info_t info = dottedcircle;
    info.cluster = rb_buffer_get_cur(buffer, 0)->cluster;
    info.mask = rb_buffer_get_cur(buffer, 0)->mask;
    rb_buffer_output_info(buffer, info);
    while (rb_buffer_get_idx(buffer) < rb_buffer_get_length(buffer))
        rb_buffer_next_glyph(buffer);
    rb_buffer_swap_buffers(buffer);
}

static void hb_form_clusters(rb_buffer_t *buffer)
{
    if (!(*rb_buffer_get_scratch_flags(buffer) & HB_BUFFER_SCRATCH_FLAG_HAS_NON_ASCII))
        return;

    if (rb_buffer_get_cluster_level(buffer) == HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES)
        foreach_grapheme(buffer, start, end) rb_buffer_merge_clusters(buffer, start, end);
    else
        foreach_grapheme(buffer, start, end) rb_buffer_unsafe_to_break(buffer, start, end);
}

static void hb_ensure_native_direction(rb_buffer_t *buffer)
{
    hb_direction_t direction = rb_buffer_get_direction(buffer);
    hb_direction_t horiz_dir = hb_script_get_horizontal_direction(rb_buffer_get_script(buffer));

    /* TODO vertical:
     * The only BTT vertical script is Ogham, but it's not clear to me whether OpenType
     * Ogham fonts are supposed to be implemented BTT or not.  Need to research that
     * first. */
    if ((HB_DIRECTION_IS_HORIZONTAL(direction) && direction != horiz_dir && horiz_dir != HB_DIRECTION_INVALID) ||
        (HB_DIRECTION_IS_VERTICAL(direction) && direction != HB_DIRECTION_TTB)) {

        if (rb_buffer_get_cluster_level(buffer) == HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS)
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

        rb_buffer_set_direction(buffer, HB_DIRECTION_REVERSE(rb_buffer_get_direction(buffer)));
    }
}

/*
 * Substitute
 */

static inline void hb_ot_mirror_chars(const hb_ot_shape_context_t *c)
{
    if (HB_DIRECTION_IS_FORWARD(c->target_direction))
        return;

    rb_buffer_t *buffer = c->buffer;
    hb_mask_t rtlm_mask = c->plan->rtlm_mask;

    unsigned int count = rb_buffer_get_length(buffer);
    hb_glyph_info_t *info = rb_buffer_get_info(buffer);
    for (unsigned int i = 0; i < count; i++) {
        hb_codepoint_t codepoint = rb_ucd_mirroring(info[i].codepoint);
        if (likely(codepoint == info[i].codepoint || !c->font->has_glyph(codepoint)))
            info[i].mask |= rtlm_mask;
        else
            info[i].codepoint = codepoint;
    }
}

static inline void hb_ot_shape_setup_masks_fraction(const hb_ot_shape_context_t *c)
{
    if (!(*rb_buffer_get_scratch_flags(c->buffer) & HB_BUFFER_SCRATCH_FLAG_HAS_NON_ASCII) || !c->plan->has_frac)
        return;

    rb_buffer_t *buffer = c->buffer;

    hb_mask_t pre_mask, post_mask;
    if (HB_DIRECTION_IS_FORWARD(rb_buffer_get_direction(buffer))) {
        pre_mask = c->plan->numr_mask | c->plan->frac_mask;
        post_mask = c->plan->frac_mask | c->plan->dnom_mask;
    } else {
        pre_mask = c->plan->frac_mask | c->plan->dnom_mask;
        post_mask = c->plan->numr_mask | c->plan->frac_mask;
    }

    unsigned int count = rb_buffer_get_length(buffer);
    hb_glyph_info_t *info = rb_buffer_get_info(buffer);
    for (unsigned int i = 0; i < count; i++) {
        if (info[i].codepoint == 0x2044u) /* FRACTION SLASH */
        {
            unsigned int start = i, end = i + 1;
            while (start &&
                   _hb_glyph_info_get_general_category(&info[start - 1]) == HB_UNICODE_GENERAL_CATEGORY_DECIMAL_NUMBER)
                start--;
            while (end < count &&
                   _hb_glyph_info_get_general_category(&info[end]) == HB_UNICODE_GENERAL_CATEGORY_DECIMAL_NUMBER)
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

static inline void hb_ot_shape_initialize_masks(const hb_ot_shape_context_t *c)
{
    rb_ot_map_t *map = c->plan->map;
    rb_buffer_t *buffer = c->buffer;

    hb_mask_t global_mask = rb_ot_map_get_global_mask(map);
    rb_buffer_reset_masks(buffer, global_mask);
}

static inline void hb_ot_shape_setup_masks(const hb_ot_shape_context_t *c)
{
    rb_ot_map_t *map = c->plan->map;
    rb_buffer_t *buffer = c->buffer;

    hb_ot_shape_setup_masks_fraction(c);

    if (c->plan->shaper->setup_masks)
        c->plan->shaper->setup_masks(c->plan, buffer, c->font);

    for (unsigned int i = 0; i < c->num_user_features; i++) {
        const hb_feature_t *feature = &c->user_features[i];
        if (!(feature->start == 0 && feature->end == (unsigned int)-1)) {
            unsigned int shift;
            hb_mask_t mask = rb_ot_map_get_mask(map, feature->tag, &shift);
            rb_buffer_set_masks(buffer, feature->value << shift, mask, feature->start, feature->end);
        }
    }
}

static void hb_ot_zero_width_default_ignorables(rb_buffer_t *buffer)
{
    if (!(*rb_buffer_get_scratch_flags(buffer) & HB_BUFFER_SCRATCH_FLAG_HAS_DEFAULT_IGNORABLES) ||
        (rb_buffer_get_flags(buffer) & HB_BUFFER_FLAG_PRESERVE_DEFAULT_IGNORABLES) ||
        (rb_buffer_get_flags(buffer) & HB_BUFFER_FLAG_REMOVE_DEFAULT_IGNORABLES))
        return;

    unsigned int count = rb_buffer_get_length(buffer);
    hb_glyph_info_t *info = rb_buffer_get_info(buffer);
    hb_glyph_position_t *pos = rb_buffer_get_pos(buffer);
    unsigned int i = 0;
    for (i = 0; i < count; i++)
        if (unlikely(_hb_glyph_info_is_default_ignorable(&info[i])))
            pos[i].x_advance = pos[i].y_advance = pos[i].x_offset = pos[i].y_offset = 0;
}

static void hb_ot_hide_default_ignorables(rb_buffer_t *buffer, hb_font_t *font)
{
    if (!(*rb_buffer_get_scratch_flags(buffer) & HB_BUFFER_SCRATCH_FLAG_HAS_DEFAULT_IGNORABLES) ||
        (rb_buffer_get_flags(buffer) & HB_BUFFER_FLAG_PRESERVE_DEFAULT_IGNORABLES))
        return;

    unsigned int count = rb_buffer_get_length(buffer);
    hb_glyph_info_t *info = rb_buffer_get_info(buffer);

    hb_codepoint_t invisible = rb_buffer_get_invisible_glyph(buffer);
    if (!(rb_buffer_get_flags(buffer) & HB_BUFFER_FLAG_REMOVE_DEFAULT_IGNORABLES) &&
        (invisible || font->get_nominal_glyph(' ', &invisible))) {
        /* Replace default-ignorables with a zero-advance invisible glyph. */
        for (unsigned int i = 0; i < count; i++) {
            if (_hb_glyph_info_is_default_ignorable(&info[i]))
                info[i].codepoint = invisible;
        }
    } else
        hb_ot_layout_delete_glyphs_inplace(buffer, _hb_glyph_info_is_default_ignorable);
}

static inline void hb_ot_map_glyphs_fast(rb_buffer_t *buffer)
{
    /* Normalization process sets up glyph_index(), we just copy it. */
    unsigned int count = rb_buffer_get_length(buffer);
    hb_glyph_info_t *info = rb_buffer_get_info(buffer);
    for (unsigned int i = 0; i < count; i++)
        info[i].codepoint = info[i].glyph_index();
}

static inline void hb_synthesize_glyph_classes(rb_buffer_t *buffer)
{
    unsigned int count = rb_buffer_get_length(buffer);
    hb_glyph_info_t *info = rb_buffer_get_info(buffer);
    for (unsigned int i = 0; i < count; i++) {
        hb_ot_layout_glyph_props_flags_t klass;

        /* Never mark default-ignorables as marks.
         * They won't get in the way of lookups anyway,
         * but having them as mark will cause them to be skipped
         * over if the lookup-flag says so, but at least for the
         * Mongolian variation selectors, looks like Uniscribe
         * marks them as non-mark.  Some Mongolian fonts without
         * GDEF rely on this.  Another notable character that
         * this applies to is COMBINING GRAPHEME JOINER. */
        klass = (_hb_glyph_info_get_general_category(&info[i]) != HB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK ||
                 _hb_glyph_info_is_default_ignorable(&info[i]))
                    ? HB_OT_LAYOUT_GLYPH_PROPS_BASE_GLYPH
                    : HB_OT_LAYOUT_GLYPH_PROPS_MARK;
        _hb_glyph_info_set_glyph_props(&info[i], klass);
    }
}

static inline void hb_ot_substitute_default(const hb_ot_shape_context_t *c)
{
    rb_buffer_t *buffer = c->buffer;

    hb_ot_mirror_chars(c);

    _hb_ot_shape_normalize(c->plan, buffer, c->font);

    hb_ot_shape_setup_masks(c);

    /* This is unfortunate to go here, but necessary... */
    if (c->plan->fallback_mark_positioning)
        _hb_ot_shape_fallback_mark_position_recategorize_marks(c->plan, c->font, buffer);

    hb_ot_map_glyphs_fast(buffer);
}

static inline void hb_ot_substitute_complex(const hb_ot_shape_context_t *c)
{
    rb_buffer_t *buffer = c->buffer;

    hb_ot_layout_substitute_start(c->font, buffer);

    if (c->plan->fallback_glyph_classes)
        hb_synthesize_glyph_classes(c->buffer);

    c->plan->substitute(c->font, buffer);
}

static inline void hb_ot_substitute_pre(const hb_ot_shape_context_t *c)
{
    hb_ot_substitute_default(c);

    hb_ot_substitute_complex(c);
}

static inline void hb_ot_substitute_post(const hb_ot_shape_context_t *c)
{
    hb_ot_hide_default_ignorables(c->buffer, c->font);
    if (c->plan->apply_morx)
        hb_aat_layout_remove_deleted_glyphs(c->buffer);

    if (c->plan->shaper->postprocess_glyphs)
        c->plan->shaper->postprocess_glyphs(c->plan, c->buffer, c->font);
}

/*
 * Position
 */

static inline void adjust_mark_offsets(hb_glyph_position_t *pos)
{
    pos->x_offset -= pos->x_advance;
    pos->y_offset -= pos->y_advance;
}

static inline void zero_mark_width(hb_glyph_position_t *pos)
{
    pos->x_advance = 0;
    pos->y_advance = 0;
}

static inline void zero_mark_widths_by_gdef(rb_buffer_t *buffer, bool adjust_offsets)
{
    unsigned int count = rb_buffer_get_length(buffer);
    hb_glyph_info_t *info = rb_buffer_get_info(buffer);
    for (unsigned int i = 0; i < count; i++)
        if (_hb_glyph_info_is_mark(&info[i])) {
            if (adjust_offsets)
                adjust_mark_offsets(&rb_buffer_get_pos(buffer)[i]);
            zero_mark_width(&rb_buffer_get_pos(buffer)[i]);
        }
}

static inline void hb_ot_position_default(const hb_ot_shape_context_t *c)
{
    hb_direction_t direction = rb_buffer_get_direction(c->buffer);
    unsigned int count = rb_buffer_get_length(c->buffer);
    hb_glyph_info_t *info = rb_buffer_get_info(c->buffer);
    hb_glyph_position_t *pos = rb_buffer_get_pos(c->buffer);

    if (HB_DIRECTION_IS_HORIZONTAL(direction)) {
        c->font->get_glyph_h_advances(count, &info[0].codepoint, sizeof(info[0]), &pos[0].x_advance, sizeof(pos[0]));
    } else {
        c->font->get_glyph_v_advances(count, &info[0].codepoint, sizeof(info[0]), &pos[0].y_advance, sizeof(pos[0]));
        for (unsigned int i = 0; i < count; i++) {
            c->font->subtract_glyph_v_origin(info[i].codepoint, &pos[i].x_offset, &pos[i].y_offset);
        }
    }
    if (*rb_buffer_get_scratch_flags(c->buffer) & HB_BUFFER_SCRATCH_FLAG_HAS_SPACE_FALLBACK)
        _hb_ot_shape_fallback_spaces(c->plan, c->font, c->buffer);
}

static inline void hb_ot_position_complex(const hb_ot_shape_context_t *c)
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
        c->plan->adjust_mark_positioning_when_zeroing && HB_DIRECTION_IS_FORWARD(rb_buffer_get_direction(c->buffer));

    /* We change glyph origin to what GPOS expects (horizontal), apply GPOS, change it back. */

    hb_ot_layout_position_start(c->font, c->buffer);

    if (c->plan->zero_marks)
        switch (c->plan->shaper->zero_width_marks) {
        case HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_EARLY:
            zero_mark_widths_by_gdef(c->buffer, adjust_offsets_when_zeroing);
            break;

        default:
        case HB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE:
        case HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE:
            break;
        }

    c->plan->position(c->font, c->buffer);

    if (c->plan->zero_marks)
        switch (c->plan->shaper->zero_width_marks) {
        case HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE:
            zero_mark_widths_by_gdef(c->buffer, adjust_offsets_when_zeroing);
            break;

        default:
        case HB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE:
        case HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_EARLY:
            break;
        }

    /* Finish off.  Has to follow a certain order. */
    hb_ot_layout_position_finish_advances(c->font, c->buffer);
    hb_ot_zero_width_default_ignorables(c->buffer);
    if (c->plan->apply_morx)
        hb_aat_layout_zero_width_deleted_glyphs(c->buffer);
    hb_ot_layout_position_finish_offsets(c->font, c->buffer);

    if (c->plan->fallback_mark_positioning)
        _hb_ot_shape_fallback_mark_position(c->plan, c->font, c->buffer, adjust_offsets_when_zeroing);
}

static inline void hb_ot_position(const hb_ot_shape_context_t *c)
{
    rb_buffer_clear_positions(c->buffer);

    hb_ot_position_default(c);

    hb_ot_position_complex(c);

    if (HB_DIRECTION_IS_BACKWARD(rb_buffer_get_direction(c->buffer)))
        rb_buffer_reverse(c->buffer);
}

static inline void hb_propagate_flags(rb_buffer_t *buffer)
{
    /* Propagate cluster-level glyph flags to be the same on all cluster glyphs.
     * Simplifies using them. */

    if (!(*rb_buffer_get_scratch_flags(buffer) & HB_BUFFER_SCRATCH_FLAG_HAS_UNSAFE_TO_BREAK))
        return;

    hb_glyph_info_t *info = rb_buffer_get_info(buffer);

    foreach_cluster(buffer, start, end)
    {
        unsigned int mask = 0;
        for (unsigned int i = start; i < end; i++)
            if (info[i].mask & HB_GLYPH_FLAG_UNSAFE_TO_BREAK) {
                mask = HB_GLYPH_FLAG_UNSAFE_TO_BREAK;
                break;
            }
        if (mask)
            for (unsigned int i = start; i < end; i++)
                info[i].mask |= mask;
    }
}

/* Pull it all together! */

static void hb_ot_shape_internal(hb_ot_shape_context_t *c)
{
    *rb_buffer_get_scratch_flags(c->buffer) = HB_BUFFER_SCRATCH_FLAG_DEFAULT;
    if (likely(!hb_unsigned_mul_overflows(rb_buffer_get_length(c->buffer), HB_BUFFER_MAX_OPS_FACTOR))) {
        rb_buffer_set_max_ops(
            c->buffer,
            hb_max(rb_buffer_get_length(c->buffer) * HB_BUFFER_MAX_OPS_FACTOR, (unsigned)HB_BUFFER_MAX_OPS_MIN));
    }

    /* Save the original direction, we use it later. */
    c->target_direction = rb_buffer_get_direction(c->buffer);

    rb_buffer_clear_output(c->buffer);

    hb_ot_shape_initialize_masks(c);
    hb_set_unicode_props(c->buffer);
    hb_insert_dotted_circle(c->buffer, c->font);

    hb_form_clusters(c->buffer);

    hb_ensure_native_direction(c->buffer);

    if (c->plan->shaper->preprocess_text)
        c->plan->shaper->preprocess_text(c->plan, c->buffer, c->font);

    hb_ot_substitute_pre(c);
    hb_ot_position(c);
    hb_ot_substitute_post(c);

    hb_propagate_flags(c->buffer);

    rb_buffer_set_direction(c->buffer, c->target_direction);

    rb_buffer_set_max_ops(c->buffer, HB_BUFFER_MAX_OPS_DEFAULT);
}

hb_bool_t _hb_ot_shape(hb_shape_plan_t *shape_plan,
                       hb_font_t *font,
                       rb_buffer_t *buffer,
                       const hb_feature_t *features,
                       unsigned int num_features)
{
    hb_ot_shape_context_t c = {shape_plan, font, font->face, buffer, features, num_features};
    hb_ot_shape_internal(&c);

    return true;
}

#endif
