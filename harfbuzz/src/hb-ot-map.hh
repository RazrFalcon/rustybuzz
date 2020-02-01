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

#pragma once

#include "hb-buffer.h"
#include "hb-ot-map.h"
#include "hb.hh"

#define HB_OT_MAP_MAX_BITS 8u
#define HB_OT_MAP_MAX_VALUE ((1u << HB_OT_MAP_MAX_BITS) - 1u)

struct hb_ot_shape_plan_t;

static const hb_tag_t table_tags[2] = {HB_OT_TAG_GSUB, HB_OT_TAG_GPOS};

namespace OT {
struct GSUB;
}
namespace OT {
struct GPOS;
}
namespace OT {
struct hb_ot_layout_lookup_accelerator_t;
}

struct hb_ot_map_t
{
    struct feature_map_t
    {
        hb_tag_t tag;          /* should be first for our bsearch to work */
        unsigned int index[2]; /* GSUB/GPOS */
        unsigned int stage[2]; /* GSUB/GPOS */
        unsigned int shift;
        hb_mask_t mask;
        hb_mask_t _1_mask; /* mask for value=1, for quick access */
        unsigned int needs_fallback : 1;
        unsigned int auto_zwnj : 1;
        unsigned int auto_zwj : 1;
        unsigned int random : 1;

        int cmp(const hb_tag_t tag_) const
        {
            return tag_ < tag ? -1 : tag_ > tag ? 1 : 0;
        }
    };

    hb_tag_t chosen_script[2];
    bool found_script[2];

    hb_mask_t global_mask;

    hb_sorted_vector_t<feature_map_t> features;
    hb_vector_t<lookup_map_t> lookups[2]; /* GSUB/GPOS */
    hb_vector_t<stage_map_t> stages[2];   /* GSUB/GPOS */
};

HB_MARK_AS_FLAG_T(hb_ot_map_feature_flags_t);

struct hb_ot_map_builder_t
{
    struct feature_info_t
    {
        hb_tag_t tag;
        unsigned int seq; /* sequence#, used for stable sorting only */
        unsigned int max_value;
        hb_ot_map_feature_flags_t flags;
        unsigned int default_value; /* for non-global features, what should the unset glyphs take */
        unsigned int stage[2];      /* GSUB/GPOS */

        HB_INTERNAL static int cmp(const void *pa, const void *pb)
        {
            const feature_info_t *a = (const feature_info_t *)pa;
            const feature_info_t *b = (const feature_info_t *)pb;
            return (a->tag != b->tag) ? (a->tag < b->tag ? -1 : 1) : (a->seq < b->seq ? -1 : a->seq > b->seq ? 1 : 0);
        }
    };

    struct stage_info_t
    {
        unsigned int index;
        pause_func_t pause_func;
    };

    hb_face_t *face;
    hb_segment_properties_t props;

    hb_tag_t chosen_script[2];
    bool found_script[2];
    unsigned int script_index[2], language_index[2];

    unsigned int current_stage[2]; /* GSUB/GPOS */
    hb_vector_t<feature_info_t> feature_infos;
    hb_vector_t<stage_info_t> stages[2]; /* GSUB/GPOS */
};
