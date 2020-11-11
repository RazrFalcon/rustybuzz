/*
 * Copyright © 2013  Red Hat, Inc.
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
 */

#ifndef RB_OT_H_IN
#error "Include <hb-ot.h> instead."
#endif

#ifndef RB_OT_SHAPE_H
#define RB_OT_SHAPE_H

#include "hb.h"

RB_BEGIN_DECLS

typedef struct rb_shape_plan_t rb_shape_plan_t;

RB_EXTERN void _rb_ot_shape(rb_shape_plan_t *shape_plan,
                            rb_face_t *face,
                            rb_buffer_t *buffer,
                            const rb_feature_t *features,
                            unsigned int num_features);

typedef struct rb_ot_shape_plan_t rb_ot_shape_plan_t;
typedef struct rb_ot_complex_shaper_t rb_ot_complex_shaper_t;
typedef struct rb_ot_map_t rb_ot_map_t;
RB_EXTERN const rb_ot_complex_shaper_t *rb_ot_shape_plan_get_ot_complex_shaper(const rb_ot_shape_plan_t *plan);
RB_EXTERN const rb_ot_map_t *rb_ot_shape_plan_get_ot_map(const rb_ot_shape_plan_t *plan);
RB_EXTERN const void *rb_ot_shape_plan_get_data(const rb_ot_shape_plan_t *plan);
RB_EXTERN rb_script_t rb_ot_shape_plan_get_script(const rb_ot_shape_plan_t *plan);
RB_EXTERN rb_direction_t rb_ot_shape_plan_get_direction(const rb_ot_shape_plan_t *plan);
RB_EXTERN bool rb_ot_shape_plan_has_gpos_mark(const rb_ot_shape_plan_t *plan);

typedef struct rb_ot_shape_planner_t rb_ot_shape_planner_t;
typedef struct rb_ot_map_builder_t rb_ot_map_builder_t;
RB_EXTERN rb_ot_map_builder_t *rb_ot_shape_planner_get_ot_map(rb_ot_shape_planner_t *planner);
RB_EXTERN rb_script_t rb_ot_shape_planner_get_script(const rb_ot_shape_planner_t *planner);

RB_END_DECLS

#endif /* RB_OT_SHAPE_H */
