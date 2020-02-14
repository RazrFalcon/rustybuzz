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

#include "hb-ot-layout.hh"
#include "hb-ot-shape-complex.hh"

typedef struct indic_shape_plan_t indic_shape_plan_t;

extern "C" {
void rb_complex_indic_collect_features(rb_ot_map_builder_t *map);
void rb_complex_indic_override_features(rb_ot_map_builder_t *map);
void hb_complex_indic_setup_syllables(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer);
void hb_complex_indic_initial_reordering(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer);
void rb_complex_indic_final_reordering(const indic_shape_plan_t *indic_plan, hb_tag_t script, rb_buffer_t *buffer);
void hb_complex_indic_final_reordering(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer);
void* rb_complex_indic_data_create(const rb_ot_map_t *map, hb_tag_t script);
void rb_complex_indic_data_destroy(void *data);
bool rb_complex_indic_decompose(const indic_shape_plan_t *indic_plan,
                                hb_face_t *face,
                                const rb_ttf_parser_t *ttf_parser,
                                hb_codepoint_t ab,
                                hb_codepoint_t *a,
                                hb_codepoint_t *b);
bool rb_complex_indic_compose(hb_codepoint_t a, hb_codepoint_t b, hb_codepoint_t *ab);
void rb_indic_setup_masks(rb_buffer_t *buffer);
void rb_complex_indic_initial_reordering(const rb_ttf_parser_t *ttf_parser,
                                         hb_face_t *face,
                                         indic_shape_plan_t *indic_plan,
                                         rb_buffer_t *buffer);
void rb_complex_indic_setup_syllables(rb_buffer_t *buffer);
}

static void collect_features_indic(hb_ot_shape_planner_t *plan)
{
    rb_ot_map_builder_t *map = plan->map;
    rb_complex_indic_collect_features(map);
}

static void override_features_indic(hb_ot_shape_planner_t *plan)
{
    rb_ot_map_builder_t *map = plan->map;
    rb_complex_indic_override_features(map);
}

static void *data_create_indic(const hb_shape_plan_t *plan)
{
    return rb_complex_indic_data_create(plan->map, plan->props.script);
}

static void data_destroy_indic(void *data)
{
    rb_complex_indic_data_destroy(data);
}

static void setup_masks_indic(const hb_shape_plan_t *plan HB_UNUSED, rb_buffer_t *buffer, hb_font_t *font HB_UNUSED)
{
    rb_indic_setup_masks(buffer);
}

void hb_complex_indic_setup_syllables(const hb_shape_plan_t *plan HB_UNUSED,
                                      hb_font_t *font HB_UNUSED,
                                      rb_buffer_t *buffer)
{
    rb_complex_indic_setup_syllables(buffer);
}

void hb_complex_indic_initial_reordering(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer)
{
    indic_shape_plan_t *indic_plan = (indic_shape_plan_t *)plan->data;
    rb_complex_indic_initial_reordering(font->ttf_parser, font->face, indic_plan, buffer);
}

void hb_complex_indic_final_reordering(const hb_shape_plan_t *plan, hb_font_t *font HB_UNUSED, rb_buffer_t *buffer)
{
    const indic_shape_plan_t *indic_plan = (const indic_shape_plan_t *)plan->data;
    rb_complex_indic_final_reordering(indic_plan, plan->props.script, buffer);
}

static void preprocess_text_indic(const hb_shape_plan_t *plan, rb_buffer_t *buffer, hb_font_t *font)
{
    rb_preprocess_text_vowel_constraints(buffer);
}

static bool
decompose_indic(const hb_ot_shape_normalize_context_t *c, hb_codepoint_t ab, hb_codepoint_t *a, hb_codepoint_t *b)
{
    const indic_shape_plan_t *indic_plan = (const indic_shape_plan_t *)c->plan->data;
    return rb_complex_indic_decompose(indic_plan, c->font->face, c->font->ttf_parser, ab, a, b);
}

static bool
compose_indic(const hb_ot_shape_normalize_context_t *c, hb_codepoint_t a, hb_codepoint_t b, hb_codepoint_t *ab)
{
    return rb_complex_indic_compose(a, b, ab);
}

const hb_ot_complex_shaper_t _hb_ot_complex_shaper_indic = {
    collect_features_indic,
    override_features_indic,
    data_create_indic,
    data_destroy_indic,
    preprocess_text_indic,
    nullptr, /* postprocess_glyphs */
    HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT,
    decompose_indic,
    compose_indic,
    setup_masks_indic,
    HB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    false, /* fallback_position */
};
