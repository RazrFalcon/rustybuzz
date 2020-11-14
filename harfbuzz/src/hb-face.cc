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
#include "hb-machinery.hh"

#include "hb-ot-kern-table.hh"
#include "hb-aat-layout-ankr-table.hh"
#include "hb-aat-layout-feat-table.hh"
#include "hb-aat-layout-kerx-table.hh"
#include "hb-aat-layout-morx-table.hh"
#include "hb-aat-layout-trak-table.hh"

/**
 * SECTION:hb-face
 * @title: hb-face
 * @short_description: Face objects
 * @include: hb.h
 *
 * Face objects represent a font face at a certain size and other
 * parameters (pixels per EM, points per EM, variation settings.)
 * Font faces are used as input to rb_shape() among other things.
 **/

rb_blob_t *rb_face_sanitize_table(rb_blob_t *blob, rb_tag_t tag, unsigned int glyph_count)
{
    rb_sanitize_context_t c;
    c.set_num_glyphs(glyph_count);
    switch (tag) {
        case RB_OT_TAG_kern:
            return c.sanitize_blob<OT::kern>(blob);

        case RB_AAT_TAG_morx:
            return c.sanitize_blob<AAT::morx>(blob);

        case RB_AAT_TAG_mort:
            return c.sanitize_blob<AAT::mort>(blob);

        case RB_AAT_TAG_kerx:
            return c.sanitize_blob<AAT::kerx>(blob);

        case RB_AAT_TAG_ankr:
            return c.sanitize_blob<AAT::ankr>(blob);

        case RB_AAT_TAG_trak:
            return c.sanitize_blob<AAT::trak>(blob);

        case RB_AAT_TAG_feat:
            return c.sanitize_blob<AAT::feat>(blob);

        default:
            assert(false);
    }
}

const OT::kern *rb_face_get_kern_table(rb_face_t *face)
{
    return rb_face_get_table_blob(face, RB_OT_TAG_kern)->as<OT::kern>();
}

const AAT::morx *rb_face_get_morx_table(rb_face_t *face)
{
    return rb_face_get_table_blob(face, RB_AAT_TAG_morx)->as<AAT::morx>();
}

const AAT::mort *rb_face_get_mort_table(rb_face_t *face)
{
    return rb_face_get_table_blob(face, RB_AAT_TAG_mort)->as<AAT::mort>();
}

const AAT::kerx *rb_face_get_kerx_table(rb_face_t *face)
{
    return rb_face_get_table_blob(face, RB_AAT_TAG_kerx)->as<AAT::kerx>();
}

const AAT::ankr *rb_face_get_ankr_table(rb_face_t *face)
{
    return rb_face_get_table_blob(face, RB_AAT_TAG_ankr)->as<AAT::ankr>();
}

const AAT::trak *rb_face_get_trak_table(rb_face_t *face)
{
    return rb_face_get_table_blob(face, RB_AAT_TAG_trak)->as<AAT::trak>();
}

const AAT::feat *rb_face_get_feat_table(rb_face_t *face)
{
    return rb_face_get_table_blob(face, RB_AAT_TAG_feat)->as<AAT::feat>();
}

rb_bool_t rb_face_get_glyph_contour_point_for_origin(rb_face_t *face,
                                                     rb_codepoint_t glyph,
                                                     unsigned int point_index,
                                                     rb_direction_t direction,
                                                     rb_position_t *x,
                                                     rb_position_t *y)
{
    *x = *y = 0;
    return false;
}
