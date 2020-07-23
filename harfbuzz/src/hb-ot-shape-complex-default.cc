/*
 * Copyright © 2010,2012  Google, Inc.
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

void rb_clear_substitution_flags(const rb_ot_shape_plan_t *plan RB_UNUSED,
                                 rb_font_t *font RB_UNUSED,
                                 rb_buffer_t *buffer)
{
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);
    unsigned int count = rb_buffer_get_length(buffer);
    for (unsigned int i = 0; i < count; i++)
        _rb_glyph_info_clear_substituted(&info[i]);
}

const rb_ot_complex_shaper_t _rb_ot_complex_shaper_arabic = {
    rb_ot_complex_collect_features_arabic,
    nullptr, /* override_features */
    rb_ot_complex_data_create_arabic,
    rb_ot_complex_data_destroy_arabic,
    nullptr, /* preprocess_text */
    rb_ot_complex_postprocess_glyphs_arabic,
    RB_OT_SHAPE_NORMALIZATION_MODE_DEFAULT,
    nullptr, /* decompose */
    nullptr, /* compose */
    rb_ot_complex_setup_masks_arabic,
    RB_TAG_NONE, /* gpos_tag */
    rb_ot_complex_reorder_marks_arabic,
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE,
    true, /* fallback_position */
};

const rb_ot_complex_shaper_t _rb_ot_complex_shaper_hangul = {
    rb_ot_complex_collect_features_hangul,
    rb_ot_complex_override_features_hangul,
    rb_ot_complex_data_create_hangul,
    rb_ot_complex_data_destroy_hangul,
    rb_ot_complex_preprocess_text_hangul,
    nullptr, /* postprocess_glyphs */
    RB_OT_SHAPE_NORMALIZATION_MODE_NONE,
    nullptr, /* decompose */
    nullptr, /* compose */
    rb_ot_complex_setup_masks_hangul,
    RB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    false, /* fallback_position */
};

const rb_ot_complex_shaper_t _rb_ot_complex_shaper_hebrew = {
    nullptr, /* collect_features */
    nullptr, /* override_features */
    nullptr, /* data_create */
    nullptr, /* data_destroy */
    nullptr, /* preprocess_text */
    nullptr, /* postprocess_glyphs */
    RB_OT_SHAPE_NORMALIZATION_MODE_DEFAULT,
    nullptr, /* decompose */
    rb_ot_complex_compose_hebrew,
    nullptr,                    /* setup_masks */
    RB_TAG('h', 'e', 'b', 'r'), /* gpos_tag. https://github.com/harfbuzz/harfbuzz/issues/347#issuecomment-267838368 */
    nullptr,                    /* reorder_marks */
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE,
    true, /* fallback_position */
};

const rb_ot_complex_shaper_t _rb_ot_complex_shaper_indic = {
    rb_ot_complex_collect_features_indic,
    rb_ot_complex_override_features_indic,
    rb_ot_complex_data_create_indic,
    rb_ot_complex_data_destroy_indic,
    rb_ot_complex_preprocess_text_indic,
    nullptr, /* postprocess_glyphs */
    RB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT,
    rb_ot_complex_decompose_indic,
    rb_ot_complex_compose_indic,
    rb_ot_complex_setup_masks_indic,
    RB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    false, /* fallback_position */
};

const rb_ot_complex_shaper_t _rb_ot_complex_shaper_khmer = {
    rb_ot_complex_collect_features_khmer,
    rb_ot_complex_override_features_khmer,
    rb_ot_complex_data_create_khmer,
    rb_ot_complex_data_destroy_khmer,
    nullptr, /* preprocess_text */
    nullptr, /* postprocess_glyphs */
    RB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT,
    rb_ot_complex_decompose_khmer,
    rb_ot_complex_compose_khmer,
    rb_ot_complex_setup_masks_khmer,
    RB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    false, /* fallback_position */
};

const rb_ot_complex_shaper_t _rb_ot_complex_shaper_myanmar = {
    rb_ot_complex_collect_features_myanmar,
    rb_ot_complex_override_features_myanmar,
    nullptr, /* data_create */
    nullptr, /* data_destroy */
    nullptr, /* preprocess_text */
    nullptr, /* postprocess_glyphs */
    RB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT,
    nullptr, /* decompose */
    nullptr, /* compose */
    rb_ot_complex_setup_masks_myanmar,
    RB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_EARLY,
    false, /* fallback_position */
};

/* Ugly Zawgyi encoding.
 * Disable all auto processing.
 * https://github.com/harfbuzz/harfbuzz/issues/1162 */
const rb_ot_complex_shaper_t _rb_ot_complex_shaper_myanmar_zawgyi = {
    nullptr, /* collect_features */
    nullptr, /* override_features */
    nullptr, /* data_create */
    nullptr, /* data_destroy */
    nullptr, /* preprocess_text */
    nullptr, /* postprocess_glyphs */
    RB_OT_SHAPE_NORMALIZATION_MODE_NONE,
    nullptr,     /* decompose */
    nullptr,     /* compose */
    nullptr,     /* setup_masks */
    RB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    false, /* fallback_position */
};

const rb_ot_complex_shaper_t _rb_ot_complex_shaper_thai = {
    nullptr, /* collect_features */
    nullptr, /* override_features */
    nullptr, /* data_create */
    nullptr, /* data_destroy */
    rb_ot_complex_preprocess_text_thai,
    nullptr, /* postprocess_glyphs */
    RB_OT_SHAPE_NORMALIZATION_MODE_DEFAULT,
    nullptr,     /* decompose */
    nullptr,     /* compose */
    nullptr,     /* setup_masks */
    RB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE,
    false, /* fallback_position */
};

const rb_ot_complex_shaper_t _rb_ot_complex_shaper_use = {
    rb_ot_complex_collect_features_use,
    nullptr, /* override_features */
    rb_ot_complex_data_create_use,
    rb_ot_complex_data_destroy_use,
    rb_ot_complex_preprocess_text_use,
    nullptr, /* postprocess_glyphs */
    RB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT,
    nullptr, /* decompose */
    rb_ot_complex_compose_use,
    rb_ot_complex_setup_masks_use,
    RB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_EARLY,
    false, /* fallback_position */
};

const rb_ot_complex_shaper_t _rb_ot_complex_shaper_default = {
    nullptr, /* collect_features */
    nullptr, /* override_features */
    nullptr, /* data_create */
    nullptr, /* data_destroy */
    nullptr, /* preprocess_text */
    nullptr, /* postprocess_glyphs */
    RB_OT_SHAPE_NORMALIZATION_MODE_DEFAULT,
    nullptr,     /* decompose */
    nullptr,     /* compose */
    nullptr,     /* setup_masks */
    RB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE,
    true, /* fallback_position */
};

/* Same as default but no mark advance zeroing / fallback positioning.
 * Dumbest shaper ever, basically. */
const rb_ot_complex_shaper_t _rb_ot_complex_shaper_dumber = {
    nullptr, /* collect_features */
    nullptr, /* override_features */
    nullptr, /* data_create */
    nullptr, /* data_destroy */
    nullptr, /* preprocess_text */
    nullptr, /* postprocess_glyphs */
    RB_OT_SHAPE_NORMALIZATION_MODE_DEFAULT,
    nullptr,     /* decompose */
    nullptr,     /* compose */
    nullptr,     /* setup_masks */
    RB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    false, /* fallback_position */
};
