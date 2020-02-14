/*
 * Copyright © 2011,2012,2013  Google, Inc.
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

#include "hb-ot-shape-complex-myanmar.hh"

extern "C" {
void hb_complex_myanmar_setup_syllables(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer);
void hb_complex_myanmar_reorder(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer);
void rb_complex_myanmar_collect_features(rb_ot_map_builder_t *map);
void rb_complex_myanmar_override_features(rb_ot_map_builder_t *map);
void rb_complex_myanmar_set_properties(hb_glyph_info_t *info);
void rb_complex_myanmar_setup_syllables(rb_buffer_t *buffer);
void rb_complex_myanmar_setup_masks(rb_buffer_t *buffer);
void rb_complex_myanmar_reorder(const void *rust_data, rb_buffer_t *buffer);
}

static void collect_features_myanmar(hb_ot_shape_planner_t *plan)
{
    rb_complex_myanmar_collect_features(plan->map);
}

static void override_features_myanmar(hb_ot_shape_planner_t *plan)
{
    rb_complex_myanmar_override_features(plan->map);
}

static void
setup_masks_myanmar(const hb_shape_plan_t *plan HB_UNUSED, rb_buffer_t *buffer, hb_font_t *font HB_UNUSED)
{
    rb_complex_myanmar_setup_masks(buffer);
}

void hb_complex_myanmar_setup_syllables(const hb_shape_plan_t *plan HB_UNUSED, hb_font_t *font HB_UNUSED, rb_buffer_t *buffer)
{
    rb_complex_myanmar_setup_syllables(buffer);
}

void hb_complex_myanmar_reorder(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer)
{
    rb_complex_myanmar_reorder(font->rust_data, buffer);
}

const hb_ot_complex_shaper_t _hb_ot_complex_shaper_myanmar = {
    collect_features_myanmar,
    override_features_myanmar,
    nullptr, /* data_create */
    nullptr, /* data_destroy */
    nullptr, /* preprocess_text */
    nullptr, /* postprocess_glyphs */
    HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT,
    nullptr, /* decompose */
    nullptr, /* compose */
    setup_masks_myanmar,
    HB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_EARLY,
    false, /* fallback_position */
};

/* Ugly Zawgyi encoding.
 * Disable all auto processing.
 * https://github.com/harfbuzz/harfbuzz/issues/1162 */
const hb_ot_complex_shaper_t _hb_ot_complex_shaper_myanmar_zawgyi = {
    nullptr, /* collect_features */
    nullptr, /* override_features */
    nullptr, /* data_create */
    nullptr, /* data_destroy */
    nullptr, /* preprocess_text */
    nullptr, /* postprocess_glyphs */
    HB_OT_SHAPE_NORMALIZATION_MODE_NONE,
    nullptr,     /* decompose */
    nullptr,     /* compose */
    nullptr,     /* setup_masks */
    HB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    false, /* fallback_position */
};

#endif
