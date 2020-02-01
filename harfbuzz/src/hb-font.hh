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

#pragma once

#include "hb.hh"

#include "hb-face.hh"
#include "hb-shaper.hh"

/*
 * hb_font_t
 */

struct hb_font_t
{
    hb_object_header_t header;

    hb_font_t *parent;
    hb_face_t *face;

    int32_t x_scale;
    int32_t y_scale;
    int64_t x_mult;
    int64_t y_mult;

    unsigned int x_ppem;
    unsigned int y_ppem;

    float ptem;

    /* Font variation coordinates. */
    unsigned int num_coords;
    int *coords;

    const void *rust_data;
    void *user_data;
    hb_destroy_func_t destroy;

    /* Convert from font-space to user-space */
    int64_t dir_mult(hb_direction_t direction)
    {
        return HB_DIRECTION_IS_VERTICAL(direction) ? y_mult : x_mult;
    }
    hb_position_t em_scale_x(int16_t v)
    {
        return em_mult(v, x_mult);
    }
    hb_position_t em_scale_y(int16_t v)
    {
        return em_mult(v, y_mult);
    }
    hb_position_t em_scalef_x(float v)
    {
        return em_scalef(v, x_scale);
    }
    hb_position_t em_scalef_y(float v)
    {
        return em_scalef(v, y_scale);
    }
    float em_fscale_x(int16_t v)
    {
        return em_fscale(v, x_scale);
    }
    float em_fscale_y(int16_t v)
    {
        return em_fscale(v, y_scale);
    }
    hb_position_t em_scale_dir(int16_t v, hb_direction_t direction)
    {
        return em_mult(v, dir_mult(direction));
    }

    /* Convert from parent-font user-space to our user-space */
    hb_position_t parent_scale_x_distance(hb_position_t v)
    {
        if (unlikely(parent && parent->x_scale != x_scale))
            return (hb_position_t)(v * (int64_t)this->x_scale / this->parent->x_scale);
        return v;
    }
    hb_position_t parent_scale_y_distance(hb_position_t v)
    {
        if (unlikely(parent && parent->y_scale != y_scale))
            return (hb_position_t)(v * (int64_t)this->y_scale / this->parent->y_scale);
        return v;
    }
    hb_position_t parent_scale_x_position(hb_position_t v)
    {
        return parent_scale_x_distance(v);
    }
    hb_position_t parent_scale_y_position(hb_position_t v)
    {
        return parent_scale_y_distance(v);
    }

    void parent_scale_position(hb_position_t *x, hb_position_t *y)
    {
        *x = parent_scale_x_position(*x);
        *y = parent_scale_y_position(*y);
    }

    /* Public getters */

    hb_bool_t get_font_h_extents(hb_font_extents_t *extents);

    bool has_glyph(hb_codepoint_t unicode)
    {
        hb_codepoint_t glyph;
        return get_nominal_glyph(unicode, &glyph);
    }

    hb_bool_t get_nominal_glyph(hb_codepoint_t unicode, hb_codepoint_t *glyph);
    unsigned int get_nominal_glyphs(unsigned int count,
                                    const hb_codepoint_t *first_unicode,
                                    unsigned int unicode_stride,
                                    hb_codepoint_t *first_glyph,
                                    unsigned int glyph_stride);

    hb_bool_t get_variation_glyph(hb_codepoint_t unicode, hb_codepoint_t variation_selector, hb_codepoint_t *glyph);

    hb_position_t get_glyph_h_advance(hb_codepoint_t glyph);

    hb_position_t get_glyph_v_advance(hb_codepoint_t glyph);

    void get_glyph_h_advances(unsigned int count,
                              const hb_codepoint_t *first_glyph,
                              unsigned int glyph_stride,
                              hb_position_t *first_advance,
                              unsigned int advance_stride);

    void get_glyph_v_advances(unsigned int count,
                              const hb_codepoint_t *first_glyph,
                              unsigned int glyph_stride,
                              hb_position_t *first_advance,
                              unsigned int advance_stride);

    hb_bool_t get_glyph_h_origin(hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y);

    hb_bool_t get_glyph_v_origin(hb_codepoint_t glyph, hb_position_t *x, hb_position_t *y);

    hb_bool_t get_glyph_extents(hb_codepoint_t glyph, hb_glyph_extents_t *extents);

    hb_bool_t
    get_glyph_contour_point(hb_codepoint_t glyph, unsigned int point_index, hb_position_t *x, hb_position_t *y);

    hb_bool_t get_glyph_name(hb_codepoint_t glyph, char *name, unsigned int size);

    /* A bit higher-level, and with fallback */

    void get_h_extents_with_fallback(hb_font_extents_t *extents)
    {
        if (!get_font_h_extents(extents)) {
            extents->ascender = y_scale * .8;
            extents->descender = extents->ascender - y_scale;
            extents->line_gap = 0;
        }
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

    void mults_changed()
    {
        signed upem = hb_face_get_upem(face);
        x_mult = ((int64_t)x_scale << 16) / upem;
        y_mult = ((int64_t)y_scale << 16) / upem;
    }

    hb_position_t em_mult(int16_t v, int64_t mult)
    {
        return (hb_position_t)((v * mult) >> 16);
    }
    hb_position_t em_scalef(float v, int scale)
    {
        return (hb_position_t)roundf(v * scale / hb_face_get_upem(face));
    }
    float em_fscale(int16_t v, int scale)
    {
        return (float)v * scale / hb_face_get_upem(face);
    }
};
DECLARE_NULL_INSTANCE(hb_font_t);
