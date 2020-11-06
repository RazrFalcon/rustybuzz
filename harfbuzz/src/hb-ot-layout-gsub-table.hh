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
RB_EXTERN rb_bool_t rb_single_subst_would_apply(const char *data, OT::rb_would_apply_context_t *c);
RB_EXTERN rb_bool_t rb_single_subst_apply(const char *data, OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_bool_t rb_multiple_subst_would_apply(const char *data, OT::rb_would_apply_context_t *c);
RB_EXTERN rb_bool_t rb_multiple_subst_apply(const char *data, OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_bool_t rb_alternate_subst_would_apply(const char *data, OT::rb_would_apply_context_t *c);
RB_EXTERN rb_bool_t rb_alternate_subst_apply(const char *data, OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_bool_t rb_ligature_subst_would_apply(const char *data, OT::rb_would_apply_context_t *c);
RB_EXTERN rb_bool_t rb_ligature_subst_apply(const char *data, OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_bool_t rb_reverse_chain_single_subst_would_apply(const char *data, OT::rb_would_apply_context_t *c);
RB_EXTERN rb_bool_t rb_reverse_chain_single_subst_apply(const char *data, OT::rb_ot_apply_context_t *c);
}

namespace OT {

struct SingleSubst
{
    bool would_apply(rb_would_apply_context_t *c) const
    {
        return rb_single_subst_would_apply((const char*)this, c);
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        return rb_single_subst_apply((const char*)this, c);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return true;
    }
};

struct MultipleSubst
{
    bool would_apply(rb_would_apply_context_t *c) const
    {
        return rb_multiple_subst_would_apply((const char*)this, c);
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        return rb_multiple_subst_apply((const char*)this, c);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return true;
    }
};

struct AlternateSubst
{
    bool would_apply(rb_would_apply_context_t *c) const
    {
        return rb_alternate_subst_would_apply((const char*)this, c);
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        return rb_alternate_subst_apply((const char*)this, c);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return true;
    }
};

struct LigatureSubst
{
    bool would_apply(rb_would_apply_context_t *c) const
    {
        return rb_ligature_subst_would_apply((const char*)this, c);
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        return rb_ligature_subst_apply((const char*)this, c);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return true;
    }
};

struct ContextSubst : Context
{
};

struct ChainContextSubst : ChainContext
{
};

struct ExtensionSubst : Extension<ExtensionSubst>
{
    typedef struct SubstLookupSubTable SubTable;
    bool is_reverse() const;
};

struct ReverseChainSingleSubst
{
    bool would_apply(rb_would_apply_context_t *c) const
    {
        return rb_reverse_chain_single_subst_would_apply((const char*)this, c);
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        return rb_reverse_chain_single_subst_apply((const char*)this, c);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return true;
    }
};

/*
 * SubstLookup
 */

struct SubstLookupSubTable
{
    friend struct Lookup;
    friend struct SubstLookup;

    enum Type {
        Single = 1,
        Multiple = 2,
        Alternate = 3,
        Ligature = 4,
        Context = 5,
        ChainContext = 6,
        Extension = 7,
        ReverseChainSingle = 8
    };

    template <typename context_t, typename... Ts>
    typename context_t::return_t dispatch(context_t *c, unsigned int lookup_type, Ts &&... ds) const
    {
        switch (lookup_type) {
        case Single:
            return c->dispatch(u.single, rb_forward<Ts>(ds)...);
        case Multiple:
            return c->dispatch(u.multiple, rb_forward<Ts>(ds)...);
        case Alternate:
            return c->dispatch(u.alternate, rb_forward<Ts>(ds)...);
        case Ligature:
            return c->dispatch(u.ligature, rb_forward<Ts>(ds)...);
        case Context:
            return c->dispatch(u.context, rb_forward<Ts>(ds)...);
        case ChainContext:
            return c->dispatch(u.chainContext, rb_forward<Ts>(ds)...);
        case Extension:
            return u.extension.dispatch(c, rb_forward<Ts>(ds)...);
        case ReverseChainSingle:
            return c->dispatch(u.reverseChainContextSingle, rb_forward<Ts>(ds)...);
        default:
            return c->default_return_value();
        }
    }

protected:
    union {
        SingleSubst single;
        MultipleSubst multiple;
        AlternateSubst alternate;
        LigatureSubst ligature;
        ContextSubst context;
        ChainContextSubst chainContext;
        ExtensionSubst extension;
        ReverseChainSingleSubst reverseChainContextSingle;
    } u;

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

    static inline bool lookup_type_is_reverse(unsigned int lookup_type)
    {
        return lookup_type == SubTable::ReverseChainSingle;
    }

    bool is_reverse() const
    {
        unsigned int type = get_type();
        if (unlikely(type == SubTable::Extension))
            return reinterpret_cast<const ExtensionSubst &>(get_subtable(0)).is_reverse();
        return lookup_type_is_reverse(type);
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        return dispatch(c);
    }

    bool would_apply(rb_would_apply_context_t *c, const rb_ot_layout_lookup_accelerator_t *accel) const
    {
        if (unlikely(!c->len))
            return false;
        if (!accel->may_have(c->glyphs[0]))
            return false;
        return dispatch(c);
    }

    static inline bool apply_recurse_func(rb_ot_apply_context_t *c, unsigned int lookup_index);

    template <typename context_t, typename... Ts> typename context_t::return_t dispatch(context_t *c, Ts &&... ds) const
    {
        return Lookup::dispatch<SubTable>(c, rb_forward<Ts>(ds)...);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return Lookup::sanitize<SubTable>(c);
    }
};

/*
 * GSUB -- Glyph Substitution
 * https://docs.microsoft.com/en-us/typography/opentype/spec/gsub
 */

struct GSUB : GSUBGPOS
{
    static constexpr rb_tag_t tableTag = RB_OT_TAG_GSUB;

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

/*static*/ inline bool ExtensionSubst::is_reverse() const
{
    return SubstLookup::lookup_type_is_reverse(get_type());
}

/*static*/ bool SubstLookup::apply_recurse_func(rb_ot_apply_context_t *c, unsigned int lookup_index)
{
    const SubstLookup &l = c->face->table.GSUB.get_relaxed()->table->get_lookup(lookup_index);
    unsigned int saved_lookup_props = c->lookup_props;
    unsigned int saved_lookup_index = c->lookup_index;
    c->set_lookup_index(lookup_index);
    c->set_lookup_props(l.get_props());
    bool ret = l.dispatch(c);
    c->set_lookup_index(saved_lookup_index);
    c->set_lookup_props(saved_lookup_props);
    return ret;
}

} /* namespace OT */

#endif /* RB_OT_LAYOUT_GSUB_TABLE_HH */
