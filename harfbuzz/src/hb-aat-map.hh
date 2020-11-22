/*
 * Copyright © 2018  Google, Inc.
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

#ifndef RB_AAT_MAP_HH
#define RB_AAT_MAP_HH

#include "hb.hh"

struct rb_aat_map_t
{
    friend struct rb_aat_map_builder_t;

public:
    void init()
    {
        memset(this, 0, sizeof(*this));
        chain_flags.init();
    }
    void fini()
    {
        chain_flags.fini();
    }

public:
    rb_vector_t<rb_mask_t> chain_flags;
};

struct rb_aat_map_builder_t
{
public:
    void init(rb_face_t *face_)
    {
        memset(this, 0, sizeof(*this));
        face = face_;
        features.init();
    }
    void fini()
    {
        features.fini();
    }

    RB_INTERNAL void compile(rb_aat_map_t *m);

public:
    struct feature_info_t
    {
        rb_aat_layout_feature_type_t type;
        rb_aat_layout_feature_selector_t setting;
        bool is_exclusive;
        unsigned seq; /* For stable sorting only. */

        RB_INTERNAL static int cmp(const void *pa, const void *pb)
        {
            const feature_info_t *a = (const feature_info_t *)pa;
            const feature_info_t *b = (const feature_info_t *)pb;
            if (a->type != b->type)
                return (a->type < b->type ? -1 : 1);
            if (!a->is_exclusive && (a->setting & ~1) != (b->setting & ~1))
                return (a->setting < b->setting ? -1 : 1);
            return (a->seq < b->seq ? -1 : a->seq > b->seq ? 1 : 0);
        }

        /* compares type & setting only, not is_exclusive flag or seq number */
        int cmp(const feature_info_t &f) const
        {
            return (f.type != type) ? (f.type < type ? -1 : 1)
                                    : (f.setting != setting) ? (f.setting < setting ? -1 : 1) : 0;
        }
    };

public:
    rb_face_t *face;

public:
    rb_sorted_vector_t<feature_info_t> features;
};

RB_BEGIN_DECLS

RB_EXTERN void rb_aat_map_init(rb_aat_map_t *map);

RB_EXTERN void rb_aat_map_fini(rb_aat_map_t *map);

RB_EXTERN void rb_aat_map_builder_init(rb_aat_map_builder_t *builder, rb_face_t *face);

RB_EXTERN void rb_aat_map_builder_fini(rb_aat_map_builder_t *builder);

RB_EXTERN void rb_aat_map_builder_add_feature(rb_aat_map_builder_t *builder, int type, int setting, bool is_exclusive);

RB_EXTERN void rb_aat_map_builder_compile(rb_aat_map_builder_t *builder, rb_aat_map_t *map);

RB_END_DECLS

#endif /* RB_AAT_MAP_HH */
