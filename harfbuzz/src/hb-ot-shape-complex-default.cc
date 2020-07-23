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

void hb_clear_substitution_flags(const hb_ot_shape_plan_t *plan HB_UNUSED,
                                 hb_font_t *font HB_UNUSED,
                                 hb_buffer_t *buffer)
{
    hb_glyph_info_t *info = hb_buffer_get_glyph_infos(buffer);
    unsigned int count = hb_buffer_get_length(buffer);
    for (unsigned int i = 0; i < count; i++)
        _hb_glyph_info_clear_substituted(&info[i]);
}

const hb_ot_complex_shaper_t _hb_ot_complex_shaper_arabic = {
    hb_ot_complex_collect_features_arabic,
    nullptr, /* override_features */
    hb_ot_complex_data_create_arabic,
    hb_ot_complex_data_destroy_arabic,
    nullptr, /* preprocess_text */
    hb_ot_complex_postprocess_glyphs_arabic,
    HB_OT_SHAPE_NORMALIZATION_MODE_DEFAULT,
    nullptr, /* decompose */
    nullptr, /* compose */
    hb_ot_complex_setup_masks_arabic,
    HB_TAG_NONE, /* gpos_tag */
    hb_ot_complex_reorder_marks_arabic,
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE,
    true, /* fallback_position */
};

const hb_ot_complex_shaper_t _hb_ot_complex_shaper_hangul = {
    hb_ot_complex_collect_features_hangul,
    hb_ot_complex_override_features_hangul,
    hb_ot_complex_data_create_hangul,
    hb_ot_complex_data_destroy_hangul,
    hb_ot_complex_preprocess_text_hangul,
    nullptr, /* postprocess_glyphs */
    HB_OT_SHAPE_NORMALIZATION_MODE_NONE,
    nullptr, /* decompose */
    nullptr, /* compose */
    hb_ot_complex_setup_masks_hangul,
    HB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    false, /* fallback_position */
};

const hb_ot_complex_shaper_t _hb_ot_complex_shaper_hebrew = {
    nullptr, /* collect_features */
    nullptr, /* override_features */
    nullptr, /* data_create */
    nullptr, /* data_destroy */
    nullptr, /* preprocess_text */
    nullptr, /* postprocess_glyphs */
    HB_OT_SHAPE_NORMALIZATION_MODE_DEFAULT,
    nullptr, /* decompose */
    hb_ot_complex_compose_hebrew,
    nullptr,                    /* setup_masks */
    HB_TAG('h', 'e', 'b', 'r'), /* gpos_tag. https://github.com/harfbuzz/harfbuzz/issues/347#issuecomment-267838368 */
    nullptr,                    /* reorder_marks */
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE,
    true, /* fallback_position */
};

const hb_ot_complex_shaper_t _hb_ot_complex_shaper_indic = {
    hb_ot_complex_collect_features_indic,
    hb_ot_complex_override_features_indic,
    hb_ot_complex_data_create_indic,
    hb_ot_complex_data_destroy_indic,
    hb_ot_complex_preprocess_text_indic,
    nullptr, /* postprocess_glyphs */
    HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT,
    hb_ot_complex_decompose_indic,
    hb_ot_complex_compose_indic,
    hb_ot_complex_setup_masks_indic,
    HB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    false, /* fallback_position */
};

const hb_ot_complex_shaper_t _hb_ot_complex_shaper_khmer = {
    hb_ot_complex_collect_features_khmer,
    hb_ot_complex_override_features_khmer,
    hb_ot_complex_data_create_khmer,
    hb_ot_complex_data_destroy_khmer,
    nullptr, /* preprocess_text */
    nullptr, /* postprocess_glyphs */
    HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT,
    hb_ot_complex_decompose_khmer,
    hb_ot_complex_compose_khmer,
    hb_ot_complex_setup_masks_khmer,
    HB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    false, /* fallback_position */
};

const hb_ot_complex_shaper_t _hb_ot_complex_shaper_myanmar = {
    hb_ot_complex_collect_features_myanmar,
    hb_ot_complex_override_features_myanmar,
    nullptr, /* data_create */
    nullptr, /* data_destroy */
    nullptr, /* preprocess_text */
    nullptr, /* postprocess_glyphs */
    HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT,
    nullptr, /* decompose */
    nullptr, /* compose */
    hb_ot_complex_setup_masks_myanmar,
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

const hb_ot_complex_shaper_t _hb_ot_complex_shaper_thai = {
    nullptr, /* collect_features */
    nullptr, /* override_features */
    nullptr, /* data_create */
    nullptr, /* data_destroy */
    hb_ot_complex_preprocess_text_thai,
    nullptr, /* postprocess_glyphs */
    HB_OT_SHAPE_NORMALIZATION_MODE_DEFAULT,
    nullptr,     /* decompose */
    nullptr,     /* compose */
    nullptr,     /* setup_masks */
    HB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE,
    false, /* fallback_position */
};

const hb_ot_complex_shaper_t _hb_ot_complex_shaper_use = {
    hb_ot_complex_collect_features_use,
    nullptr, /* override_features */
    hb_ot_complex_data_create_use,
    hb_ot_complex_data_destroy_use,
    hb_ot_complex_preprocess_text_use,
    nullptr, /* postprocess_glyphs */
    HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT,
    nullptr, /* decompose */
    hb_ot_complex_compose_use,
    hb_ot_complex_setup_masks_use,
    HB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_EARLY,
    false, /* fallback_position */
};

const hb_ot_complex_shaper_t _hb_ot_complex_shaper_default = {
    nullptr, /* collect_features */
    nullptr, /* override_features */
    nullptr, /* data_create */
    nullptr, /* data_destroy */
    nullptr, /* preprocess_text */
    nullptr, /* postprocess_glyphs */
    HB_OT_SHAPE_NORMALIZATION_MODE_DEFAULT,
    nullptr,     /* decompose */
    nullptr,     /* compose */
    nullptr,     /* setup_masks */
    HB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE,
    true, /* fallback_position */
};

/* Same as default but no mark advance zeroing / fallback positioning.
 * Dumbest shaper ever, basically. */
const hb_ot_complex_shaper_t _hb_ot_complex_shaper_dumber = {
    nullptr, /* collect_features */
    nullptr, /* override_features */
    nullptr, /* data_create */
    nullptr, /* data_destroy */
    nullptr, /* preprocess_text */
    nullptr, /* postprocess_glyphs */
    HB_OT_SHAPE_NORMALIZATION_MODE_DEFAULT,
    nullptr,     /* decompose */
    nullptr,     /* compose */
    nullptr,     /* setup_masks */
    HB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    false, /* fallback_position */
};
