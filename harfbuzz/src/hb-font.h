/*
 * Copyright © 2009  Red Hat, Inc.
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
 * Red Hat Author(s): Behdad Esfahbod
 */

#ifndef RB_H_IN
#error "Include <hb.h> instead."
#endif

#ifndef RB_FONT_H
#define RB_FONT_H

#include "hb-common.h"
#include "hb-face.h"

RB_BEGIN_DECLS

typedef struct rb_font_t rb_font_t;

/* font and glyph extents */

/* Note that typically ascender is positive and descender negative in coordinate systems that grow up. */
typedef struct rb_font_extents_t
{
    rb_position_t ascender;  /* typographic ascender. */
    rb_position_t descender; /* typographic descender. */
    rb_position_t line_gap;  /* suggested line spacing gap. */
    /*< private >*/
    rb_position_t reserved9;
    rb_position_t reserved8;
    rb_position_t reserved7;
    rb_position_t reserved6;
    rb_position_t reserved5;
    rb_position_t reserved4;
    rb_position_t reserved3;
    rb_position_t reserved2;
    rb_position_t reserved1;
} rb_font_extents_t;

/* Note that height is negative in coordinate systems that grow up. */
typedef struct rb_glyph_extents_t
{
    rb_position_t x_bearing; /* left side of glyph from origin. */
    rb_position_t y_bearing; /* top side of glyph from origin. */
    rb_position_t width;     /* distance from left to right side. */
    rb_position_t height;    /* distance from top to bottom side. */
} rb_glyph_extents_t;

/* func dispatch */

RB_EXTERN rb_bool_t rb_font_get_h_extents(rb_font_t *font, rb_font_extents_t *extents);
RB_EXTERN rb_bool_t rb_font_get_v_extents(rb_font_t *font, rb_font_extents_t *extents);

RB_EXTERN rb_bool_t rb_font_get_nominal_glyph(rb_font_t *font, rb_codepoint_t unicode, rb_codepoint_t *glyph);
RB_EXTERN rb_bool_t rb_font_get_variation_glyph(rb_font_t *font,
                                                rb_codepoint_t unicode,
                                                rb_codepoint_t variation_selector,
                                                rb_codepoint_t *glyph);

RB_EXTERN unsigned int rb_font_get_nominal_glyphs(rb_font_t *font,
                                                  unsigned int count,
                                                  const rb_codepoint_t *first_unicode,
                                                  unsigned int unicode_stride,
                                                  rb_codepoint_t *first_glyph,
                                                  unsigned int glyph_stride);

RB_EXTERN rb_position_t rb_font_get_glyph_h_advance(rb_font_t *font, rb_codepoint_t glyph);
RB_EXTERN rb_position_t rb_font_get_glyph_v_advance(rb_font_t *font, rb_codepoint_t glyph);

RB_EXTERN void rb_font_get_glyph_h_advances(rb_font_t *font,
                                            unsigned int count,
                                            const rb_codepoint_t *first_glyph,
                                            unsigned glyph_stride,
                                            rb_position_t *first_advance,
                                            unsigned advance_stride);
RB_EXTERN void rb_font_get_glyph_v_advances(rb_font_t *font,
                                            unsigned int count,
                                            const rb_codepoint_t *first_glyph,
                                            unsigned glyph_stride,
                                            rb_position_t *first_advance,
                                            unsigned advance_stride);

RB_EXTERN rb_bool_t rb_font_get_glyph_extents(rb_font_t *font, rb_codepoint_t glyph, rb_glyph_extents_t *extents);

/* high-level funcs, with fallback */

RB_EXTERN rb_bool_t rb_font_get_glyph_contour_point_for_origin(rb_font_t *font,
                                                               rb_codepoint_t glyph,
                                                               unsigned int point_index,
                                                               rb_direction_t direction,
                                                               rb_position_t *x,
                                                               rb_position_t *y);

/*
 * rb_font_t
 */

/* Fonts are very light-weight objects */

RB_EXTERN rb_face_t *rb_font_get_face(rb_font_t *font);

RB_EXTERN int rb_font_get_upem(rb_font_t *font);
RB_EXTERN unsigned int rb_font_get_ppem_x(rb_font_t *font);
RB_EXTERN unsigned int rb_font_get_ppem_y(rb_font_t *font);
RB_EXTERN float rb_font_get_ptem(rb_font_t *font);

RB_EXTERN const int *rb_font_get_coords(rb_font_t *font);
RB_EXTERN unsigned int rb_font_get_num_coords(rb_font_t *font);

RB_EXTERN rb_bool_t rb_font_has_glyph(rb_font_t *font, rb_codepoint_t unicode);

RB_EXTERN void rb_font_get_h_extents_with_fallback(rb_font_t *font, rb_font_extents_t *extents);

RB_EXTERN void
rb_font_get_glyph_v_origin_with_fallback(rb_font_t *font, rb_codepoint_t glyph, rb_position_t *x, rb_position_t *y);

RB_EXTERN rb_bool_t rb_font_get_glyph_h_origin(rb_font_t *font,
                                               rb_codepoint_t glyph,
                                               rb_position_t *x,
                                               rb_position_t *y);
RB_EXTERN rb_bool_t rb_font_get_glyph_v_origin(rb_font_t *font,
                                               rb_codepoint_t glyph,
                                               rb_position_t *x,
                                               rb_position_t *y);

RB_EXTERN void
rb_font_subtract_glyph_v_origin(rb_font_t *font, rb_codepoint_t glyph, rb_position_t *x, rb_position_t *y);

RB_EXTERN void
rb_font_guess_v_origin_minus_h_origin(rb_font_t *font, rb_codepoint_t glyph, rb_position_t *x, rb_position_t *y);

RB_EXTERN rb_bool_t rb_font_has_vorg_data(rb_font_t *font);
RB_EXTERN int rb_font_get_y_origin(rb_font_t *font, rb_codepoint_t glyph);

RB_END_DECLS

#endif /* RB_FONT_H */
