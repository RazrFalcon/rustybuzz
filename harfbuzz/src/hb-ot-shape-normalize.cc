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

#include "hb-ot-shape-complex.hh"
#include "hb-ot-shape-normalize.hh"
#include "hb-ot-shape.hh"

/*
 * HIGHLEVEL DESIGN:
 *
 * This file exports one main function: _hb_ot_shape_normalize().
 *
 * This function closely reflects the Unicode Normalization Algorithm,
 * yet it's different.
 *
 * Each shaper specifies whether it prefers decomposed (NFD) or composed (NFC).
 * The logic however tries to use whatever the font can support.
 *
 * In general what happens is that: each grapheme is decomposed in a chain
 * of 1:2 decompositions, marks reordered, and then recomposed if desired,
 * so far it's like Unicode Normalization.  However, the decomposition and
 * recomposition only happens if the font supports the resulting characters.
 *
 * The goals are:
 *
 *   - Try to render all canonically equivalent strings similarly.  To really
 *     achieve this we have to always do the full decomposition and then
 *     selectively recompose from there.  It's kinda too expensive though, so
 *     we skip some cases.  For example, if composed is desired, we simply
 *     don't touch 1-character clusters that are supported by the font, even
 *     though their NFC may be different.
 *
 *   - When a font has a precomposed character for a sequence but the 'ccmp'
 *     feature in the font is not adequate, use the precomposed character
 *     which typically has better mark positioning.
 *
 *   - When a font does not support a combining mark, but supports it precomposed
 *     with previous base, use that.  This needs the itemizer to have this
 *     knowledge too.  We need to provide assistance to the itemizer.
 *
 *   - When a font does not support a character but supports its canonical
 *     decomposition, well, use the decomposition.
 *
 *   - The complex shapers can customize the compose and decompose functions to
 *     offload some of their requirements to the normalizer.  For example, the
 *     Indic shaper may want to disallow recomposing of two matras.
 */

static bool
decompose_unicode(const hb_ot_shape_normalize_context_t *c, hb_codepoint_t ab, hb_codepoint_t *a, hb_codepoint_t *b)
{
    return (bool)hb_ucd_decompose(ab, a, b);
}

static bool
compose_unicode(const hb_ot_shape_normalize_context_t *c, hb_codepoint_t a, hb_codepoint_t b, hb_codepoint_t *ab)
{
    return (bool)hb_ucd_compose(a, b, ab);
}

static inline void set_glyph(hb_glyph_info_t &info, hb_font_t *font)
{
    (void)font->get_nominal_glyph(info.codepoint, &info.glyph_index());
}

static inline void output_char(rb_buffer_t *buffer, hb_codepoint_t unichar, hb_codepoint_t glyph)
{
    rb_buffer_get_cur(buffer, 0)->glyph_index() = glyph;
    rb_buffer_output_glyph(buffer, unichar); /* This is very confusing indeed. */
    _hb_glyph_info_set_unicode_props(rb_buffer_get_prev(buffer), buffer);
}

static inline void next_char(rb_buffer_t *buffer, hb_codepoint_t glyph)
{
    rb_buffer_get_cur(buffer, 0)->glyph_index() = glyph;
    rb_buffer_next_glyph(buffer);
}

static inline void skip_char(rb_buffer_t *buffer)
{
    rb_buffer_skip_glyph(buffer);
}

/* Returns 0 if didn't decompose, number of resulting characters otherwise. */
static inline unsigned int decompose(const hb_ot_shape_normalize_context_t *c, bool shortest, hb_codepoint_t ab)
{
    hb_codepoint_t a = 0, b = 0, a_glyph = 0, b_glyph = 0;
    rb_buffer_t *const buffer = c->buffer;
    hb_font_t *const font = c->font;

    if (!c->decompose(c, ab, &a, &b) || (b && !font->get_nominal_glyph(b, &b_glyph)))
        return 0;

    bool has_a = (bool)font->get_nominal_glyph(a, &a_glyph);
    if (shortest && has_a) {
        /* Output a and b */
        output_char(buffer, a, a_glyph);
        if (likely(b)) {
            output_char(buffer, b, b_glyph);
            return 2;
        }
        return 1;
    }

    unsigned int ret;
    if ((ret = decompose(c, shortest, a))) {
        if (b) {
            output_char(buffer, b, b_glyph);
            return ret + 1;
        }
        return ret;
    }

    if (has_a) {
        output_char(buffer, a, a_glyph);
        if (likely(b)) {
            output_char(buffer, b, b_glyph);
            return 2;
        }
        return 1;
    }

    return 0;
}

static inline void decompose_current_character(const hb_ot_shape_normalize_context_t *c, bool shortest)
{
    rb_buffer_t *const buffer = c->buffer;
    hb_codepoint_t u = rb_buffer_get_cur(buffer, 0)->codepoint;
    hb_codepoint_t glyph = 0;

    if (shortest && c->font->get_nominal_glyph(u, &glyph)) {
        next_char(buffer, glyph);
        return;
    }

    if (decompose(c, shortest, u)) {
        skip_char(buffer);
        return;
    }

    if (!shortest && c->font->get_nominal_glyph(u, &glyph)) {
        next_char(buffer, glyph);
        return;
    }

    if (_hb_glyph_info_is_unicode_space(rb_buffer_get_cur(buffer, 0))) {
        hb_codepoint_t space_glyph;
        space_t space_type = rb_ucd_space_fallback_type(u);
        if (space_type != NOT_SPACE && c->font->get_nominal_glyph(0x0020u, &space_glyph)) {
            _hb_glyph_info_set_unicode_space_fallback_type(rb_buffer_get_cur(buffer, 0), space_type);
            next_char(buffer, space_glyph);
            *rb_buffer_get_scratch_flags(buffer) |= HB_BUFFER_SCRATCH_FLAG_HAS_SPACE_FALLBACK;
            return;
        }
    }

    if (u == 0x2011u) {
        /* U+2011 is the only sensible character that is a no-break version of another character
         * and not a space.  The space ones are handled already.  Handle this lone one. */
        hb_codepoint_t other_glyph;
        if (c->font->get_nominal_glyph(0x2010u, &other_glyph)) {
            next_char(buffer, other_glyph);
            return;
        }
    }

    next_char(buffer, glyph); /* glyph is initialized in earlier branches. */
}

static inline void handle_variation_selector_cluster(const hb_ot_shape_normalize_context_t *c,
                                                     unsigned int end,
                                                     bool short_circuit HB_UNUSED)
{
    /* TODO Currently if there's a variation-selector we give-up, it's just too hard. */
    rb_buffer_t *const buffer = c->buffer;
    hb_font_t *const font = c->font;
    for (; rb_buffer_get_idx(buffer) < end - 1;) {
        if (unlikely(hb_ucd_is_variation_selector(rb_buffer_get_cur(buffer, 1)->codepoint))) {
            if (font->get_variation_glyph(rb_buffer_get_cur(buffer, 0)->codepoint,
                                          rb_buffer_get_cur(buffer, 1)->codepoint,
                                          &rb_buffer_get_cur(buffer, 0)->glyph_index())) {
                hb_codepoint_t unicode = rb_buffer_get_cur(buffer, 0)->codepoint;
                rb_buffer_replace_glyphs(buffer, 2, 1, &unicode);
            } else {
                /* Just pass on the two characters separately, let GSUB do its magic. */
                set_glyph(*rb_buffer_get_cur(buffer, 0), font);
                rb_buffer_next_glyph(buffer);
                set_glyph(*rb_buffer_get_cur(buffer, 0), font);
                rb_buffer_next_glyph(buffer);
            }
            /* Skip any further variation selectors. */
            while (rb_buffer_get_idx(buffer) < end &&
                   unlikely(hb_ucd_is_variation_selector(rb_buffer_get_cur(buffer, 0)->codepoint))) {
                set_glyph(*rb_buffer_get_cur(buffer, 0), font);
                rb_buffer_next_glyph(buffer);
            }
        } else {
            set_glyph(*rb_buffer_get_cur(buffer, 0), font);
            rb_buffer_next_glyph(buffer);
        }
    }
    if (likely(rb_buffer_get_idx(buffer) < end)) {
        set_glyph(*rb_buffer_get_cur(buffer, 0), font);
        rb_buffer_next_glyph(buffer);
    }
}

static inline void
decompose_multi_char_cluster(const hb_ot_shape_normalize_context_t *c, unsigned int end, bool short_circuit)
{
    rb_buffer_t *const buffer = c->buffer;
    for (unsigned int i = rb_buffer_get_idx(buffer); i < end; i++)
        if (unlikely(hb_ucd_is_variation_selector(rb_buffer_get_info(buffer)[i].codepoint))) {
            handle_variation_selector_cluster(c, end, short_circuit);
            return;
        }

    while (rb_buffer_get_idx(buffer) < end)
        decompose_current_character(c, short_circuit);
}

static int compare_combining_class(const hb_glyph_info_t *pa, const hb_glyph_info_t *pb)
{
    unsigned int a = _hb_glyph_info_get_modified_combining_class(pa);
    unsigned int b = _hb_glyph_info_get_modified_combining_class(pb);

    return a < b ? -1 : a == b ? 0 : +1;
}

void _hb_ot_shape_normalize(const hb_shape_plan_t *plan, rb_buffer_t *buffer, hb_font_t *font)
{
    if (unlikely(!rb_buffer_get_length(buffer)))
        return;

    hb_ot_shape_normalization_mode_t mode = plan->shaper->normalization_preference;
    if (mode == HB_OT_SHAPE_NORMALIZATION_MODE_AUTO) {
        if (plan->has_gpos_mark)
            // https://github.com/harfbuzz/harfbuzz/issues/653#issuecomment-423905920
            // mode = HB_OT_SHAPE_NORMALIZATION_MODE_DECOMPOSED;
            mode = HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS;
        else
            mode = HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS;
    }

    const hb_ot_shape_normalize_context_t c = {plan,
                                               buffer,
                                               font,
                                               plan->shaper->decompose ? plan->shaper->decompose : decompose_unicode,
                                               plan->shaper->compose ? plan->shaper->compose : compose_unicode};

    bool always_short_circuit = mode == HB_OT_SHAPE_NORMALIZATION_MODE_NONE;
    bool might_short_circuit =
        always_short_circuit || (mode != HB_OT_SHAPE_NORMALIZATION_MODE_DECOMPOSED &&
                                 mode != HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT);
    unsigned int count;

    /* We do a fairly straightforward yet custom normalization process in three
     * separate rounds: decompose, reorder, recompose (if desired).  Currently
     * this makes two buffer swaps.  We can make it faster by moving the last
     * two rounds into the inner loop for the first round, but it's more readable
     * this way. */

    /* First round, decompose */

    bool all_simple = true;
    {
        rb_buffer_clear_output(buffer);
        count = rb_buffer_get_length(buffer);
        rb_buffer_set_idx(buffer, 0);
        do {
            unsigned int end;
            for (end = rb_buffer_get_idx(buffer) + 1; end < count; end++)
                if (unlikely(HB_UNICODE_GENERAL_CATEGORY_IS_MARK(
                        _hb_glyph_info_get_general_category(&rb_buffer_get_info(buffer)[end]))))
                    break;

            if (end < count)
                end--; /* Leave one base for the marks to cluster with. */

            /* From idx to end are simple clusters. */
            if (might_short_circuit) {
                unsigned int done = font->get_nominal_glyphs(end - rb_buffer_get_idx(buffer),
                                                             &rb_buffer_get_cur(buffer, 0)->codepoint,
                                                             sizeof(rb_buffer_get_info(buffer)[0]),
                                                             &rb_buffer_get_cur(buffer, 0)->glyph_index(),
                                                             sizeof(rb_buffer_get_info(buffer)[0]));
                rb_buffer_next_glyphs(buffer, done);
            }
            while (rb_buffer_get_idx(buffer) < end)
                decompose_current_character(&c, might_short_circuit);

            if (rb_buffer_get_idx(buffer) == count)
                break;

            all_simple = false;

            /* Find all the marks now. */
            for (end = rb_buffer_get_idx(buffer) + 1; end < count; end++)
                if (!HB_UNICODE_GENERAL_CATEGORY_IS_MARK(
                        _hb_glyph_info_get_general_category(&rb_buffer_get_info(buffer)[end])))
                    break;

            /* idx to end is one non-simple cluster. */
            decompose_multi_char_cluster(&c, end, always_short_circuit);
        } while (rb_buffer_get_idx(buffer) < count);
        rb_buffer_swap_buffers(buffer);
    }

    /* Second round, reorder (inplace) */

    if (!all_simple) {
        count = rb_buffer_get_length(buffer);
        for (unsigned int i = 0; i < count; i++) {
            if (_hb_glyph_info_get_modified_combining_class(&rb_buffer_get_info(buffer)[i]) == 0)
                continue;

            unsigned int end;
            for (end = i + 1; end < count; end++)
                if (_hb_glyph_info_get_modified_combining_class(&rb_buffer_get_info(buffer)[end]) == 0)
                    break;

            /* We are going to do a O(n^2).  Only do this if the sequence is short. */
            if (end - i > HB_OT_SHAPE_COMPLEX_MAX_COMBINING_MARKS) {
                i = end;
                continue;
            }

            rb_buffer_sort(buffer, i, end, compare_combining_class);

            if (plan->shaper->reorder_marks)
                plan->shaper->reorder_marks(plan, buffer, i, end);

            i = end;
        }
    }
    if (*rb_buffer_get_scratch_flags(buffer) & HB_BUFFER_SCRATCH_FLAG_HAS_CGJ) {
        /* For all CGJ, check if it prevented any reordering at all.
         * If it did NOT, then make it skippable.
         * https://github.com/harfbuzz/harfbuzz/issues/554
         */
        for (unsigned int i = 1; i + 1 < rb_buffer_get_length(buffer); i++)
            if (rb_buffer_get_info(buffer)[i].codepoint == 0x034Fu /*CGJ*/ &&
                info_cc(rb_buffer_get_info(buffer)[i - 1]) <= info_cc(rb_buffer_get_info(buffer)[i + 1])) {
                _hb_glyph_info_unhide(&rb_buffer_get_info(buffer)[i]);
            }
    }

    /* Third round, recompose */

    if (!all_simple && (mode == HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS ||
                        mode == HB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT)) {
        /* As noted in the comment earlier, we don't try to combine
         * ccc=0 chars with their previous Starter. */

        rb_buffer_clear_output(buffer);
        count = rb_buffer_get_length(buffer);
        unsigned int starter = 0;
        rb_buffer_next_glyph(buffer);
        while (rb_buffer_get_idx(buffer) < count) {
            hb_codepoint_t composed, glyph;
            if (/* We don't try to compose a non-mark character with it's preceding starter.
                 * This is both an optimization to avoid trying to compose every two neighboring
                 * glyphs in most scripts AND a desired feature for Hangul.  Apparently Hangul
                 * fonts are not designed to mix-and-match pre-composed syllables and Jamo. */
                HB_UNICODE_GENERAL_CATEGORY_IS_MARK(
                    _hb_glyph_info_get_general_category(rb_buffer_get_cur(buffer, 0)))) {
                if (/* If there's anything between the starter and this char, they should have CCC
                     * smaller than this character's. */
                    (starter == rb_buffer_get_out_len(buffer) - 1 ||
                     info_cc(*rb_buffer_get_prev(buffer)) < info_cc(*rb_buffer_get_cur(buffer, 0))) &&
                    /* And compose. */
                    c.compose(&c,
                              rb_buffer_get_out_info(buffer)[starter].codepoint,
                              rb_buffer_get_cur(buffer, 0)->codepoint,
                              &composed) &&
                    /* And the font has glyph for the composite. */
                    font->get_nominal_glyph(composed, &glyph)) {
                    /* Composes. */
                    rb_buffer_next_glyph(buffer); /* Copy to out-buffer. */
                    rb_buffer_merge_out_clusters(buffer, starter, rb_buffer_get_out_len(buffer));
                    rb_buffer_set_out_len(buffer,
                                          rb_buffer_get_out_len(buffer) - 1); /* Remove the second composable. */
                    /* Modify starter and carry on. */
                    rb_buffer_get_out_info(buffer)[starter].codepoint = composed;
                    rb_buffer_get_out_info(buffer)[starter].glyph_index() = glyph;
                    _hb_glyph_info_set_unicode_props(&rb_buffer_get_out_info(buffer)[starter], buffer);

                    continue;
                }
            }

            /* Blocked, or doesn't compose. */
            rb_buffer_next_glyph(buffer);

            if (info_cc(*rb_buffer_get_prev(buffer)) == 0)
                starter = rb_buffer_get_out_len(buffer) - 1;
        }
        rb_buffer_swap_buffers(buffer);
    }
}

#endif
