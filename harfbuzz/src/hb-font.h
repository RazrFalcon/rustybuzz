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

#pragma once

#include "hb-common.h"
#include "hb-face.h"

HB_BEGIN_DECLS

typedef struct hb_font_t hb_font_t;

/* font and glyph extents */

/* Note that typically ascender is positive and descender negative in coordinate systems that grow up. */
typedef struct hb_font_extents_t
{
  hb_position_t ascender; /* typographic ascender. */
  hb_position_t descender; /* typographic descender. */
  hb_position_t line_gap; /* suggested line spacing gap. */
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
  hb_position_t width; /* distance from left to right side. */
  hb_position_t height; /* distance from top to bottom side. */
} hb_glyph_extents_t;

/* high-level funcs, with fallback */

/* Calls either hb_font_get_nominal_glyph() if variation_selector is 0,
 * otherwise calls hb_font_get_variation_glyph(). */
HB_EXTERN hb_bool_t
hb_font_get_glyph (hb_font_t *font,
		   hb_codepoint_t unicode, hb_codepoint_t variation_selector,
		   hb_codepoint_t *glyph);

/*
 * hb_font_t
 */

/* Fonts are very light-weight objects */

HB_EXTERN hb_font_t *
hb_font_create (hb_face_t *face, const void *rust_data);

HB_EXTERN hb_font_t *
hb_font_create_sub_font (hb_font_t *parent);

HB_EXTERN hb_font_t *
hb_font_get_empty (void);

HB_EXTERN hb_font_t *
hb_font_reference (hb_font_t *font);

HB_EXTERN void
hb_font_destroy (hb_font_t *font);

HB_EXTERN void
hb_font_make_immutable (hb_font_t *font);

HB_EXTERN hb_bool_t
hb_font_is_immutable (hb_font_t *font);

HB_EXTERN void
hb_font_set_parent (hb_font_t *font,
		    hb_font_t *parent);

HB_EXTERN hb_font_t *
hb_font_get_parent (hb_font_t *font);

HB_EXTERN void
hb_font_set_face (hb_font_t *font,
		  hb_face_t *face);

HB_EXTERN hb_face_t *
hb_font_get_face (hb_font_t *font);


HB_EXTERN void
hb_font_set_scale (hb_font_t *font,
		   int x_scale,
		   int y_scale);

HB_EXTERN void
hb_font_get_scale (hb_font_t *font,
		   int *x_scale,
		   int *y_scale);

/*
 * A zero value means "no hinting in that direction"
 */
HB_EXTERN void
hb_font_set_ppem (hb_font_t *font,
		  unsigned int x_ppem,
		  unsigned int y_ppem);

HB_EXTERN void
hb_font_get_ppem (hb_font_t *font,
		  unsigned int *x_ppem,
		  unsigned int *y_ppem);

/*
 * Point size per EM.  Used for optical-sizing in CoreText.
 * A value of zero means "not set".
 */
HB_EXTERN void
hb_font_set_ptem (hb_font_t *font, float ptem);

HB_EXTERN float
hb_font_get_ptem (hb_font_t *font);

HB_EXTERN void
hb_font_set_variations (hb_font_t *font,
			const int *coords,
			unsigned int coords_length);

HB_EXTERN const int *
hb_font_get_var_coords_normalized (hb_font_t *font,
				   unsigned int *length);

HB_EXTERN unsigned int
hb_font_get_glyph_count (hb_font_t *face);

HB_EXTERN unsigned
rb_font_get_advance (const void *rust_data, hb_codepoint_t glyph, hb_bool_t is_vertical);

HB_EXTERN unsigned
rb_font_get_advance_var (hb_font_t *font, const void *rust_data,
			 hb_codepoint_t glyph, hb_bool_t is_vertical,
			 const int *coords, unsigned int coord_count);

HB_EXTERN int
rb_font_get_side_bearing (const void *rust_data, hb_codepoint_t glyph, hb_bool_t is_vertical);

HB_EXTERN int
rb_font_get_side_bearing_var (hb_font_t *font, const void *rust_data,
			      hb_codepoint_t glyph, hb_bool_t is_vertical,
			      const int *coords, unsigned int coord_count);

HB_EXTERN unsigned
hb_ot_glyf_get_advance_var (hb_font_t *font, hb_codepoint_t glyph, hb_bool_t is_vertical);

HB_EXTERN int
hb_ot_glyf_get_side_bearing_var (hb_font_t *font, hb_codepoint_t glyph, hb_bool_t is_vertical);

HB_EXTERN unsigned int
rb_face_get_glyph_count (const void *rust_data);

HB_EXTERN unsigned int
rb_face_get_upem (const void *rust_data);

HB_EXTERN unsigned int
rb_face_index_to_loc_format (const void *rust_data);

HB_END_DECLS
