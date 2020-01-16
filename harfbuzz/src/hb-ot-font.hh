#pragma once

#include "hb-font.h"

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
hb_ot_get_font_h_extents (hb_font_t *font, hb_font_extents_t *metrics);

hb_bool_t
hb_ot_get_font_v_extents (hb_font_t *font, hb_font_extents_t *metrics);
