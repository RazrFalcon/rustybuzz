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

extern "C" {
RB_EXTERN rb_bool_t rb_ot_metrics_get_position_common(rb_font_t *font, rb_tag_t tag, int *position);
}

/**
 * SECTION:hb-ot-font
 * @title: hb-ot-font
 * @short_description: OpenType font implementation
 * @include: hb-ot.h
 *
 * Functions for using OpenType fonts with rb_shape().  Note that fonts returned
 * by rb_font_create() default to using these functions, so most clients would
 * never need to call these functions directly.
 **/

unsigned int rb_ot_get_nominal_glyphs(rb_font_t *font,
                                      unsigned int count,
                                      const rb_codepoint_t *first_unicode,
                                      unsigned int unicode_stride,
                                      rb_codepoint_t *first_glyph,
                                      unsigned int glyph_stride)
{
    unsigned int done;
    for (done = 0; done < count && rb_ot_get_nominal_glyph(font, *first_unicode, first_glyph); done++) {
        first_unicode = &StructAtOffsetUnaligned<rb_codepoint_t>(first_unicode, unicode_stride);
        first_glyph = &StructAtOffsetUnaligned<rb_codepoint_t>(first_glyph, glyph_stride);
    }
    return done;
}

void rb_ot_get_glyph_h_advances(rb_font_t *font,
                                unsigned count,
                                const rb_codepoint_t *first_glyph,
                                unsigned glyph_stride,
                                rb_position_t *first_advance,
                                unsigned advance_stride)
{
    for (unsigned int i = 0; i < count; i++) {
        *first_advance = rb_font_get_advance(font, *first_glyph, 0);
        first_glyph = &StructAtOffsetUnaligned<rb_codepoint_t>(first_glyph, glyph_stride);
        first_advance = &StructAtOffsetUnaligned<rb_position_t>(first_advance, advance_stride);
    }
}

void rb_ot_get_glyph_v_advances(rb_font_t *font,
                                unsigned count,
                                const rb_codepoint_t *first_glyph,
                                unsigned glyph_stride,
                                rb_position_t *first_advance,
                                unsigned advance_stride)
{
    for (unsigned int i = 0; i < count; i++) {
        *first_advance = -rb_font_get_advance(font, *first_glyph, 1);
        first_glyph = &StructAtOffsetUnaligned<rb_codepoint_t>(first_glyph, glyph_stride);
        first_advance = &StructAtOffsetUnaligned<rb_position_t>(first_advance, advance_stride);
    }
}

rb_bool_t rb_ot_get_glyph_v_origin(rb_font_t *font, rb_codepoint_t glyph, rb_position_t *x, rb_position_t *y)
{
    *x = rb_font_get_glyph_h_advance(font, glyph) / 2;

    if (rb_font_has_vorg_data(font)) {
        *y = rb_font_get_y_origin(font, glyph);
        return true;
    }

    rb_glyph_extents_t extents = {0};
    rb_ot_get_glyph_extents(font, glyph, &extents);

    rb_position_t tsb = rb_font_get_side_bearing(font, glyph, true);
    *y = extents.y_bearing + tsb;
    return true;
}

rb_bool_t rb_ot_get_font_h_extents(rb_font_t *font, rb_font_extents_t *metrics)
{
    return rb_ot_metrics_get_position_common(font, RB_OT_METRICS_TAG_HORIZONTAL_ASCENDER, &metrics->ascender) &&
           rb_ot_metrics_get_position_common(font, RB_OT_METRICS_TAG_HORIZONTAL_DESCENDER, &metrics->descender) &&
           rb_ot_metrics_get_position_common(font, RB_OT_METRICS_TAG_HORIZONTAL_LINE_GAP, &metrics->line_gap);
}

rb_bool_t rb_ot_get_font_v_extents(rb_font_t *font, rb_font_extents_t *metrics)
{
    return rb_ot_metrics_get_position_common(font, RB_OT_METRICS_TAG_VERTICAL_ASCENDER, &metrics->ascender) &&
           rb_ot_metrics_get_position_common(font, RB_OT_METRICS_TAG_VERTICAL_DESCENDER, &metrics->descender) &&
           rb_ot_metrics_get_position_common(font, RB_OT_METRICS_TAG_VERTICAL_LINE_GAP, &metrics->line_gap);
}
