/*
 * Copyright © 2009  Red Hat, Inc.
 * Copyright © 2011  Google, Inc.
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

#pragma once

#include "hb.hh"

#include "hb-ot-face.hh"
#include "hb-shaper.hh"

/*
 * hb_face_t
 */

struct hb_face_t
{
    hb_object_header_t header;

    hb_reference_table_func_t reference_table_func;
    void *user_data;
    hb_destroy_func_t destroy;

    const rb_ttf_parser_t *ttf_parser;

    unsigned int index;                 /* Face index in a collection, zero-based. */
    mutable hb_atomic_int_t upem;       /* Units-per-EM. */
    mutable hb_atomic_int_t num_glyphs; /* Number of glyphs. */

    hb_ot_face_t table; /* All the face's tables. */

    hb_blob_t *reference_table(hb_tag_t tag) const
    {
        hb_blob_t *blob;

        if (unlikely(!reference_table_func))
            return hb_blob_get_empty();

        blob = reference_table_func(/*XXX*/ const_cast<hb_face_t *>(this), tag, user_data);
        if (unlikely(!blob))
            return hb_blob_get_empty();

        return blob;
    }
};
DECLARE_NULL_INSTANCE(hb_face_t);
