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

#ifndef HB_H_IN
#error "Include <hb.h> instead."
#endif

#pragma once

#include "hb-blob.h"
#include "hb-common.h"
#include "hb-set.h"

HB_BEGIN_DECLS

HB_EXTERN unsigned int hb_face_count(hb_blob_t *blob);

/*
 * hb_face_t
 */

typedef struct rb_ttf_parser_t rb_ttf_parser_t;

typedef struct hb_face_t hb_face_t;

HB_EXTERN hb_face_t *hb_face_create(hb_blob_t *blob, const rb_ttf_parser_t *ttf_parser, unsigned int index);

typedef hb_blob_t *(*hb_reference_table_func_t)(hb_face_t *face, hb_tag_t tag, void *user_data);

/* calls destroy() when not needing user_data anymore */
HB_EXTERN hb_face_t *
hb_face_create_for_tables(hb_reference_table_func_t reference_table_func, void *user_data, hb_destroy_func_t destroy);

HB_EXTERN hb_face_t *hb_face_get_empty(void);

HB_EXTERN hb_face_t *hb_face_reference(hb_face_t *face);

HB_EXTERN void hb_face_destroy(hb_face_t *face);

HB_EXTERN void hb_face_make_immutable(hb_face_t *face);

HB_EXTERN hb_bool_t hb_face_is_immutable(const hb_face_t *face);

HB_EXTERN hb_blob_t *hb_face_reference_table(const hb_face_t *face, hb_tag_t tag);

HB_EXTERN unsigned int hb_face_get_index(const hb_face_t *face);

HB_EXTERN void hb_face_set_upem(hb_face_t *face, unsigned int upem);

HB_EXTERN unsigned int hb_face_get_upem(const hb_face_t *face);

HB_EXTERN unsigned int hb_face_get_glyph_count(const hb_face_t *face);

HB_END_DECLS
