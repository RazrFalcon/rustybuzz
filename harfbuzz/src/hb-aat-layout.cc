/*
 * Copyright © 2017  Google, Inc.
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
 * Google Author(s): Behdad Esfahbod
 */

#include "hb.hh"

#include "hb-aat-layout.hh"
#include "hb-aat-layout-morx-table.hh"

/*
 * rb_aat_apply_context_t
 */

/* Note: This context is used for kerning, even without AAT, hence the condition. */
AAT::rb_aat_apply_context_t::rb_aat_apply_context_t(const rb_shape_plan_t *plan_,
                                                    rb_face_t *face_,
                                                    rb_buffer_t *buffer_,
                                                    rb_blob_t *blob)
    : plan(plan_)
    , face(face_)
    , buffer(buffer_)
    , sanitizer()
    , lookup_index(0)
{
    sanitizer.init(blob);
    sanitizer.set_num_glyphs(rb_face_get_glyph_count(face));
    sanitizer.start_processing();
    sanitizer.set_max_ops(RB_SANITIZE_MAX_OPS_MAX);
}

AAT::rb_aat_apply_context_t::~rb_aat_apply_context_t()
{
    sanitizer.end_processing();
}

/**
 * SECTION:hb-aat-layout
 * @title: hb-aat-layout
 * @short_description: Apple Advanced Typography Layout
 * @include: hb-aat.h
 *
 * Functions for querying OpenType Layout features in the font face.
 **/

/*
 * mort/morx/kerx/trak
 */

void rb_aat_layout_compile_map(const rb_aat_map_builder_t *mapper, rb_aat_map_t *map)
{
    const AAT::morx &morx = *rb_face_get_morx_table(mapper->face);
    if (morx.has_data()) {
        morx.compile_flags(mapper, map);
        return;
    }

    const AAT::mort &mort = *rb_face_get_mort_table(mapper->face);
    if (mort.has_data()) {
        mort.compile_flags(mapper, map);
        return;
    }
}

/*
 * rb_aat_layout_has_substitution:
 * @face:
 *
 * Returns:
 * Since: 2.3.0
 */
rb_bool_t rb_aat_layout_has_substitution(rb_face_t *face)
{
    return rb_face_get_morx_table(face)->has_data() || rb_face_get_mort_table(face)->has_data();
}

void rb_aat_layout_substitute(const rb_shape_plan_t *plan, rb_face_t *face, rb_buffer_t *buffer)
{
    rb_blob_t *morx_blob = rb_face_get_table_blob(face, RB_AAT_TAG_morx);
    const AAT::morx &morx = *morx_blob->as<AAT::morx>();
    if (morx.has_data()) {
        AAT::rb_aat_apply_context_t c(plan, face, buffer, morx_blob);
        morx.apply(&c);
        return;
    }

    rb_blob_t *mort_blob = rb_face_get_table_blob(face, RB_AAT_TAG_mort);
    const AAT::mort &mort = *mort_blob->as<AAT::mort>();
    if (mort.has_data()) {
        AAT::rb_aat_apply_context_t c(plan, face, buffer, mort_blob);
        mort.apply(&c);
        return;
    }
}

void rb_aat_layout_zero_width_deleted_glyphs(rb_buffer_t *buffer)
{
    unsigned int count = rb_buffer_get_length(buffer);
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);
    rb_glyph_position_t *pos = rb_buffer_get_glyph_positions(buffer);
    for (unsigned int i = 0; i < count; i++)
        if (unlikely(info[i].codepoint == AAT::DELETED_GLYPH))
            pos[i].x_advance = pos[i].y_advance = pos[i].x_offset = pos[i].y_offset = 0;
}

static rb_bool_t is_deleted_glyph(const rb_glyph_info_t *info)
{
    return info->codepoint == AAT::DELETED_GLYPH;
}

void rb_aat_layout_remove_deleted_glyphs(rb_buffer_t *buffer)
{
    rb_buffer_delete_glyphs_inplace(buffer, is_deleted_glyph);
}
