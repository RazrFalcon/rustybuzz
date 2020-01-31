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
#include "hb-shaper.hh"
#include "hb-font.hh"
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
 * hb_shape_plan_t
 */


/**
 * hb_shape_plan_create: (Xconstructor)
 * @face:
 * @props:
 * @user_features: (array length=num_user_features):
 * @num_user_features:
 * @shaper_list: (array zero-terminated=1):
 *
 *
 *
 * Return value: (transfer full):
 *
 * Since: 0.9.7
 **/
hb_shape_plan_t *
hb_shape_plan_create (hb_face_t                     *face,
		      const hb_segment_properties_t *props,
		      const hb_feature_t            *user_features,
		      unsigned int                   num_user_features)
{
  return hb_shape_plan_create2 (face, props,
				user_features, num_user_features,
				nullptr, 0);
}

hb_shape_plan_t *
hb_shape_plan_create2 (hb_face_t                     *face,
		       const hb_segment_properties_t *props,
		       const hb_feature_t            *user_features,
		       unsigned int                   num_user_features,
		       const int                     *coords,
		       unsigned int                   num_coords)
{
  assert (props->direction != HB_DIRECTION_INVALID);

  hb_shape_plan_t *shape_plan;

  if (unlikely (!props))
    return hb_shape_plan_get_empty ();
  if (!(shape_plan = hb_object_create<hb_shape_plan_t> ()))
    return hb_shape_plan_get_empty ();

  if (unlikely (!face))
    face = hb_face_get_empty ();
  hb_face_make_immutable (face);

  unsigned int variations_index[2];
  for (unsigned int table_index = 0; table_index < 2; table_index++)
    rb_ot_layout_table_find_feature_variations (face->rust_data,
                                                table_tags[table_index],
                                                coords,
                                                num_coords,
                                                &variations_index[table_index]);

  if (unlikely (!shape_plan->ot.init0 (face, variations_index, props, user_features, num_user_features, coords, num_coords)))
    return hb_shape_plan_get_empty ();

  return shape_plan;
}

/**
 * hb_shape_plan_get_empty:
 *
 *
 *
 * Return value: (transfer full):
 *
 * Since: 0.9.7
 **/
hb_shape_plan_t *
hb_shape_plan_get_empty ()
{
  return const_cast<hb_shape_plan_t *> (&Null(hb_shape_plan_t));
}

/**
 * hb_shape_plan_reference: (skip)
 * @shape_plan: a shape plan.
 *
 *
 *
 * Return value: (transfer full):
 *
 * Since: 0.9.7
 **/
hb_shape_plan_t *
hb_shape_plan_reference (hb_shape_plan_t *shape_plan)
{
  return hb_object_reference (shape_plan);
}

/**
 * hb_shape_plan_destroy: (skip)
 * @shape_plan: a shape plan.
 *
 *
 *
 * Since: 0.9.7
 **/
void
hb_shape_plan_destroy (hb_shape_plan_t *shape_plan)
{
  if (!hb_object_destroy (shape_plan)) return;

  shape_plan->ot.fini ();
  free (shape_plan);
}

/**
 * hb_shape_plan_execute:
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
hb_bool_t
hb_shape_plan_execute (hb_shape_plan_t    *shape_plan,
		       hb_font_t          *font,
		       hb_buffer_t        *buffer,
		       const hb_feature_t *features,
		       unsigned int        num_features)
{
  if (unlikely (!hb_buffer_get_length(buffer)))
    return true;

  if (unlikely (hb_object_is_inert (shape_plan)))
    return false;

  return _hb_ot_shape (shape_plan, font, buffer, features, num_features); 
}
