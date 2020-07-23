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

#ifndef HB_OT_H_IN
#error "Include <hb-ot.h> instead."
#endif

#ifndef HB_OT_SHAPE_H
#define HB_OT_SHAPE_H

#include "hb.h"

HB_BEGIN_DECLS

HB_EXTERN void
hb_ot_shape_plan_collect_lookups(hb_shape_plan_t *shape_plan, hb_tag_t table_tag, hb_set_t *lookup_indexes /* OUT */);

HB_EXTERN void _hb_ot_shape(hb_shape_plan_t *shape_plan,
                            hb_font_t *font,
                            hb_buffer_t *buffer,
                            const hb_feature_t *features,
                            unsigned int num_features);

typedef struct hb_ot_shape_plan_t hb_ot_shape_plan_t;
typedef struct hb_ot_map_t hb_ot_map_t;
HB_EXTERN const hb_ot_map_t *hb_ot_shape_plan_get_ot_map(const hb_ot_shape_plan_t *plan);
HB_EXTERN const void *hb_ot_shape_plan_get_data(const hb_ot_shape_plan_t *plan);
HB_EXTERN hb_script_t hb_ot_shape_plan_get_script(const hb_ot_shape_plan_t *plan);
HB_EXTERN bool hb_ot_shape_plan_has_gpos_mark(const hb_ot_shape_plan_t *plan);

typedef struct hb_ot_shape_planner_t hb_ot_shape_planner_t;
typedef struct hb_ot_map_builder_t hb_ot_map_builder_t;
HB_EXTERN hb_ot_map_builder_t *hb_ot_shape_planner_get_ot_map(hb_ot_shape_planner_t *planner);
HB_EXTERN hb_script_t hb_ot_shape_planner_get_script(hb_ot_shape_planner_t *planner);

HB_END_DECLS

#endif /* HB_OT_SHAPE_H */
