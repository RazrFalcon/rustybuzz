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

#include "hb.hh"
#include "hb-shape-plan.hh"
#include "hb-buffer.hh"

/**
 * SECTION:hb-shape-plan
 * @title: hb-shape-plan
 * @short_description: Object representing a shaping plan
 * @include: hb.h
 *
 * Shape plans are not used for shaping directly, but can be access to query
 * certain information about how shaping will perform given a set of input
 * parameters (script, language, direction, features, etc.)
 * Most client would not need to deal with shape plans directly.
 **/

/*
 * rb_shape_plan_t
 */

rb_shape_plan_t *rb_shape_plan_create(rb_face_t *face,
                                      const rb_segment_properties_t *props,
                                      const rb_feature_t *user_features,
                                      unsigned int num_user_features,
                                      const int *coords,
                                      unsigned int num_coords)
{
    assert(props->direction != RB_DIRECTION_INVALID);

    rb_shape_plan_t *shape_plan;

    unsigned int variations_index[2] = {0};
    for (unsigned int table_index = 0; table_index < 2; table_index++)
        rb_ot_layout_table_find_feature_variations(
            face, table_tags[table_index], coords, num_coords, &variations_index[table_index]);

    if (unlikely(!props))
        goto bail;
    if (!(shape_plan = rb_object_create<rb_shape_plan_t>()))
        goto bail;

    if (unlikely(!face))
        face = rb_face_get_empty();
    rb_face_make_immutable(face);
    shape_plan->face_unsafe = face;

    if (unlikely(!shape_plan->ot.init0(face, props, user_features, num_user_features, variations_index)))
        goto bail3;

    return shape_plan;

bail3:
    free(shape_plan);
bail:
    return rb_shape_plan_get_empty();
}

/**
 * rb_shape_plan_get_empty:
 *
 *
 *
 * Return value: (transfer full):
 *
 * Since: 0.9.7
 **/
rb_shape_plan_t *rb_shape_plan_get_empty()
{
    return const_cast<rb_shape_plan_t *>(&Null(rb_shape_plan_t));
}

/**
 * rb_shape_plan_reference: (skip)
 * @shape_plan: a shape plan.
 *
 *
 *
 * Return value: (transfer full):
 *
 * Since: 0.9.7
 **/
rb_shape_plan_t *rb_shape_plan_reference(rb_shape_plan_t *shape_plan)
{
    return rb_object_reference(shape_plan);
}

/**
 * rb_shape_plan_destroy: (skip)
 * @shape_plan: a shape plan.
 *
 *
 *
 * Since: 0.9.7
 **/
void rb_shape_plan_destroy(rb_shape_plan_t *shape_plan)
{
    if (!rb_object_destroy(shape_plan))
        return;

    shape_plan->ot.fini();
    free(shape_plan);
}

/**
 * rb_shape_plan_execute:
 * @shape_plan: a shape plan.
 * @font: a font.
 * @buffer: a buffer.
 * @features: (array length=num_features):
 * @num_features:
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.7
 **/
rb_bool_t rb_shape_plan_execute(rb_shape_plan_t *shape_plan,
                                rb_font_t *font,
                                rb_buffer_t *buffer,
                                const rb_feature_t *features,
                                unsigned int num_features)
{
    if (unlikely(!rb_buffer_get_length(buffer)))
        return true;

    if (unlikely(rb_object_is_inert(shape_plan)))
        return false;

    assert(shape_plan->face_unsafe == rb_font_get_face(font));

    _rb_ot_shape(shape_plan, font, buffer, features, num_features);
    return true;
}
