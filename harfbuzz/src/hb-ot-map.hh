/*
 * Copyright © 2009,2010  Red Hat, Inc.
 * Copyright © 2010,2011,2012,2013  Google, Inc.
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

#ifndef RB_OT_MAP_HH
#define RB_OT_MAP_HH

#include "hb-buffer.hh"

#define RB_OT_MAP_MAX_BITS 8u
#define RB_OT_MAP_MAX_VALUE ((1u << RB_OT_MAP_MAX_BITS) - 1u)

struct rb_ot_shape_plan_t;

static const rb_tag_t table_tags[2] = {RB_OT_TAG_GSUB, RB_OT_TAG_GPOS};

typedef void (*rb_ot_pause_func_t)(const struct rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer);

struct rb_ot_map_lookup_map_t
{
    unsigned short index;
    bool auto_zwnj;
    bool auto_zwj;
    bool random;
    rb_mask_t mask;

    RB_INTERNAL static int cmp(const void *pa, const void *pb)
    {
        const rb_ot_map_lookup_map_t *a = (const rb_ot_map_lookup_map_t *)pa;
        const rb_ot_map_lookup_map_t *b = (const rb_ot_map_lookup_map_t *)pb;
        return a->index < b->index ? -1 : a->index > b->index ? 1 : 0;
    }
};

struct rb_ot_map_t
{
    friend struct rb_ot_map_builder_t;

public:
    struct feature_map_t
    {
        rb_tag_t tag;          /* should be first for our bsearch to work */
        unsigned int index[2]; /* GSUB/GPOS */
        unsigned int stage[2]; /* GSUB/GPOS */
        unsigned int shift;
        rb_mask_t mask;
        rb_mask_t _1_mask; /* mask for value=1, for quick access */
        unsigned int needs_fallback : 1;
        unsigned int auto_zwnj : 1;
        unsigned int auto_zwj : 1;
        unsigned int random : 1;

        int cmp(const rb_tag_t tag_) const
        {
            return tag_ < tag ? -1 : tag_ > tag ? 1 : 0;
        }
    };

    struct stage_map_t
    {
        unsigned int last_lookup; /* Cumulative */
        rb_ot_pause_func_t pause_func;
    };

    void init()
    {
        memset(this, 0, sizeof(*this));

        features.init();
        for (unsigned int table_index = 0; table_index < 2; table_index++) {
            lookups[table_index].init();
            stages[table_index].init();
        }
    }
    void fini()
    {
        features.fini();
        for (unsigned int table_index = 0; table_index < 2; table_index++) {
            lookups[table_index].fini();
            stages[table_index].fini();
        }
    }

    rb_mask_t get_global_mask() const
    {
        return global_mask;
    }

    rb_mask_t get_mask(rb_tag_t feature_tag, unsigned int *shift = nullptr) const
    {
        const feature_map_t *map = features.bsearch(feature_tag);
        if (shift)
            *shift = map ? map->shift : 0;
        return map ? map->mask : 0;
    }

    bool needs_fallback(rb_tag_t feature_tag) const
    {
        const feature_map_t *map = features.bsearch(feature_tag);
        return map ? map->needs_fallback : false;
    }

    rb_mask_t get_1_mask(rb_tag_t feature_tag) const
    {
        const feature_map_t *map = features.bsearch(feature_tag);
        return map ? map->_1_mask : 0;
    }

    unsigned int get_feature_index(unsigned int table_index, rb_tag_t feature_tag) const
    {
        const feature_map_t *map = features.bsearch(feature_tag);
        return map ? map->index[table_index] : RB_OT_LAYOUT_NO_FEATURE_INDEX;
    }

    unsigned int get_feature_stage(unsigned int table_index, rb_tag_t feature_tag) const
    {
        const feature_map_t *map = features.bsearch(feature_tag);
        return map ? map->stage[table_index] : UINT_MAX;
    }

    void get_stage_lookups(unsigned int table_index,
                           unsigned int stage,
                           const struct rb_ot_map_lookup_map_t **plookups,
                           unsigned int *lookup_count) const
    {
        if (unlikely(stage == UINT_MAX)) {
            *plookups = nullptr;
            *lookup_count = 0;
            return;
        }
        assert(stage <= stages[table_index].length);
        unsigned int start = stage ? stages[table_index][stage - 1].last_lookup : 0;
        unsigned int end =
            stage < stages[table_index].length ? stages[table_index][stage].last_lookup : lookups[table_index].length;
        *plookups = end == start ? nullptr : &lookups[table_index][start];
        *lookup_count = end - start;
    }

    RB_INTERNAL void collect_lookups(unsigned int table_index, rb_set_t *lookups) const;
    template <typename Proxy>
    RB_INTERNAL void
    apply(const Proxy &proxy, const struct rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer) const;
    RB_INTERNAL void substitute(const struct rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer) const;
    RB_INTERNAL void position(const struct rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer) const;

public:
    rb_tag_t chosen_script[2];
    bool found_script[2];

private:
    rb_mask_t global_mask;

    rb_sorted_vector_t<feature_map_t> features;
    rb_vector_t<rb_ot_map_lookup_map_t> lookups[2]; /* GSUB/GPOS */
    rb_vector_t<stage_map_t> stages[2];             /* GSUB/GPOS */
};

enum rb_ot_map_feature_flags_t {
    F_NONE = 0x0000u,
    F_GLOBAL = 0x0001u,       /* Feature applies to all characters; results in no mask allocated for it. */
    F_HAS_FALLBACK = 0x0002u, /* Has fallback implementation, so include mask bit even if feature not found. */
    F_MANUAL_ZWNJ = 0x0004u,  /* Don't skip over ZWNJ when matching **context**. */
    F_MANUAL_ZWJ = 0x0008u,   /* Don't skip over ZWJ when matching **input**. */
    F_MANUAL_JOINERS = F_MANUAL_ZWNJ | F_MANUAL_ZWJ,
    F_GLOBAL_MANUAL_JOINERS = F_GLOBAL | F_MANUAL_JOINERS,
    F_GLOBAL_HAS_FALLBACK = F_GLOBAL | F_HAS_FALLBACK,
    F_GLOBAL_SEARCH = 0x0010u, /* If feature not found in LangSys, look for it in global feature list and pick one. */
    F_RANDOM = 0x0020u         /* Randomly select a glyph from an AlternateSubstFormat1 subtable. */
};
RB_MARK_AS_FLAG_T(rb_ot_map_feature_flags_t);

struct rb_ot_map_feature_t
{
    rb_tag_t tag;
    rb_ot_map_feature_flags_t flags;
};

struct rb_ot_map_builder_t
{
public:
    RB_INTERNAL rb_ot_map_builder_t(rb_face_t *face_, const rb_segment_properties_t *props_);

    RB_INTERNAL ~rb_ot_map_builder_t();

    RB_INTERNAL void add_feature(rb_tag_t tag, rb_ot_map_feature_flags_t flags = F_NONE, unsigned int value = 1);

    void add_feature(const rb_ot_map_feature_t &feat)
    {
        add_feature(feat.tag, feat.flags);
    }

    void enable_feature(rb_tag_t tag, rb_ot_map_feature_flags_t flags = F_NONE, unsigned int value = 1)
    {
        add_feature(tag, F_GLOBAL | flags, value);
    }

    void disable_feature(rb_tag_t tag)
    {
        add_feature(tag, F_GLOBAL, 0);
    }

    void add_gsub_pause(rb_ot_pause_func_t pause_func)
    {
        add_pause(0, pause_func);
    }
    void add_gpos_pause(rb_ot_pause_func_t pause_func)
    {
        add_pause(1, pause_func);
    }

    RB_INTERNAL void compile(rb_ot_map_t &m, unsigned int *variations_index);

private:
    RB_INTERNAL void add_lookups(rb_ot_map_t &m,
                                 unsigned int table_index,
                                 unsigned int feature_index,
                                 unsigned int variations_index,
                                 rb_mask_t mask,
                                 bool auto_zwnj = true,
                                 bool auto_zwj = true,
                                 bool random = false);

    struct feature_info_t
    {
        rb_tag_t tag;
        unsigned int seq; /* sequence#, used for stable sorting only */
        unsigned int max_value;
        rb_ot_map_feature_flags_t flags;
        unsigned int default_value; /* for non-global features, what should the unset glyphs take */
        unsigned int stage[2];      /* GSUB/GPOS */

        RB_INTERNAL static int cmp(const void *pa, const void *pb)
        {
            const feature_info_t *a = (const feature_info_t *)pa;
            const feature_info_t *b = (const feature_info_t *)pb;
            return (a->tag != b->tag) ? (a->tag < b->tag ? -1 : 1) : (a->seq < b->seq ? -1 : a->seq > b->seq ? 1 : 0);
        }
    };

    struct stage_info_t
    {
        unsigned int index;
        rb_ot_pause_func_t pause_func;
    };

    RB_INTERNAL void add_pause(unsigned int table_index, rb_ot_pause_func_t pause_func);

public:
    rb_face_t *face;
    rb_segment_properties_t props;

    rb_tag_t chosen_script[2];
    bool found_script[2];
    unsigned int script_index[2], language_index[2];

private:
    unsigned int current_stage[2]; /* GSUB/GPOS */
    rb_vector_t<feature_info_t> feature_infos;
    rb_vector_t<stage_info_t> stages[2]; /* GSUB/GPOS */
};

extern "C" {
RB_EXTERN rb_mask_t rb_ot_map_get_1_mask(const rb_ot_map_t *map, rb_tag_t tag);
RB_EXTERN rb_mask_t rb_ot_map_global_mask(const rb_ot_map_t *map);
RB_EXTERN bool rb_ot_map_get_found_script(const rb_ot_map_t *map, unsigned int index);
RB_EXTERN rb_tag_t rb_ot_map_get_chosen_script(const rb_ot_map_t *map, unsigned int index);
RB_EXTERN unsigned int
rb_ot_map_get_feature_stage(const rb_ot_map_t *map, unsigned int table_index, rb_tag_t feature_tag);
RB_EXTERN void rb_ot_map_get_stage_lookups(const rb_ot_map_t *map,
                                           unsigned int table_index,
                                           unsigned int stage,
                                           const struct rb_ot_map_lookup_map_t **plookups,
                                           unsigned int *lookup_count);

RB_EXTERN void rb_ot_map_builder_add_feature(rb_ot_map_builder_t *builder,
                                             rb_tag_t tag,
                                             rb_ot_map_feature_flags_t flags,
                                             unsigned int value);
RB_EXTERN void rb_ot_map_builder_add_gsub_pause(rb_ot_map_builder_t *builder, rb_ot_pause_func_t pause_func);
RB_EXTERN void rb_ot_map_builder_add_gpos_pause(rb_ot_map_builder_t *builder, rb_ot_pause_func_t pause_func);
}

#endif /* RB_OT_MAP_HH */
