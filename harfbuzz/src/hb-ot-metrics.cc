/*
 * Copyright © 2018-2019  Ebrahim Byagowi
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
 */

#include "hb.hh"

#include "hb-ot-var-mvar-table.hh"
#include "hb-ot-os2-table.hh"
#include "hb-ot-hhea-table.hh"
#include "hb-ot-metrics.hh"
#include "hb-ot-face.hh"


static float
_fix_ascender_descender (float value, hb_ot_metrics_tag_t metrics_tag)
{
  if (metrics_tag == HB_OT_METRICS_TAG_HORIZONTAL_ASCENDER ||
      metrics_tag == HB_OT_METRICS_TAG_VERTICAL_ASCENDER)
    return fabs ((double) value);
  if (metrics_tag == HB_OT_METRICS_TAG_HORIZONTAL_DESCENDER ||
      metrics_tag == HB_OT_METRICS_TAG_VERTICAL_DESCENDER)
    return -fabs ((double) value);
  return value;
}

/* The common part of _get_position logic needed on hb-ot-font and here
   to be able to have slim builds without the not always needed parts */
bool 
_hb_ot_metrics_get_position_common (hb_font_t *font,
        hb_ot_metrics_tag_t metrics_tag,
        hb_position_t *position )
{
  hb_face_t *face = font->face;
  switch ((unsigned) metrics_tag)
  {
  case HB_OT_METRICS_TAG_HORIZONTAL_ASCENDER: {
    if (face->table.OS2->use_typo_metrics () && face->table.OS2->has_data ()) {
      const auto n = _fix_ascender_descender (face->table.OS2->sTypoAscender + face->table.MVAR->get_var (metrics_tag, font->coords, font->num_coords), metrics_tag);
      *position = font->em_scalef_y (n);
      return true; 
    } else if (face->table.hhea->has_data ()) {
      const auto n = _fix_ascender_descender ( face->table.hhea->ascender + face->table.MVAR->get_var (metrics_tag, font->coords, font->num_coords), metrics_tag);
      *position = font->em_scalef_y (n);
      return true; 
    } else {
      return false; 
    }
  }
  case HB_OT_METRICS_TAG_HORIZONTAL_DESCENDER:
    return (face->table.OS2->use_typo_metrics () && (face->table.OS2->has_data () && (position && (*position = font->em_scalef_y (_fix_ascender_descender ( face->table.OS2->sTypoDescender + face->table.MVAR->get_var (metrics_tag, font->coords, font->num_coords), metrics_tag))), true))) ||
    (face->table.hhea->has_data () && (position && (*position = font->em_scalef_y (_fix_ascender_descender ( face->table.hhea->descender + face->table.MVAR->get_var (metrics_tag, font->coords, font->num_coords), metrics_tag))), true));
  case HB_OT_METRICS_TAG_HORIZONTAL_LINE_GAP:
    return (face->table.OS2->use_typo_metrics () && (face->table.OS2->has_data () && (position && (*position = font->em_scalef_y (_fix_ascender_descender ( face->table.OS2->sTypoLineGap + face->table.MVAR->get_var (metrics_tag, font->coords, font->num_coords), metrics_tag))), true))) ||
    (face->table.hhea->has_data () && (position && (*position = font->em_scalef_y (_fix_ascender_descender ( face->table.hhea->lineGap + face->table.MVAR->get_var (metrics_tag, font->coords, font->num_coords), metrics_tag))), true));
  case HB_OT_METRICS_TAG_VERTICAL_ASCENDER: return (face->table.vhea->has_data () && (position && (*position = font->em_scalef_x (_fix_ascender_descender ( face->table.vhea->ascender + face->table.MVAR->get_var (metrics_tag, font->coords, font->num_coords), metrics_tag))), true));
  case HB_OT_METRICS_TAG_VERTICAL_DESCENDER: return (face->table.vhea->has_data () && (position && (*position = font->em_scalef_x (_fix_ascender_descender ( face->table.vhea->descender + face->table.MVAR->get_var (metrics_tag, font->coords, font->num_coords), metrics_tag))), true));
  case HB_OT_METRICS_TAG_VERTICAL_LINE_GAP: return (face->table.vhea->has_data () && (position && (*position = font->em_scalef_x (_fix_ascender_descender ( face->table.vhea->lineGap + face->table.MVAR->get_var (metrics_tag, font->coords, font->num_coords), metrics_tag))), true));
  default: assert (0); return false;
  }
}

#ifndef HB_NO_METRICS

#endif
