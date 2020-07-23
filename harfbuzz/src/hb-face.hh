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

#ifndef RB_FACE_HH
#define RB_FACE_HH

#include "hb.hh"

#include "hb-shape-plan.hh"
#include "hb-ot-face.hh"

/*
 * rb_face_t
 */

struct rb_face_t
{
    rb_object_header_t header;

    rb_reference_table_func_t reference_table_func;
    void *user_data;
    rb_destroy_func_t destroy;

    unsigned int index;                 /* Face index in a collection, zero-based. */
    mutable rb_atomic_int_t upem;       /* Units-per-EM. */
    mutable rb_atomic_int_t num_glyphs; /* Number of glyphs. */

    rb_ot_face_t table; /* All the face's tables. */

    rb_blob_t *reference_table(rb_tag_t tag) const
    {
        rb_blob_t *blob;

        if (unlikely(!reference_table_func))
            return rb_blob_get_empty();

        blob = reference_table_func(/*XXX*/ const_cast<rb_face_t *>(this), tag, user_data);
        if (unlikely(!blob))
            return rb_blob_get_empty();

        return blob;
    }

    RB_PURE_FUNC unsigned int get_upem() const
    {
        unsigned int ret = upem.get_relaxed();
        if (unlikely(!ret)) {
            return load_upem();
        }
        return ret;
    }

    unsigned int get_num_glyphs() const
    {
        unsigned int ret = num_glyphs.get_relaxed();
        if (unlikely(ret == UINT_MAX))
            return load_num_glyphs();
        return ret;
    }

private:
    RB_INTERNAL unsigned int load_upem() const;
    RB_INTERNAL unsigned int load_num_glyphs() const;
};
DECLARE_NULL_INSTANCE(rb_face_t);

#endif /* RB_FACE_HH */
