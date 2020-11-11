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

extern "C" {
RB_EXTERN rb_bool_t rb_face_metrics_get_position_common(rb_face_t *face, rb_tag_t tag, int *position);
}

enum rb_ot_metrics_tag_t {
    RB_FACE_METRICS_TAG_HORIZONTAL_ASCENDER = RB_TAG('h', 'a', 's', 'c'),
    RB_FACE_METRICS_TAG_HORIZONTAL_DESCENDER = RB_TAG('h', 'd', 's', 'c'),
    RB_FACE_METRICS_TAG_HORIZONTAL_LINE_GAP = RB_TAG('h', 'l', 'g', 'p'),
};

rb_bool_t rb_face_has_glyph(rb_face_t *face, rb_codepoint_t unicode)
{
    rb_codepoint_t glyph;
    return rb_face_get_nominal_glyph(face, unicode, &glyph);
}

rb_position_t rb_face_get_glyph_h_advance(rb_face_t *face, rb_codepoint_t glyph)
{
    rb_position_t ret;
    rb_face_get_glyph_h_advances(face, 1, &glyph, 0, &ret, 0);
    return ret;
}

void rb_face_get_glyph_h_advances(rb_face_t *face,
                                  unsigned int count,
                                  const rb_codepoint_t *first_glyph,
                                  unsigned glyph_stride,
                                  rb_position_t *first_advance,
                                  unsigned advance_stride)
{
    for (unsigned int i = 0; i < count; i++) {
        *first_advance = rb_face_get_advance(face, *first_glyph, 0);
        first_glyph = &StructAtOffsetUnaligned<rb_codepoint_t>(first_glyph, glyph_stride);
        first_advance = &StructAtOffsetUnaligned<rb_position_t>(first_advance, advance_stride);
    }
}

rb_position_t rb_face_get_glyph_v_advance(rb_face_t *face, rb_codepoint_t glyph)
{
    rb_position_t ret;
    rb_face_get_glyph_v_advances(face, 1, &glyph, 0, &ret, 0);
    return ret;
}

void rb_face_get_glyph_v_advances(rb_face_t *face,
                                  unsigned count,
                                  const rb_codepoint_t *first_glyph,
                                  unsigned glyph_stride,
                                  rb_position_t *first_advance,
                                  unsigned advance_stride)
{
    for (unsigned int i = 0; i < count; i++) {
        *first_advance = -rb_face_get_advance(face, *first_glyph, 1);
        first_glyph = &StructAtOffsetUnaligned<rb_codepoint_t>(first_glyph, glyph_stride);
        first_advance = &StructAtOffsetUnaligned<rb_position_t>(first_advance, advance_stride);
    }
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

static rb_bool_t rb_face_get_h_extents(rb_face_t *face, rb_face_extents_t *extents)
{
    return rb_face_metrics_get_position_common(face, RB_FACE_METRICS_TAG_HORIZONTAL_ASCENDER, &extents->ascender) &&
           rb_face_metrics_get_position_common(face, RB_FACE_METRICS_TAG_HORIZONTAL_DESCENDER, &extents->descender) &&
           rb_face_metrics_get_position_common(face, RB_FACE_METRICS_TAG_HORIZONTAL_LINE_GAP, &extents->line_gap);
}

static rb_bool_t rb_face_get_glyph_h_origin(rb_face_t *face, rb_codepoint_t glyph, rb_position_t *x, rb_position_t *y)
{
    *x = *y = 0;
    return true;
}

static rb_bool_t rb_face_get_glyph_v_origin(rb_face_t *face, rb_codepoint_t glyph, rb_position_t *x, rb_position_t *y)
{
    *x = rb_face_get_glyph_h_advance(face, glyph) / 2;

    if (rb_face_has_vorg_data(face)) {
        *y = rb_face_get_y_origin(face, glyph);
        return true;
    }

    rb_glyph_extents_t extents = {0};
    rb_face_get_glyph_extents(face, glyph, &extents);

    rb_position_t tsb = rb_face_get_side_bearing(face, glyph, true);
    *y = extents.y_bearing + tsb;
    return true;
}

static void rb_face_get_h_extents_with_fallback(rb_face_t *face, rb_face_extents_t *extents)
{
    if (!rb_face_get_h_extents(face, extents)) {
        extents->ascender = rb_face_get_upem(face) * .8;
        extents->descender = extents->ascender - rb_face_get_upem(face);
        extents->line_gap = 0;
    }
}

static void rb_face_guess_v_origin_minus_h_origin(rb_face_t *face, rb_codepoint_t glyph, rb_position_t *x, rb_position_t *y)
{
    *x = rb_face_get_glyph_h_advance(face, glyph) / 2;

    /* TODO cache this somehow?! */
    rb_face_extents_t extents;
    rb_face_get_h_extents_with_fallback(face, &extents);
    *y = extents.ascender;
}

static void rb_face_get_glyph_v_origin_with_fallback(rb_face_t *face, rb_codepoint_t glyph, rb_position_t *x, rb_position_t *y)
{
    if (!rb_face_get_glyph_v_origin(face, glyph, x, y) && rb_face_get_glyph_h_origin(face, glyph, x, y)) {
        rb_position_t dx, dy;
        rb_face_guess_v_origin_minus_h_origin(face, glyph, &dx, &dy);
        *x += dx;
        *y += dy;
    }
}

void rb_face_subtract_glyph_v_origin(rb_face_t *face, rb_codepoint_t glyph, rb_position_t *x, rb_position_t *y)
{
    rb_position_t origin_x, origin_y;

    rb_face_get_glyph_v_origin_with_fallback(face, glyph, &origin_x, &origin_y);

    *x -= origin_x;
    *y -= origin_y;
}

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
