/*
 * Copyright © 2009  Red Hat, Inc.
 * Copyright © 2018  Ebrahim Byagowi
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

#include "hb.hh"
#include "hb-blob.hh"

#include <stdio.h>
#include <stdlib.h>

/**
 * SECTION: hb-blob
 * @title: hb-blob
 * @short_description: Binary data containers
 * @include: hb.h
 *
 * Blobs wrap a chunk of binary data to handle lifecycle management of data
 * while it is passed between client and HarfBuzz.  Blobs are primarily used
 * to create font faces, but also to access font face tables, as well as
 * pass around other binary data.
 **/

/**
 * rb_blob_create: (skip)
 * @data: Pointer to blob data.
 * @length: Length of @data in bytes.
 * @mode: Memory mode for @data.
 * @user_data: Data parameter to pass to @destroy.
 * @destroy: Callback to call when @data is not needed anymore.
 *
 * Creates a new "blob" object wrapping @data.  The @mode parameter is used
 * to negotiate ownership and lifecycle of @data.
 *
 * Return value: New blob, or the empty blob if something failed or if @length is
 * zero.  Destroy with rb_blob_destroy().
 *
 * Since: 0.9.2
 **/
rb_blob_t *rb_blob_create(const char *data, unsigned int length, void *user_data, rb_destroy_func_t destroy)
{
    rb_blob_t *blob;

    if (!length || length >= 1u << 31 || !(blob = rb_object_create<rb_blob_t>())) {
        if (destroy)
            destroy(user_data);
        return rb_blob_get_empty();
    }

    blob->data = data;
    blob->length = length;

    blob->user_data = user_data;
    blob->destroy = destroy;

    return blob;
}

static void _rb_blob_destroy(void *data)
{
    rb_blob_destroy((rb_blob_t *)data);
}

/**
 * rb_blob_create_sub_blob:
 * @parent: Parent blob.
 * @offset: Start offset of sub-blob within @parent, in bytes.
 * @length: Length of sub-blob.
 *
 * Returns a blob that represents a range of bytes in @parent.  The new
 * blob is always created with %RB_MEMORY_MODE_READONLY, meaning that it
 * will never modify data in the parent blob.  The parent data is not
 * expected to be modified, and will result in undefined behavior if it
 * is.
 *
 * Makes @parent immutable.
 *
 * Return value: New blob, or the empty blob if something failed or if
 * @length is zero or @offset is beyond the end of @parent's data.  Destroy
 * with rb_blob_destroy().
 *
 * Since: 0.9.2
 **/
rb_blob_t *rb_blob_create_sub_blob(rb_blob_t *parent, unsigned int offset, unsigned int length)
{
    rb_blob_t *blob;

    if (!length || !parent || offset >= parent->length)
        return rb_blob_get_empty();

    rb_blob_make_immutable(parent);

    blob = rb_blob_create(
        parent->data + offset, rb_min(length, parent->length - offset), rb_blob_reference(parent), _rb_blob_destroy);

    return blob;
}

/**
 * rb_blob_get_empty:
 *
 * Returns the singleton empty blob.
 *
 * See TODO:link object types for more information.
 *
 * Return value: (transfer full): the empty blob.
 *
 * Since: 0.9.2
 **/
rb_blob_t *rb_blob_get_empty()
{
    return const_cast<rb_blob_t *>(&Null(rb_blob_t));
}

/**
 * rb_blob_reference: (skip)
 * @blob: a blob.
 *
 * Increases the reference count on @blob.
 *
 * See TODO:link object types for more information.
 *
 * Return value: @blob.
 *
 * Since: 0.9.2
 **/
rb_blob_t *rb_blob_reference(rb_blob_t *blob)
{
    return rb_object_reference(blob);
}

/**
 * rb_blob_destroy: (skip)
 * @blob: a blob.
 *
 * Decreases the reference count on @blob, and if it reaches zero, destroys
 * @blob, freeing all memory, possibly calling the destroy-callback the blob
 * was created for if it has not been called already.
 *
 * See TODO:link object types for more information.
 *
 * Since: 0.9.2
 **/
void rb_blob_destroy(rb_blob_t *blob)
{
    if (!rb_object_destroy(blob))
        return;

    blob->fini_shallow();

    free(blob);
}

/**
 * rb_blob_make_immutable:
 * @blob: a blob.
 *
 *
 *
 * Since: 0.9.2
 **/
void rb_blob_make_immutable(rb_blob_t *blob)
{
    if (rb_object_is_immutable(blob))
        return;

    rb_object_make_immutable(blob);
}

/**
 * rb_blob_is_immutable:
 * @blob: a blob.
 *
 *
 *
 * Return value: TODO
 *
 * Since: 0.9.2
 **/
rb_bool_t rb_blob_is_immutable(rb_blob_t *blob)
{
    return rb_object_is_immutable(blob);
}

/**
 * rb_blob_get_length:
 * @blob: a blob.
 *
 *
 *
 * Return value: the length of blob data in bytes.
 *
 * Since: 0.9.2
 **/
unsigned int rb_blob_get_length(rb_blob_t *blob)
{
    return blob->length;
}

/**
 * rb_blob_get_data:
 * @blob: a blob.
 * @length: (out):
 *
 *
 *
 * Returns: (transfer none) (array length=length):
 *
 * Since: 0.9.2
 **/
const char *rb_blob_get_data(rb_blob_t *blob, unsigned int *length)
{
    if (length)
        *length = blob->length;

    return blob->data;
}
