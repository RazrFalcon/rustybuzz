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

#ifndef RB_OT_LAYOUT_GSUB_TABLE_HH
#define RB_OT_LAYOUT_GSUB_TABLE_HH

#include "hb-ot-layout-gsubgpos.hh"

extern "C" {
RB_EXTERN rb_bool_t rb_subst_lookup_apply(const char *data, OT::rb_ot_apply_context_t *c, unsigned int kind);
RB_EXTERN rb_bool_t rb_subst_lookup_is_reverse(const char *data, unsigned int kind);
}

namespace OT {

/*
 * SubstLookup
 */

struct SubstLookupSubTable
{
    bool apply(rb_ot_apply_context_t *c, unsigned int lookup_type) const
    {
        return rb_subst_lookup_apply((const char*)this, c, lookup_type);
    }

    bool is_reverse(unsigned int lookup_type) const
    {
        return rb_subst_lookup_is_reverse((const char*)this, lookup_type);
    }

public:
    DEFINE_SIZE_MIN(0);
};

struct SubstLookup : Lookup
{
    typedef SubstLookupSubTable SubTable;

    const SubTable &get_subtable(unsigned int i) const
    {
        return Lookup::get_subtable<SubTable>(i);
    }

    bool is_reverse() const
    {
        return get_subtable(0).is_reverse(get_type());
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
 * GSUB -- Glyph Substitution
 * https://docs.microsoft.com/en-us/typography/opentype/spec/gsub
 */

struct GSUB : GSUBGPOS
{
    static constexpr rb_tag_t tableTag = RB_OT_TAG_GSUB;
    static constexpr unsigned table_index = 0u;
    static constexpr bool inplace = false;
    typedef SubstLookup Lookup;

    const SubstLookup &get_lookup(unsigned int i) const
    {
        return static_cast<const SubstLookup &>(GSUBGPOS::get_lookup(i));
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return GSUBGPOS::sanitize<SubstLookup>(c);
    }

    RB_INTERNAL bool is_blocklisted(rb_blob_t *blob, rb_face_t *face) const;

    typedef GSUBGPOS::accelerator_t<GSUB> accelerator_t;
};

struct GSUB_accelerator_t : GSUB::accelerator_t
{
};

/* Out-of-class implementation for methods recursing */

/*static*/ bool SubstLookup::apply_recurse_func(rb_ot_apply_context_t *c, unsigned int lookup_index)
{
    const SubstLookup &l = c->face->table.GSUB.get_relaxed()->table->get_lookup(lookup_index);
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

#endif /* RB_OT_LAYOUT_GSUB_TABLE_HH */
