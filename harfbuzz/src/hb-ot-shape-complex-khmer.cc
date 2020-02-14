/*
 * Copyright © 2011,2012  Google, Inc.
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

#ifndef HB_NO_OT_SHAPE

#include "hb-ot-layout.hh"
#include "hb-ot-shape-complex-khmer.hh"

typedef struct khmer_shape_plan_t khmer_shape_plan_t;

extern "C" {
void* rb_complex_khmer_data_create(const rb_ot_map_t *map);
void rb_complex_khmer_data_destroy(void *data);
void hb_complex_khmer_setup_syllables(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer);
void hb_complex_khmer_reorder(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer);
void rb_complex_khmer_collect_features(rb_ot_map_builder_t *map);
void rb_complex_khmer_override_features(rb_ot_map_builder_t *map);
bool rb_complex_khmer_decompose(hb_codepoint_t ab, hb_codepoint_t *a, hb_codepoint_t *b);
bool rb_complex_khmer_compose(hb_codepoint_t a, hb_codepoint_t b, hb_codepoint_t *ab);
void rb_complex_khmer_reorder(const khmer_shape_plan_t *plan, const void *rust_data, rb_buffer_t *buffer);
void rb_complex_khmer_setup_masks(rb_buffer_t *buffer);
void rb_complex_khmer_setup_syllables(rb_buffer_t *buffer);
}

static void collect_features_khmer(hb_ot_shape_planner_t *plan)
{
    rb_complex_khmer_collect_features(plan->map);
}

static void override_features_khmer(hb_ot_shape_planner_t *plan)
{
    rb_ot_map_builder_t *map = plan->map;
    rb_complex_khmer_override_features(map);
}

static void *data_create_khmer(const hb_shape_plan_t *plan)
{
    return rb_complex_khmer_data_create(plan->map);
}

static void data_destroy_khmer(void *data)
{
    rb_complex_khmer_data_destroy(data);
}

static void setup_masks_khmer(const hb_shape_plan_t *plan HB_UNUSED, rb_buffer_t *buffer, hb_font_t *font HB_UNUSED)
{
    rb_complex_khmer_setup_masks(buffer);
}

void
hb_complex_khmer_setup_syllables(const hb_shape_plan_t *plan HB_UNUSED, hb_font_t *font HB_UNUSED, rb_buffer_t *buffer)
{
    rb_complex_khmer_setup_syllables(buffer);
}

void hb_complex_khmer_reorder(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer)
{
    const khmer_shape_plan_t *khmer_plan = (const khmer_shape_plan_t *)plan->data;
    rb_complex_khmer_reorder(khmer_plan, font->rust_data, buffer);
}

static bool
decompose_khmer(const hb_ot_shape_normalize_context_t *c, hb_codepoint_t ab, hb_codepoint_t *a, hb_codepoint_t *b)
{
    return rb_complex_khmer_decompose(ab, a, b);
}

static bool
compose_khmer(const hb_ot_shape_normalize_context_t *c, hb_codepoint_t a, hb_codepoint_t b, hb_codepoint_t *ab)
{
    return rb_complex_khmer_compose(a, b, ab);
}

const hb_ot_complex_shaper_t _hb_ot_complex_shaper_khmer = {
    collect_features_khmer,
    override_features_khmer,
    data_create_khmer,
    data_destroy_khmer,
    nullptr, /* preprocess_text */
    nullptr, /* postprocess_glyphs */
    HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT,
    decompose_khmer,
    compose_khmer,
    setup_masks_khmer,
    HB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    false, /* fallback_position */
};

#endif
