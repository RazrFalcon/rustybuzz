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

#ifndef RB_OT_SHAPE_COMPLEX_HH
#define RB_OT_SHAPE_COMPLEX_HH

#include "hb.hh"

#include "hb-ot-layout.hh"
#include "hb-ot-shape.hh"
#include "hb-ot-shape-normalize.hh"

/* buffer var allocations, used by complex shapers */
#define complex_var_u8_0() var2.u8[2]
#define complex_var_u8_1() var2.u8[3]

#define RB_OT_SHAPE_COMPLEX_MAX_COMBINING_MARKS 32

enum rb_ot_shape_zero_width_marks_type_t {
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE,
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_EARLY,
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE
};

/* Master OT shaper list */
#define RB_COMPLEX_SHAPERS_IMPLEMENT_SHAPERS                                                                           \
    RB_COMPLEX_SHAPER_IMPLEMENT(arabic)                                                                                \
    RB_COMPLEX_SHAPER_IMPLEMENT(default)                                                                               \
    RB_COMPLEX_SHAPER_IMPLEMENT(dumber)                                                                                \
    RB_COMPLEX_SHAPER_IMPLEMENT(hangul)                                                                                \
    RB_COMPLEX_SHAPER_IMPLEMENT(hebrew)                                                                                \
    RB_COMPLEX_SHAPER_IMPLEMENT(indic)                                                                                 \
    RB_COMPLEX_SHAPER_IMPLEMENT(khmer)                                                                                 \
    RB_COMPLEX_SHAPER_IMPLEMENT(myanmar)                                                                               \
    RB_COMPLEX_SHAPER_IMPLEMENT(myanmar_zawgyi)                                                                        \
    RB_COMPLEX_SHAPER_IMPLEMENT(thai)                                                                                  \
    RB_COMPLEX_SHAPER_IMPLEMENT(use)                                                                                   \
    /* ^--- Add new shapers here; keep sorted. */

typedef void (*rb_ot_reorder_marks_func_t)(const rb_ot_shape_plan_t *plan, rb_buffer_t *buffer, unsigned int start, unsigned int end);

struct rb_ot_complex_shaper_t
{
    /* collect_features()
     * Called during shape_plan().
     * Shapers should use plan->map to add their features and callbacks.
     * May be NULL.
     */
    void (*collect_features)(rb_ot_shape_planner_t *plan);

    /* override_features()
     * Called during shape_plan().
     * Shapers should use plan->map to override features and add callbacks after
     * common features are added.
     * May be NULL.
     */
    void (*override_features)(rb_ot_shape_planner_t *plan);

    /* data_create()
     * Called at the end of shape_plan().
     * Whatever shapers return will be accessible through plan->data later.
     * If nullptr is returned, means a plan failure.
     */
    void *(*data_create)(const rb_ot_shape_plan_t *plan);

    /* data_destroy()
     * Called when the shape_plan is being destroyed.
     * plan->data is passed here for destruction.
     * If nullptr is returned, means a plan failure.
     * May be NULL.
     */
    void (*data_destroy)(void *data);

    /* preprocess_text()
     * Called during shape().
     * Shapers can use to modify text before shaping starts.
     * May be NULL.
     */
    void (*preprocess_text)(const rb_ot_shape_plan_t *plan, rb_buffer_t *buffer, rb_font_t *font);

    /* postprocess_glyphs()
     * Called during shape().
     * Shapers can use to modify glyphs after shaping ends.
     * May be NULL.
     */
    void (*postprocess_glyphs)(const rb_ot_shape_plan_t *plan, rb_buffer_t *buffer, rb_font_t *font);

    rb_ot_shape_normalization_mode_t normalization_preference;

    /* decompose()
     * Called during shape()'s normalization.
     * May be NULL.
     */
    rb_ot_decompose_func_t decompose;

    /* compose()
     * Called during shape()'s normalization.
     * May be NULL.
     */
    rb_ot_compose_func_t compose;

    /* setup_masks()
     * Called during shape().
     * Shapers should use map to get feature masks and set on buffer.
     * Shapers may NOT modify characters.
     * May be NULL.
     */
    void (*setup_masks)(const rb_ot_shape_plan_t *plan, rb_buffer_t *buffer, rb_font_t *font);

    /* gpos_tag()
     * If not RB_TAG_NONE, then must match found GPOS script tag for
     * GPOS to be applied.  Otherwise, fallback positioning will be used.
     */
    rb_tag_t gpos_tag;

    /* reorder_marks()
     * Called during shape().
     * Shapers can use to modify ordering of combining marks.
     * May be NULL.
     */
    rb_ot_reorder_marks_func_t reorder_marks;

    rb_ot_shape_zero_width_marks_type_t zero_width_marks;

    bool fallback_position;
};

#define RB_COMPLEX_SHAPER_IMPLEMENT(name) extern RB_INTERNAL const rb_ot_complex_shaper_t _rb_ot_complex_shaper_##name;
RB_COMPLEX_SHAPERS_IMPLEMENT_SHAPERS
#undef RB_COMPLEX_SHAPER_IMPLEMENT

static inline const rb_ot_complex_shaper_t *rb_ot_shape_complex_categorize(const rb_ot_shape_planner_t *planner)
{
    switch ((rb_tag_t)planner->props.script) {
    default:
        return &_rb_ot_complex_shaper_default;

    /* Unicode-1.1 additions */
    case RB_SCRIPT_ARABIC:

    /* Unicode-3.0 additions */
    case RB_SCRIPT_MONGOLIAN:
    case RB_SCRIPT_SYRIAC:

    /* Unicode-5.0 additions */
    case RB_SCRIPT_NKO:
    case RB_SCRIPT_PHAGS_PA:

    /* Unicode-6.0 additions */
    case RB_SCRIPT_MANDAIC:

    /* Unicode-7.0 additions */
    case RB_SCRIPT_MANICHAEAN:
    case RB_SCRIPT_PSALTER_PAHLAVI:

    /* Unicode-9.0 additions */
    case RB_SCRIPT_ADLAM:

    /* Unicode-11.0 additions */
    case RB_SCRIPT_HANIFI_ROHINGYA:
    case RB_SCRIPT_SOGDIAN:

        /* For Arabic script, use the Arabic shaper even if no OT script tag was found.
         * This is because we do fallback shaping for Arabic script (and not others).
         * But note that Arabic shaping is applicable only to horizontal layout; for
         * vertical text, just use the generic shaper instead. */
        if ((planner->map.chosen_script[0] != RB_OT_TAG_DEFAULT_SCRIPT || planner->props.script == RB_SCRIPT_ARABIC) &&
            RB_DIRECTION_IS_HORIZONTAL(planner->props.direction))
            return &_rb_ot_complex_shaper_arabic;
        else
            return &_rb_ot_complex_shaper_default;

    /* Unicode-1.1 additions */
    case RB_SCRIPT_THAI:
    case RB_SCRIPT_LAO:

        return &_rb_ot_complex_shaper_thai;

    /* Unicode-1.1 additions */
    case RB_SCRIPT_HANGUL:

        return &_rb_ot_complex_shaper_hangul;

    /* Unicode-1.1 additions */
    case RB_SCRIPT_HEBREW:

        return &_rb_ot_complex_shaper_hebrew;

    /* Unicode-1.1 additions */
    case RB_SCRIPT_BENGALI:
    case RB_SCRIPT_DEVANAGARI:
    case RB_SCRIPT_GUJARATI:
    case RB_SCRIPT_GURMUKHI:
    case RB_SCRIPT_KANNADA:
    case RB_SCRIPT_MALAYALAM:
    case RB_SCRIPT_ORIYA:
    case RB_SCRIPT_TAMIL:
    case RB_SCRIPT_TELUGU:

    /* Unicode-3.0 additions */
    case RB_SCRIPT_SINHALA:

        /* If the designer designed the font for the 'DFLT' script,
         * (or we ended up arbitrarily pick 'latn'), use the default shaper.
         * Otherwise, use the specific shaper.
         *
         * If it's indy3 tag, send to USE. */
        if (planner->map.chosen_script[0] == RB_TAG('D', 'F', 'L', 'T') ||
            planner->map.chosen_script[0] == RB_TAG('l', 'a', 't', 'n'))
            return &_rb_ot_complex_shaper_default;
        else if ((planner->map.chosen_script[0] & 0x000000FF) == '3')
            return &_rb_ot_complex_shaper_use;
        else
            return &_rb_ot_complex_shaper_indic;

    case RB_SCRIPT_KHMER:
        return &_rb_ot_complex_shaper_khmer;

    case RB_SCRIPT_MYANMAR:
        /* If the designer designed the font for the 'DFLT' script,
         * (or we ended up arbitrarily pick 'latn'), use the default shaper.
         * Otherwise, use the specific shaper.
         *
         * If designer designed for 'mymr' tag, also send to default
         * shaper.  That's tag used from before Myanmar shaping spec
         * was developed.  The shaping spec uses 'mym2' tag. */
        if (planner->map.chosen_script[0] == RB_TAG('D', 'F', 'L', 'T') ||
            planner->map.chosen_script[0] == RB_TAG('l', 'a', 't', 'n') ||
            planner->map.chosen_script[0] == RB_TAG('m', 'y', 'm', 'r'))
            return &_rb_ot_complex_shaper_default;
        else
            return &_rb_ot_complex_shaper_myanmar;

    /* https://github.com/harfbuzz/harfbuzz/issues/1162 */
    case RB_SCRIPT_MYANMAR_ZAWGYI:

        return &_rb_ot_complex_shaper_myanmar_zawgyi;

    /* Unicode-2.0 additions */
    case RB_SCRIPT_TIBETAN:

    /* Unicode-3.0 additions */
    // case RB_SCRIPT_MONGOLIAN:
    // case RB_SCRIPT_SINHALA:

    /* Unicode-3.2 additions */
    case RB_SCRIPT_BUHID:
    case RB_SCRIPT_HANUNOO:
    case RB_SCRIPT_TAGALOG:
    case RB_SCRIPT_TAGBANWA:

    /* Unicode-4.0 additions */
    case RB_SCRIPT_LIMBU:
    case RB_SCRIPT_TAI_LE:

    /* Unicode-4.1 additions */
    case RB_SCRIPT_BUGINESE:
    case RB_SCRIPT_KHAROSHTHI:
    case RB_SCRIPT_SYLOTI_NAGRI:
    case RB_SCRIPT_TIFINAGH:

    /* Unicode-5.0 additions */
    case RB_SCRIPT_BALINESE:
    // case RB_SCRIPT_NKO:
    // case RB_SCRIPT_PHAGS_PA:

    /* Unicode-5.1 additions */
    case RB_SCRIPT_CHAM:
    case RB_SCRIPT_KAYAH_LI:
    case RB_SCRIPT_LEPCHA:
    case RB_SCRIPT_REJANG:
    case RB_SCRIPT_SAURASHTRA:
    case RB_SCRIPT_SUNDANESE:

    /* Unicode-5.2 additions */
    case RB_SCRIPT_EGYPTIAN_HIEROGLYPHS:
    case RB_SCRIPT_JAVANESE:
    case RB_SCRIPT_KAITHI:
    case RB_SCRIPT_MEETEI_MAYEK:
    case RB_SCRIPT_TAI_THAM:
    case RB_SCRIPT_TAI_VIET:

    /* Unicode-6.0 additions */
    case RB_SCRIPT_BATAK:
    case RB_SCRIPT_BRAHMI:
    // case RB_SCRIPT_MANDAIC:

    /* Unicode-6.1 additions */
    case RB_SCRIPT_CHAKMA:
    case RB_SCRIPT_SHARADA:
    case RB_SCRIPT_TAKRI:

    /* Unicode-7.0 additions */
    case RB_SCRIPT_DUPLOYAN:
    case RB_SCRIPT_GRANTHA:
    case RB_SCRIPT_KHOJKI:
    case RB_SCRIPT_KHUDAWADI:
    case RB_SCRIPT_MAHAJANI:
    // case RB_SCRIPT_MANICHAEAN:
    case RB_SCRIPT_MODI:
    case RB_SCRIPT_PAHAWH_HMONG:
    // case RB_SCRIPT_PSALTER_PAHLAVI:
    case RB_SCRIPT_SIDDHAM:
    case RB_SCRIPT_TIRHUTA:

    /* Unicode-8.0 additions */
    case RB_SCRIPT_AHOM:

    /* Unicode-9.0 additions */
    // case RB_SCRIPT_ADLAM:
    case RB_SCRIPT_BHAIKSUKI:
    case RB_SCRIPT_MARCHEN:
    case RB_SCRIPT_NEWA:

    /* Unicode-10.0 additions */
    case RB_SCRIPT_MASARAM_GONDI:
    case RB_SCRIPT_SOYOMBO:
    case RB_SCRIPT_ZANABAZAR_SQUARE:

    /* Unicode-11.0 additions */
    case RB_SCRIPT_DOGRA:
    case RB_SCRIPT_GUNJALA_GONDI:
    // case RB_SCRIPT_HANIFI_ROHINGYA:
    case RB_SCRIPT_MAKASAR:
    // case RB_SCRIPT_SOGDIAN:

    /* Unicode-12.0 additions */
    case RB_SCRIPT_NANDINAGARI:

    /* Unicode-13.0 additions */
    case RB_SCRIPT_CHORASMIAN:
    case RB_SCRIPT_DIVES_AKURU:

        /* If the designer designed the font for the 'DFLT' script,
         * (or we ended up arbitrarily pick 'latn'), use the default shaper.
         * Otherwise, use the specific shaper.
         * Note that for some simple scripts, there may not be *any*
         * GSUB/GPOS needed, so there may be no scripts found! */
        if (planner->map.chosen_script[0] == RB_TAG('D', 'F', 'L', 'T') ||
            planner->map.chosen_script[0] == RB_TAG('l', 'a', 't', 'n'))
            return &_rb_ot_complex_shaper_default;
        else
            return &_rb_ot_complex_shaper_use;
    }
}

extern "C" {
typedef struct rb_ot_complex_shaper_t rb_ot_complex_shaper_t;

RB_EXTERN rb_ot_shape_normalization_mode_t rb_ot_complex_shaper_get_normalization_preference(const rb_ot_complex_shaper_t *shaper);
RB_EXTERN rb_ot_decompose_func_t rb_ot_complex_shaper_get_decompose(const rb_ot_complex_shaper_t *shaper);
RB_EXTERN rb_ot_compose_func_t rb_ot_complex_shaper_get_compose(const rb_ot_complex_shaper_t *shaper);
RB_EXTERN rb_ot_reorder_marks_func_t rb_ot_complex_shaper_get_reorder_marks(const rb_ot_complex_shaper_t *shaper);
}

extern "C" {
typedef struct rb_ot_arabic_shape_plan_t rb_ot_arabic_shape_plan_t;

RB_EXTERN void *rb_ot_complex_data_create_arabic(const rb_ot_shape_plan_t *plan);
RB_EXTERN void rb_ot_complex_data_destroy_arabic(void *data);
RB_EXTERN void rb_ot_complex_setup_masks_arabic_plan(const rb_ot_arabic_shape_plan_t *arabic_plan,
                                                     rb_buffer_t *buffer,
                                                     rb_script_t script);
RB_EXTERN void rb_ot_complex_collect_features_arabic(rb_ot_shape_planner_t *plan);
RB_EXTERN void
rb_ot_complex_postprocess_glyphs_arabic(const rb_ot_shape_plan_t *plan, rb_buffer_t *buffer, rb_font_t *font);
RB_EXTERN void rb_ot_complex_setup_masks_arabic(const rb_ot_shape_plan_t *plan, rb_buffer_t *buffer, rb_font_t *font);
RB_EXTERN void rb_ot_complex_reorder_marks_arabic(const rb_ot_shape_plan_t *plan,
                                                  rb_buffer_t *buffer,
                                                  unsigned int start,
                                                  unsigned int end);
}

extern "C" {
RB_EXTERN void *rb_ot_complex_data_create_hangul(const rb_ot_shape_plan_t *plan);
RB_EXTERN void rb_ot_complex_data_destroy_hangul(void *data);
RB_EXTERN void rb_ot_complex_collect_features_hangul(rb_ot_shape_planner_t *plan);
RB_EXTERN void rb_ot_complex_override_features_hangul(rb_ot_shape_planner_t *plan);
RB_EXTERN void
rb_ot_complex_preprocess_text_hangul(const rb_ot_shape_plan_t *plan, rb_buffer_t *buffer, rb_font_t *font);
RB_EXTERN void rb_ot_complex_setup_masks_hangul(const rb_ot_shape_plan_t *plan, rb_buffer_t *buffer, rb_font_t *font);
}

extern "C" {
RB_EXTERN rb_bool_t rb_ot_complex_compose_hebrew(const rb_ot_shape_normalize_context_t *c,
                                                 rb_codepoint_t a,
                                                 rb_codepoint_t b,
                                                 rb_codepoint_t *ab);
}

extern "C" {
typedef struct rb_ot_indic_shape_plan_t rb_ot_indic_shape_plan_t;

RB_EXTERN void rb_ot_complex_collect_features_indic(rb_ot_shape_planner_t *plan);
RB_EXTERN void rb_ot_complex_override_features_indic(rb_ot_shape_planner_t *plan);
RB_EXTERN void *rb_ot_complex_data_create_indic(const rb_ot_shape_plan_t *plan);
RB_EXTERN void rb_ot_complex_data_destroy_indic(void *data);
RB_EXTERN void
rb_ot_complex_preprocess_text_indic(const rb_ot_shape_plan_t *plan, rb_buffer_t *buffer, rb_font_t *font);
RB_EXTERN rb_bool_t rb_ot_complex_decompose_indic(const rb_ot_shape_normalize_context_t *c,
                                                  rb_codepoint_t ab,
                                                  rb_codepoint_t *a,
                                                  rb_codepoint_t *b);
RB_EXTERN rb_bool_t rb_ot_complex_compose_indic(const rb_ot_shape_normalize_context_t *c,
                                                rb_codepoint_t a,
                                                rb_codepoint_t b,
                                                rb_codepoint_t *ab);
RB_EXTERN void rb_ot_complex_setup_masks_indic(const rb_ot_shape_plan_t *plan, rb_buffer_t *buffer, rb_font_t *font);
}

extern "C" {
RB_EXTERN void rb_ot_complex_collect_features_khmer(rb_ot_shape_planner_t *plan);
RB_EXTERN void rb_ot_complex_override_features_khmer(rb_ot_shape_planner_t *plan);
RB_EXTERN void *rb_ot_complex_data_create_khmer(const rb_ot_shape_plan_t *plan);
RB_EXTERN void rb_ot_complex_data_destroy_khmer(void *data);
RB_EXTERN rb_bool_t rb_ot_complex_decompose_khmer(const rb_ot_shape_normalize_context_t *c,
                                                  rb_codepoint_t ab,
                                                  rb_codepoint_t *a,
                                                  rb_codepoint_t *b);
RB_EXTERN rb_bool_t rb_ot_complex_compose_khmer(const rb_ot_shape_normalize_context_t *c,
                                                rb_codepoint_t a,
                                                rb_codepoint_t b,
                                                rb_codepoint_t *ab);
RB_EXTERN void rb_ot_complex_setup_masks_khmer(const rb_ot_shape_plan_t *plan, rb_buffer_t *buffer, rb_font_t *font);
}

extern "C" {
RB_EXTERN void rb_ot_complex_collect_features_myanmar(rb_ot_shape_planner_t *plan);
RB_EXTERN void rb_ot_complex_override_features_myanmar(rb_ot_shape_planner_t *plan);
RB_EXTERN void rb_ot_complex_setup_masks_myanmar(const rb_ot_shape_plan_t *plan, rb_buffer_t *buffer, rb_font_t *font);
}

extern "C" {
RB_EXTERN void rb_ot_complex_preprocess_text_thai(const rb_ot_shape_plan_t *plan, rb_buffer_t *buffer, rb_font_t *font);
}

extern "C" {
RB_EXTERN void rb_ot_complex_collect_features_use(rb_ot_shape_planner_t *plan);
RB_EXTERN void rb_clear_substitution_flags(const rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer);
RB_EXTERN void *rb_ot_complex_data_create_use(const rb_ot_shape_plan_t *plan);
RB_EXTERN void rb_ot_complex_data_destroy_use(void *data);
RB_EXTERN void rb_ot_complex_preprocess_text_use(const rb_ot_shape_plan_t *plan, rb_buffer_t *buffer, rb_font_t *font);
RB_EXTERN rb_bool_t rb_ot_complex_compose_use(const rb_ot_shape_normalize_context_t *c,
                                              rb_codepoint_t a,
                                              rb_codepoint_t b,
                                              rb_codepoint_t *ab);
RB_EXTERN void rb_ot_complex_setup_masks_use(const rb_ot_shape_plan_t *plan, rb_buffer_t *buffer, rb_font_t *font);
}

#endif /* RB_OT_SHAPE_COMPLEX_HH */
