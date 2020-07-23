/*
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
 * Google Author(s): Behdad Esfahbod
 */

#ifndef RB_H_IN
#error "Include <hb.h> instead."
#endif

#ifndef RB_SHAPE_PLAN_H
#define RB_SHAPE_PLAN_H

#include "hb-common.h"
#include "hb-font.h"
#include "hb-buffer.h"

RB_BEGIN_DECLS

typedef struct rb_shape_plan_t rb_shape_plan_t;

RB_EXTERN rb_shape_plan_t *rb_shape_plan_create(rb_face_t *face,
                                                const rb_segment_properties_t *props,
                                                const rb_feature_t *user_features,
                                                unsigned int num_user_features,
                                                const int *coords,
                                                unsigned int num_coords);

RB_EXTERN rb_shape_plan_t *rb_shape_plan_get_empty(void);

RB_EXTERN rb_shape_plan_t *rb_shape_plan_reference(rb_shape_plan_t *shape_plan);

RB_EXTERN void rb_shape_plan_destroy(rb_shape_plan_t *shape_plan);

RB_EXTERN rb_bool_t rb_shape_plan_execute(rb_shape_plan_t *shape_plan,
                                          rb_font_t *font,
                                          rb_buffer_t *buffer,
                                          const rb_feature_t *features,
                                          unsigned int num_features);

RB_END_DECLS

#endif /* RB_SHAPE_PLAN_H */
