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

RB_BEGIN_DECLS

#define RB_OT_MAP_MAX_BITS 8u
#define RB_OT_MAP_MAX_VALUE ((1u << RB_OT_MAP_MAX_BITS) - 1u)

static const rb_tag_t table_tags[2] = {RB_TAG('G', 'S', 'U', 'B'), RB_TAG('G', 'P', 'O', 'S')};

typedef struct rb_ot_map_t rb_ot_map_t;
typedef struct rb_ot_map_builder_t rb_ot_map_builder_t;

typedef void (*rb_ot_pause_func_t)(const rb_ot_shape_plan_t *plan, rb_face_t *face, rb_buffer_t *buffer);

typedef enum rb_ot_map_feature_flags_t {
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
} rb_ot_map_feature_flags_t;
RB_MARK_AS_FLAG_T(rb_ot_map_feature_flags_t);

typedef struct rb_ot_map_feature_t {
    rb_tag_t tag;
    rb_ot_map_feature_flags_t flags;
} rb_ot_map_feature_t;

RB_EXTERN rb_ot_map_t *rb_ot_map_create();
RB_EXTERN void rb_ot_map_destroy(rb_ot_map_t *map);
RB_EXTERN rb_mask_t rb_ot_map_get_global_mask(const rb_ot_map_t *map);
RB_EXTERN rb_mask_t rb_ot_map_get_mask(const rb_ot_map_t *map, rb_tag_t feature_tag, unsigned int *shift);
RB_EXTERN rb_mask_t rb_ot_map_get_1_mask(const rb_ot_map_t *map, rb_tag_t feature_tag);
RB_EXTERN unsigned int rb_ot_map_get_feature_index(const rb_ot_map_t *map, unsigned int table_index, rb_tag_t feature_tag);
RB_EXTERN rb_tag_t rb_ot_map_get_chosen_script(const rb_ot_map_t *map, unsigned int table_index);

RB_EXTERN rb_ot_map_builder_t *rb_ot_map_builder_create(rb_face_t *face, const rb_segment_properties_t *props);
RB_EXTERN void rb_ot_map_builder_destroy(rb_ot_map_builder_t *builder);
RB_EXTERN void rb_ot_map_builder_compile(rb_ot_map_builder_t *builder, rb_ot_map_t *map, unsigned int *variation_index);
RB_EXTERN void rb_ot_map_builder_add_feature(rb_ot_map_builder_t *builder, rb_tag_t tag, rb_ot_map_feature_flags_t flags, unsigned int value);
RB_EXTERN void rb_ot_map_builder_enable_feature(rb_ot_map_builder_t *builder, rb_tag_t tag, rb_ot_map_feature_flags_t flags, unsigned int value);
RB_EXTERN void rb_ot_map_builder_add_gsub_pause(rb_ot_map_builder_t *builder, rb_ot_pause_func_t pause);

RB_END_DECLS

#endif /* RB_OT_MAP_HH */
