/*
 * Copyright © 2012  Google, Inc.
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

#ifndef RB_OT_SHAPE_NORMALIZE_HH
#define RB_OT_SHAPE_NORMALIZE_HH

#include "hb.hh"

/* buffer var allocations, used during the normalization process */
#define glyph_index() var1.u32

struct rb_ot_shape_plan_t;

enum rb_ot_shape_normalization_mode_t {
    RB_OT_SHAPE_NORMALIZATION_MODE_NONE,
    RB_OT_SHAPE_NORMALIZATION_MODE_DECOMPOSED,
    RB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS,                  /* Never composes base-to-base */
    RB_OT_SHAPE_NORMALIZATION_MODE_COMPOSED_DIACRITICS_NO_SHORT_CIRCUIT, /* Always fully decomposes and then recompose
                                                                            back */

    RB_OT_SHAPE_NORMALIZATION_MODE_AUTO, /* See hb-ot-shape-normalize.cc for logic. */
    RB_OT_SHAPE_NORMALIZATION_MODE_DEFAULT = RB_OT_SHAPE_NORMALIZATION_MODE_AUTO
};

RB_INTERNAL void _rb_ot_shape_normalize(const rb_ot_shape_plan_t *shaper, rb_buffer_t *buffer, rb_font_t *font);

struct rb_ot_shape_normalize_context_t
{
    const rb_ot_shape_plan_t *plan;
    rb_buffer_t *buffer;
    rb_font_t *font;
    bool (*decompose)(const rb_ot_shape_normalize_context_t *c,
                      rb_codepoint_t ab,
                      rb_codepoint_t *a,
                      rb_codepoint_t *b);
    bool (*compose)(const rb_ot_shape_normalize_context_t *c, rb_codepoint_t a, rb_codepoint_t b, rb_codepoint_t *ab);
};

extern "C" {
RB_EXTERN const rb_ot_shape_plan_t *rb_ot_shape_normalize_context_get_plan(const rb_ot_shape_normalize_context_t *c);
RB_EXTERN const rb_font_t *rb_ot_shape_normalize_context_get_font(const rb_ot_shape_normalize_context_t *c);
}

#endif /* RB_OT_SHAPE_NORMALIZE_HH */
