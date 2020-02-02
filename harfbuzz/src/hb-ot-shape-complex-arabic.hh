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

#pragma once

#include "hb.hh"

#include "hb-ot-shape-complex.hh"

struct arabic_shape_plan_t;

HB_INTERNAL void *data_create_arabic(const hb_shape_plan_t *plan);

HB_INTERNAL void data_destroy_arabic(void *data);

HB_INTERNAL void
setup_masks_arabic_plan(const arabic_shape_plan_t *arabic_plan, rb_buffer_t *buffer, hb_script_t script);

extern "C" {
void hb_complex_arabic_record_stch(const hb_shape_plan_t *plan, hb_font_t *font HB_UNUSED, rb_buffer_t *buffer);

void hb_complex_arabic_fallback_shape(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer);

void rb_complex_arabic_collect_features(rb_ot_map_builder_t *map, hb_tag_t script);

void rb_complex_arabic_joining(rb_buffer_t *buffer);

void rb_complex_arabic_apply_stch(rb_buffer_t *buffer, hb_font_t *font);

void rb_complex_arabic_reorder_marks(rb_buffer_t *buffer, unsigned int start, unsigned int end);
}
