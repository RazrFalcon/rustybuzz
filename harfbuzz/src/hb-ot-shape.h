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
typedef struct rb_ot_map_t rb_ot_map_t;
typedef struct rb_ot_map_builder_t rb_ot_map_builder_t;
typedef struct rb_ot_shape_plan_t rb_ot_shape_plan_t;
typedef struct rb_ot_shape_planner_t rb_ot_shape_planner_t;
typedef struct rb_ot_complex_shaper_t rb_ot_complex_shaper_t;

typedef enum {
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_NONE = 0,
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_EARLY = 1,
    RB_OT_SHAPE_ZERO_WIDTH_MARKS_BY_GDEF_LATE = 2
} rb_ot_shape_zero_width_marks_mode_t;

RB_EXTERN void rb_ot_shape(rb_shape_plan_t *shape_plan,
                            rb_face_t *face,
                            rb_buffer_t *buffer,
                            const rb_feature_t *features,
                            unsigned int num_features);

// C++ -> Rust
RB_EXTERN const rb_ot_complex_shaper_t *rb_ot_shape_plan_get_ot_complex_shaper(const rb_ot_shape_plan_t *plan);
RB_EXTERN const rb_ot_map_t *rb_ot_shape_plan_get_ot_map(const rb_ot_shape_plan_t *plan);
RB_EXTERN const void *rb_ot_shape_plan_get_data(const rb_ot_shape_plan_t *plan);
RB_EXTERN rb_script_t rb_ot_shape_plan_get_script(const rb_ot_shape_plan_t *plan);
RB_EXTERN rb_direction_t rb_ot_shape_plan_get_direction(const rb_ot_shape_plan_t *plan);
RB_EXTERN bool rb_ot_shape_plan_has_gpos_mark(const rb_ot_shape_plan_t *plan);
RB_EXTERN rb_ot_map_builder_t *rb_ot_shape_planner_get_ot_map(rb_ot_shape_planner_t *planner);
RB_EXTERN rb_script_t rb_ot_shape_planner_get_script(const rb_ot_shape_planner_t *planner);
RB_EXTERN rb_direction_t rb_ot_shape_planner_get_direction(const rb_ot_shape_planner_t *planner);

// Rust -> C++
RB_EXTERN const rb_ot_complex_shaper_t *rb_ot_shape_complex_categorize(const rb_ot_shape_planner_t *planner);
RB_EXTERN const rb_ot_complex_shaper_t *rb_ot_complex_shaper_reconsider_shaper_if_applying_morx(const rb_ot_complex_shaper_t * shaper);
RB_EXTERN void rb_ot_complex_shaper_collect_features(const rb_ot_complex_shaper_t *shaper, rb_ot_shape_planner_t *planner);
RB_EXTERN void rb_ot_complex_shaper_override_features(const rb_ot_complex_shaper_t *shaper, rb_ot_shape_planner_t *planner);
RB_EXTERN rb_bool_t rb_ot_complex_shaper_data_create(const rb_ot_complex_shaper_t *shaper, const rb_ot_shape_plan_t *plan, void **data);
RB_EXTERN void rb_ot_complex_shaper_data_destroy(const rb_ot_complex_shaper_t *shaper, void *data);
RB_EXTERN void rb_ot_complex_shaper_preprocess_text(const rb_ot_complex_shaper_t *shaper, rb_ot_shape_plan_t *planner, rb_buffer_t *buffer, rb_face_t *face);
RB_EXTERN void rb_ot_complex_shaper_postprocess_glyphs(const rb_ot_complex_shaper_t *shaper, rb_ot_shape_plan_t *planner, rb_buffer_t *buffer, rb_face_t *face);
RB_EXTERN void rb_ot_complex_shaper_setup_masks(const rb_ot_complex_shaper_t *shaper, rb_ot_shape_plan_t *planner, rb_buffer_t *buffer, rb_face_t *face);
RB_EXTERN rb_tag_t rb_ot_complex_shaper_get_gpos_tag(const rb_ot_complex_shaper_t *shaper);
RB_EXTERN void rb_ot_complex_shaper_reorder_marks(const rb_ot_complex_shaper_t *shaper, const rb_ot_shape_plan_t *plan, rb_buffer_t *buffer, unsigned int start, unsigned int end);
RB_EXTERN rb_ot_shape_zero_width_marks_mode_t rb_ot_complex_shaper_get_zero_width_marks_mode(const rb_ot_complex_shaper_t *shaper);
RB_EXTERN rb_bool_t rb_ot_complex_shaper_get_fallback_position(const rb_ot_complex_shaper_t *shaper);

RB_END_DECLS

#endif /* RB_OT_SHAPE_H */
