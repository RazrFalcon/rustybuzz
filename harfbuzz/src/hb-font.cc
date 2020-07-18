/*
 * Copyright © 2009  Red Hat, Inc.
 * Copyright © 2012  Google, Inc.
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
 * Google Author(s): Behdad Esfahbod
 */

#include "hb.hh"

#include "hb-machinery.hh"

#include "hb-ot.h"
#include "hb-face.hh"
#include "hb-ot-font.h"
#include "hb-ot-var-avar-table.hh"
#include "hb-ot-var-fvar-table.hh"

/**
 * SECTION:hb-font
 * @title: hb-font
 * @short_description: Font objects
 * @include: hb.h
 *
 * Font objects represent a font face at a certain size and other
 * parameters (pixels per EM, points per EM, variation settings.)
 * Fonts are created from font faces, and are used as input to
 * hb_shape() among other things.
 **/

/* Public getters */

/**
 * hb_font_get_h_extents:
 * @font: a font.
 * @extents: (out):
 *
 *
 *
 * Return value:
 *
 * Since: 1.1.3
 **/
hb_bool_t hb_font_get_h_extents(hb_font_t *font, hb_font_extents_t *extents)
{
    memset(extents, 0, sizeof(*extents));
    return hb_ot_get_font_h_extents(font, extents);
}

/**
 * hb_font_get_v_extents:
 * @font: a font.
 * @extents: (out):
 *
 *
 *
 * Return value:
 *
 * Since: 1.1.3
 **/
hb_bool_t hb_font_get_v_extents(hb_font_t *font, hb_font_extents_t *extents)
{
    memset(extents, 0, sizeof(*extents));
    return hb_ot_get_font_v_extents(font, extents);
}

/**
 * hb_font_get_glyph:
 * @font: a font.
 * @unicode:
 * @variation_selector:
 * @glyph: (out):
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
hb_bool_t
hb_font_get_glyph(hb_font_t *font, hb_codepoint_t unicode, hb_codepoint_t variation_selector, hb_codepoint_t *glyph)
{
    if (unlikely(variation_selector))
        return hb_font_get_variation_glyph(font, unicode, variation_selector, glyph);
    return hb_ot_get_nominal_glyph(font, unicode, glyph);
}

/**
 * hb_font_get_nominal_glyph:
 * @font: a font.
 * @unicode:
 * @glyph: (out):
 *
 *
 *
 * Return value:
 *
 * Since: 1.2.3
 **/
hb_bool_t hb_font_get_nominal_glyph(hb_font_t *font, hb_codepoint_t unicode, hb_codepoint_t *glyph)
{
    *glyph = 0;
    return hb_ot_get_nominal_glyph(font, unicode, glyph);
}

/**
 * hb_font_get_nominal_glyphs:
 * @font: a font.
 *
 *
 *
 * Return value:
 *
 * Since: 2.6.3
 **/
unsigned int hb_font_get_nominal_glyphs(hb_font_t *font,
                                        unsigned int count,
                                        const hb_codepoint_t *first_unicode,
                                        unsigned int unicode_stride,
                                        hb_codepoint_t *first_glyph,
                                        unsigned int glyph_stride)
{
    return hb_ot_get_nominal_glyphs(font, count, first_unicode, unicode_stride, first_glyph, glyph_stride);
}

/**
 * hb_font_get_variation_glyph:
 * @font: a font.
 * @unicode:
 * @variation_selector:
 * @glyph: (out):
 *
 *
 *
 * Return value:
 *
 * Since: 1.2.3
 **/
hb_bool_t hb_font_get_variation_glyph(hb_font_t *font,
                                      hb_codepoint_t unicode,
                                      hb_codepoint_t variation_selector,
                                      hb_codepoint_t *glyph)
{
    *glyph = 0;
    return hb_ot_get_variation_glyph(font, unicode, variation_selector, glyph);
}

/**
 * hb_font_get_glyph_h_advance:
 * @font: a font.
 * @glyph:
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
hb_position_t hb_font_get_glyph_h_advance(hb_font_t *font, hb_codepoint_t glyph)
{
    hb_position_t ret;
    hb_font_get_glyph_h_advances(font, 1, &glyph, 0, &ret, 0);
    return ret;
}

/**
 * hb_font_get_glyph_v_advance:
 * @font: a font.
 * @glyph:
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
hb_position_t hb_font_get_glyph_v_advance(hb_font_t *font, hb_codepoint_t glyph)
{
    hb_position_t ret;
    hb_font_get_glyph_v_advances(font, 1, &glyph, 0, &ret, 0);
    return ret;
}

/**
 * hb_font_get_glyph_h_advances:
 * @font: a font.
 *
 *
 *
 * Since: 1.8.6
 **/
void hb_font_get_glyph_h_advances(hb_font_t *font,
                                  unsigned int count,
                                  const hb_codepoint_t *first_glyph,
                                  unsigned glyph_stride,
                                  hb_position_t *first_advance,
                                  unsigned advance_stride)
{
    return hb_ot_get_glyph_h_advances(font, count, first_glyph, glyph_stride, first_advance, advance_stride);
}
/**
 * hb_font_get_glyph_v_advances:
 * @font: a font.
 *
 *
 *
 * Since: 1.8.6
 **/
void hb_font_get_glyph_v_advances(hb_font_t *font,
                                  unsigned int count,
                                  const hb_codepoint_t *first_glyph,
                                  unsigned glyph_stride,
                                  hb_position_t *first_advance,
                                  unsigned advance_stride)
{
    return hb_ot_get_glyph_v_advances(font, count, first_glyph, glyph_stride, first_advance, advance_stride);
}

/**
 * hb_font_get_glyph_extents:
 * @font: a font.
 * @glyph:
 * @extents: (out):
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
hb_bool_t hb_font_get_glyph_extents(hb_font_t *font, hb_codepoint_t glyph, hb_glyph_extents_t *extents)
{
    memset(extents, 0, sizeof(*extents));
    return hb_ot_get_glyph_extents(font, glyph, extents);
}

/**
 * hb_font_get_glyph_contour_point:
 * @font: a font.
 * @glyph:
 * @point_index:
 * @x: (out):
 * @y: (out):
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
hb_bool_t hb_font_get_glyph_contour_point(
    hb_font_t *font, hb_codepoint_t glyph, unsigned int point_index, hb_position_t *x, hb_position_t *y)
{
    *x = *y = 0;
    return false;
}

/**
 * hb_font_get_glyph_name:
 * @font: a font.
 * @glyph:
 * @name: (array length=size):
 * @size:
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
hb_bool_t hb_font_get_glyph_name(hb_font_t *font, hb_codepoint_t glyph, char *name, unsigned int size)
{
    if (size)
        *name = '\0';
    return hb_ot_get_glyph_name(font, glyph, name, size);
}

/* A bit higher-level, and with fallback */

/**
 * hb_font_get_extents_for_direction:
 * @font: a font.
 * @direction:
 * @extents: (out):
 *
 *
 *
 * Since: 1.1.3
 **/
void hb_font_get_extents_for_direction(hb_font_t *font, hb_direction_t direction, hb_font_extents_t *extents)
{
    if (likely(HB_DIRECTION_IS_HORIZONTAL(direction)))
        hb_font_get_h_extents_with_fallback(font, extents);
    else
        hb_font_get_v_extents_with_fallback(font, extents);
}
/**
 * hb_font_get_glyph_advance_for_direction:
 * @font: a font.
 * @glyph:
 * @direction:
 * @x: (out):
 * @y: (out):
 *
 *
 *
 * Since: 0.9.2
 **/
void hb_font_get_glyph_advance_for_direction(
    hb_font_t *font, hb_codepoint_t glyph, hb_direction_t direction, hb_position_t *x, hb_position_t *y)
{
    *x = *y = 0;
    if (likely(HB_DIRECTION_IS_HORIZONTAL(direction)))
        *x = hb_font_get_glyph_h_advance(font, glyph);
    else
        *y = hb_font_get_glyph_v_advance(font, glyph);
}
/**
 * hb_font_get_glyph_advances_for_direction:
 * @font: a font.
 * @direction:
 *
 *
 *
 * Since: 1.8.6
 **/
HB_EXTERN void hb_font_get_glyph_advances_for_direction(hb_font_t *font,
                                                        hb_direction_t direction,
                                                        unsigned int count,
                                                        const hb_codepoint_t *first_glyph,
                                                        unsigned glyph_stride,
                                                        hb_position_t *first_advance,
                                                        unsigned advance_stride)
{
    if (likely(HB_DIRECTION_IS_HORIZONTAL(direction)))
        hb_font_get_glyph_h_advances(font, count, first_glyph, glyph_stride, first_advance, advance_stride);
    else
        hb_font_get_glyph_v_advances(font, count, first_glyph, glyph_stride, first_advance, advance_stride);
}

/**
 * hb_font_get_glyph_origin_for_direction:
 * @font: a font.
 * @glyph:
 * @direction:
 * @x: (out):
 * @y: (out):
 *
 *
 *
 * Since: 0.9.2
 **/
void hb_font_get_glyph_origin_for_direction(
    hb_font_t *font, hb_codepoint_t glyph, hb_direction_t direction, hb_position_t *x, hb_position_t *y)
{
    if (likely(HB_DIRECTION_IS_HORIZONTAL(direction)))
        hb_font_get_glyph_h_origin_with_fallback(font, glyph, x, y);
    else
        hb_font_get_glyph_v_origin_with_fallback(font, glyph, x, y);
}

/**
 * hb_font_add_glyph_origin_for_direction:
 * @font: a font.
 * @glyph:
 * @direction:
 * @x: (out):
 * @y: (out):
 *
 *
 *
 * Since: 0.9.2
 **/
void hb_font_add_glyph_origin_for_direction(
    hb_font_t *font, hb_codepoint_t glyph, hb_direction_t direction, hb_position_t *x, hb_position_t *y)
{
    hb_position_t origin_x, origin_y;

    hb_font_get_glyph_origin_for_direction(font, glyph, direction, &origin_x, &origin_y);

    *x += origin_x;
    *y += origin_y;
}

/**
 * hb_font_subtract_glyph_origin_for_direction:
 * @font: a font.
 * @glyph:
 * @direction:
 * @x: (out):
 * @y: (out):
 *
 *
 *
 * Since: 0.9.2
 **/
void hb_font_subtract_glyph_origin_for_direction(
    hb_font_t *font, hb_codepoint_t glyph, hb_direction_t direction, hb_position_t *x, hb_position_t *y)
{
    hb_position_t origin_x, origin_y;

    hb_font_get_glyph_origin_for_direction(font, glyph, direction, &origin_x, &origin_y);

    *x -= origin_x;
    *y -= origin_y;
}

/**
 * hb_font_get_glyph_extents_for_origin:
 * @font: a font.
 * @glyph:
 * @direction:
 * @extents: (out):
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
hb_bool_t hb_font_get_glyph_extents_for_origin(hb_font_t *font,
                                               hb_codepoint_t glyph,
                                               hb_direction_t direction,
                                               hb_glyph_extents_t *extents)
{
    hb_bool_t ret = hb_font_get_glyph_extents(font, glyph, extents);

    if (ret)
        hb_font_subtract_glyph_origin_for_direction(font, glyph, direction, &extents->x_bearing, &extents->y_bearing);

    return ret;
}

/**
 * hb_font_get_glyph_contour_point_for_origin:
 * @font: a font.
 * @glyph:
 * @point_index:
 * @direction:
 * @x: (out):
 * @y: (out):
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
hb_bool_t hb_font_get_glyph_contour_point_for_origin(hb_font_t *font,
                                                     hb_codepoint_t glyph,
                                                     unsigned int point_index,
                                                     hb_direction_t direction,
                                                     hb_position_t *x,
                                                     hb_position_t *y)
{
    hb_bool_t ret = hb_font_get_glyph_contour_point(font, glyph, point_index, x, y);

    if (ret)
        hb_font_subtract_glyph_origin_for_direction(font, glyph, direction, x, y);

    return ret;
}

/* Generates gidDDD if glyph has no name. */
/**
 * hb_font_glyph_to_string:
 * @font: a font.
 * @glyph:
 * @s: (array length=size):
 * @size:
 *
 *
 *
 * Since: 0.9.2
 **/
void hb_font_glyph_to_string(hb_font_t *font, hb_codepoint_t glyph, char *s, unsigned int size)
{
    if (hb_font_get_glyph_name(font, glyph, s, size))
        return;

    if (size && snprintf(s, size, "gid%u", glyph) < 0)
        *s = '\0';
}

/*
 * hb_font_t
 */

hb_bool_t hb_font_has_glyph(hb_font_t *font, hb_codepoint_t unicode)
{
    hb_codepoint_t glyph;
    return hb_ot_get_nominal_glyph(font, unicode, &glyph);
}

hb_bool_t hb_font_get_glyph_h_origin(hb_font_t *font, hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
{
    *x = *y = 0;
    return true;
}

hb_bool_t hb_font_get_glyph_v_origin(hb_font_t *font, hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
{
    *x = *y = 0;
    return hb_ot_get_glyph_v_origin(font, glyph, x, y);
}

/* A bit higher-level, and with fallback */

void hb_font_get_h_extents_with_fallback(hb_font_t *font, hb_font_extents_t *extents)
{
    if (!hb_font_get_h_extents(font, extents)) {
        extents->ascender = hb_font_get_upem(font) * .8;
        extents->descender = extents->ascender - hb_font_get_upem(font);
        extents->line_gap = 0;
    }
}

void hb_font_get_v_extents_with_fallback(hb_font_t *font, hb_font_extents_t *extents)
{
    if (!hb_font_get_v_extents(font, extents)) {
        extents->ascender = hb_font_get_upem(font) / 2;
        extents->descender = extents->ascender - hb_font_get_upem(font);
        extents->line_gap = 0;
    }
}

void hb_font_guess_v_origin_minus_h_origin(hb_font_t *font, hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
{
    *x = hb_font_get_glyph_h_advance(font, glyph) / 2;

    /* TODO cache this somehow?! */
    hb_font_extents_t extents;
    hb_font_get_h_extents_with_fallback(font, &extents);
    *y = extents.ascender;
}

void hb_font_get_glyph_h_origin_with_fallback(hb_font_t *font, hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
{
    if (!hb_font_get_glyph_h_origin(font, glyph, x, y) && hb_font_get_glyph_v_origin(font, glyph, x, y)) {
        hb_position_t dx, dy;
        hb_font_guess_v_origin_minus_h_origin(font, glyph, &dx, &dy);
        *x -= dx;
        *y -= dy;
    }
}
void hb_font_get_glyph_v_origin_with_fallback(hb_font_t *font, hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
{
    if (!hb_font_get_glyph_v_origin(font, glyph, x, y) && hb_font_get_glyph_h_origin(font, glyph, x, y)) {
        hb_position_t dx, dy;
        hb_font_guess_v_origin_minus_h_origin(font, glyph, &dx, &dy);
        *x += dx;
        *y += dy;
    }
}

void hb_font_add_glyph_h_origin(hb_font_t *font, hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
{
    hb_position_t origin_x, origin_y;

    hb_font_get_glyph_h_origin_with_fallback(font, glyph, &origin_x, &origin_y);

    *x += origin_x;
    *y += origin_y;
}
void hb_font_add_glyph_v_origin(hb_font_t *font, hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
{
    hb_position_t origin_x, origin_y;

    hb_font_get_glyph_v_origin_with_fallback(font, glyph, &origin_x, &origin_y);

    *x += origin_x;
    *y += origin_y;
}
void hb_font_subtract_glyph_h_origin(hb_font_t *font, hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
{
    hb_position_t origin_x, origin_y;

    hb_font_get_glyph_h_origin_with_fallback(font, glyph, &origin_x, &origin_y);

    *x -= origin_x;
    *y -= origin_y;
}
void hb_font_subtract_glyph_v_origin(hb_font_t *font, hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
{
    hb_position_t origin_x, origin_y;

    hb_font_get_glyph_v_origin_with_fallback(font, glyph, &origin_x, &origin_y);

    *x -= origin_x;
    *y -= origin_y;
}
