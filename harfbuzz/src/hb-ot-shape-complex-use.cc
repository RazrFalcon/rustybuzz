/*
 * Copyright © 2015  Mozilla Foundation.
 * Copyright © 2015  Google, Inc.
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
 * Mozilla Author(s): Jonathan Kew
 * Google Author(s): Behdad Esfahbod
 */

#include "hb.hh"

#include "hb-ot-shape-complex-arabic.hh"
#include "hb-ot-shape-complex-use.hh"

/* buffer var allocations */
#define use_category() complex_var_u8_0()

extern "C" {
USE_TABLE_ELEMENT_TYPE rb_complex_universal_get_category(hb_codepoint_t u);
void rb_complex_universal_collect_features(rb_ot_map_builder_t *map);
void hb_complex_universal_setup_syllables(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer);
void hb_complex_universal_record_rphf(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer);
void hb_complex_universal_record_pref(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer);
void hb_complex_universal_reorder(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer);
void hb_complex_universal_clear_substitution_flags(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer);
void rb_complex_universal_insert_dotted_circles(const rb_ttf_parser_t *ttf_parser, rb_buffer_t *buffer);
void rb_complex_universal_find_syllables(rb_buffer_t *buffer);
void rb_complex_universal_setup_topographical_masks(rb_ot_map_t *map, rb_buffer_t *buffer);
void rb_complex_universal_reorder_syllable(unsigned int start, unsigned int end, rb_buffer_t *buffer);
}

static void collect_features_use(hb_ot_shape_planner_t *plan)
{
    rb_complex_universal_collect_features(plan->map);
}

struct use_shape_plan_t
{
    hb_mask_t rphf_mask;

    arabic_shape_plan_t *arabic_plan;
};

static bool has_arabic_joining(hb_script_t script)
{
    /* List of scripts that have data in arabic-table. */
    switch ((int)script) {
    /* Unicode-1.1 additions */
    case HB_SCRIPT_ARABIC:

    /* Unicode-3.0 additions */
    case HB_SCRIPT_MONGOLIAN:
    case HB_SCRIPT_SYRIAC:

    /* Unicode-5.0 additions */
    case HB_SCRIPT_NKO:
    case HB_SCRIPT_PHAGS_PA:

    /* Unicode-6.0 additions */
    case HB_SCRIPT_MANDAIC:

    /* Unicode-7.0 additions */
    case HB_SCRIPT_MANICHAEAN:
    case HB_SCRIPT_PSALTER_PAHLAVI:

    /* Unicode-9.0 additions */
    case HB_SCRIPT_ADLAM:

        return true;

    default:
        return false;
    }
}

static void *data_create_use(const hb_shape_plan_t *plan)
{
    use_shape_plan_t *use_plan = (use_shape_plan_t *)calloc(1, sizeof(use_shape_plan_t));
    if (unlikely(!use_plan))
        return nullptr;

    use_plan->rphf_mask = rb_ot_map_get_1_mask(plan->map, HB_TAG('r', 'p', 'h', 'f'));

    if (has_arabic_joining(plan->props.script)) {
        use_plan->arabic_plan = (arabic_shape_plan_t *)data_create_arabic(plan);
        if (unlikely(!use_plan->arabic_plan)) {
            free(use_plan);
            return nullptr;
        }
    }

    return use_plan;
}

static void data_destroy_use(void *data)
{
    use_shape_plan_t *use_plan = (use_shape_plan_t *)data;

    if (use_plan->arabic_plan)
        data_destroy_arabic(use_plan->arabic_plan);

    free(data);
}

enum use_syllable_type_t {
    use_independent_cluster,
    use_virama_terminated_cluster,
    use_sakot_terminated_cluster,
    use_standard_cluster,
    use_number_joiner_terminated_cluster,
    use_numeral_cluster,
    use_symbol_cluster,
    use_broken_cluster,
    use_non_cluster,
};

static void setup_masks_use(const hb_shape_plan_t *plan, rb_buffer_t *buffer, hb_font_t *font HB_UNUSED)
{
    const use_shape_plan_t *use_plan = (const use_shape_plan_t *)plan->data;

    /* Do this before allocating use_category(). */
    if (use_plan->arabic_plan) {
        setup_masks_arabic_plan(use_plan->arabic_plan, buffer, plan->props.script);
    }

    /* We cannot setup masks here.  We save information about characters
     * and setup masks later on in a pause-callback. */

    unsigned int count = rb_buffer_get_length(buffer);
    hb_glyph_info_t *info = rb_buffer_get_info(buffer);
    for (unsigned int i = 0; i < count; i++)
        info[i].use_category() = rb_complex_universal_get_category(info[i].codepoint);
}

static void setup_rphf_mask(const hb_shape_plan_t *plan, rb_buffer_t *buffer)
{
    const use_shape_plan_t *use_plan = (const use_shape_plan_t *)plan->data;

    hb_mask_t mask = use_plan->rphf_mask;
    if (!mask)
        return;

    hb_glyph_info_t *info = rb_buffer_get_info(buffer);

    foreach_syllable(buffer, start, end)
    {
        unsigned int limit = info[start].use_category() == USE_R ? 1 : hb_min(3u, end - start);
        for (unsigned int i = start; i < start + limit; i++)
            info[i].mask |= mask;
    }
}

static void setup_topographical_masks(const hb_shape_plan_t *plan, rb_buffer_t *buffer)
{
    const use_shape_plan_t *use_plan = (const use_shape_plan_t *)plan->data;
    if (use_plan->arabic_plan)
        return;

    rb_complex_universal_setup_topographical_masks(plan->map, buffer);
}

void hb_complex_universal_setup_syllables(const hb_shape_plan_t *plan, hb_font_t *font HB_UNUSED, rb_buffer_t *buffer)
{
    rb_complex_universal_find_syllables(buffer);
    foreach_syllable(buffer, start, end) rb_buffer_unsafe_to_break(buffer, start, end);
    setup_rphf_mask(plan, buffer);
    setup_topographical_masks(plan, buffer);
}

void hb_complex_universal_record_rphf(const hb_shape_plan_t *plan, hb_font_t *font HB_UNUSED, rb_buffer_t *buffer)
{
    const use_shape_plan_t *use_plan = (const use_shape_plan_t *)plan->data;

    hb_mask_t mask = use_plan->rphf_mask;
    if (!mask)
        return;
    hb_glyph_info_t *info = rb_buffer_get_info(buffer);

    foreach_syllable(buffer, start, end)
    {
        /* Mark a substituted repha as USE_R. */
        for (unsigned int i = start; i < end && (info[i].mask & mask); i++)
            if (_hb_glyph_info_substituted(&info[i])) {
                info[i].use_category() = USE_R;
                break;
            }
    }
}

void hb_complex_universal_record_pref(const hb_shape_plan_t *plan HB_UNUSED, hb_font_t *font HB_UNUSED, rb_buffer_t *buffer)
{
    hb_glyph_info_t *info = rb_buffer_get_info(buffer);

    foreach_syllable(buffer, start, end)
    {
        /* Mark a substituted pref as VPre, as they behave the same way. */
        for (unsigned int i = start; i < end; i++)
            if (_hb_glyph_info_substituted(&info[i])) {
                info[i].use_category() = USE_VPre;
                break;
            }
    }
}

static inline void
insert_dotted_circles_use(const hb_shape_plan_t *plan HB_UNUSED, hb_font_t *font, rb_buffer_t *buffer)
{
    rb_complex_universal_insert_dotted_circles(font->ttf_parser, buffer);
}

void hb_complex_universal_reorder(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer)
{
    insert_dotted_circles_use(plan, font, buffer);

    foreach_syllable(buffer, start, end) rb_complex_universal_reorder_syllable(start, end, buffer);
}

static void preprocess_text_use(const hb_shape_plan_t *plan, rb_buffer_t *buffer, hb_font_t *font)
{
    rb_preprocess_text_vowel_constraints(buffer);
}

static bool
compose_use(const hb_ot_shape_normalize_context_t *c, hb_codepoint_t a, hb_codepoint_t b, hb_codepoint_t *ab)
{
    /* Avoid recomposing split matras. */
    if (HB_UNICODE_GENERAL_CATEGORY_IS_MARK(rb_ucd_general_category(a)))
        return false;

    return (bool)hb_ucd_compose(a, b, ab);
}

void hb_complex_universal_clear_substitution_flags(const hb_shape_plan_t *plan HB_UNUSED, hb_font_t *font HB_UNUSED, rb_buffer_t *buffer)
{
    hb_glyph_info_t *info = rb_buffer_get_info(buffer);
    unsigned int count = rb_buffer_get_length(buffer);
    for (unsigned int i = 0; i < count; i++)
        _hb_glyph_info_clear_substituted(&info[i]);
}

const hb_ot_complex_shaper_t _hb_ot_complex_shaper_use = {
    collect_features_use,
    nullptr, /* override_features */
    data_create_use,
    data_destroy_use,
    preprocess_text_use,
    nullptr, /* postprocess_glyphs */
    HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT,
    nullptr, /* decompose */
    compose_use,
    setup_masks_use,
    HB_TAG_NONE, /* gpos_tag */
    nullptr,     /* reorder_marks */
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_EARLY,
    false, /* fallback_position */
};
