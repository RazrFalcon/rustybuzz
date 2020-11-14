/*
 * Copyright © 2009  Red Hat, Inc.
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

#ifndef RB_H_IN
#error "Include <hb.h> instead."
#endif

#ifndef RB_FACE_H
#define RB_FACE_H

#include "hb-common.h"
#include "hb-blob.h"

RB_BEGIN_DECLS

typedef struct rb_face_t rb_face_t;

RB_EXTERN rb_blob_t *rb_face_get_table_blob(rb_face_t *face, rb_tag_t tag);

RB_EXTERN rb_blob_t *rb_face_sanitize_table(rb_blob_t *blob, rb_tag_t tag, unsigned int glyph_count);

RB_EXTERN unsigned int rb_face_get_glyph_count(const rb_face_t *face);

RB_EXTERN float rb_face_get_ptem(rb_face_t *face);

RB_EXTERN rb_bool_t rb_face_get_glyph_contour_point_for_origin(rb_face_t *face,
                                                               rb_codepoint_t glyph,
                                                               unsigned int point_index,
                                                               rb_direction_t direction,
                                                               rb_position_t *x,
                                                               rb_position_t *y);

RB_END_DECLS

#endif /* RB_FACE_H */
