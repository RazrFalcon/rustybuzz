#pragma once

#include "hb-font.h"

hb_bool_t
hb_ot_get_nominal_glyph (void *font_data,
			 hb_codepoint_t unicode,
			 hb_codepoint_t *glyph);

unsigned int
hb_ot_get_nominal_glyphs (void *font_data,
			  unsigned int count,
			  const hb_codepoint_t *first_unicode,
			  unsigned int unicode_stride,
			  hb_codepoint_t *first_glyph,
			  unsigned int glyph_stride);

hb_bool_t
hb_ot_get_variation_glyph (void *font_data,
			   hb_codepoint_t unicode,
			   hb_codepoint_t variation_selector,
			   hb_codepoint_t *glyph);

void
hb_ot_get_glyph_h_advances (hb_font_t* font, void* font_data,
			    unsigned count,
			    const hb_codepoint_t *first_glyph,
			    unsigned glyph_stride,
			    hb_position_t *first_advance,
			    unsigned advance_stride);

void
hb_ot_get_glyph_v_advances (hb_font_t* font, void* font_data,
			    unsigned count,
			    const hb_codepoint_t *first_glyph,
			    unsigned glyph_stride,
			    hb_position_t *first_advance,
			    unsigned advance_stride);

hb_bool_t
hb_ot_get_glyph_v_origin (hb_font_t *font,
			  void *font_data,
			  hb_codepoint_t glyph,
			  hb_position_t *x,
			  hb_position_t *y);

hb_bool_t
hb_ot_get_glyph_extents (hb_font_t *font,
			 void *font_data,
			 hb_codepoint_t glyph,
			 hb_glyph_extents_t *extents);

hb_bool_t
hb_ot_get_glyph_name (void *font_data,
		      hb_codepoint_t glyph,
		      char *name, unsigned int size);

hb_bool_t
hb_ot_get_glyph_from_name (void *font_data,
			   const char *name, int len,
			   hb_codepoint_t *glyph);

hb_bool_t
hb_ot_get_font_h_extents (hb_font_t *font, hb_font_extents_t *metrics);

hb_bool_t
hb_ot_get_font_v_extents (hb_font_t *font, hb_font_extents_t *metrics);

