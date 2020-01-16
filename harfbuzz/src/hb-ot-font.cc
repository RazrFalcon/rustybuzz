/*
 * Copyright © 2011,2014  Google, Inc.
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

#include "hb.hh"

#ifndef HB_NO_OT_FONT

#include "hb-ot.h"

#include "hb-font.hh"
#include "hb-machinery.hh"
#include "hb-ot-face.hh"
#include "hb-ot-font.hh"

#include "hb-ot-glyf-table.hh"
#include "hb-ot-hmtx-table.hh"
#include "hb-ot-os2-table.hh"
#include "hb-ot-post-table.hh"
#include "hb-ot-vorg-table.hh"
#include "hb-ot-color-cbdt-table.hh"
#include "hb-ot-color-sbix-table.hh"


/**
 * SECTION:hb-ot-font
 * @title: hb-ot-font
 * @short_description: OpenType font implementation
 * @include: hb-ot.h
 *
 * Functions for using OpenType fonts with hb_shape().  Note that fonts returned
 * by hb_font_create() default to using these functions, so most clients would
 * never need to call these functions directly.
 **/

struct hb_glyph_bbox_t
{
  int16_t min_x;
  int16_t min_y;
  int16_t max_x;
  int16_t max_y;
};

extern "C" {
  bool rb_ot_get_glyph_bbox (const void *rust_data,
                                hb_codepoint_t glyph,
                                hb_glyph_bbox_t *bbox);
}

void
hb_ot_get_glyph_h_advances (hb_font_t* font, void* font_data,
			    unsigned count,
			    const hb_codepoint_t *first_glyph,
			    unsigned glyph_stride,
			    hb_position_t *first_advance,
			    unsigned advance_stride)
{
  const hb_ot_face_t *ot_face = (const hb_ot_face_t *) font_data;
  const OT::hmtx_accelerator_t &hmtx = *ot_face->hmtx;

  for (unsigned int i = 0; i < count; i++)
  {
    *first_advance = font->em_scale_x (hmtx.get_advance (*first_glyph, font));
    first_glyph = &StructAtOffsetUnaligned<hb_codepoint_t> (first_glyph, glyph_stride);
    first_advance = &StructAtOffsetUnaligned<hb_position_t> (first_advance, advance_stride);
  }
}

void
hb_ot_get_glyph_v_advances (hb_font_t* font, void* font_data,
			    unsigned count,
			    const hb_codepoint_t *first_glyph,
			    unsigned glyph_stride,
			    hb_position_t *first_advance,
			    unsigned advance_stride)
{
  const hb_ot_face_t *ot_face = (const hb_ot_face_t *) font_data;
  const OT::vmtx_accelerator_t &vmtx = *ot_face->vmtx;

  for (unsigned int i = 0; i < count; i++)
  {
    *first_advance = font->em_scale_y (-(int) vmtx.get_advance (*first_glyph, font));
    first_glyph = &StructAtOffsetUnaligned<hb_codepoint_t> (first_glyph, glyph_stride);
    first_advance = &StructAtOffsetUnaligned<hb_position_t> (first_advance, advance_stride);
  }
}

hb_bool_t
hb_ot_get_glyph_v_origin (hb_font_t *font,
			  void *font_data,
			  hb_codepoint_t glyph,
			  hb_position_t *x,
			  hb_position_t *y)
{
  const hb_ot_face_t *ot_face = (const hb_ot_face_t *) font_data;

  *x = font->get_glyph_h_advance (glyph) / 2;

  const OT::VORG &VORG = *ot_face->VORG;
  if (VORG.has_data ())
  {
    *y = font->em_scale_y (VORG.get_y_origin (glyph));
    return true;
  }

  hb_glyph_extents_t extents = {0};
  if (ot_face->glyf->get_extents (font, glyph, &extents))
  {
    const OT::vmtx_accelerator_t &vmtx = *ot_face->vmtx;
    hb_position_t tsb = vmtx.get_side_bearing (font, glyph);
    *y = extents.y_bearing + font->em_scale_y (tsb);
    return true;
  }

  hb_font_extents_t font_extents;
  font->get_h_extents_with_fallback (&font_extents);
  *y = font_extents.ascender;

  return true;
}

hb_bool_t
hb_ot_get_glyph_extents (hb_font_t *font,
			 void *font_data,
			 hb_codepoint_t glyph,
			 hb_glyph_extents_t *extents)
{
  const hb_ot_face_t *ot_face = (const hb_ot_face_t *) font_data;
  bool ret = false;

  if (!ret) ret = ot_face->sbix->get_extents (font, glyph, extents);
  if (!ret) ret = ot_face->glyf->get_extents (font, glyph, extents);
  if (!ret) {
    hb_glyph_bbox_t param;
    ret = rb_ot_get_glyph_bbox (font->rust_data, glyph, &param);
    if (ret) {    
      if (param.min_x >= param.max_x)
      {
        extents->width = 0;
        extents->x_bearing = 0;
      }
      else
      {
        extents->x_bearing = font->em_scalef_x (param.min_x);
        extents->width = font->em_scalef_x (param.max_x - param.min_x);
      }
      if (param.min_y >= param.max_y)
      {
        extents->height = 0;
        extents->y_bearing = 0;
      }
      else
      {
        extents->y_bearing = font->em_scalef_y (param.max_y);
        extents->height = font->em_scalef_y (param.min_y - param.max_y);
      }
    }
  }
  if (!ret) ret = ot_face->CBDT->get_extents (font, glyph, extents);

  // TODO Hook up side-bearings variations.
  return ret;
}

hb_bool_t
hb_ot_get_glyph_name (void *font_data,
		      hb_codepoint_t glyph,
		      char *name, unsigned int size)
{
  const hb_ot_face_t *ot_face = (const hb_ot_face_t *) font_data;
  return ot_face->post->get_glyph_name (glyph, name, size);
}

hb_bool_t
hb_ot_get_font_h_extents (hb_font_t *font, hb_font_extents_t *metrics)
{
  return _hb_ot_metrics_get_position_common (font, HB_OT_METRICS_TAG_HORIZONTAL_ASCENDER, &metrics->ascender) &&
	 _hb_ot_metrics_get_position_common (font, HB_OT_METRICS_TAG_HORIZONTAL_DESCENDER, &metrics->descender) &&
	 _hb_ot_metrics_get_position_common (font, HB_OT_METRICS_TAG_HORIZONTAL_LINE_GAP, &metrics->line_gap);
}

hb_bool_t
hb_ot_get_font_v_extents (hb_font_t *font, hb_font_extents_t *metrics)
{
  return _hb_ot_metrics_get_position_common (font, HB_OT_METRICS_TAG_VERTICAL_ASCENDER, &metrics->ascender) &&
	 _hb_ot_metrics_get_position_common (font, HB_OT_METRICS_TAG_VERTICAL_DESCENDER, &metrics->descender) &&
	 _hb_ot_metrics_get_position_common (font, HB_OT_METRICS_TAG_VERTICAL_LINE_GAP, &metrics->line_gap);
}

#ifndef HB_NO_VAR
int
_glyf_get_side_bearing_var (hb_font_t *font, hb_codepoint_t glyph, bool is_vertical)
{
  return font->face->table.glyf->get_side_bearing_var (font, glyph, is_vertical);
}

unsigned
_glyf_get_advance_var (hb_font_t *font, hb_codepoint_t glyph, bool is_vertical)
{
  return font->face->table.glyf->get_advance_var (font, glyph, is_vertical);
}
#endif


#endif
