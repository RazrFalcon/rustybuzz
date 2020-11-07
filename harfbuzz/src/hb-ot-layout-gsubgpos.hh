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

extern "C" {
namespace OT { struct rb_ot_apply_context_t; }
RB_EXTERN rb_bool_t rb_ot_apply_context_check_glyph_property(const OT::rb_ot_apply_context_t *c, const rb_glyph_info_t *info, unsigned int match_props);
}

namespace OT {

struct rb_ot_apply_context_t : rb_dispatch_context_t<rb_ot_apply_context_t, bool>
{
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

    rb_font_t *font;
    rb_face_t *face;
    rb_buffer_t *buffer;
    recurse_func_t recurse_func;

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
        : font(font_)
        , face(rb_font_get_face(font))
        , buffer(buffer_)
        , recurse_func(nullptr)
        , lookup_mask(1)
        , table_index(table_index_)
        , lookup_index((unsigned int)-1)
        , lookup_props(0)
        , nesting_level_left(RB_MAX_NESTING_LEVEL)
        , auto_zwnj(true)
        , auto_zwj(true)
        , random(false)
        , random_state(1)
    {}

    void set_lookup_mask(rb_mask_t mask)
    {
        lookup_mask = mask;
    }
    void set_auto_zwj(bool auto_zwj_)
    {
        auto_zwj = auto_zwj_;
    }
    void set_auto_zwnj(bool auto_zwnj_)
    {
        auto_zwnj = auto_zwnj_;
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
RB_EXTERN const rb_font_t *rb_ot_apply_context_get_font(const OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_buffer_t   *rb_ot_apply_context_get_buffer(OT::rb_ot_apply_context_t *c);
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
