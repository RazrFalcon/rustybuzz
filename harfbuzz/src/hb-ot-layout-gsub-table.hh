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
RB_EXTERN rb_bool_t rb_single_subst_would_apply(const OT::rb_would_apply_context_t *c, const char *data, unsigned int length);
RB_EXTERN rb_bool_t rb_single_subst_apply(OT::rb_ot_apply_context_t *c, const char *data, unsigned int length);
RB_EXTERN rb_bool_t rb_multiple_subst_would_apply(const OT::rb_would_apply_context_t *c, const char *data, unsigned int length);
RB_EXTERN rb_bool_t rb_multiple_subst_apply(OT::rb_ot_apply_context_t *c, const char *data, unsigned int length);
RB_EXTERN rb_bool_t rb_alternate_subst_would_apply(const OT::rb_would_apply_context_t *c, const char *data, unsigned int length);
RB_EXTERN rb_bool_t rb_alternate_subst_apply(OT::rb_ot_apply_context_t *c, const char *data, unsigned int length);
}

namespace OT {

struct SingleSubst
{
    const Coverage &get_coverage() const
    {
        return this + coverage;
    }

    bool would_apply(rb_would_apply_context_t *c) const
    {
        return rb_single_subst_would_apply(c, (const char*)this, -1);
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        return rb_single_subst_apply(c, (const char*)this, -1);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return (format != 1 && format != 2) || coverage.sanitize(c, this);
    }

protected:
    HBUINT16 format;
    OffsetTo<Coverage> coverage;
};

struct MultipleSubst
{
    const Coverage &get_coverage() const
    {
        return this + coverage;
    }

    bool would_apply(rb_would_apply_context_t *c) const
    {
        return rb_multiple_subst_would_apply(c, (const char*)this, -1);
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        return rb_multiple_subst_apply(c, (const char*)this, -1);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return format != 1 || coverage.sanitize(c, this);
    }

protected:
    HBUINT16 format;
    OffsetTo<Coverage> coverage;
};

struct AlternateSubst
{
    const Coverage &get_coverage() const
    {
        return this + coverage;
    }

    bool would_apply(rb_would_apply_context_t *c) const
    {
        return rb_alternate_subst_would_apply(c, (const char*)this, -1);
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        return rb_alternate_subst_apply(c, (const char*)this, -1);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return format != 1 || coverage.sanitize(c, this);
    }

protected:
    HBUINT16 format;
    OffsetTo<Coverage> coverage;
};

struct Ligature
{
    bool would_apply(rb_would_apply_context_t *c) const
    {
        if (c->len != component.lenP1)
            return false;

        for (unsigned int i = 1; i < c->len; i++)
            if (likely(c->glyphs[i] != component[i]))
                return false;

        return true;
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        unsigned int count = component.lenP1;

        if (unlikely(!count))
            return false;

        /* Special-case to make it in-place and not consider this
         * as a "ligated" substitution. */
        if (unlikely(count == 1)) {
            c->replace_glyph(ligGlyph);
            return true;
        }

        unsigned int total_component_count = 0;

        unsigned int match_length = 0;
        unsigned int match_positions[RB_MAX_CONTEXT_LENGTH];

        if (likely(!match_input(
                c, count, &component[1], match_glyph, nullptr, &match_length, match_positions, &total_component_count)))
            return false;

        ligate_input(c, count, match_positions, match_length, ligGlyph, total_component_count);

        return true;
    }

public:
    bool sanitize(rb_sanitize_context_t *c) const
    {
        return ligGlyph.sanitize(c) && component.sanitize(c);
    }

protected:
    HBGlyphID ligGlyph;                   /* GlyphID of ligature to substitute */
    HeadlessArrayOf<HBGlyphID> component; /* Array of component GlyphIDs--start
                                           * with the second  component--ordered
                                           * in writing direction */
public:
    DEFINE_SIZE_ARRAY(4, component);
};

struct LigatureSet
{
    bool would_apply(rb_would_apply_context_t *c) const
    {
        return +rb_iter(ligature) | rb_map(rb_add(this)) | rb_map([c](const Ligature &_) { return _.would_apply(c); }) |
               rb_any;
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        unsigned int num_ligs = ligature.len;
        for (unsigned int i = 0; i < num_ligs; i++) {
            const Ligature &lig = this + ligature[i];
            if (lig.apply(c))
                return true;
        }

        return false;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return ligature.sanitize(c, this);
    }

protected:
    OffsetArrayOf<Ligature> ligature; /* Array LigatureSet tables
                                       * ordered by preference */
public:
    DEFINE_SIZE_ARRAY(2, ligature);
};

struct LigatureSubstFormat1
{
    const Coverage &get_coverage() const
    {
        return this + coverage;
    }

    bool would_apply(rb_would_apply_context_t *c) const
    {
        unsigned int index = (this + coverage).get_coverage(c->glyphs[0]);
        if (likely(index == NOT_COVERED))
            return false;

        const LigatureSet &lig_set = this + ligatureSet[index];
        return lig_set.would_apply(c);
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        unsigned int index = (this + coverage).get_coverage(rb_buffer_get_cur(c->buffer, 0)->codepoint);
        if (likely(index == NOT_COVERED))
            return false;

        const LigatureSet &lig_set = this + ligatureSet[index];
        return lig_set.apply(c);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return coverage.sanitize(c, this) && ligatureSet.sanitize(c, this);
    }

protected:
    HBUINT16 format;                        /* Format identifier--format = 1 */
    OffsetTo<Coverage> coverage;            /* Offset to Coverage table--from
                                             * beginning of Substitution table */
    OffsetArrayOf<LigatureSet> ligatureSet; /* Array LigatureSet tables
                                             * ordered by Coverage Index */
public:
    DEFINE_SIZE_ARRAY(6, ligatureSet);
};

struct LigatureSubst
{
    template <typename context_t, typename... Ts> typename context_t::return_t dispatch(context_t *c, Ts &&... ds) const
    {
        if (unlikely(!c->may_dispatch(this, &u.format)))
            return c->no_dispatch_return_value();
        switch (u.format) {
        case 1:
            return c->dispatch(u.format1, rb_forward<Ts>(ds)...);
        default:
            return c->default_return_value();
        }
    }

protected:
    union {
        HBUINT16 format; /* Format identifier */
        LigatureSubstFormat1 format1;
    } u;
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

struct ReverseChainSingleSubstFormat1
{
    const Coverage &get_coverage() const
    {
        return this + coverage;
    }

    bool would_apply(rb_would_apply_context_t *c) const
    {
        return c->len == 1 && (this + coverage).get_coverage(c->glyphs[0]) != NOT_COVERED;
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        if (unlikely(c->nesting_level_left != RB_MAX_NESTING_LEVEL))
            return false; /* No chaining to this type */

        unsigned int index = (this + coverage).get_coverage(rb_buffer_get_cur(c->buffer, 0)->codepoint);
        if (likely(index == NOT_COVERED))
            return false;

        const OffsetArrayOf<Coverage> &lookahead = StructAfter<OffsetArrayOf<Coverage>>(backtrack);
        const ArrayOf<HBGlyphID> &substitute = StructAfter<ArrayOf<HBGlyphID>>(lookahead);

        if (unlikely(index >= substitute.len))
            return false;

        unsigned int start_index = 0, end_index = 0;
        if (match_backtrack(c, backtrack.len, (HBUINT16 *)backtrack.arrayZ, match_coverage, this, &start_index) &&
            match_lookahead(c, lookahead.len, (HBUINT16 *)lookahead.arrayZ, match_coverage, this, 1, &end_index)) {
            rb_buffer_unsafe_to_break_from_outbuffer(c->buffer, start_index, end_index);
            c->replace_glyph_inplace(substitute[index]);
            /* Note: We DON'T decrease buffer->idx.  The main loop does it
             * for us.  This is useful for preventing surprises if someone
             * calls us through a Context lookup. */
            return true;
        }

        return false;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        if (!(coverage.sanitize(c, this) && backtrack.sanitize(c, this)))
            return false;
        const OffsetArrayOf<Coverage> &lookahead = StructAfter<OffsetArrayOf<Coverage>>(backtrack);
        if (!lookahead.sanitize(c, this))
            return false;
        const ArrayOf<HBGlyphID> &substitute = StructAfter<ArrayOf<HBGlyphID>>(lookahead);
        return substitute.sanitize(c);
    }

protected:
    HBUINT16 format;                    /* Format identifier--format = 1 */
    OffsetTo<Coverage> coverage;        /* Offset to Coverage table--from
                                         * beginning of table */
    OffsetArrayOf<Coverage> backtrack;  /* Array of coverage tables
                                         * in backtracking sequence, in glyph
                                         * sequence order */
    OffsetArrayOf<Coverage> lookaheadX; /* Array of coverage tables
                                         * in lookahead sequence, in glyph
                                         * sequence order */
    ArrayOf<HBGlyphID> substituteX;     /* Array of substitute
                                         * GlyphIDs--ordered by Coverage Index */
public:
    DEFINE_SIZE_MIN(10);
};

struct ReverseChainSingleSubst
{
    template <typename context_t, typename... Ts> typename context_t::return_t dispatch(context_t *c, Ts &&... ds) const
    {
        if (unlikely(!c->may_dispatch(this, &u.format)))
            return c->no_dispatch_return_value();
        switch (u.format) {
        case 1:
            return c->dispatch(u.format1, rb_forward<Ts>(ds)...);
        default:
            return c->default_return_value();
        }
    }

protected:
    union {
        HBUINT16 format; /* Format identifier */
        ReverseChainSingleSubstFormat1 format1;
    } u;
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
            return u.ligature.dispatch(c, rb_forward<Ts>(ds)...);
        case Context:
            return u.context.dispatch(c, rb_forward<Ts>(ds)...);
        case ChainContext:
            return u.chainContext.dispatch(c, rb_forward<Ts>(ds)...);
        case Extension:
            return u.extension.dispatch(c, rb_forward<Ts>(ds)...);
        case ReverseChainSingle:
            return u.reverseChainContextSingle.dispatch(c, rb_forward<Ts>(ds)...);
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

    template <typename set_t> void collect_coverage(set_t *glyphs) const
    {
        rb_collect_coverage_context_t<set_t> c(glyphs);
        dispatch(&c);
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
