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

/*
 * rb_face_t
 */

DEFINE_NULL_INSTANCE(rb_face_t) = {
    RB_OBJECT_HEADER_STATIC,

    nullptr, /* reference_table_func */
    nullptr, /* user_data */
    nullptr, /* destroy */

    RB_ATOMIC_INT_INIT(1000), /* upem */
    RB_ATOMIC_INT_INIT(0),    /* num_glyphs */

    /* Zero for the rest is fine. */
};

/**
 * rb_face_create_for_tables:
 * @reference_table_func: (closure user_data) (destroy destroy) (scope notified):
 * @user_data:
 * @destroy:
 *
 *
 *
 * Return value: (transfer full)
 *
 * Since: 0.9.2
 **/
rb_face_t *
rb_face_create_for_tables(rb_reference_table_func_t reference_table_func, void *user_data, rb_destroy_func_t destroy)
{
    rb_face_t *face;

    if (!reference_table_func || !(face = rb_object_create<rb_face_t>())) {
        if (destroy)
            destroy(user_data);
        return rb_face_get_empty();
    }

    face->reference_table_func = reference_table_func;
    face->user_data = user_data;
    face->destroy = destroy;

    face->num_glyphs.set_relaxed(-1);

    face->table.init0(face);

    return face;
}

typedef struct rb_face_for_data_closure_t
{
    rb_blob_t *blob;
    unsigned int index;
} rb_face_for_data_closure_t;

static rb_face_for_data_closure_t *_rb_face_for_data_closure_create(rb_blob_t *blob, unsigned int index)
{
    rb_face_for_data_closure_t *closure;

    closure = (rb_face_for_data_closure_t *)calloc(1, sizeof(rb_face_for_data_closure_t));
    if (unlikely(!closure))
        return nullptr;

    closure->blob = blob;
    closure->index = index;

    return closure;
}

static void _rb_face_for_data_closure_destroy(void *data)
{
    rb_face_for_data_closure_t *closure = (rb_face_for_data_closure_t *)data;

    rb_blob_destroy(closure->blob);
    free(closure);
}

static rb_blob_t *_rb_face_for_data_reference_table(rb_face_t *face RB_UNUSED, rb_tag_t tag, void *user_data)
{
    rb_face_for_data_closure_t *data = (rb_face_for_data_closure_t *)user_data;

    if (tag == RB_TAG_NONE)
        return rb_blob_reference(data->blob);

    const OT::OpenTypeFontFile &ot_file = *data->blob->as<OT::OpenTypeFontFile>();
    unsigned int base_offset;
    const OT::OpenTypeFontFace &ot_face = ot_file.get_face(data->index, &base_offset);

    const OT::OpenTypeTable &table = ot_face.get_table_by_tag(tag);

    rb_blob_t *blob = rb_blob_create_sub_blob(data->blob, base_offset + table.offset, table.length);

    return blob;
}

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

    rb_face_for_data_closure_t *closure = _rb_face_for_data_closure_create(
        rb_sanitize_context_t().sanitize_blob<OT::OpenTypeFontFile>(rb_blob_reference(blob)), index);

    if (unlikely(!closure))
        return rb_face_get_empty();

    face = rb_face_create_for_tables(_rb_face_for_data_reference_table, closure, _rb_face_for_data_closure_destroy);

    return face;
}

/**
 * rb_face_get_empty:
 *
 *
 *
 * Return value: (transfer full)
 *
 * Since: 0.9.2
 **/
rb_face_t *rb_face_get_empty()
{
    return const_cast<rb_face_t *>(&Null(rb_face_t));
}

/**
 * rb_face_reference: (skip)
 * @face: a face.
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
rb_face_t *rb_face_reference(rb_face_t *face)
{
    return rb_object_reference(face);
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

    if (face->destroy)
        face->destroy(face->user_data);

    free(face);
}

/**
 * rb_face_make_immutable:
 * @face: a face.
 *
 *
 *
 * Since: 0.9.2
 **/
void rb_face_make_immutable(rb_face_t *face)
{
    if (rb_object_is_immutable(face))
        return;

    rb_object_make_immutable(face);
}

/**
 * rb_face_is_immutable:
 * @face: a face.
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
rb_bool_t rb_face_is_immutable(const rb_face_t *face)
{
    return rb_object_is_immutable(face);
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

    return face->reference_table(tag);
}

/**
 * rb_face_reference_blob:
 * @face: a face.
 *
 *
 *
 * Return value: (transfer full):
 *
 * Since: 0.9.2
 **/
rb_blob_t *rb_face_reference_blob(rb_face_t *face)
{
    return face->reference_table(RB_TAG_NONE);
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
    return face->get_num_glyphs();
}
