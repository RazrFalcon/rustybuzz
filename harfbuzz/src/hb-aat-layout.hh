/*
 * Copyright © 2017  Google, Inc.
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

#ifndef RB_AAT_LAYOUT_HH
#define RB_AAT_LAYOUT_HH

#include "hb.hh"

#include "hb-ot-shape.hh"

struct rb_aat_feature_mapping_t
{
    rb_tag_t otFeatureTag;
    rb_aat_layout_feature_type_t aatFeatureType;
    rb_aat_layout_feature_selector_t selectorToEnable;
    rb_aat_layout_feature_selector_t selectorToDisable;

    int cmp(rb_tag_t key) const
    {
        return key < otFeatureTag ? -1 : key > otFeatureTag ? 1 : 0;
    }
};

RB_INTERNAL const rb_aat_feature_mapping_t *rb_aat_layout_find_feature_mapping(rb_tag_t tag);

RB_INTERNAL void rb_aat_layout_compile_map(const rb_aat_map_builder_t *mapper, rb_aat_map_t *map);

RB_INTERNAL void rb_aat_layout_substitute(const rb_ot_shape_plan_t *plan, rb_face_t *face, rb_buffer_t *buffer);

RB_INTERNAL void rb_aat_layout_zero_width_deleted_glyphs(rb_buffer_t *buffer);

RB_INTERNAL void rb_aat_layout_remove_deleted_glyphs(rb_buffer_t *buffer);

RB_INTERNAL void rb_aat_layout_position(const rb_ot_shape_plan_t *plan, rb_face_t *face, rb_buffer_t *buffer);

RB_INTERNAL void rb_aat_layout_track(const rb_ot_shape_plan_t *plan, rb_face_t *face, rb_buffer_t *buffer);

#endif /* RB_AAT_LAYOUT_HH */
