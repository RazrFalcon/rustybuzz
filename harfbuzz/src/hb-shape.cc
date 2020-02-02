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

#include "hb-buffer.hh"
#include "hb-font.hh"
#include "hb-machinery.hh"
#include "hb-shaper.hh"

/**
 * SECTION:hb-shape
 * @title: hb-shape
 * @short_description: Conversion of text strings into positioned glyphs
 * @include: hb.h
 *
 * Shaping is the central operation of HarfBuzz. Shaping operates on buffers,
 * which are sequences of Unicode characters that use the same font and have
 * the same text direction, script, and language. After shaping the buffer
 * contains the output glyphs and their positions.
 **/

/**
 * hb_shape_full:
 * @font: an #hb_font_t to use for shaping
 * @buffer: an #hb_buffer_t to shape
 * @features: (array length=num_features) (allow-none): an array of user
 *    specified #hb_feature_t or %NULL
 * @num_features: the length of @features array
 * @shaper_list: (array zero-terminated=1) (allow-none): a %NULL-terminated
 *    array of shapers to use or %NULL
 *
 * See hb_shape() for details. If @shaper_list is not %NULL, the specified
 * shapers will be used in the given order, otherwise the default shapers list
 * will be used.
 *
 * Return value: false if all shapers failed, true otherwise
 *
 * Since: 0.9.2
 **/
hb_bool_t hb_shape_full(hb_font_t *font, hb_buffer_t *buffer, const hb_feature_t *features, unsigned int num_features)
{
    hb_segment_properties_t props = hb_buffer_get_segment_properties(buffer);
    hb_shape_plan_t *shape_plan =
        hb_shape_plan_create2(font->face, &props, features, num_features, font->coords, font->num_coords);
    hb_bool_t res = hb_shape_plan_execute(shape_plan, font, buffer, features, num_features);
    hb_shape_plan_destroy(shape_plan);

    return res;
}

/**
 * hb_shape:
 * @font: an #hb_font_t to use for shaping
 * @buffer: an #hb_buffer_t to shape
 * @features: (array length=num_features) (allow-none): an array of user
 *    specified #hb_feature_t or %NULL
 * @num_features: the length of @features array
 *
 * Shapes @buffer using @font turning its Unicode characters content to
 * positioned glyphs. If @features is not %NULL, it will be used to control the
 * features applied during shaping. If two @features have the same tag but
 * overlapping ranges the value of the feature with the higher index takes
 * precedence.
 *
 * Since: 0.9.2
 **/
void hb_shape(hb_font_t *font, hb_buffer_t *buffer, const hb_feature_t *features, unsigned int num_features)
{
    hb_shape_full(font, buffer, features, num_features);
}
