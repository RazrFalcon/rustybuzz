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

#include "hb-ot.h"

#include "hb-machinery.hh"
#include "hb-ot-face.hh"

#include "hb-ot-cmap-table.hh"

extern "C" {
HB_EXTERN hb_bool_t hb_ot_metrics_get_position_common(hb_font_t *font, hb_tag_t tag, int *position);
}

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

hb_bool_t hb_ot_get_nominal_glyph(hb_font_t *font, hb_codepoint_t unicode, hb_codepoint_t *glyph)
{
    return hb_font_get_face(font)->table.cmap->get_nominal_glyph(unicode, glyph);
}

unsigned int hb_ot_get_nominal_glyphs(hb_font_t *font,
                                      unsigned int count,
                                      const hb_codepoint_t *first_unicode,
                                      unsigned int unicode_stride,
                                      hb_codepoint_t *first_glyph,
                                      unsigned int glyph_stride)
{
    return hb_font_get_face(font)->table.cmap->get_nominal_glyphs(
        count, first_unicode, unicode_stride, first_glyph, glyph_stride);
}

hb_bool_t hb_ot_get_variation_glyph(hb_font_t *font,
                                    hb_codepoint_t unicode,
                                    hb_codepoint_t variation_selector,
                                    hb_codepoint_t *glyph)
{
    return hb_font_get_face(font)->table.cmap->get_variation_glyph(unicode, variation_selector, glyph);
}

void hb_ot_get_glyph_h_advances(hb_font_t *font,
                                unsigned count,
                                const hb_codepoint_t *first_glyph,
                                unsigned glyph_stride,
                                hb_position_t *first_advance,
                                unsigned advance_stride)
{
    for (unsigned int i = 0; i < count; i++) {
        *first_advance = hb_font_get_advance(font, *first_glyph, 0);
        first_glyph = &StructAtOffsetUnaligned<hb_codepoint_t>(first_glyph, glyph_stride);
        first_advance = &StructAtOffsetUnaligned<hb_position_t>(first_advance, advance_stride);
    }
}

void hb_ot_get_glyph_v_advances(hb_font_t *font,
                                unsigned count,
                                const hb_codepoint_t *first_glyph,
                                unsigned glyph_stride,
                                hb_position_t *first_advance,
                                unsigned advance_stride)
{
    for (unsigned int i = 0; i < count; i++) {
        *first_advance = -hb_font_get_advance(font, *first_glyph, 1);
        first_glyph = &StructAtOffsetUnaligned<hb_codepoint_t>(first_glyph, glyph_stride);
        first_advance = &StructAtOffsetUnaligned<hb_position_t>(first_advance, advance_stride);
    }
}

hb_bool_t hb_ot_get_glyph_v_origin(hb_font_t *font, hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
{
    *x = hb_font_get_glyph_h_advance(font, glyph) / 2;

    if (hb_font_has_vorg_data(font)) {
        *y = hb_font_get_y_origin(font, glyph);
        return true;
    }

    hb_glyph_extents_t extents = {0};
    hb_ot_get_glyph_extents(font, glyph, &extents);

    hb_position_t tsb = hb_font_get_side_bearing(font, glyph, true);
    *y = extents.y_bearing + tsb;
    return true;
}

hb_bool_t hb_ot_get_font_h_extents(hb_font_t *font, hb_font_extents_t *metrics)
{
    return hb_ot_metrics_get_position_common(font, HB_OT_METRICS_TAG_HORIZONTAL_ASCENDER, &metrics->ascender) &&
           hb_ot_metrics_get_position_common(font, HB_OT_METRICS_TAG_HORIZONTAL_DESCENDER, &metrics->descender) &&
           hb_ot_metrics_get_position_common(font, HB_OT_METRICS_TAG_HORIZONTAL_LINE_GAP, &metrics->line_gap);
}

hb_bool_t hb_ot_get_font_v_extents(hb_font_t *font, hb_font_extents_t *metrics)
{
    return hb_ot_metrics_get_position_common(font, HB_OT_METRICS_TAG_VERTICAL_ASCENDER, &metrics->ascender) &&
           hb_ot_metrics_get_position_common(font, HB_OT_METRICS_TAG_VERTICAL_DESCENDER, &metrics->descender) &&
           hb_ot_metrics_get_position_common(font, HB_OT_METRICS_TAG_VERTICAL_LINE_GAP, &metrics->line_gap);
}
