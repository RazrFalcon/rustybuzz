/*
 * Copyright © 1998-2004  David Turner and Werner Lemberg
 * Copyright © 2006  Behdad Esfahbod
 * Copyright © 2007,2008,2009  Red Hat, Inc.
 * Copyright © 2012,2013  Google, Inc.
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

#include "hb-open-type.hh"
#include "hb-ot-layout.hh"
#include "hb-ot-face.hh"
#include "hb-ot-map.hh"
#include "hb-map.hh"

#include "hb-ot-kern-table.hh"
#include "hb-ot-layout-gsub-table.hh"
#include "hb-ot-layout-gpos-table.hh"
#include "hb-ot-layout-gsubgpos.hh"

#include "hb-aat-layout-morx-table.hh"

extern "C" {
unsigned int   rb_would_apply_context_get_len(const OT::rb_would_apply_context_t *c) { return c->len; }
rb_codepoint_t rb_would_apply_context_get_glyph(const OT::rb_would_apply_context_t *c, unsigned int index) { return c->glyphs[index]; }
rb_bool_t      rb_would_apply_context_get_zero_context(const OT::rb_would_apply_context_t *c) { return (rb_bool_t)c->zero_context; }
const rb_font_t *rb_ot_apply_context_get_font(const OT::rb_ot_apply_context_t *c) { return c->font; }
rb_buffer_t   *rb_ot_apply_context_get_buffer(OT::rb_ot_apply_context_t *c) { return c->buffer; }
rb_direction_t rb_ot_apply_context_get_direction(const OT::rb_ot_apply_context_t *c) { return c->direction; }
rb_mask_t      rb_ot_apply_context_get_lookup_mask(const OT::rb_ot_apply_context_t *c) { return c->lookup_mask; }
unsigned int   rb_ot_apply_context_get_table_index(const OT::rb_ot_apply_context_t *c) { return c->table_index; }
unsigned int   rb_ot_apply_context_get_lookup_index(const OT::rb_ot_apply_context_t *c) { return c->lookup_index; }
unsigned int   rb_ot_apply_context_get_lookup_props(const OT::rb_ot_apply_context_t *c) { return c->lookup_props; }
unsigned int   rb_ot_apply_context_get_nesting_level_left(const OT::rb_ot_apply_context_t *c) { return c->nesting_level_left; }
rb_bool_t      rb_ot_apply_context_get_auto_zwnj(const OT::rb_ot_apply_context_t *c) { return c->auto_zwnj; }
rb_bool_t      rb_ot_apply_context_get_auto_zwj(const OT::rb_ot_apply_context_t *c) { return c->auto_zwj; }
rb_bool_t      rb_ot_apply_context_get_random(const OT::rb_ot_apply_context_t *c) { return (rb_bool_t)c->random; }
uint32_t       rb_ot_apply_context_random_number(OT::rb_ot_apply_context_t *c) { return c->random_number(); }
rb_bool_t      rb_ot_apply_context_recurse(OT::rb_ot_apply_context_t *c, unsigned int sub_lookup_index) { return (rb_bool_t)c->recurse(sub_lookup_index); }
}

/**
 * SECTION:hb-ot-layout
 * @title: hb-ot-layout
 * @short_description: OpenType Layout
 * @include: hb-ot.h
 *
 * Functions for querying OpenType Layout features in the font face.
 **/

/*
 * kern
 */

/**
 * rb_ot_layout_has_kerning:
 * @face: The #rb_face_t to work on
 *
 * Tests whether a face includes any kerning data in the 'kern' table.
 * Does NOT test for kerning lookups in the GPOS table.
 *
 * Return value: true if data found, false otherwise
 *
 **/
bool rb_ot_layout_has_kerning(rb_face_t *face)
{
    return face->table.kern->has_data();
}

/**
 * rb_ot_layout_has_machine_kerning:
 * @face: The #rb_face_t to work on
 *
 * Tests whether a face includes any state-machine kerning in the 'kern' table.
 * Does NOT examine the GPOS table.
 *
 * Return value: true if data found, false otherwise
 *
 **/
bool rb_ot_layout_has_machine_kerning(rb_face_t *face)
{
    return face->table.kern->has_state_machine();
}

/**
 * rb_ot_layout_has_cross_kerning:
 * @face: The #rb_face_t to work on
 *
 * Tests whether a face has any cross-stream kerning (i.e., kerns
 * that make adjustments perpendicular to the direction of the text
 * flow: Y adjustments in horizontal text or X adjustments in
 * vertical text) in the 'kern' table.
 *
 * Does NOT examine the GPOS table.
 *
 * Return value: true is data found, false otherwise
 *
 **/
bool rb_ot_layout_has_cross_kerning(rb_face_t *face)
{
    return face->table.kern->has_cross_stream();
}

void rb_ot_layout_kern(const rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer)
{
    rb_blob_t *blob = rb_font_get_face(font)->table.kern.get_blob();
    const AAT::kern &kern = *blob->as<AAT::kern>();

    AAT::rb_aat_apply_context_t c(plan, font, buffer, blob);

    kern.apply(&c);
}

/* Public API */

/*
 * GSUB/GPOS
 */

bool OT::GSUB::is_blocklisted(rb_blob_t *blob RB_UNUSED, rb_face_t *face) const
{
    return false;
}

bool OT::GPOS::is_blocklisted(rb_blob_t *blob RB_UNUSED, rb_face_t *face RB_UNUSED) const
{
    return false;
}

/*
 * OT::GSUB
 */

/**
 * rb_ot_layout_lookup_would_substitute:
 * @face: #rb_face_t to work upon
 * @lookup_index: The index of the lookup to query
 * @glyphs: The sequence of glyphs to query for substitution
 * @glyphs_length: The length of the glyph sequence
 * @zero_context: #rb_bool_t indicating whether substitutions should be context-free
 *
 * Tests whether a specified lookup in the specified face would
 * trigger a substitution on the given glyph sequence.
 *
 * Return value: true if a substitution would be triggered, false otherwise
 *
 * Since: 0.9.7
 **/
rb_bool_t rb_ot_layout_lookup_would_substitute(rb_face_t *face,
                                               unsigned int lookup_index,
                                               const rb_codepoint_t *glyphs,
                                               unsigned int glyphs_length,
                                               rb_bool_t zero_context)
{
    if (unlikely(lookup_index >= face->table.GSUB->lookup_count))
        return false;
    OT::rb_would_apply_context_t c(face, glyphs, glyphs_length, (bool)zero_context);

    const OT::SubstLookup &l = face->table.GSUB->table->get_lookup(lookup_index);
    return l.would_apply(&c);
}

/*
 * Parts of different types are implemented here such that they have direct
 * access to GSUB/GPOS lookups.
 */

template <typename Table>
static inline bool apply_forward(OT::rb_ot_apply_context_t *c, const typename Table::Lookup &lookup)
{
    bool ret = false;
    rb_buffer_t *buffer = c->buffer;
    while (rb_buffer_get_index(buffer) < rb_buffer_get_length(buffer) && rb_buffer_is_allocation_successful(buffer)) {
        bool applied = false;
        if ((rb_buffer_get_cur(buffer, 0)->mask & c->lookup_mask) &&
            c->check_glyph_property(rb_buffer_get_cur(buffer, 0), c->lookup_props)) {
            applied = lookup.apply(c);
        }

        if (applied)
            ret = true;
        else
            rb_buffer_next_glyph(buffer);
    }
    return ret;
}

template <typename Table>
static inline bool apply_backward(OT::rb_ot_apply_context_t *c, const typename Table::Lookup &lookup)
{
    bool ret = false;
    rb_buffer_t *buffer = c->buffer;
    do {
        if ((rb_buffer_get_cur(buffer, 0)->mask & c->lookup_mask) &&
            c->check_glyph_property(rb_buffer_get_cur(buffer, 0), c->lookup_props))
            ret |= lookup.apply(c);

        /* The reverse lookup doesn't "advance" cursor (for good reason). */
        rb_buffer_set_index(buffer, rb_buffer_get_index(buffer) - 1);
    } while ((int)rb_buffer_get_index(buffer) >= 0);
    return ret;
}

template <typename Table>
static inline void apply_string(OT::rb_ot_apply_context_t *c, const typename Table::Lookup &lookup)
{
    rb_buffer_t *buffer = c->buffer;

    if (unlikely(!rb_buffer_get_length(buffer) || !c->lookup_mask))
        return;

    c->set_lookup_props(lookup.get_props());

    if (likely(!lookup.is_reverse())) {
        /* in/out forward substitution/positioning */
        if (Table::table_index == 0u)
            rb_buffer_clear_output(buffer);
        rb_buffer_set_index(buffer, 0);

        bool ret;
        ret = apply_forward<Table>(c, lookup);
        if (ret) {
            if (!Table::inplace)
                rb_buffer_swap_buffers(buffer);
            else
                assert(!rb_buffer_has_separate_output(buffer));
        }
    } else {
        /* in-place backward substitution/positioning */
        if (Table::table_index == 0u)
            rb_buffer_remove_output(buffer);
        rb_buffer_set_index(buffer, rb_buffer_get_length(buffer) - 1);

        apply_backward<Table>(c, lookup);
    }
}

template <typename Table>
inline void
rb_ot_map_t::apply(const Table &table, const rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer) const
{
    const unsigned int table_index = table.table_index;
    unsigned int i = 0;
    OT::rb_ot_apply_context_t c(table_index, font, buffer);
    c.set_recurse_func(Table::Lookup::apply_recurse_func);

    for (unsigned int stage_index = 0; stage_index < stages[table_index].length; stage_index++) {
        const stage_map_t *stage = &stages[table_index][stage_index];
        for (; i < stage->last_lookup; i++) {
            unsigned int lookup_index = lookups[table_index][i].index;
            c.set_lookup_index(lookup_index);
            c.set_lookup_mask(lookups[table_index][i].mask);
            c.set_auto_zwj(lookups[table_index][i].auto_zwj);
            c.set_auto_zwnj(lookups[table_index][i].auto_zwnj);
            if (lookups[table_index][i].random) {
                c.set_random(true);
                rb_buffer_unsafe_to_break_all(buffer);
            }
            apply_string<Table>(&c, table.get_lookup(lookup_index));
        }

        if (stage->pause_func) {
            rb_buffer_clear_output(buffer);
            stage->pause_func(plan, font, buffer);
        }
    }
}

void rb_ot_map_t::substitute(const rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer) const
{
    apply(*rb_font_get_face(font)->table.GSUB->table, plan, font, buffer);
}

void rb_ot_map_t::position(const rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer) const
{
    apply(*rb_font_get_face(font)->table.GPOS->table, plan, font, buffer);
}

void rb_layout_clear_syllables(const rb_ot_shape_plan_t *plan RB_UNUSED, rb_font_t *font RB_UNUSED, rb_buffer_t *buffer)
{
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);
    unsigned int count = rb_buffer_get_length(buffer);
    for (unsigned int i = 0; i < count; i++)
        info[i].syllable() = 0;
}
