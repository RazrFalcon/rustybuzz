/*
 * Copyright © 2010  Google, Inc.
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

#pragma once

#include "hb.hh"

#include "hb-aat-map.hh"
#include "hb-ot-map.hh"

struct hb_shape_plan_key_t;

struct hb_shape_plan_t
{
    hb_segment_properties_t props;
    const struct hb_ot_complex_shaper_t *shaper;
    rb_ot_map_t *map;
    hb_aat_map_t aat_map;
    const void *data;
    hb_mask_t frac_mask, numr_mask, dnom_mask;
    hb_mask_t rtlm_mask;
    hb_mask_t kern_mask;
    hb_mask_t trak_mask;

    bool requested_kerning : 1;
    bool requested_tracking : 1;
    bool has_frac : 1;
    bool has_gpos_mark : 1;
    bool zero_marks : 1;
    bool fallback_glyph_classes : 1;
    bool fallback_mark_positioning : 1;
    bool adjust_mark_positioning_when_zeroing : 1;

    bool apply_gpos : 1;
    bool apply_kern : 1;
    bool apply_kerx : 1;
    bool apply_morx : 1;
    bool apply_trak : 1;

    HB_INTERNAL bool init0(hb_face_t *face,
                           unsigned int *variations_index,
                           const hb_segment_properties_t *props,
                           const hb_feature_t *user_features,
                           unsigned int num_user_features,
                           const int *coords,
                           unsigned int num_coords);
    HB_INTERNAL void fini();

    HB_INTERNAL void substitute(hb_font_t *font, rb_buffer_t *buffer) const;
    HB_INTERNAL void position(hb_font_t *font, rb_buffer_t *buffer) const;
};

struct hb_shape_plan_t;

struct hb_ot_shape_planner_t
{
    /* In the order that they are filled in. */
    hb_face_t *face;
    hb_segment_properties_t props;
    rb_ot_map_builder_t *map;
    hb_aat_map_builder_t aat_map;
    bool apply_morx : 1;
    bool script_zero_marks : 1;
    bool script_fallback_mark_positioning : 1;
    const struct hb_ot_complex_shaper_t *shaper;

    HB_INTERNAL hb_ot_shape_planner_t(hb_face_t *face, const hb_segment_properties_t *props);
    HB_INTERNAL ~hb_ot_shape_planner_t();

    HB_INTERNAL void compile(hb_shape_plan_t &plan, unsigned int *variations_index);
};
