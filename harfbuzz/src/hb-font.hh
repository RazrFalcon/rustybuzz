/*
 * Copyright © 2009  Red Hat, Inc.
 * Copyright © 2011  Google, Inc.
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

#ifndef HB_FONT_HH
#define HB_FONT_HH

#include "hb.hh"

#include "hb-face.hh"
#include "hb-ot-font.h"

/*
 * hb_font_t
 */

struct hb_font_t
{
    hb_object_header_t header;

    hb_face_t *face;

    int32_t upem;

    unsigned int x_ppem;
    unsigned int y_ppem;

    float ptem;

    /* Font variation coordinates. */
    unsigned int num_coords;
    int *coords;
    float *design_coords;

    hb_bool_t get_font_h_extents(hb_font_extents_t *extents)
    {
        memset(extents, 0, sizeof(*extents));
        return hb_ot_get_font_h_extents(this, extents);
    }
    hb_bool_t get_font_v_extents(hb_font_extents_t *extents)
    {
        memset(extents, 0, sizeof(*extents));
        return hb_ot_get_font_v_extents(this, extents);
    }

    bool has_glyph(hb_codepoint_t unicode)
    {
        hb_codepoint_t glyph;
        return get_nominal_glyph(unicode, &glyph);
    }

    hb_bool_t get_nominal_glyph(hb_codepoint_t unicode, hb_codepoint_t *glyph)
    {
        *glyph = 0;
        return hb_ot_get_nominal_glyph(this, unicode, glyph);
    }

    unsigned int get_nominal_glyphs(unsigned int count,
                                    const hb_codepoint_t *first_unicode,
                                    unsigned int unicode_stride,
                                    hb_codepoint_t *first_glyph,
                                    unsigned int glyph_stride)
    {
        return hb_ot_get_nominal_glyphs(this, count, first_unicode, unicode_stride, first_glyph, glyph_stride);
    }

    hb_bool_t get_variation_glyph(hb_codepoint_t unicode, hb_codepoint_t variation_selector, hb_codepoint_t *glyph)
    {
        *glyph = 0;
        return hb_ot_get_variation_glyph(this, unicode, variation_selector, glyph);
    }

    hb_position_t get_glyph_h_advance(hb_codepoint_t glyph)
    {
        hb_position_t ret;
        get_glyph_h_advances(1, &glyph, 0, &ret, 0);
        return ret;
    }

    hb_position_t get_glyph_v_advance(hb_codepoint_t glyph)
    {
        hb_position_t ret;
        get_glyph_v_advances(1, &glyph, 0, &ret, 0);
        return ret;
    }

    void get_glyph_h_advances(unsigned int count,
                              const hb_codepoint_t *first_glyph,
                              unsigned int glyph_stride,
                              hb_position_t *first_advance,
                              unsigned int advance_stride)
    {
        return hb_ot_get_glyph_h_advances(this, count, first_glyph, glyph_stride, first_advance, advance_stride);
    }

    void get_glyph_v_advances(unsigned int count,
                              const hb_codepoint_t *first_glyph,
                              unsigned int glyph_stride,
                              hb_position_t *first_advance,
                              unsigned int advance_stride)
    {
        return hb_ot_get_glyph_v_advances(this, count, first_glyph, glyph_stride, first_advance, advance_stride);
    }

    hb_bool_t get_glyph_h_origin(hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
    {
        *x = *y = 0;
        return true;
    }

    hb_bool_t get_glyph_v_origin(hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
    {
        *x = *y = 0;
        return hb_ot_get_glyph_v_origin(this, glyph, x, y);
    }

    hb_bool_t get_glyph_extents(hb_codepoint_t glyph, hb_glyph_extents_t *extents)
    {
        memset(extents, 0, sizeof(*extents));
        return hb_ot_get_glyph_extents(this, glyph, extents);
    }

    hb_bool_t
    get_glyph_contour_point(hb_codepoint_t glyph, unsigned int point_index, hb_position_t *x, hb_position_t *y)
    {
        *x = *y = 0;
        return false;
    }

    hb_bool_t get_glyph_name(hb_codepoint_t glyph, char *name, unsigned int size)
    {
        if (size)
            *name = '\0';
        return hb_ot_get_glyph_name(this, glyph, name, size);
    }

    /* A bit higher-level, and with fallback */

    void get_h_extents_with_fallback(hb_font_extents_t *extents)
    {
        if (!get_font_h_extents(extents)) {
            extents->ascender = upem * .8;
            extents->descender = extents->ascender - upem;
            extents->line_gap = 0;
        }
    }
    void get_v_extents_with_fallback(hb_font_extents_t *extents)
    {
        if (!get_font_v_extents(extents)) {
            extents->ascender = upem / 2;
            extents->descender = extents->ascender - upem;
            extents->line_gap = 0;
        }
    }

    void get_extents_for_direction(hb_direction_t direction, hb_font_extents_t *extents)
    {
        if (likely(HB_DIRECTION_IS_HORIZONTAL(direction)))
            get_h_extents_with_fallback(extents);
        else
            get_v_extents_with_fallback(extents);
    }

    void
    get_glyph_advance_for_direction(hb_codepoint_t glyph, hb_direction_t direction, hb_position_t *x, hb_position_t *y)
    {
        *x = *y = 0;
        if (likely(HB_DIRECTION_IS_HORIZONTAL(direction)))
            *x = get_glyph_h_advance(glyph);
        else
            *y = get_glyph_v_advance(glyph);
    }
    void get_glyph_advances_for_direction(hb_direction_t direction,
                                          unsigned int count,
                                          const hb_codepoint_t *first_glyph,
                                          unsigned glyph_stride,
                                          hb_position_t *first_advance,
                                          unsigned advance_stride)
    {
        if (likely(HB_DIRECTION_IS_HORIZONTAL(direction)))
            get_glyph_h_advances(count, first_glyph, glyph_stride, first_advance, advance_stride);
        else
            get_glyph_v_advances(count, first_glyph, glyph_stride, first_advance, advance_stride);
    }

    void guess_v_origin_minus_h_origin(hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
    {
        *x = get_glyph_h_advance(glyph) / 2;

        /* TODO cache this somehow?! */
        hb_font_extents_t extents;
        get_h_extents_with_fallback(&extents);
        *y = extents.ascender;
    }

    void get_glyph_h_origin_with_fallback(hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
    {
        if (!get_glyph_h_origin(glyph, x, y) && get_glyph_v_origin(glyph, x, y)) {
            hb_position_t dx, dy;
            guess_v_origin_minus_h_origin(glyph, &dx, &dy);
            *x -= dx;
            *y -= dy;
        }
    }
    void get_glyph_v_origin_with_fallback(hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
    {
        if (!get_glyph_v_origin(glyph, x, y) && get_glyph_h_origin(glyph, x, y)) {
            hb_position_t dx, dy;
            guess_v_origin_minus_h_origin(glyph, &dx, &dy);
            *x += dx;
            *y += dy;
        }
    }

    void
    get_glyph_origin_for_direction(hb_codepoint_t glyph, hb_direction_t direction, hb_position_t *x, hb_position_t *y)
    {
        if (likely(HB_DIRECTION_IS_HORIZONTAL(direction)))
            get_glyph_h_origin_with_fallback(glyph, x, y);
        else
            get_glyph_v_origin_with_fallback(glyph, x, y);
    }

    void add_glyph_h_origin(hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
    {
        hb_position_t origin_x, origin_y;

        get_glyph_h_origin_with_fallback(glyph, &origin_x, &origin_y);

        *x += origin_x;
        *y += origin_y;
    }
    void add_glyph_v_origin(hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
    {
        hb_position_t origin_x, origin_y;

        get_glyph_v_origin_with_fallback(glyph, &origin_x, &origin_y);

        *x += origin_x;
        *y += origin_y;
    }
    void
    add_glyph_origin_for_direction(hb_codepoint_t glyph, hb_direction_t direction, hb_position_t *x, hb_position_t *y)
    {
        hb_position_t origin_x, origin_y;

        get_glyph_origin_for_direction(glyph, direction, &origin_x, &origin_y);

        *x += origin_x;
        *y += origin_y;
    }

    void subtract_glyph_h_origin(hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
    {
        hb_position_t origin_x, origin_y;

        get_glyph_h_origin_with_fallback(glyph, &origin_x, &origin_y);

        *x -= origin_x;
        *y -= origin_y;
    }
    void subtract_glyph_v_origin(hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y)
    {
        hb_position_t origin_x, origin_y;

        get_glyph_v_origin_with_fallback(glyph, &origin_x, &origin_y);

        *x -= origin_x;
        *y -= origin_y;
    }
    void subtract_glyph_origin_for_direction(hb_codepoint_t glyph,
                                             hb_direction_t direction,
                                             hb_position_t *x,
                                             hb_position_t *y)
    {
        hb_position_t origin_x, origin_y;

        get_glyph_origin_for_direction(glyph, direction, &origin_x, &origin_y);

        *x -= origin_x;
        *y -= origin_y;
    }

    hb_bool_t get_glyph_extents_for_origin(hb_codepoint_t glyph, hb_direction_t direction, hb_glyph_extents_t *extents)
    {
        hb_bool_t ret = get_glyph_extents(glyph, extents);

        if (ret)
            subtract_glyph_origin_for_direction(glyph, direction, &extents->x_bearing, &extents->y_bearing);

        return ret;
    }

    hb_bool_t get_glyph_contour_point_for_origin(
        hb_codepoint_t glyph, unsigned int point_index, hb_direction_t direction, hb_position_t *x, hb_position_t *y)
    {
        hb_bool_t ret = get_glyph_contour_point(glyph, point_index, x, y);

        if (ret)
            subtract_glyph_origin_for_direction(glyph, direction, x, y);

        return ret;
    }

    /* Generates gidDDD if glyph has no name. */
    void glyph_to_string(hb_codepoint_t glyph, char *s, unsigned int size)
    {
        if (get_glyph_name(glyph, s, size))
            return;

        if (size && snprintf(s, size, "gid%u", glyph) < 0)
            *s = '\0';
    }
};
DECLARE_NULL_INSTANCE(hb_font_t);

#endif /* HB_FONT_HH */
