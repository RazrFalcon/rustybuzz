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
#include "hb-ot-post-table.hh"
#include "hb-ot-hhea-table.hh"
#include "hb-ot-metrics.hh"
#include "hb-ot-face.hh"

static float _fix_ascender_descender(float value, hb_ot_metrics_tag_t metrics_tag)
{
    if (metrics_tag == HB_OT_METRICS_TAG_HORIZONTAL_ASCENDER || metrics_tag == HB_OT_METRICS_TAG_VERTICAL_ASCENDER)
        return fabs((double)value);
    if (metrics_tag == HB_OT_METRICS_TAG_HORIZONTAL_DESCENDER || metrics_tag == HB_OT_METRICS_TAG_VERTICAL_DESCENDER)
        return -fabs((double)value);
    return value;
}

/* The common part of _get_position logic needed on hb-ot-font and here
   to be able to have slim builds without the not always needed parts */
bool _hb_ot_metrics_get_position_common(hb_font_t *font,
                                        hb_ot_metrics_tag_t metrics_tag,
                                        hb_position_t *position /* OUT.  May be NULL. */)
{
    hb_face_t *face = font->face;
    switch ((unsigned)metrics_tag) {
#ifndef HB_NO_VAR
#define GET_VAR face->table.MVAR->get_var(metrics_tag, font->coords, font->num_coords)
#else
#define GET_VAR .0f
#endif
#define GET_METRIC_X(TABLE, ATTR)                                                                                      \
    (face->table.TABLE->has_data() &&                                                                                  \
     (position &&                                                                                                      \
          (*position = (hb_position_t)roundf(_fix_ascender_descender(face->table.TABLE->ATTR + GET_VAR, metrics_tag))),    \
      true))
#define GET_METRIC_Y(TABLE, ATTR)                                                                                      \
    (face->table.TABLE->has_data() &&                                                                                  \
     (position &&                                                                                                      \
          (*position = (hb_position_t)roundf(_fix_ascender_descender(face->table.TABLE->ATTR + GET_VAR, metrics_tag))),    \
      true))
    case HB_OT_METRICS_TAG_HORIZONTAL_ASCENDER:
        return (face->table.OS2->use_typo_metrics() && GET_METRIC_Y(OS2, sTypoAscender)) ||
               GET_METRIC_Y(hhea, ascender);
    case HB_OT_METRICS_TAG_HORIZONTAL_DESCENDER:
        return (face->table.OS2->use_typo_metrics() && GET_METRIC_Y(OS2, sTypoDescender)) ||
               GET_METRIC_Y(hhea, descender);
    case HB_OT_METRICS_TAG_HORIZONTAL_LINE_GAP:
        return (face->table.OS2->use_typo_metrics() && GET_METRIC_Y(OS2, sTypoLineGap)) || GET_METRIC_Y(hhea, lineGap);
    case HB_OT_METRICS_TAG_VERTICAL_ASCENDER:
        return GET_METRIC_X(vhea, ascender);
    case HB_OT_METRICS_TAG_VERTICAL_DESCENDER:
        return GET_METRIC_X(vhea, descender);
    case HB_OT_METRICS_TAG_VERTICAL_LINE_GAP:
        return GET_METRIC_X(vhea, lineGap);
#undef GET_METRIC_Y
#undef GET_METRIC_X
#undef GET_VAR
    default:
        assert(0);
        return false;
    }
}

#ifndef HB_NO_METRICS

#endif
