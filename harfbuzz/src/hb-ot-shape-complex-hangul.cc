/*
 * Copyright © 2013  Google, Inc.
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
 * Google Author(s): Behdad Esfahbod
 */

#include "hb.hh"

#include "hb-ot-shape-complex.hh"

/* Hangul shaper */

extern "C" {
void* rb_complex_hangul_data_create(const rb_ot_map_t *map);
void rb_complex_hangul_data_destroy(void *data);
void rb_complex_hangul_collect_features(rb_ot_map_builder_t *builder);
void rb_complex_hangul_override_features(rb_ot_map_builder_t *builder);
void rb_complex_hangul_preprocess_text(rb_buffer_t *buffer, hb_font_t *font);
void rb_complex_hangul_setup_masks(const void *hangul_plan, rb_buffer_t *buffer);
}

static void collect_features_hangul(hb_ot_shape_planner_t *plan)
{
    rb_complex_hangul_collect_features(plan->map);
}

static void override_features_hangul(hb_ot_shape_planner_t *plan)
{
    rb_complex_hangul_override_features(plan->map);
}

static void *data_create_hangul(const hb_shape_plan_t *plan)
{
    return rb_complex_hangul_data_create(plan->map);
}

static void preprocess_text_hangul(const hb_shape_plan_t *plan HB_UNUSED, rb_buffer_t *buffer, hb_font_t *font)
{
    rb_complex_hangul_preprocess_text(buffer, font);
}

static void setup_masks_hangul(const hb_shape_plan_t *plan, rb_buffer_t *buffer, hb_font_t *font HB_UNUSED)
{
    rb_complex_hangul_setup_masks(plan->data, buffer);
}

const hb_ot_complex_shaper_t _hb_ot_complex_shaper_hangul = {
    collect_features_hangul,
    override_features_hangul,
    data_create_hangul,
    rb_complex_hangul_data_destroy,
    preprocess_text_hangul,
    nullptr, /* postprocess_glyphs */
    HB_OT_SHAPE_NORMALIZATION_MODE_NONE,
    nullptr, /* decompose */
    nullptr, /* compose */
    setup_masks_hangul,
    HB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    false, /* fallback_position */
};
