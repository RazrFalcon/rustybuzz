/*
 * Copyright © 2007,2008,2009,2010  Red Hat, Inc.
 * Copyright © 2010,2012,2013  Google, Inc.
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

#ifndef RB_OT_LAYOUT_GPOS_TABLE_HH
#define RB_OT_LAYOUT_GPOS_TABLE_HH

#include "hb-ot-layout-gsubgpos.hh"

extern "C" {
RB_EXTERN rb_bool_t rb_pos_lookup_apply(const char *data, OT::rb_ot_apply_context_t *c, unsigned int kind);
}

namespace OT {

/* buffer **position** var allocations */
#define attach_chain()                                                                                                 \
    var.i16[0] /* glyph to which this attaches to, relative to current glyphs; negative for going back, positive for   \
                  forward. */
#define attach_type() var.u8[2] /* attachment type */
/* Note! if attach_chain() is zero, the value of attach_type() is irrelevant. */

enum attach_type_t {
    ATTACH_TYPE_NONE = 0X00,

    /* Each attachment should be either a mark or a cursive; can't be both. */
    ATTACH_TYPE_MARK = 0X01,
    ATTACH_TYPE_CURSIVE = 0X02,
};

/*
 * PosLookup
 */

struct PosLookupSubTable
{
    bool apply(rb_ot_apply_context_t *c, unsigned int lookup_type) const
    {
        return rb_pos_lookup_apply((const char*)this, c, lookup_type);
    }

public:
    DEFINE_SIZE_MIN(0);
};

struct PosLookup : Lookup
{
    typedef struct PosLookupSubTable SubTable;

    const SubTable &get_subtable(unsigned int i) const
    {
        return Lookup::get_subtable<SubTable>(i);
    }

    bool is_reverse() const
    {
        return false;
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        unsigned int lookup_type = get_type();
        unsigned int count = get_subtable_count();
        for (unsigned int i = 0; i < count; i++) {
            if (get_subtable(i).apply(c, lookup_type))
                return true;
        }
        return false;
    }

    static inline bool apply_recurse_func(rb_ot_apply_context_t *c, unsigned int lookup_index);
};

/*
 * GPOS -- Glyph Positioning
 * https://docs.microsoft.com/en-us/typography/opentype/spec/gpos
 */

struct GPOS : GSUBGPOS
{
    static constexpr rb_tag_t tableTag = RB_OT_TAG_GPOS;
    static constexpr unsigned table_index = 1u;
    static constexpr bool inplace = true;
    typedef PosLookup Lookup;

    const PosLookup &get_lookup(unsigned int i) const
    {
        return static_cast<const PosLookup &>(GSUBGPOS::get_lookup(i));
    }

    static inline void position_start(rb_font_t *font, rb_buffer_t *buffer);
    static inline void position_finish_advances(rb_font_t *font, rb_buffer_t *buffer);
    static inline void position_finish_offsets(rb_font_t *font, rb_buffer_t *buffer);

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return GSUBGPOS::sanitize<PosLookup>(c);
    }

    RB_INTERNAL bool is_blocklisted(rb_blob_t *blob, rb_face_t *face) const;

    typedef GSUBGPOS::accelerator_t<GPOS> accelerator_t;
};

static void
propagate_attachment_offsets(rb_glyph_position_t *pos, unsigned int len, unsigned int i, rb_direction_t direction)
{
    /* Adjusts offsets of attached glyphs (both cursive and mark) to accumulate
     * offset of glyph they are attached to. */
    int chain = pos[i].attach_chain(), type = pos[i].attach_type();
    if (likely(!chain))
        return;

    pos[i].attach_chain() = 0;

    unsigned int j = (int)i + chain;

    if (unlikely(j >= len))
        return;

    propagate_attachment_offsets(pos, len, j, direction);

    assert(!!(type & ATTACH_TYPE_MARK) ^ !!(type & ATTACH_TYPE_CURSIVE));

    if (type & ATTACH_TYPE_CURSIVE) {
        if (RB_DIRECTION_IS_HORIZONTAL(direction))
            pos[i].y_offset += pos[j].y_offset;
        else
            pos[i].x_offset += pos[j].x_offset;
    } else /*if (type & ATTACH_TYPE_MARK)*/
    {
        pos[i].x_offset += pos[j].x_offset;
        pos[i].y_offset += pos[j].y_offset;

        assert(j < i);
        if (RB_DIRECTION_IS_FORWARD(direction))
            for (unsigned int k = j; k < i; k++) {
                pos[i].x_offset -= pos[k].x_advance;
                pos[i].y_offset -= pos[k].y_advance;
            }
        else
            for (unsigned int k = j + 1; k < i + 1; k++) {
                pos[i].x_offset += pos[k].x_advance;
                pos[i].y_offset += pos[k].y_advance;
            }
    }
}

void GPOS::position_start(rb_font_t *font RB_UNUSED, rb_buffer_t *buffer)
{
    unsigned int count = rb_buffer_get_length(buffer);
    for (unsigned int i = 0; i < count; i++)
        rb_buffer_get_glyph_positions(buffer)[i].attach_chain() =
            rb_buffer_get_glyph_positions(buffer)[i].attach_type() = 0;
}

void GPOS::position_finish_advances(rb_font_t *font RB_UNUSED, rb_buffer_t *buffer RB_UNUSED)
{
    //_rb_buffer_assert_gsubgpos_vars (buffer);
}

void GPOS::position_finish_offsets(rb_font_t *font RB_UNUSED, rb_buffer_t *buffer)
{
    unsigned int len = rb_buffer_get_length(buffer);
    rb_glyph_position_t *pos = rb_buffer_get_glyph_positions(buffer);
    rb_direction_t direction = rb_buffer_get_direction(buffer);

    /* Handle attachments */
    if (rb_buffer_get_scratch_flags(buffer) & RB_BUFFER_SCRATCH_FLAG_HAS_GPOS_ATTACHMENT)
        for (unsigned int i = 0; i < len; i++)
            propagate_attachment_offsets(pos, len, i, direction);
}

struct GPOS_accelerator_t : GPOS::accelerator_t
{
};

/* Out-of-class implementation for methods recursing */

/*static*/ bool PosLookup::apply_recurse_func(rb_ot_apply_context_t *c, unsigned int lookup_index)
{
    const PosLookup &l = c->face->table.GPOS.get_relaxed()->table->get_lookup(lookup_index);
    unsigned int saved_lookup_props = c->lookup_props;
    unsigned int saved_lookup_index = c->lookup_index;
    c->set_lookup_index(lookup_index);
    c->set_lookup_props(l.get_props());
    bool ret = l.apply(c);
    c->set_lookup_index(saved_lookup_index);
    c->set_lookup_props(saved_lookup_props);
    return ret;
}

} /* namespace OT */

#endif /* RB_OT_LAYOUT_GPOS_TABLE_HH */
