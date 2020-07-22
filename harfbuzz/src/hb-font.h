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

#ifndef HB_H_IN
#error "Include <hb.h> instead."
#endif

#ifndef HB_FONT_H
#define HB_FONT_H

#include "hb-common.h"
#include "hb-face.h"

HB_BEGIN_DECLS

typedef struct hb_font_t hb_font_t;

/* font and glyph extents */

/* Note that typically ascender is positive and descender negative in coordinate systems that grow up. */
typedef struct hb_font_extents_t
{
    hb_position_t ascender;  /* typographic ascender. */
    hb_position_t descender; /* typographic descender. */
    hb_position_t line_gap;  /* suggested line spacing gap. */
    /*< private >*/
    hb_position_t reserved9;
    hb_position_t reserved8;
    hb_position_t reserved7;
    hb_position_t reserved6;
    hb_position_t reserved5;
    hb_position_t reserved4;
    hb_position_t reserved3;
    hb_position_t reserved2;
    hb_position_t reserved1;
} hb_font_extents_t;

/* Note that height is negative in coordinate systems that grow up. */
typedef struct hb_glyph_extents_t
{
    hb_position_t x_bearing; /* left side of glyph from origin. */
    hb_position_t y_bearing; /* top side of glyph from origin. */
    hb_position_t width;     /* distance from left to right side. */
    hb_position_t height;    /* distance from top to bottom side. */
} hb_glyph_extents_t;

/* func dispatch */

HB_EXTERN hb_bool_t hb_font_get_h_extents(hb_font_t *font, hb_font_extents_t *extents);
HB_EXTERN hb_bool_t hb_font_get_v_extents(hb_font_t *font, hb_font_extents_t *extents);

HB_EXTERN hb_bool_t hb_font_get_nominal_glyph(hb_font_t *font, hb_codepoint_t unicode, hb_codepoint_t *glyph);
HB_EXTERN hb_bool_t hb_font_get_variation_glyph(hb_font_t *font,
                                                hb_codepoint_t unicode,
                                                hb_codepoint_t variation_selector,
                                                hb_codepoint_t *glyph);

HB_EXTERN unsigned int hb_font_get_nominal_glyphs(hb_font_t *font,
                                                  unsigned int count,
                                                  const hb_codepoint_t *first_unicode,
                                                  unsigned int unicode_stride,
                                                  hb_codepoint_t *first_glyph,
                                                  unsigned int glyph_stride);

HB_EXTERN hb_position_t hb_font_get_glyph_h_advance(hb_font_t *font, hb_codepoint_t glyph);
HB_EXTERN hb_position_t hb_font_get_glyph_v_advance(hb_font_t *font, hb_codepoint_t glyph);

HB_EXTERN void hb_font_get_glyph_h_advances(hb_font_t *font,
                                            unsigned int count,
                                            const hb_codepoint_t *first_glyph,
                                            unsigned glyph_stride,
                                            hb_position_t *first_advance,
                                            unsigned advance_stride);
HB_EXTERN void hb_font_get_glyph_v_advances(hb_font_t *font,
                                            unsigned int count,
                                            const hb_codepoint_t *first_glyph,
                                            unsigned glyph_stride,
                                            hb_position_t *first_advance,
                                            unsigned advance_stride);

HB_EXTERN hb_bool_t hb_font_get_glyph_extents(hb_font_t *font, hb_codepoint_t glyph, hb_glyph_extents_t *extents);

/* high-level funcs, with fallback */

HB_EXTERN hb_bool_t hb_font_get_glyph_contour_point_for_origin(hb_font_t *font,
                                                               hb_codepoint_t glyph,
                                                               unsigned int point_index,
                                                               hb_direction_t direction,
                                                               hb_position_t *x,
                                                               hb_position_t *y);

/*
 * hb_font_t
 */

/* Fonts are very light-weight objects */

HB_EXTERN hb_face_t *hb_font_get_face(hb_font_t *font);

HB_EXTERN int hb_font_get_upem(hb_font_t *font);
HB_EXTERN unsigned int hb_font_get_ppem_x(hb_font_t *font);
HB_EXTERN unsigned int hb_font_get_ppem_y(hb_font_t *font);
HB_EXTERN float hb_font_get_ptem(hb_font_t *font);

HB_EXTERN const int* hb_font_get_coords(hb_font_t *font);
HB_EXTERN unsigned int hb_font_get_num_coords(hb_font_t *font);

HB_EXTERN hb_bool_t hb_font_has_glyph(hb_font_t *font, hb_codepoint_t unicode);

HB_EXTERN void hb_font_get_h_extents_with_fallback(hb_font_t *font, hb_font_extents_t *extents);

HB_EXTERN void
hb_font_get_glyph_v_origin_with_fallback(hb_font_t *font, hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y);

HB_EXTERN hb_bool_t hb_font_get_glyph_h_origin(hb_font_t *font,
                                               hb_codepoint_t glyph,
                                               hb_position_t *x,
                                               hb_position_t *y);
HB_EXTERN hb_bool_t hb_font_get_glyph_v_origin(hb_font_t *font,
                                               hb_codepoint_t glyph,
                                               hb_position_t *x,
                                               hb_position_t *y);

HB_EXTERN void
hb_font_subtract_glyph_v_origin(hb_font_t *font, hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y);

HB_EXTERN void
hb_font_guess_v_origin_minus_h_origin(hb_font_t *font, hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y);

HB_EXTERN hb_bool_t hb_font_has_vorg_data(hb_font_t *font);
HB_EXTERN int hb_font_get_y_origin(hb_font_t *font, hb_codepoint_t glyph);

HB_END_DECLS

#endif /* HB_FONT_H */
