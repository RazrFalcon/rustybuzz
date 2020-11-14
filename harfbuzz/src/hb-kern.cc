/*
 * Copyright © 1998-2004  David Turner and Werner Lemberg
 * Copyright © 2006  Behdad Esfahbod
 * Copyright © 2007,2008,2009  Red Hat, Inc.
 * Copyright © 2012,2013  Google, Inc.
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
 * Google Author(s): Behdad Esfahbod
 */

#include "hb.hh"

#include "hb-face.hh"
#include "hb-ot-kern-table.hh"


rb_bool_t rb_ot_layout_has_kerning(rb_face_t *face)
{
    return rb_face_get_kern_table(face)->has_data();
}

rb_bool_t rb_ot_layout_has_machine_kerning(rb_face_t *face)
{
    return rb_face_get_kern_table(face)->has_state_machine();
}

rb_bool_t rb_ot_layout_has_cross_kerning(rb_face_t *face)
{
    return rb_face_get_kern_table(face)->has_cross_stream();
}

void rb_ot_layout_kern(const rb_shape_plan_t *plan, rb_face_t *face, rb_buffer_t *buffer)
{
    rb_blob_t *blob = rb_face_get_table_blob(face, RB_OT_TAG_kern);
    const AAT::kern &kern = *blob->as<AAT::kern>();
    AAT::rb_aat_apply_context_t c(plan, face, buffer, blob);
    kern.apply(&c);
}
