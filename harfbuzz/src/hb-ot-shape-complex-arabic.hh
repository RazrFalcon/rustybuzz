/*
 * Copyright © 2015  Mozilla Foundation.
 * Copyright © 2015  Google, Inc.
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
 * Mozilla Author(s): Jonathan Kew
 * Google Author(s): Behdad Esfahbod
 */

#ifndef HB_OT_SHAPE_COMPLEX_ARABIC_HH
#define HB_OT_SHAPE_COMPLEX_ARABIC_HH

#include "hb.hh"

#include "hb-ot-shape-complex.hh"

typedef struct hb_ot_arabic_shape_plan_t hb_ot_arabic_shape_plan_t;

extern "C" {
HB_EXTERN void *hb_ot_complex_data_create_arabic(const hb_ot_shape_plan_t *plan);

HB_EXTERN void hb_ot_complex_data_destroy_arabic(void *data);

HB_EXTERN void hb_ot_complex_setup_masks_arabic_plan(const hb_ot_arabic_shape_plan_t *arabic_plan,
                                                     hb_buffer_t *buffer,
                                                     hb_script_t script);

HB_EXTERN void hb_ot_complex_collect_features_arabic(hb_ot_shape_planner_t *plan);
HB_EXTERN void
hb_ot_complex_postprocess_glyphs_arabic(const hb_ot_shape_plan_t *plan, hb_buffer_t *buffer, hb_font_t *font);
HB_EXTERN void hb_ot_complex_setup_masks_arabic(const hb_ot_shape_plan_t *plan, hb_buffer_t *buffer, hb_font_t *font);
HB_EXTERN void hb_ot_complex_reorder_marks_arabic(const hb_ot_shape_plan_t *plan,
                                                  hb_buffer_t *buffer,
                                                  unsigned int start,
                                                  unsigned int end);
}

#endif /* HB_OT_SHAPE_COMPLEX_ARABIC_HH */
