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

#ifndef RB_BLOB_H
#define RB_BLOB_H

#include "hb-common.h"

RB_BEGIN_DECLS

typedef struct rb_blob_t rb_blob_t;

RB_EXTERN rb_blob_t *rb_blob_create(const char *data, unsigned int length, void *user_data, rb_destroy_func_t destroy);

RB_EXTERN rb_blob_t *rb_blob_create_sub_blob(rb_blob_t *parent, unsigned int offset, unsigned int length);

RB_EXTERN rb_blob_t *rb_blob_get_empty(void);

RB_EXTERN rb_blob_t *rb_blob_reference(rb_blob_t *blob);

RB_EXTERN void rb_blob_destroy(rb_blob_t *blob);

RB_END_DECLS

#endif /* RB_BLOB_H */
