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

#ifndef HB_OT_H_IN
#error "Include <hb-ot.h> instead."
#endif

#ifndef HB_OT_FONT_H
#define HB_OT_FONT_H

#include "hb.h"

HB_BEGIN_DECLS

HB_EXTERN hb_bool_t hb_ot_get_nominal_glyph(hb_font_t *font, hb_codepoint_t unicode, hb_codepoint_t *glyph);

HB_EXTERN unsigned int hb_ot_get_nominal_glyphs(hb_font_t *font,
                                                unsigned int count,
                                                const hb_codepoint_t *first_unicode,
                                                unsigned int unicode_stride,
                                                hb_codepoint_t *first_glyph,
                                                unsigned int glyph_stride);

HB_EXTERN hb_bool_t hb_ot_get_variation_glyph(hb_font_t *font,
                                              hb_codepoint_t unicode,
                                              hb_codepoint_t variation_selector,
                                              hb_codepoint_t *glyph);

HB_EXTERN void hb_ot_get_glyph_h_advances(hb_font_t *font,
                                          unsigned count,
                                          const hb_codepoint_t *first_glyph,
                                          unsigned glyph_stride,
                                          hb_position_t *first_advance,
                                          unsigned advance_stride);

HB_EXTERN void hb_ot_get_glyph_v_advances(hb_font_t *font,
                                          unsigned count,
                                          const hb_codepoint_t *first_glyph,
                                          unsigned glyph_stride,
                                          hb_position_t *first_advance,
                                          unsigned advance_stride);

HB_EXTERN hb_bool_t hb_ot_get_glyph_v_origin(hb_font_t *font, hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y);

HB_EXTERN hb_bool_t hb_ot_get_glyph_extents(hb_font_t *font, hb_codepoint_t glyph, hb_glyph_extents_t *extents);

HB_EXTERN hb_bool_t hb_ot_get_glyph_name(hb_font_t *font, hb_codepoint_t glyph, char *name, unsigned int size);

HB_EXTERN hb_bool_t hb_ot_get_font_h_extents(hb_font_t *font, hb_font_extents_t *metrics);

HB_EXTERN hb_bool_t hb_ot_get_font_v_extents(hb_font_t *font, hb_font_extents_t *metrics);

HB_END_DECLS

#endif /* HB_OT_FONT_H */
