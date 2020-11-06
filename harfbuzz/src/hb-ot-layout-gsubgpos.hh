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

extern "C" {
RB_EXTERN rb_bool_t rb_ot_apply_context_check_glyph_property(const OT::rb_ot_apply_context_t *c, const rb_glyph_info_t *info, unsigned int match_props);
}

namespace OT {

struct rb_would_apply_context_t : rb_dispatch_context_t<rb_would_apply_context_t, bool>
{
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

struct rb_ot_apply_context_t;

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

    return_t recurse(unsigned int sub_lookup_index)
    {
        if (unlikely(nesting_level_left == 0 || !recurse_func || rb_buffer_decrement_max_ops(buffer, 1) < 0))
            return false;

        nesting_level_left--;
        bool ret = recurse_func(this, sub_lookup_index);
        nesting_level_left++;
        return ret;
    }

    skipping_iterator_t iter_input;

    rb_font_t *font;
    rb_face_t *face;
    rb_buffer_t *buffer;
    recurse_func_t recurse_func;
    const GDEF &gdef;

    rb_direction_t direction;
    rb_mask_t lookup_mask;
    unsigned int table_index; /* GSUB/GPOS */
    unsigned int lookup_index;
    unsigned int lookup_props;
    unsigned int nesting_level_left;

    bool auto_zwnj;
    bool auto_zwj;
    bool random;

    uint32_t random_state;

    rb_ot_apply_context_t(unsigned int table_index_, rb_font_t *font_, rb_buffer_t *buffer_)
        : iter_input()
        , font(font_)
        , face(rb_font_get_face(font))
        , buffer(buffer_)
        , recurse_func(nullptr)
        , gdef(*face->table.GDEF->table)
        , direction(rb_buffer_get_direction(buffer_))
        , lookup_mask(1)
        , table_index(table_index_)
        , lookup_index((unsigned int)-1)
        , lookup_props(0)
        , nesting_level_left(RB_MAX_NESTING_LEVEL)
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

    bool check_glyph_property(const rb_glyph_info_t *info, unsigned int match_props) const
    {
        return rb_ot_apply_context_check_glyph_property(this, info, match_props);
    }
};

/*
 * GSUB/GPOS Common
 */

struct GSUBGPOS
{
    bool has_data() const
    {
        return version.to_int();
    }

    unsigned int get_lookup_count() const
    {
        return (this + lookupList).len;
    }
    const Lookup &get_lookup(unsigned int i) const
    {
        return (this + lookupList)[i];
    }

    unsigned int get_size() const
    {
        return min_size;
    }

    template <typename TLookup> bool sanitize(rb_sanitize_context_t *c) const
    {
        typedef OffsetListOf<TLookup> TLookupList;
        if (unlikely(!(version.sanitize(c) && likely(version.major == 1) &&
                       reinterpret_cast<const OffsetTo<TLookupList> &>(lookupList).sanitize(c, this))))
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
        }

        void fini()
        {
            this->table.destroy();
        }

        rb_blob_ptr_t<T> table;
        unsigned int lookup_count;
    };

protected:
    FixedVersion<> version;                   /* Version of the GSUB/GPOS table--initially set
                                               * to 0x00010000u */
    Offset16 scriptList;                      /* Offset to ScriptList table */
    Offset16 featureList;                     /* Offset to FeatureList table */
    OffsetTo<LookupList> lookupList;          /* LookupList table */

public:
    DEFINE_SIZE_MIN(10);
};

} /* namespace OT */

extern "C" {
RB_EXTERN unsigned int   rb_would_apply_context_get_len(const OT::rb_would_apply_context_t *c);
RB_EXTERN rb_codepoint_t rb_would_apply_context_get_glyph(const OT::rb_would_apply_context_t *c, unsigned int index);
RB_EXTERN rb_bool_t      rb_would_apply_context_get_zero_context(const OT::rb_would_apply_context_t *c);
RB_EXTERN const rb_font_t *rb_ot_apply_context_get_font(const OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_buffer_t   *rb_ot_apply_context_get_buffer(OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_direction_t rb_ot_apply_context_get_direction(const OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_mask_t      rb_ot_apply_context_get_lookup_mask(const OT::rb_ot_apply_context_t *c);
RB_EXTERN unsigned int   rb_ot_apply_context_get_table_index(const OT::rb_ot_apply_context_t *c);
RB_EXTERN unsigned int   rb_ot_apply_context_get_lookup_index(const OT::rb_ot_apply_context_t *c);
RB_EXTERN unsigned int   rb_ot_apply_context_get_lookup_props(const OT::rb_ot_apply_context_t *c);
RB_EXTERN unsigned int   rb_ot_apply_context_get_nesting_level_left(const OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_bool_t      rb_ot_apply_context_get_auto_zwnj(const OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_bool_t      rb_ot_apply_context_get_auto_zwj(const OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_bool_t      rb_ot_apply_context_get_random(const OT::rb_ot_apply_context_t *c);
RB_EXTERN uint32_t       rb_ot_apply_context_random_number(OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_bool_t      rb_ot_apply_context_recurse(OT::rb_ot_apply_context_t *c, unsigned int sub_lookup_index);
}

#endif /* RB_OT_LAYOUT_GSUBGPOS_HH */
