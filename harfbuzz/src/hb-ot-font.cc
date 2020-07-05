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

#include "hb-font.hh"
#include "hb-machinery.hh"
#include "hb-ot-face.hh"

#include "hb-ot-cmap-table.hh"
#include "hb-ot-glyf-table.hh"
#include "hb-ot-cff1-table.hh"
#include "hb-ot-cff2-table.hh"
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

hb_bool_t hb_ot_get_nominal_glyph(hb_font_t *font, hb_codepoint_t unicode, hb_codepoint_t *glyph)
{
    return font->face->table.cmap->get_nominal_glyph(unicode, glyph);
}

unsigned int hb_ot_get_nominal_glyphs(hb_font_t *font,
                                      unsigned int count,
                                      const hb_codepoint_t *first_unicode,
                                      unsigned int unicode_stride,
                                      hb_codepoint_t *first_glyph,
                                      unsigned int glyph_stride)
{
    return font->face->table.cmap->get_nominal_glyphs(count, first_unicode, unicode_stride, first_glyph, glyph_stride);
}

hb_bool_t hb_ot_get_variation_glyph(hb_font_t *font,
                                    hb_codepoint_t unicode,
                                    hb_codepoint_t variation_selector,
                                    hb_codepoint_t *glyph)
{
    return font->face->table.cmap->get_variation_glyph(unicode, variation_selector, glyph);
}

void hb_ot_get_glyph_h_advances(hb_font_t *font,
                                unsigned count,
                                const hb_codepoint_t *first_glyph,
                                unsigned glyph_stride,
                                hb_position_t *first_advance,
                                unsigned advance_stride)
{
    const OT::hmtx_accelerator_t &hmtx = *font->face->table.hmtx;

    for (unsigned int i = 0; i < count; i++) {
        *first_advance = font->em_scale_x(hmtx.get_advance(*first_glyph, font));
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
    const OT::vmtx_accelerator_t &vmtx = *font->face->table.vmtx;

    for (unsigned int i = 0; i < count; i++) {
        *first_advance = font->em_scale_y(-(int)vmtx.get_advance(*first_glyph, font));
        first_glyph = &StructAtOffsetUnaligned<hb_codepoint_t>(first_glyph, glyph_stride);
        first_advance = &StructAtOffsetUnaligned<hb_position_t>(first_advance, advance_stride);
    }
}

hb_bool_t hb_ot_get_glyph_v_origin(hb_font_t *font, hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
{
    *x = font->get_glyph_h_advance(glyph) / 2;

    const OT::VORG &VORG = *font->face->table.VORG;
    if (VORG.has_data()) {
        *y = font->em_scale_y(VORG.get_y_origin(glyph));
        return true;
    }

    hb_glyph_extents_t extents = {0};
    if (font->face->table.glyf->get_extents(font, glyph, &extents)) {
        const OT::vmtx_accelerator_t &vmtx = *font->face->table.vmtx;
        hb_position_t tsb = vmtx.get_side_bearing(font, glyph);
        *y = extents.y_bearing + font->em_scale_y(tsb);
        return true;
    }

    hb_font_extents_t font_extents;
    font->get_h_extents_with_fallback(&font_extents);
    *y = font_extents.ascender;

    return true;
}

hb_bool_t hb_ot_get_glyph_extents(hb_font_t *font, hb_codepoint_t glyph, hb_glyph_extents_t *extents)
{
    if (font->face->table.sbix->get_extents(font, glyph, extents))
        return true;
    if (font->face->table.glyf->get_extents(font, glyph, extents))
        return true;
    if (font->face->table.cff1->get_extents(font, glyph, extents))
        return true;
    if (font->face->table.cff2->get_extents(font, glyph, extents))
        return true;
    if (font->face->table.CBDT->get_extents(font, glyph, extents))
        return true;

    // TODO Hook up side-bearings variations.
    return false;
}

hb_bool_t hb_ot_get_glyph_name(hb_font_t *font, hb_codepoint_t glyph, char *name, unsigned int size)
{
    if (font->face->table.post->get_glyph_name(glyph, name, size))
        return true;
    if (font->face->table.cff1->get_glyph_name(glyph, name, size))
        return true;
    return false;
}
hb_bool_t hb_ot_get_glyph_from_name(hb_font_t *font, const char *name, int len, hb_codepoint_t *glyph)
{
    if (font->face->table.post->get_glyph_from_name(name, len, glyph))
        return true;
    if (font->face->table.cff1->get_glyph_from_name(name, len, glyph))
        return true;
    return false;
}

hb_bool_t hb_ot_get_font_h_extents(hb_font_t *font, hb_font_extents_t *metrics)
{
    return _hb_ot_metrics_get_position_common(font, HB_OT_METRICS_TAG_HORIZONTAL_ASCENDER, &metrics->ascender) &&
           _hb_ot_metrics_get_position_common(font, HB_OT_METRICS_TAG_HORIZONTAL_DESCENDER, &metrics->descender) &&
           _hb_ot_metrics_get_position_common(font, HB_OT_METRICS_TAG_HORIZONTAL_LINE_GAP, &metrics->line_gap);
}

hb_bool_t hb_ot_get_font_v_extents(hb_font_t *font, hb_font_extents_t *metrics)
{
    return _hb_ot_metrics_get_position_common(font, HB_OT_METRICS_TAG_VERTICAL_ASCENDER, &metrics->ascender) &&
           _hb_ot_metrics_get_position_common(font, HB_OT_METRICS_TAG_VERTICAL_DESCENDER, &metrics->descender) &&
           _hb_ot_metrics_get_position_common(font, HB_OT_METRICS_TAG_VERTICAL_LINE_GAP, &metrics->line_gap);
}

int _glyf_get_side_bearing_var(hb_font_t *font, hb_codepoint_t glyph, bool is_vertical)
{
    return font->face->table.glyf->get_side_bearing_var(font, glyph, is_vertical);
}

unsigned _glyf_get_advance_var(hb_font_t *font, hb_codepoint_t glyph, bool is_vertical)
{
    return font->face->table.glyf->get_advance_var(font, glyph, is_vertical);
}
