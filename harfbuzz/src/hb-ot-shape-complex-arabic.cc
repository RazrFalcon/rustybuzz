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

#include "hb-ot-shape-complex-arabic.hh"
#include "hb-ot-shape.hh"

/* buffer var allocations */
#define arabic_shaping_action() complex_var_u8_0() /* arabic shaping action */

#define HB_BUFFER_SCRATCH_FLAG_ARABIC_HAS_STCH HB_BUFFER_SCRATCH_FLAG_COMPLEX0

/*
 * Bits used in the joining tables
 */
enum hb_arabic_joining_type_t {
    JOINING_TYPE_U = 0,
    JOINING_TYPE_L = 1,
    JOINING_TYPE_R = 2,
    JOINING_TYPE_D = 3,
    JOINING_TYPE_C = JOINING_TYPE_D,
    JOINING_GROUP_ALAPH = 4,
    JOINING_GROUP_DALATH_RISH = 5,
    NUM_STATE_MACHINE_COLS = 6,

    JOINING_TYPE_T = 7,
    JOINING_TYPE_X = 8 /* means: use general-category to choose between U or T. */
};

#define FEATURE_IS_SYRIAC(tag) hb_in_range<unsigned char>((unsigned char)(tag), '2', '3')

// clang-format off
static const hb_tag_t arabic_features[] =
{
  HB_TAG('i','s','o','l'),
  HB_TAG('f','i','n','a'),
  HB_TAG('f','i','n','2'),
  HB_TAG('f','i','n','3'),
  HB_TAG('m','e','d','i'),
  HB_TAG('m','e','d','2'),
  HB_TAG('i','n','i','t'),
  HB_TAG_NONE
};
// clang-format on

/* Same order as the feature array */
enum arabic_action_t {
    ISOL,
    FINA,
    FIN2,
    FIN3,
    MEDI,
    MED2,
    INIT,

    NONE,

    ARABIC_NUM_FEATURES = NONE,

    /* We abuse the same byte for other things... */
    STCH_FIXED,
    STCH_REPEATING,
};

#include "hb-ot-shape-complex-arabic-fallback.hh"

struct arabic_shape_plan_t
{
    /* The "+ 1" in the next array is to accommodate for the "NONE" command,
     * which is not an OpenType feature, but this simplifies the code by not
     * having to do a "if (... < NONE) ..." and just rely on the fact that
     * mask_array[NONE] == 0. */
    hb_mask_t mask_array[ARABIC_NUM_FEATURES + 1];

    hb_atomic_ptr_t<arabic_fallback_plan_t> fallback_plan;

    unsigned int do_fallback : 1;
    unsigned int has_stch : 1;
};

void *hb_complex_arabic_data_create(const hb_shape_plan_t *plan)
{
    arabic_shape_plan_t *arabic_plan = (arabic_shape_plan_t *)calloc(1, sizeof(arabic_shape_plan_t));
    if (unlikely(!arabic_plan))
        return nullptr;

    arabic_plan->do_fallback = plan->props.script == HB_SCRIPT_ARABIC;
    arabic_plan->has_stch = !!rb_ot_map_get_1_mask(plan->map, HB_TAG('s', 't', 'c', 'h'));
    for (unsigned int i = 0; i < ARABIC_NUM_FEATURES; i++) {
        arabic_plan->mask_array[i] = rb_ot_map_get_1_mask(plan->map, arabic_features[i]);
        arabic_plan->do_fallback =
            arabic_plan->do_fallback &&
            (FEATURE_IS_SYRIAC(arabic_features[i]) || rb_ot_map_needs_fallback(plan->map, arabic_features[i]));
    }

    return arabic_plan;
}

void hb_complex_arabic_data_destroy(void *data)
{
    arabic_shape_plan_t *arabic_plan = (arabic_shape_plan_t *)data;

    arabic_fallback_plan_destroy(arabic_plan->fallback_plan);

    free(data);
}

static void mongolian_variation_selectors(rb_buffer_t *buffer)
{
    /* Copy arabic_shaping_action() from base to Mongolian variation selectors. */
    unsigned int count = rb_buffer_get_length(buffer);
    hb_glyph_info_t *info = rb_buffer_get_info(buffer);
    for (unsigned int i = 1; i < count; i++)
        if (unlikely(hb_in_range<hb_codepoint_t>(info[i].codepoint, 0x180Bu, 0x180Du)))
            info[i].arabic_shaping_action() = info[i - 1].arabic_shaping_action();
}

void setup_masks_arabic_plan(const arabic_shape_plan_t *arabic_plan, rb_buffer_t *buffer, hb_script_t script)
{
    rb_complex_arabic_joining(buffer);
    if (script == HB_SCRIPT_MONGOLIAN)
        mongolian_variation_selectors(buffer);

    unsigned int count = rb_buffer_get_length(buffer);
    hb_glyph_info_t *info = rb_buffer_get_info(buffer);
    for (unsigned int i = 0; i < count; i++) {
        info[i].mask |= arabic_plan->mask_array[info[i].arabic_shaping_action()];
    }
}

void hb_complex_arabic_setup_masks(const hb_shape_plan_t *plan, rb_buffer_t *buffer, hb_font_t *font HB_UNUSED)
{
    const arabic_shape_plan_t *arabic_plan = (const arabic_shape_plan_t *)plan->data;
    setup_masks_arabic_plan(arabic_plan, buffer, plan->props.script);
}

void hb_complex_arabic_fallback_shape(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer)
{
    const arabic_shape_plan_t *arabic_plan = (const arabic_shape_plan_t *)plan->data;

    if (!arabic_plan->do_fallback)
        return;

retry:
    arabic_fallback_plan_t *fallback_plan = arabic_plan->fallback_plan;
    if (unlikely(!fallback_plan)) {
        /* This sucks.  We need a font to build the fallback plan... */
        fallback_plan = arabic_fallback_plan_create(plan, font);
        if (unlikely(!arabic_plan->fallback_plan.cmpexch(nullptr, fallback_plan))) {
            arabic_fallback_plan_destroy(fallback_plan);
            goto retry;
        }
    }

    arabic_fallback_plan_shape(fallback_plan, font, buffer);
}

/*
 * Stretch feature: "stch".
 * See example here:
 * https://docs.microsoft.com/en-us/typography/script-development/syriac
 * We implement this in a generic way, such that the Arabic subtending
 * marks can use it as well.
 */

void hb_complex_arabic_record_stch(const hb_shape_plan_t *plan, hb_font_t *font HB_UNUSED, rb_buffer_t *buffer)
{
    const arabic_shape_plan_t *arabic_plan = (const arabic_shape_plan_t *)plan->data;
    if (!arabic_plan->has_stch)
        return;

    /* 'stch' feature was just applied.  Look for anything that multiplied,
     * and record it for stch treatment later.  Note that rtlm, frac, etc
     * are applied before stch, but we assume that they didn't result in
     * anything multiplying into 5 pieces, so it's safe-ish... */

    unsigned int count = rb_buffer_get_length(buffer);
    hb_glyph_info_t *info = rb_buffer_get_info(buffer);
    for (unsigned int i = 0; i < count; i++)
        if (unlikely(_hb_glyph_info_multiplied(&info[i]))) {
            unsigned int comp = _hb_glyph_info_get_lig_comp(&info[i]);
            info[i].arabic_shaping_action() = comp % 2 ? STCH_REPEATING : STCH_FIXED;
            *rb_buffer_get_scratch_flags(buffer) |= HB_BUFFER_SCRATCH_FLAG_ARABIC_HAS_STCH;
        }
}
