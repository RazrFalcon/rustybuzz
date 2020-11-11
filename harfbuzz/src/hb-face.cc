/*
 * Copyright © 2009  Red Hat, Inc.
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
 * Red Hat Author(s): Behdad Esfahbod
 * Google Author(s): Behdad Esfahbod
 */

#include "hb.hh"

#include "hb-face.hh"
#include "hb-blob.hh"
#include "hb-open-file.hh"
#include "hb-ot-face.hh"
#include "hb-ot-maxp-table.hh"

/**
 * SECTION:hb-face
 * @title: hb-face
 * @short_description: Font face objects
 * @include: hb.h
 *
 * Font face is objects represent a single face in a font family.
 * More exactly, a font face represents a single face in a binary font file.
 * Font faces are typically built from a binary blob and a face index.
 * Font faces are used to create fonts.
 **/

/**
 * rb_face_create: (Xconstructor)
 * @blob:
 * @index:
 *
 *
 *
 * Return value: (transfer full):
 *
 * Since: 0.9.2
 **/
rb_face_t *rb_face_create(rb_blob_t *blob, unsigned int index)
{
    rb_face_t *face;

    if (unlikely(!blob))
        blob = rb_blob_get_empty();

    blob = rb_sanitize_context_t().sanitize_blob<OT::OpenTypeFontFile>(rb_blob_reference(blob));

    if (!(face = rb_object_create<rb_face_t>())) {
        rb_blob_destroy(blob);
        return const_cast<rb_face_t *>(&Null(rb_face_t));
    }

    face->blob = blob;
    face->index = index;
    face->num_glyphs.set_relaxed(UINT_MAX);
    face->table.init0(face);

    return face;
}

/**
 * rb_face_destroy: (skip)
 * @face: a face.
 *
 *
 *
 * Since: 0.9.2
 **/
void rb_face_destroy(rb_face_t *face)
{
    if (!rb_object_destroy(face))
        return;

    face->table.fini();
    rb_blob_destroy(face->blob);

    free(face);
}

/**
 * rb_face_reference_table:
 * @face: a face.
 * @tag:
 *
 *
 *
 * Return value: (transfer full):
 *
 * Since: 0.9.2
 **/
rb_blob_t *rb_face_reference_table(const rb_face_t *face, rb_tag_t tag)
{
    if (unlikely(tag == RB_TAG_NONE))
        return rb_blob_get_empty();

    unsigned int base_offset;
    const OT::OpenTypeFontFile &ot_file = *face->blob->as<OT::OpenTypeFontFile>();
    const OT::OpenTypeFontFace &ot_face = ot_file.get_face(face->index, &base_offset);
    const OT::OpenTypeTable &table = ot_face.get_table_by_tag(tag);

    return rb_blob_create_sub_blob(face->blob, base_offset + table.offset, table.length);
}

const char *rb_face_get_table_data(const rb_face_t *face, rb_tag_t tag, unsigned int *len)
{
    rb_blob_t* blob = rb_face_reference_table(face, tag);
    *len = blob->length;
    const char *data = blob->data;
    // This blob was just pointing into it's parent's data, so we don't need it.
    rb_blob_destroy(blob);
    return data;
}

/**
 * rb_face_get_glyph_count:
 * @face: a face.
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.7
 **/
unsigned int rb_face_get_glyph_count(const rb_face_t *face)
{
    unsigned int ret = face->num_glyphs.get_relaxed();
    if (unlikely(ret == UINT_MAX)) {
        rb_sanitize_context_t c = rb_sanitize_context_t();
        c.set_num_glyphs(0); /* So we don't recurse ad infinitum. */
        rb_blob_t *maxp_blob = c.reference_table<OT::maxp>(face);
        const OT::maxp *maxp_table = maxp_blob->as<OT::maxp>();

        unsigned int ret = maxp_table->get_num_glyphs();
        face->num_glyphs.set_relaxed(ret);
        rb_blob_destroy(maxp_blob);
    }
    return ret;
}
