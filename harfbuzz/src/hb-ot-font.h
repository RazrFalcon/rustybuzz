/*
 * Copyright © 2014  Google, Inc.
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
 * Google Author(s): Behdad Esfahbod, Roozbeh Pournader
 */

#ifndef RB_OT_H_IN
#error "Include <hb-ot.h> instead."
#endif

#ifndef RB_OT_FONT_H
#define RB_OT_FONT_H

#include "hb.h"

RB_BEGIN_DECLS

RB_EXTERN rb_bool_t rb_ot_get_nominal_glyph(rb_font_t *font, rb_codepoint_t unicode, rb_codepoint_t *glyph);

RB_EXTERN unsigned int rb_ot_get_nominal_glyphs(rb_font_t *font,
                                                unsigned int count,
                                                const rb_codepoint_t *first_unicode,
                                                unsigned int unicode_stride,
                                                rb_codepoint_t *first_glyph,
                                                unsigned int glyph_stride);

RB_EXTERN rb_bool_t rb_ot_get_variation_glyph(rb_font_t *font,
                                              rb_codepoint_t unicode,
                                              rb_codepoint_t variation_selector,
                                              rb_codepoint_t *glyph);

RB_EXTERN void rb_ot_get_glyph_h_advances(rb_font_t *font,
                                          unsigned count,
                                          const rb_codepoint_t *first_glyph,
                                          unsigned glyph_stride,
                                          rb_position_t *first_advance,
                                          unsigned advance_stride);

RB_EXTERN void rb_ot_get_glyph_v_advances(rb_font_t *font,
                                          unsigned count,
                                          const rb_codepoint_t *first_glyph,
                                          unsigned glyph_stride,
                                          rb_position_t *first_advance,
                                          unsigned advance_stride);

RB_EXTERN rb_bool_t rb_ot_get_glyph_v_origin(rb_font_t *font, rb_codepoint_t glyph, rb_position_t *x, rb_position_t *y);

RB_EXTERN rb_bool_t rb_ot_get_glyph_extents(rb_font_t *font, rb_codepoint_t glyph, rb_glyph_extents_t *extents);

RB_EXTERN rb_bool_t rb_ot_get_glyph_name(rb_font_t *font, rb_codepoint_t glyph, char *name, unsigned int size);

RB_EXTERN rb_bool_t rb_ot_get_font_h_extents(rb_font_t *font, rb_font_extents_t *metrics);

RB_EXTERN rb_bool_t rb_ot_get_font_v_extents(rb_font_t *font, rb_font_extents_t *metrics);

RB_EXTERN unsigned int rb_font_get_advance(rb_font_t *font, rb_codepoint_t glyph, rb_bool_t is_vertical);
RB_EXTERN int rb_font_get_side_bearing(rb_font_t *font, rb_codepoint_t glyph, rb_bool_t is_vertical);

RB_END_DECLS

#endif /* RB_OT_FONT_H */
