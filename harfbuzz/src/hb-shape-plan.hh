/*
 * Copyright © 2012,2018  Google, Inc.
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
 * Google Author(s): Behdad Esfahbod
 */

#ifndef RB_SHAPE_PLAN_HH
#define RB_SHAPE_PLAN_HH

#include "hb.hh"
#include "hb-ot-shape.hh"

struct rb_shape_plan_t
{
    rb_object_header_t header;
    rb_face_t *face_unsafe; /* We don't carry a reference to face. */
    rb_ot_shape_plan_t ot;
};

rb_shape_plan_t *rb_shape_plan_create(rb_face_t *face,
                                      const rb_segment_properties_t *props,
                                      const rb_feature_t *user_features,
                                      unsigned int num_user_features,
                                      const int *coords,
                                      unsigned int num_coords);

rb_shape_plan_t *rb_shape_plan_get_empty(void);

void rb_shape_plan_destroy(rb_shape_plan_t *shape_plan);

rb_bool_t rb_shape_plan_execute(rb_shape_plan_t *shape_plan,
                                rb_font_t *font,
                                rb_buffer_t *buffer,
                                const rb_feature_t *features,
                                unsigned int num_features);

#endif /* RB_SHAPE_PLAN_HH */
