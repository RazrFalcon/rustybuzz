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

#ifndef RB_OT_SHAPE_HH
#define RB_OT_SHAPE_HH

#include "hb.hh"

#include "hb-ot-map.hh"
#include "hb-aat-map.hh"

struct rb_ot_shape_plan_t
{
    rb_segment_properties_t props;
    const struct rb_ot_complex_shaper_t *shaper;
    rb_ot_map_t map;
    rb_aat_map_t aat_map;
    const void *data;
    rb_mask_t frac_mask, numr_mask, dnom_mask;
    rb_mask_t rtlm_mask;
    rb_mask_t kern_mask;
    rb_mask_t trak_mask;

    bool requested_kerning : 1;
    bool requested_tracking : 1;
    bool has_frac : 1;
    bool has_vert : 1;
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

    void collect_lookups(rb_tag_t table_tag, rb_set_t *lookups) const
    {
        unsigned int table_index;
        switch (table_tag) {
        case RB_OT_TAG_GSUB:
            table_index = 0;
            break;
        case RB_OT_TAG_GPOS:
            table_index = 1;
            break;
        default:
            return;
        }
        map.collect_lookups(table_index, lookups);
    }

    RB_INTERNAL bool init0(rb_face_t *face,
                           const rb_segment_properties_t *props,
                           const rb_feature_t *user_features,
                           unsigned int num_user_features,
                           unsigned int *variations_index);
    RB_INTERNAL void fini();

    RB_INTERNAL void substitute(rb_font_t *font, rb_buffer_t *buffer) const;
    RB_INTERNAL void position(rb_font_t *font, rb_buffer_t *buffer) const;
};

struct rb_shape_plan_t;

struct rb_ot_shape_planner_t
{
    /* In the order that they are filled in. */
    rb_face_t *face;
    rb_segment_properties_t props;
    rb_ot_map_builder_t map;
    rb_aat_map_builder_t aat_map;
    bool apply_morx : 1;
    bool script_zero_marks : 1;
    bool script_fallback_mark_positioning : 1;
    const struct rb_ot_complex_shaper_t *shaper;

    RB_INTERNAL rb_ot_shape_planner_t(rb_face_t *face, const rb_segment_properties_t *props);

    RB_INTERNAL void compile(rb_ot_shape_plan_t &plan, unsigned int *variations_index);
};

#endif /* RB_OT_SHAPE_HH */
