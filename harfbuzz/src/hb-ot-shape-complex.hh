/*
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
 * Google Author(s): Behdad Esfahbod
 */

#pragma once

#include "hb.hh"

#include "hb-ot-layout.hh"
#include "hb-ot-shape-normalize.hh"
#include "hb-ot-shape.hh"

/* buffer var allocations, used by complex shapers */
#define complex_var_u8_0() var2.u8[2]
#define complex_var_u8_1() var2.u8[3]

#define HB_OT_SHAPE_COMPLEX_MAX_COMBINING_MARKS 32

extern "C" {
void rb_preprocess_text_vowel_constraints(rb_buffer_t *buffer);
}

enum hb_ot_shape_zero_width_marks_type_t {
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_EARLY,
    HB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE
};

typedef struct hb_ot_complex_shaper_t hb_ot_complex_shaper_t;

extern "C" {
void hb_ot_complex_shaper_collect_features(const hb_ot_complex_shaper_t *shaper, hb_ot_shape_planner_t *planner);
void hb_ot_complex_shaper_override_features(const hb_ot_complex_shaper_t *shaper, hb_ot_shape_planner_t *planner);
void *hb_ot_complex_shaper_data_create(const hb_ot_complex_shaper_t *shaper, hb_shape_plan_t *plan);
void hb_ot_complex_shaper_data_destroy(const hb_ot_complex_shaper_t *shaper, void *data);
void hb_ot_complex_shaper_preprocess_text(const hb_ot_complex_shaper_t *shaper,
                                          const hb_shape_plan_t *plan,
                                          rb_buffer_t *buffer,
                                          hb_font_t *font);
void hb_ot_complex_shaper_postprocess_glyphs(const hb_ot_complex_shaper_t *shaper,
                                             const hb_shape_plan_t *plan,
                                             rb_buffer_t *buffer,
                                             hb_font_t *font);
hb_ot_shape_normalization_mode_t hb_ot_complex_shaper_normalization_preference(const hb_ot_complex_shaper_t *shaper);
bool hb_ot_complex_shaper_decompose(const hb_ot_complex_shaper_t *shaper,
                                    const hb_ot_shape_normalize_context_t *c,
                                    hb_codepoint_t ab,
                                    hb_codepoint_t *a,
                                    hb_codepoint_t *b);
bool hb_ot_complex_shaper_compose(const hb_ot_complex_shaper_t *shaper,
                                  const hb_ot_shape_normalize_context_t *c,
                                  hb_codepoint_t a,
                                  hb_codepoint_t b,
                                  hb_codepoint_t *ab);
void hb_ot_complex_shaper_setup_masks(const hb_ot_complex_shaper_t *shaper,
                                      const hb_shape_plan_t *plan,
                                      rb_buffer_t *buffer,
                                      hb_font_t *font);
hb_tag_t hb_ot_complex_shaper_gpos_tag(const hb_ot_complex_shaper_t *shaper);
void hb_ot_complex_shaper_reorder_marks(const hb_ot_complex_shaper_t *shaper,
                                        const hb_shape_plan_t *plan,
                                        rb_buffer_t *buffer,
                                        unsigned int start,
                                        unsigned int end);
hb_ot_shape_zero_width_marks_type_t hb_ot_complex_shaper_zero_width_marks(const hb_ot_complex_shaper_t *shaper);
bool hb_ot_complex_shaper_fallback_position(const hb_ot_complex_shaper_t *shaper);
}

extern "C" {
hb_ot_complex_shaper_t *rb_create_default_shaper();
hb_ot_complex_shaper_t *rb_create_arabic_shaper();
hb_ot_complex_shaper_t *rb_create_hangul_shaper();
hb_ot_complex_shaper_t *rb_create_hebrew_shaper();
hb_ot_complex_shaper_t *rb_create_indic_shaper();
hb_ot_complex_shaper_t *rb_create_khmer_shaper();
hb_ot_complex_shaper_t *rb_create_myanmar_shaper();
hb_ot_complex_shaper_t *rb_create_myanmar_zawgyi_shaper();
hb_ot_complex_shaper_t *rb_create_thai_shaper();
hb_ot_complex_shaper_t *rb_create_use_shaper();
}

inline const hb_ot_complex_shaper_t *hb_ot_shape_complex_categorize(const hb_ot_shape_planner_t *planner)
{
    switch ((hb_tag_t)planner->props.script) {
    default:
        return rb_create_default_shaper();

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

    /* Unicode-11.0 additions */
    case HB_SCRIPT_HANIFI_ROHINGYA:
    case HB_SCRIPT_SOGDIAN:

        /* For Arabic script, use the Arabic shaper even if no OT script tag was found.
         * This is because we do fallback shaping for Arabic script (and not others).
         * But note that Arabic shaping is applicable only to horizontal layout; for
         * vertical text, just use the generic shaper instead. */
        if ((rb_ot_map_builder_chosen_script(planner->map, 0) != HB_OT_TAG_DEFAULT_SCRIPT ||
             planner->props.script == HB_SCRIPT_ARABIC) &&
            HB_DIRECTION_IS_HORIZONTAL(planner->props.direction))
            return rb_create_arabic_shaper();
        else
            return rb_create_default_shaper();

    /* Unicode-1.1 additions */
    case HB_SCRIPT_THAI:
    case HB_SCRIPT_LAO:

        return rb_create_thai_shaper();

    /* Unicode-1.1 additions */
    case HB_SCRIPT_HANGUL:

        return rb_create_hangul_shaper();

    /* Unicode-1.1 additions */
    case HB_SCRIPT_HEBREW:

        return rb_create_hebrew_shaper();

    /* Unicode-1.1 additions */
    case HB_SCRIPT_BENGALI:
    case HB_SCRIPT_DEVANAGARI:
    case HB_SCRIPT_GUJARATI:
    case HB_SCRIPT_GURMUKHI:
    case HB_SCRIPT_KANNADA:
    case HB_SCRIPT_MALAYALAM:
    case HB_SCRIPT_ORIYA:
    case HB_SCRIPT_TAMIL:
    case HB_SCRIPT_TELUGU:

    /* Unicode-3.0 additions */
    case HB_SCRIPT_SINHALA:

        /* If the designer designed the font for the 'DFLT' script,
         * (or we ended up arbitrarily pick 'latn'), use the default shaper.
         * Otherwise, use the specific shaper.
         *
         * If it's indy3 tag, send to USE. */
        if (rb_ot_map_builder_chosen_script(planner->map, 0) == HB_TAG('D', 'F', 'L', 'T') ||
            rb_ot_map_builder_chosen_script(planner->map, 0) == HB_TAG('l', 'a', 't', 'n'))
            return rb_create_default_shaper();
        else if ((rb_ot_map_builder_chosen_script(planner->map, 0) & 0x000000FF) == '3')
            return rb_create_use_shaper();
        else
            return rb_create_indic_shaper();

    case HB_SCRIPT_KHMER:
        return rb_create_khmer_shaper();

    case HB_SCRIPT_MYANMAR:
        /* If the designer designed the font for the 'DFLT' script,
         * (or we ended up arbitrarily pick 'latn'), use the default shaper.
         * Otherwise, use the specific shaper.
         *
         * If designer designed for 'mymr' tag, also send to default
         * shaper.  That's tag used from before Myanmar shaping spec
         * was developed.  The shaping spec uses 'mym2' tag. */
        if (rb_ot_map_builder_chosen_script(planner->map, 0) == HB_TAG('D', 'F', 'L', 'T') ||
            rb_ot_map_builder_chosen_script(planner->map, 0) == HB_TAG('l', 'a', 't', 'n') ||
            rb_ot_map_builder_chosen_script(planner->map, 0) == HB_TAG('m', 'y', 'm', 'r'))
            return rb_create_default_shaper();
        else
            return rb_create_myanmar_shaper();

    /* https://github.com/harfbuzz/harfbuzz/issues/1162 */
    case HB_SCRIPT_MYANMAR_ZAWGYI:

        return rb_create_myanmar_zawgyi_shaper();

    /* Unicode-2.0 additions */
    case HB_SCRIPT_TIBETAN:

    /* Unicode-3.0 additions */
    // case HB_SCRIPT_MONGOLIAN:
    // case HB_SCRIPT_SINHALA:

    /* Unicode-3.2 additions */
    case HB_SCRIPT_BUHID:
    case HB_SCRIPT_HANUNOO:
    case HB_SCRIPT_TAGALOG:
    case HB_SCRIPT_TAGBANWA:

    /* Unicode-4.0 additions */
    case HB_SCRIPT_LIMBU:
    case HB_SCRIPT_TAI_LE:

    /* Unicode-4.1 additions */
    case HB_SCRIPT_BUGINESE:
    case HB_SCRIPT_KHAROSHTHI:
    case HB_SCRIPT_SYLOTI_NAGRI:
    case HB_SCRIPT_TIFINAGH:

    /* Unicode-5.0 additions */
    case HB_SCRIPT_BALINESE:
    // case HB_SCRIPT_NKO:
    // case HB_SCRIPT_PHAGS_PA:

    /* Unicode-5.1 additions */
    case HB_SCRIPT_CHAM:
    case HB_SCRIPT_KAYAH_LI:
    case HB_SCRIPT_LEPCHA:
    case HB_SCRIPT_REJANG:
    case HB_SCRIPT_SAURASHTRA:
    case HB_SCRIPT_SUNDANESE:

    /* Unicode-5.2 additions */
    case HB_SCRIPT_EGYPTIAN_HIEROGLYPHS:
    case HB_SCRIPT_JAVANESE:
    case HB_SCRIPT_KAITHI:
    case HB_SCRIPT_MEETEI_MAYEK:
    case HB_SCRIPT_TAI_THAM:
    case HB_SCRIPT_TAI_VIET:

    /* Unicode-6.0 additions */
    case HB_SCRIPT_BATAK:
    case HB_SCRIPT_BRAHMI:
    // case HB_SCRIPT_MANDAIC:

    /* Unicode-6.1 additions */
    case HB_SCRIPT_CHAKMA:
    case HB_SCRIPT_SHARADA:
    case HB_SCRIPT_TAKRI:

    /* Unicode-7.0 additions */
    case HB_SCRIPT_DUPLOYAN:
    case HB_SCRIPT_GRANTHA:
    case HB_SCRIPT_KHOJKI:
    case HB_SCRIPT_KHUDAWADI:
    case HB_SCRIPT_MAHAJANI:
    // case HB_SCRIPT_MANICHAEAN:
    case HB_SCRIPT_MODI:
    case HB_SCRIPT_PAHAWH_HMONG:
    // case HB_SCRIPT_PSALTER_PAHLAVI:
    case HB_SCRIPT_SIDDHAM:
    case HB_SCRIPT_TIRHUTA:

    /* Unicode-8.0 additions */
    case HB_SCRIPT_AHOM:

    /* Unicode-9.0 additions */
    // case HB_SCRIPT_ADLAM:
    case HB_SCRIPT_BHAIKSUKI:
    case HB_SCRIPT_MARCHEN:
    case HB_SCRIPT_NEWA:

    /* Unicode-10.0 additions */
    case HB_SCRIPT_MASARAM_GONDI:
    case HB_SCRIPT_SOYOMBO:
    case HB_SCRIPT_ZANABAZAR_SQUARE:

    /* Unicode-11.0 additions */
    case HB_SCRIPT_DOGRA:
    case HB_SCRIPT_GUNJALA_GONDI:
    // case HB_SCRIPT_HANIFI_ROHINGYA:
    case HB_SCRIPT_MAKASAR:
    // case HB_SCRIPT_SOGDIAN:

    /* Unicode-12.0 additions */
    case HB_SCRIPT_NANDINAGARI:

        /* If the designer designed the font for the 'DFLT' script,
         * (or we ended up arbitrarily pick 'latn'), use the default shaper.
         * Otherwise, use the specific shaper.
         * Note that for some simple scripts, there may not be *any*
         * GSUB/GPOS needed, so there may be no scripts found! */
        if (rb_ot_map_builder_chosen_script(planner->map, 0) == HB_TAG('D', 'F', 'L', 'T') ||
            rb_ot_map_builder_chosen_script(planner->map, 0) == HB_TAG('l', 'a', 't', 'n'))
            return rb_create_default_shaper();
        else
            return rb_create_use_shaper();
    }
}
