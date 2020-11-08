/*
 * Copyright © 2018  Google, Inc.
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
 * Google Author(s): Behdad Esfahbod
 */

#include "hb-ot-face.hh"

#include "hb-ot-kern-table.hh"

extern "C" {
const char *rb_face_get_table_data(const rb_face_t *face, rb_tag_t tag, unsigned int *len)
{
    rb_blob_t* blob = face->reference_table(tag);
    *len = blob->length;
    return blob->data;
}
}

void rb_ot_face_t::init0(rb_face_t *face)
{
    this->face = face;
    head.init0();
    kern.init0();
    morx.init0();
    mort.init0();
    kerx.init0();
    ankr.init0();
    trak.init0();
    feat.init0();
}

void rb_ot_face_t::fini()
{
    head.fini();
    kern.fini();
    morx.fini();
    mort.fini();
    kerx.fini();
    ankr.fini();
    trak.fini();
    feat.fini();
}
