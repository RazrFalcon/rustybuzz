/*
 * Copyright © 2007,2008,2009,2010  Red Hat, Inc.
 * Copyright © 2010,2012  Google, Inc.
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

#ifndef RB_OT_LAYOUT_GSUBGPOS_HH
#define RB_OT_LAYOUT_GSUBGPOS_HH

#include "hb.hh"
#include "hb-buffer.hh"
#include "hb-map.hh"
#include "hb-set.hh"
#include "hb-ot-map.hh"
#include "hb-ot-layout-common.hh"
#include "hb-ot-layout-gdef-table.hh"

namespace OT {

struct rb_would_apply_context_t : rb_dispatch_context_t<rb_would_apply_context_t, bool>
{
    template <typename T> return_t dispatch(const T &obj)
    {
        return obj.would_apply(this);
    }
    static return_t default_return_value()
    {
        return false;
    }
    bool stop_sublookup_iteration(return_t r) const
    {
        return r;
    }

    rb_face_t *face;
    const rb_codepoint_t *glyphs;
    unsigned int len;
    bool zero_context;

    rb_would_apply_context_t(rb_face_t *face_, const rb_codepoint_t *glyphs_, unsigned int len_, bool zero_context_)
        : face(face_)
        , glyphs(glyphs_)
        , len(len_)
        , zero_context(zero_context_)
    {
    }
};

template <typename set_t>
struct rb_collect_coverage_context_t : rb_dispatch_context_t<rb_collect_coverage_context_t<set_t>, const Coverage &>
{
    typedef const Coverage &return_t; // Stoopid that we have to dupe this here.
    template <typename T> return_t dispatch(const T &obj)
    {
        return obj.get_coverage();
    }
    static return_t default_return_value()
    {
        return Null(Coverage);
    }
    bool stop_sublookup_iteration(return_t r) const
    {
        r.collect_coverage(set);
        return false;
    }

    rb_collect_coverage_context_t(set_t *set_)
        : set(set_)
    {
    }

    set_t *set;
};

struct rb_ot_apply_context_t : rb_dispatch_context_t<rb_ot_apply_context_t, bool>
{
    struct matcher_t
    {
        matcher_t()
            : lookup_props(0)
            , ignore_zwnj(false)
            , ignore_zwj(false)
            , mask(-1)
            ,
#define arg1(arg) (arg) /* Remove the macro to see why it's needed! */
            syllable arg1(0)
            ,
#undef arg1
            match_func(nullptr)
            , match_data(nullptr)
        {
        }

        typedef bool (*match_func_t)(rb_codepoint_t glyph_id, const HBUINT16 &value, const void *data);

        void set_ignore_zwnj(bool ignore_zwnj_)
        {
            ignore_zwnj = ignore_zwnj_;
        }
        void set_ignore_zwj(bool ignore_zwj_)
        {
            ignore_zwj = ignore_zwj_;
        }
        void set_lookup_props(unsigned int lookup_props_)
        {
            lookup_props = lookup_props_;
        }
        void set_mask(rb_mask_t mask_)
        {
            mask = mask_;
        }
        void set_syllable(uint8_t syllable_)
        {
            syllable = syllable_;
        }
        void set_match_func(match_func_t match_func_, const void *match_data_)
        {
            match_func = match_func_;
            match_data = match_data_;
        }

        enum may_match_t { MATCH_NO, MATCH_YES, MATCH_MAYBE };

        may_match_t may_match(const rb_glyph_info_t &info, const HBUINT16 *glyph_data) const
        {
            if (!(info.mask & mask) || (syllable && syllable != info.syllable()))
                return MATCH_NO;

            if (match_func)
                return match_func(info.codepoint, *glyph_data, match_data) ? MATCH_YES : MATCH_NO;

            return MATCH_MAYBE;
        }

        enum may_skip_t { SKIP_NO, SKIP_YES, SKIP_MAYBE };

        may_skip_t may_skip(const rb_ot_apply_context_t *c, const rb_glyph_info_t &info) const
        {
            if (!c->check_glyph_property(&info, lookup_props))
                return SKIP_YES;

            if (unlikely(_rb_glyph_info_is_default_ignorable_and_not_hidden(&info) &&
                         (ignore_zwnj || !_rb_glyph_info_is_zwnj(&info)) &&
                         (ignore_zwj || !_rb_glyph_info_is_zwj(&info))))
                return SKIP_MAYBE;

            return SKIP_NO;
        }

    protected:
        unsigned int lookup_props;
        bool ignore_zwnj;
        bool ignore_zwj;
        rb_mask_t mask;
        uint8_t syllable;
        match_func_t match_func;
        const void *match_data;
    };

    struct skipping_iterator_t
    {
        void init(rb_ot_apply_context_t *c_, bool context_match = false)
        {
            c = c_;
            match_glyph_data = nullptr;
            matcher.set_match_func(nullptr, nullptr);
            matcher.set_lookup_props(c->lookup_props);
            /* Ignore ZWNJ if we are matching GPOS, or matching GSUB context and asked to. */
            matcher.set_ignore_zwnj(c->table_index == 1 || (context_match && c->auto_zwnj));
            /* Ignore ZWJ if we are matching context, or asked to. */
            matcher.set_ignore_zwj(context_match || c->auto_zwj);
            matcher.set_mask(context_match ? -1 : c->lookup_mask);
        }
        void set_lookup_props(unsigned int lookup_props)
        {
            matcher.set_lookup_props(lookup_props);
        }
        void set_match_func(matcher_t::match_func_t match_func_, const void *match_data_, const HBUINT16 glyph_data[])
        {
            matcher.set_match_func(match_func_, match_data_);
            match_glyph_data = glyph_data;
        }

        void reset(unsigned int start_index_, unsigned int num_items_)
        {
            idx = start_index_;
            num_items = num_items_;
            end = rb_buffer_get_length(c->buffer);
            matcher.set_syllable(
                start_index_ == rb_buffer_get_index(c->buffer) ? rb_buffer_get_cur(c->buffer, 0)->syllable() : 0);
        }

        void reject()
        {
            num_items++;
            if (match_glyph_data)
                match_glyph_data--;
        }

        matcher_t::may_skip_t may_skip(const rb_glyph_info_t &info) const
        {
            return matcher.may_skip(c, info);
        }

        bool next()
        {
            assert(num_items > 0);
            while (idx + num_items < end) {
                idx++;
                const rb_glyph_info_t &info = rb_buffer_get_glyph_infos(c->buffer)[idx];

                matcher_t::may_skip_t skip = matcher.may_skip(c, info);
                if (unlikely(skip == matcher_t::SKIP_YES))
                    continue;

                matcher_t::may_match_t match = matcher.may_match(info, match_glyph_data);
                if (match == matcher_t::MATCH_YES || (match == matcher_t::MATCH_MAYBE && skip == matcher_t::SKIP_NO)) {
                    num_items--;
                    if (match_glyph_data)
                        match_glyph_data++;
                    return true;
                }

                if (skip == matcher_t::SKIP_NO)
                    return false;
            }
            return false;
        }
        bool prev()
        {
            assert(num_items > 0);
            while (idx > num_items - 1) {
                idx--;
                const rb_glyph_info_t &info = rb_buffer_get_out_infos(c->buffer)[idx];

                matcher_t::may_skip_t skip = matcher.may_skip(c, info);
                if (unlikely(skip == matcher_t::SKIP_YES))
                    continue;

                matcher_t::may_match_t match = matcher.may_match(info, match_glyph_data);
                if (match == matcher_t::MATCH_YES || (match == matcher_t::MATCH_MAYBE && skip == matcher_t::SKIP_NO)) {
                    num_items--;
                    if (match_glyph_data)
                        match_glyph_data++;
                    return true;
                }

                if (skip == matcher_t::SKIP_NO)
                    return false;
            }
            return false;
        }

        unsigned int idx;

    protected:
        rb_ot_apply_context_t *c;
        matcher_t matcher;
        const HBUINT16 *match_glyph_data;

        unsigned int num_items;
        unsigned int end;
    };

    typedef return_t (*recurse_func_t)(rb_ot_apply_context_t *c, unsigned int lookup_index);
    template <typename T> return_t dispatch(const T &obj)
    {
        return obj.apply(this);
    }
    static return_t default_return_value()
    {
        return false;
    }
    bool stop_sublookup_iteration(return_t r) const
    {
        return r;
    }
    return_t recurse(unsigned int sub_lookup_index)
    {
        if (unlikely(nesting_level_left == 0 || !recurse_func || rb_buffer_decrement_max_ops(buffer, 1) < 0))
            return default_return_value();

        nesting_level_left--;
        bool ret = recurse_func(this, sub_lookup_index);
        nesting_level_left++;
        return ret;
    }

    skipping_iterator_t iter_input, iter_context;

    rb_font_t *font;
    rb_face_t *face;
    rb_buffer_t *buffer;
    recurse_func_t recurse_func;
    const GDEF &gdef;
    const VariationStore &var_store;

    rb_direction_t direction;
    rb_mask_t lookup_mask;
    unsigned int table_index; /* GSUB/GPOS */
    unsigned int lookup_index;
    unsigned int lookup_props;
    unsigned int nesting_level_left;

    bool has_glyph_classes;
    bool auto_zwnj;
    bool auto_zwj;
    bool random;

    uint32_t random_state;

    rb_ot_apply_context_t(unsigned int table_index_, rb_font_t *font_, rb_buffer_t *buffer_)
        : iter_input()
        , iter_context()
        , font(font_)
        , face(rb_font_get_face(font))
        , buffer(buffer_)
        , recurse_func(nullptr)
        , gdef(*face->table.GDEF->table)
        , var_store(gdef.get_var_store())
        , direction(rb_buffer_get_direction(buffer_))
        , lookup_mask(1)
        , table_index(table_index_)
        , lookup_index((unsigned int)-1)
        , lookup_props(0)
        , nesting_level_left(RB_MAX_NESTING_LEVEL)
        , has_glyph_classes(gdef.has_glyph_classes())
        , auto_zwnj(true)
        , auto_zwj(true)
        , random(false)
        , random_state(1)
    {
        init_iters();
    }

    void init_iters()
    {
        iter_input.init(this, false);
        iter_context.init(this, true);
    }

    void set_lookup_mask(rb_mask_t mask)
    {
        lookup_mask = mask;
        init_iters();
    }
    void set_auto_zwj(bool auto_zwj_)
    {
        auto_zwj = auto_zwj_;
        init_iters();
    }
    void set_auto_zwnj(bool auto_zwnj_)
    {
        auto_zwnj = auto_zwnj_;
        init_iters();
    }
    void set_random(bool random_)
    {
        random = random_;
    }
    void set_recurse_func(recurse_func_t func)
    {
        recurse_func = func;
    }
    void set_lookup_index(unsigned int lookup_index_)
    {
        lookup_index = lookup_index_;
    }
    void set_lookup_props(unsigned int lookup_props_)
    {
        lookup_props = lookup_props_;
        init_iters();
    }

    uint32_t random_number()
    {
        /* http://www.cplusplus.com/reference/random/minstd_rand/ */
        random_state = random_state * 48271 % 2147483647;
        return random_state;
    }

    bool match_properties_mark(rb_codepoint_t glyph, unsigned int glyph_props, unsigned int match_props) const
    {
        /* If using mark filtering sets, the high short of
         * match_props has the set index.
         */
        if (match_props & LookupFlag::UseMarkFilteringSet)
            return gdef.mark_set_covers(match_props >> 16, glyph);

        /* The second byte of match_props has the meaning
         * "ignore marks of attachment type different than
         * the attachment type specified."
         */
        if (match_props & LookupFlag::MarkAttachmentType)
            return (match_props & LookupFlag::MarkAttachmentType) == (glyph_props & LookupFlag::MarkAttachmentType);

        return true;
    }

    bool check_glyph_property(const rb_glyph_info_t *info, unsigned int match_props) const
    {
        rb_codepoint_t glyph = info->codepoint;
        unsigned int glyph_props = _rb_glyph_info_get_glyph_props(info);

        /* Not covered, if, for example, glyph class is ligature and
         * match_props includes LookupFlags::IgnoreLigatures
         */
        if (glyph_props & match_props & LookupFlag::IgnoreFlags)
            return false;

        if (unlikely(glyph_props & RB_OT_LAYOUT_GLYPH_PROPS_MARK))
            return match_properties_mark(glyph, glyph_props, match_props);

        return true;
    }

    void _set_glyph_class(rb_codepoint_t glyph_index,
                          unsigned int class_guess = 0,
                          bool ligature = false,
                          bool component = false) const
    {
        unsigned int props = _rb_glyph_info_get_glyph_props(rb_buffer_get_cur(buffer, 0));

        props |= RB_OT_LAYOUT_GLYPH_PROPS_SUBSTITUTED;
        if (ligature) {
            props |= RB_OT_LAYOUT_GLYPH_PROPS_LIGATED;
            /* In the only place that the MULTIPLIED bit is used, Uniscribe
             * seems to only care about the "last" transformation between
             * Ligature and Multiple substitutions.  Ie. if you ligate, expand,
             * and ligate again, it forgives the multiplication and acts as
             * if only ligation happened.  As such, clear MULTIPLIED bit.
             */
            props &= ~RB_OT_LAYOUT_GLYPH_PROPS_MULTIPLIED;
        }
        if (component)
            props |= RB_OT_LAYOUT_GLYPH_PROPS_MULTIPLIED;

        if (likely(has_glyph_classes))
            props = (props & ~RB_OT_LAYOUT_GLYPH_PROPS_CLASS_MASK) | gdef.get_glyph_props(glyph_index);
        else if (class_guess)
            props = (props & ~RB_OT_LAYOUT_GLYPH_PROPS_CLASS_MASK) | class_guess;

        _rb_glyph_info_set_glyph_props(rb_buffer_get_cur(buffer, 0), props);
    }

    void replace_glyph(rb_codepoint_t glyph_index) const
    {
        _set_glyph_class(glyph_index);
        rb_buffer_replace_glyph(buffer, glyph_index);
    }
    void replace_glyph_inplace(rb_codepoint_t glyph_index) const
    {
        _set_glyph_class(glyph_index);
        rb_buffer_get_cur(buffer, 0)->codepoint = glyph_index;
    }
    void replace_glyph_with_ligature(rb_codepoint_t glyph_index, unsigned int class_guess) const
    {
        _set_glyph_class(glyph_index, class_guess, true);
        rb_buffer_replace_glyph(buffer, glyph_index);
    }
    void output_glyph_for_component(rb_codepoint_t glyph_index, unsigned int class_guess) const
    {
        _set_glyph_class(glyph_index, class_guess, false, true);
        rb_buffer_output_glyph(buffer, glyph_index);
    }
};

struct rb_get_subtables_context_t : rb_dispatch_context_t<rb_get_subtables_context_t>
{
    template <typename Type> static inline bool apply_to(const void *obj, OT::rb_ot_apply_context_t *c)
    {
        const Type *typed_obj = (const Type *)obj;
        return typed_obj->apply(c);
    }

    typedef bool (*rb_apply_func_t)(const void *obj, OT::rb_ot_apply_context_t *c);

    struct rb_applicable_t
    {
        template <typename T> void init(const T &obj_, rb_apply_func_t apply_func_)
        {
            obj = &obj_;
            apply_func = apply_func_;
            digest.init();
            obj_.get_coverage().collect_coverage(&digest);
        }

        bool apply(OT::rb_ot_apply_context_t *c) const
        {
            return digest.may_have(rb_buffer_get_cur(c->buffer, 0)->codepoint) && apply_func(obj, c);
        }

    private:
        const void *obj;
        rb_apply_func_t apply_func;
        rb_set_digest_t digest;
    };

    typedef rb_vector_t<rb_applicable_t> array_t;

    /* Dispatch interface. */
    template <typename T> return_t dispatch(const T &obj)
    {
        rb_applicable_t *entry = array.push();
        entry->init(obj, apply_to<T>);
        return rb_empty_t();
    }
    static return_t default_return_value()
    {
        return rb_empty_t();
    }

    rb_get_subtables_context_t(array_t &array_)
        : array(array_)
    {
    }

    array_t &array;
};

/* Contextual lookups */

extern "C" {
RB_EXTERN rb_bool_t rb_context_lookup_would_apply(const rb_would_apply_context_t *c, const char *data, unsigned int length);
RB_EXTERN rb_bool_t rb_context_lookup_apply(rb_ot_apply_context_t *c, const char *data, unsigned int length);
RB_EXTERN rb_bool_t rb_chain_context_lookup_would_apply(const rb_would_apply_context_t *c, const char *data, unsigned int length);
RB_EXTERN rb_bool_t rb_chain_context_lookup_apply(rb_ot_apply_context_t *c, const char *data, unsigned int length);
}

struct ContextFormat1Or2
{
    const Coverage &get_coverage() const
    {
        return this + coverage;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return coverage.sanitize(c, this);
    }

protected:
    HBUINT16 format;                /* Format identifier--format = 1 */
    OffsetTo<Coverage> coverage;    /* Offset to Coverage table--from
                                     * beginning of table */
public:
    DEFINE_SIZE_STATIC(4);
};

struct ContextFormat3
{
    const Coverage &get_coverage() const
    {
        return this + coverage;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return coverage.sanitize(c, this);
    }

protected:
    HBUINT16 format;
    HBUINT16 glyphCount;
    HBUINT16 lookupCount;
    OffsetTo<Coverage> coverage;

public:
    DEFINE_SIZE_STATIC(8);
};

struct Context
{
    const Coverage &get_coverage() const
    {
        switch (u.format) {
        case 1:
        case 2:
            return u.format1or2.get_coverage();
        case 3:
            return u.format3.get_coverage();
        default:
            return Null(Coverage);
        }
    }

    bool would_apply(rb_would_apply_context_t *c) const
    {
        return rb_context_lookup_would_apply(c, (const char*)this, -1);
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        return rb_context_lookup_apply(c, (const char*)this, -1);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        switch (u.format) {
        case 1:
        case 2:
            return u.format1or2.sanitize(c);
        case 3:
            return u.format3.sanitize(c);
        default:
            return true;
        }
    }

protected:
    union {
        HBUINT16 format; /* Format identifier */
        ContextFormat1Or2 format1or2;
        ContextFormat3 format3;
    } u;
};

struct ChainContextFormat3
{
    const Coverage &get_coverage() const
    {
        const OffsetArrayOf<Coverage> &input = StructAfter<OffsetArrayOf<Coverage>>(backtrack);
        return this + input[0];
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        if (!backtrack.sanitize(c, this))
            return false;
        const OffsetArrayOf<Coverage> &input = StructAfter<OffsetArrayOf<Coverage>>(backtrack);
        if (!input.sanitize(c, this))
            return false;
        if (!input.len)
            return false; /* To be consistent with Context. */
        return true;
    }

protected:
    HBUINT16 format;                    /* Format identifier--format = 3 */
    OffsetArrayOf<Coverage> backtrack;  /* Array of coverage tables
                                         * in backtracking sequence, in  glyph
                                         * sequence order */
    OffsetArrayOf<Coverage> inputX;     /* Array of coverage
                                         * tables in input sequence, in glyph
                                         * sequence order */
public:
    DEFINE_SIZE_MIN(6);
};

struct ChainContext
{
    const Coverage &get_coverage() const
    {
        switch (u.format) {
        case 1:
        case 2:
            return u.format1or2.get_coverage();
        case 3:
            return u.format3.get_coverage();
        default:
            return Null(Coverage);
        }
    }

    bool would_apply(rb_would_apply_context_t *c) const
    {
        return rb_chain_context_lookup_would_apply(c, (const char*)this, -1);
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        return rb_chain_context_lookup_apply(c, (const char*)this, -1);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        switch (u.format) {
        case 1:
        case 2:
            return u.format1or2.sanitize(c);
        case 3:
            return u.format3.sanitize(c);
        default:
            return true;
        }
    }

protected:
    union {
        HBUINT16 format; /* Format identifier */
        ContextFormat1Or2 format1or2;
        ChainContextFormat3 format3;
    } u;
};

template <typename T> struct ExtensionFormat1
{
    unsigned int get_type() const
    {
        return extensionLookupType;
    }

    template <typename X> const X &get_subtable() const
    {
        return this + reinterpret_cast<const LOffsetTo<typename T::SubTable> &>(extensionOffset);
    }

    template <typename context_t, typename... Ts> typename context_t::return_t dispatch(context_t *c, Ts &&... ds) const
    {
        if (unlikely(!c->may_dispatch(this, this)))
            return c->no_dispatch_return_value();
        return get_subtable<typename T::SubTable>().dispatch(c, get_type(), rb_forward<Ts>(ds)...);
    }

    /* This is called from may_dispatch() above with rb_sanitize_context_t. */
    bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_struct(this) && extensionLookupType != T::SubTable::Extension;
    }

protected:
    HBUINT16 format;              /* Format identifier. Set to 1. */
    HBUINT16 extensionLookupType; /* Lookup type of subtable referenced
                                   * by ExtensionOffset (i.e. the
                                   * extension subtable). */
    Offset32 extensionOffset;     /* Offset to the extension subtable,
                                   * of lookup type subtable. */
public:
    DEFINE_SIZE_STATIC(8);
};

template <typename T> struct Extension
{
    unsigned int get_type() const
    {
        switch (u.format) {
        case 1:
            return u.format1.get_type();
        default:
            return 0;
        }
    }

    template <typename context_t, typename... Ts> typename context_t::return_t dispatch(context_t *c, Ts &&... ds) const
    {
        if (unlikely(!c->may_dispatch(this, &u.format)))
            return c->no_dispatch_return_value();
        switch (u.format) {
        case 1:
            return u.format1.dispatch(c, rb_forward<Ts>(ds)...);
        default:
            return c->default_return_value();
        }
    }

protected:
    union {
        HBUINT16 format; /* Format identifier */
        ExtensionFormat1<T> format1;
    } u;
};

/*
 * GSUB/GPOS Common
 */

struct rb_ot_layout_lookup_accelerator_t
{
    template <typename TLookup> void init(const TLookup &lookup)
    {
        digest.init();
        lookup.collect_coverage(&digest);

        subtables.init();
        OT::rb_get_subtables_context_t c_get_subtables(subtables);
        lookup.dispatch(&c_get_subtables);
    }
    void fini()
    {
        subtables.fini();
    }

    bool may_have(rb_codepoint_t g) const
    {
        return digest.may_have(g);
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        for (unsigned int i = 0; i < subtables.length; i++) {
            if (subtables[i].apply(c)) {
                return true;
            }
        }
        return false;
    }

private:
    rb_set_digest_t digest;
    rb_get_subtables_context_t::array_t subtables;
};

struct GSUBGPOS
{
    bool has_data() const
    {
        return version.to_int();
    }
    unsigned int get_script_count() const
    {
        return (this + scriptList).len;
    }
    const Tag &get_script_tag(unsigned int i) const
    {
        return (this + scriptList).get_tag(i);
    }
    const Script &get_script(unsigned int i) const
    {
        return (this + scriptList)[i];
    }
    bool find_script_index(rb_tag_t tag, unsigned int *index) const
    {
        return (this + scriptList).find_index(tag, index);
    }

    unsigned int get_feature_count() const
    {
        return (this + featureList).len;
    }
    rb_tag_t get_feature_tag(unsigned int i) const
    {
        return i == Index::NOT_FOUND_INDEX ? RB_TAG_NONE : (this + featureList).get_tag(i);
    }
    const Feature &get_feature(unsigned int i) const
    {
        return (this + featureList)[i];
    }
    bool find_feature_index(rb_tag_t tag, unsigned int *index) const
    {
        return (this + featureList).find_index(tag, index);
    }

    unsigned int get_lookup_count() const
    {
        return (this + lookupList).len;
    }
    const Lookup &get_lookup(unsigned int i) const
    {
        return (this + lookupList)[i];
    }

    bool find_variations_index(const int *coords, unsigned int num_coords, unsigned int *index) const
    {
        return (version.to_int() >= 0x00010001u ? this + featureVars : Null(FeatureVariations))
            .find_index(coords, num_coords, index);
    }
    const Feature &get_feature_variation(unsigned int feature_index, unsigned int variations_index) const
    {
        if (FeatureVariations::NOT_FOUND_INDEX != variations_index && version.to_int() >= 0x00010001u) {
            const Feature *feature = (this + featureVars).find_substitute(variations_index, feature_index);
            if (feature)
                return *feature;
        }
        return get_feature(feature_index);
    }

    unsigned int get_size() const
    {
        return min_size + (version.to_int() >= 0x00010001u ? featureVars.static_size : 0);
    }

    template <typename TLookup> bool sanitize(rb_sanitize_context_t *c) const
    {
        typedef OffsetListOf<TLookup> TLookupList;
        if (unlikely(!(version.sanitize(c) && likely(version.major == 1) && scriptList.sanitize(c, this) &&
                       featureList.sanitize(c, this) &&
                       reinterpret_cast<const OffsetTo<TLookupList> &>(lookupList).sanitize(c, this))))
            return false;

        if (unlikely(!(version.to_int() < 0x00010001u || featureVars.sanitize(c, this))))
            return false;

        return true;
    }

    template <typename T> struct accelerator_t
    {
        void init(rb_face_t *face)
        {
            this->table = rb_sanitize_context_t().reference_table<T>(face);
            if (unlikely(this->table->is_blocklisted(this->table.get_blob(), face))) {
                rb_blob_destroy(this->table.get_blob());
                this->table = rb_blob_get_empty();
            }

            this->lookup_count = table->get_lookup_count();

            this->accels = (rb_ot_layout_lookup_accelerator_t *)calloc(this->lookup_count,
                                                                       sizeof(rb_ot_layout_lookup_accelerator_t));
            if (unlikely(!this->accels))
                this->lookup_count = 0;

            for (unsigned int i = 0; i < this->lookup_count; i++)
                this->accels[i].init(table->get_lookup(i));
        }

        void fini()
        {
            for (unsigned int i = 0; i < this->lookup_count; i++)
                this->accels[i].fini();
            free(this->accels);
            this->table.destroy();
        }

        rb_blob_ptr_t<T> table;
        unsigned int lookup_count;
        rb_ot_layout_lookup_accelerator_t *accels;
    };

protected:
    FixedVersion<> version;                   /* Version of the GSUB/GPOS table--initially set
                                               * to 0x00010000u */
    OffsetTo<ScriptList> scriptList;          /* ScriptList table */
    OffsetTo<FeatureList> featureList;        /* FeatureList table */
    OffsetTo<LookupList> lookupList;          /* LookupList table */
    LOffsetTo<FeatureVariations> featureVars; /* Offset to Feature Variations
                                                 table--from beginning of table
                                               * (may be NULL).  Introduced
                                               * in version 0x00010001. */
public:
    DEFINE_SIZE_MIN(10);
};

} /* namespace OT */

extern "C" {
RB_EXTERN unsigned int   rb_would_apply_context_get_len(const OT::rb_would_apply_context_t *c);
RB_EXTERN rb_codepoint_t rb_would_apply_context_get_glyph(const OT::rb_would_apply_context_t *c, unsigned int index);
RB_EXTERN rb_bool_t      rb_would_apply_context_get_zero_context(const OT::rb_would_apply_context_t *c);
RB_EXTERN rb_buffer_t   *rb_ot_apply_context_get_buffer(const OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_direction_t rb_ot_apply_context_get_direction(const OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_mask_t      rb_ot_apply_context_get_lookup_mask(const OT::rb_ot_apply_context_t *c);
RB_EXTERN unsigned int   rb_ot_apply_context_get_table_index(const OT::rb_ot_apply_context_t *c);
RB_EXTERN unsigned int   rb_ot_apply_context_get_lookup_index(const OT::rb_ot_apply_context_t *c);
RB_EXTERN unsigned int   rb_ot_apply_context_get_lookup_props(const OT::rb_ot_apply_context_t *c);
RB_EXTERN unsigned int   rb_ot_apply_context_get_nesting_level_left(const OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_bool_t      rb_ot_apply_context_get_has_glyph_classes(const OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_bool_t      rb_ot_apply_context_get_auto_zwnj(const OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_bool_t      rb_ot_apply_context_get_auto_zwj(const OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_bool_t      rb_ot_apply_context_get_random(const OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_bool_t      rb_ot_apply_context_gdef_mark_set_covers(const OT::rb_ot_apply_context_t *c, unsigned int set_index, rb_codepoint_t glyph_id);
RB_EXTERN unsigned int   rb_ot_apply_context_gdef_get_glyph_props(const OT::rb_ot_apply_context_t *c, rb_codepoint_t glyph_id);
RB_EXTERN uint32_t       rb_ot_apply_context_random_number(OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_bool_t      rb_ot_apply_context_recurse(OT::rb_ot_apply_context_t *c, unsigned int sub_lookup_index);
}

#endif /* RB_OT_LAYOUT_GSUBGPOS_HH */
