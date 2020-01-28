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


/**
 * hb_face_count:
 * @blob: a blob.
 *
 * Get number of faces in a blob.
 *
 * Return value: Number of faces in @blob
 *
 * Since: 1.7.7
 **/
unsigned int
hb_face_count (hb_blob_t *blob)
{
  if (unlikely (!blob))
    return 0;

  /* TODO We shouldn't be sanitizing blob.  Port to run sanitizer and return if not sane. */
  /* Make API signature const after. */
  hb_blob_t *sanitized = hb_sanitize_context_t ().sanitize_blob<OT::OpenTypeFontFile> (hb_blob_reference (blob));
  const OT::OpenTypeFontFile& ot = *sanitized->as<OT::OpenTypeFontFile> ();
  unsigned int ret = ot.get_face_count ();
  hb_blob_destroy (sanitized);

  return ret;
}

/*
 * hb_face_t
 */

DEFINE_NULL_INSTANCE (hb_face_t) =
{
  HB_OBJECT_HEADER_STATIC,

  nullptr, /* reference_table_func */
  nullptr, /* user_data */
  nullptr, /* destroy */
  
  nullptr, /* rust data */

  0,    /* index */
  HB_ATOMIC_INT_INIT (1000), /* upem */
  HB_ATOMIC_INT_INIT (0),    /* num_glyphs */

  /* Zero for the rest is fine. */
};


/**
 * hb_face_create_for_tables:
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
hb_face_t *
hb_face_create_for_tables (hb_reference_table_func_t  reference_table_func,
			   void                      *user_data,
			   hb_destroy_func_t          destroy)
{
  hb_face_t *face;

  if (!reference_table_func || !(face = hb_object_create<hb_face_t> ())) {
    if (destroy)
      destroy (user_data);
    return hb_face_get_empty ();
  }

  face->reference_table_func = reference_table_func;
  face->user_data = user_data;
  face->destroy = destroy;

  face->num_glyphs.set_relaxed (-1);

  face->table.init0 (face);

  return face;
}


typedef struct hb_face_for_data_closure_t {
  hb_blob_t *blob;
  unsigned int  index;
} hb_face_for_data_closure_t;

static hb_face_for_data_closure_t *
_hb_face_for_data_closure_create (hb_blob_t *blob, unsigned int index)
{
  hb_face_for_data_closure_t *closure;

  closure = (hb_face_for_data_closure_t *) calloc (1, sizeof (hb_face_for_data_closure_t));
  if (unlikely (!closure))
    return nullptr;

  closure->blob = blob;
  closure->index = index;

  return closure;
}

static void
_hb_face_for_data_closure_destroy (void *data)
{
  hb_face_for_data_closure_t *closure = (hb_face_for_data_closure_t *) data;

  hb_blob_destroy (closure->blob);
  free (closure);
}

static hb_blob_t *
_hb_face_for_data_reference_table (hb_face_t *face HB_UNUSED, hb_tag_t tag, void *user_data)
{
  hb_face_for_data_closure_t *data = (hb_face_for_data_closure_t *) user_data;

  if (tag == HB_TAG_NONE)
    return hb_blob_reference (data->blob);

  const OT::OpenTypeFontFile &ot_file = *data->blob->as<OT::OpenTypeFontFile> ();
  unsigned int base_offset;
  const OT::OpenTypeFontFace &ot_face = ot_file.get_face (data->index, &base_offset);

  const OT::OpenTypeTable &table = ot_face.get_table_by_tag (tag);

  hb_blob_t *blob = hb_blob_create_sub_blob (data->blob, base_offset + table.offset, table.length);

  return blob;
}

/**
 * hb_face_create: (Xconstructor)
 * @blob:
 * @index:
 *
 *
 *
 * Return value: (transfer full):
 *
 * Since: 0.9.2
 **/
hb_face_t *
hb_face_create (hb_blob_t    *blob,
		const void   *rust_data,
		unsigned int  index)
{
  hb_face_t *face;

  if (unlikely (!blob))
    blob = hb_blob_get_empty ();

  hb_face_for_data_closure_t *closure = _hb_face_for_data_closure_create (hb_sanitize_context_t ().sanitize_blob<OT::OpenTypeFontFile> (hb_blob_reference (blob)), index);

  if (unlikely (!closure))
    return hb_face_get_empty ();

  face = hb_face_create_for_tables (_hb_face_for_data_reference_table,
				    closure,
				    _hb_face_for_data_closure_destroy);

  face->index = index;
  face->rust_data = rust_data;

  return face;
}

/**
 * hb_face_get_empty:
 *
 *
 *
 * Return value: (transfer full)
 *
 * Since: 0.9.2
 **/
hb_face_t *
hb_face_get_empty ()
{
  return const_cast<hb_face_t *> (&Null(hb_face_t));
}


/**
 * hb_face_reference: (skip)
 * @face: a face.
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
hb_face_t *
hb_face_reference (hb_face_t *face)
{
  return hb_object_reference (face);
}

/**
 * hb_face_destroy: (skip)
 * @face: a face.
 *
 *
 *
 * Since: 0.9.2
 **/
void
hb_face_destroy (hb_face_t *face)
{
  if (!hb_object_destroy (face)) return;

  face->table.fini ();

  if (face->destroy)
    face->destroy (face->user_data);

  free (face);
}

/**
 * hb_face_make_immutable:
 * @face: a face.
 *
 *
 *
 * Since: 0.9.2
 **/
void
hb_face_make_immutable (hb_face_t *face)
{
  if (hb_object_is_immutable (face))
    return;

  hb_object_make_immutable (face);
}

/**
 * hb_face_is_immutable:
 * @face: a face.
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
hb_bool_t
hb_face_is_immutable (const hb_face_t *face)
{
  return hb_object_is_immutable (face);
}


/**
 * hb_face_reference_table:
 * @face: a face.
 * @tag:
 *
 *
 *
 * Return value: (transfer full):
 *
 * Since: 0.9.2
 **/
hb_blob_t *
hb_face_reference_table (const hb_face_t *face,
			 hb_tag_t tag)
{
  if (unlikely (tag == HB_TAG_NONE))
    return hb_blob_get_empty ();

  return face->reference_table (tag);
}

/**
 * hb_face_get_index:
 * @face: a face.
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
unsigned int
hb_face_get_index (const hb_face_t *face)
{
  return face->index;
}

/**
 * hb_face_set_upem:
 * @face: a face.
 * @upem:
 *
 *
 *
 * Since: 0.9.2
 **/
void
hb_face_set_upem (hb_face_t    *face,
		  unsigned int  upem)
{
  if (hb_object_is_immutable (face))
    return;

  face->upem.set_relaxed (upem);
}

/**
 * hb_face_get_upem:
 * @face: a face.
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
unsigned int
hb_face_get_upem (const hb_face_t *face)
{
  return rb_face_get_upem (face->rust_data);
}

/**
 * hb_face_get_glyph_count:
 * @face: a face.
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.7
 **/
unsigned int
hb_face_get_glyph_count (const hb_face_t *face)
{
  return rb_face_get_glyph_count (face->rust_data);
}

/*
 * face-builder: A face that has add_table().
 */

struct hb_face_builder_data_t
{
  struct table_entry_t
  {
    int cmp (hb_tag_t t) const
    {
      if (t < tag) return -1;
      if (t > tag) return -1;
      return 0;
    }

    hb_tag_t   tag;
    hb_blob_t *blob;
  };

  hb_vector_t<table_entry_t> tables;
};
