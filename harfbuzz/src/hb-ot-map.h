#pragma once

#include "hb.h"

typedef struct rb_ot_map_builder_t rb_ot_map_builder_t;
typedef struct rb_ot_map_t rb_ot_map_t;
typedef struct lookup_map_t lookup_map_t;
typedef struct hb_shape_plan_t hb_shape_plan_t;

typedef void (*pause_func_t)(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer);

typedef enum hb_ot_map_feature_flags_t {
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
} hb_ot_map_feature_flags_t;

typedef struct lookup_map_t
{
    unsigned short index;
    bool auto_zwnj;
    bool auto_zwj;
    bool random;
    hb_mask_t mask;
} lookup_map_t;

typedef struct stage_map_t
{
    unsigned int last_lookup; /* Cumulative */
    pause_func_t pause_func;
} stage_map_t;

typedef struct hb_ot_map_feature_t
{
    hb_tag_t tag;
    hb_ot_map_feature_flags_t flags;
} hb_ot_map_feature_t;

extern "C" {

// Map

HB_EXTERN rb_ot_map_t *rb_ot_map_init();

HB_EXTERN void rb_ot_map_fini(rb_ot_map_t *map);

HB_EXTERN hb_mask_t rb_ot_map_get_global_mask(const rb_ot_map_t *map);

HB_EXTERN hb_mask_t rb_ot_map_get_mask(const rb_ot_map_t *map, hb_tag_t feature_tag, unsigned int *shift);

HB_EXTERN bool rb_ot_map_needs_fallback(const rb_ot_map_t *map, hb_tag_t feature_tag);

HB_EXTERN hb_mask_t rb_ot_map_get_1_mask(const rb_ot_map_t *map, hb_tag_t feature_tag);

HB_EXTERN unsigned int
rb_ot_map_get_feature_index(const rb_ot_map_t *map, unsigned int table_index, hb_tag_t feature_tag);

HB_EXTERN hb_tag_t rb_ot_map_get_chosen_script(const rb_ot_map_t *map, unsigned int table_index);

HB_EXTERN const lookup_map_t *rb_ot_map_get_lookup(const rb_ot_map_t *map, unsigned int table_index, unsigned int i);

HB_EXTERN unsigned int rb_ot_map_get_stages_length(const rb_ot_map_t *map, unsigned int table_index);

HB_EXTERN const stage_map_t *rb_ot_map_get_stage(const rb_ot_map_t *map, unsigned int table_index, unsigned int i);

void hb_ot_layout_substitute(const rb_ot_map_t *map, const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer);

void hb_ot_layout_position(const rb_ot_map_t *map, const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer);

// Builder

HB_EXTERN rb_ot_map_builder_t *rb_ot_map_builder_init(const rb_ttf_parser_t *ttf_parser, const hb_segment_properties_t *props_);

HB_EXTERN void rb_ot_map_builder_fini(rb_ot_map_builder_t *builder);

HB_EXTERN void rb_ot_map_builder_compile(rb_ot_map_builder_t *builder,
                                         rb_ot_map_t *m,
                                         const rb_ttf_parser_t *ttf_parser,
                                         unsigned int *variations_index);

HB_EXTERN void rb_ot_map_builder_add_feature(rb_ot_map_builder_t *builder,
                                             hb_tag_t tag,
                                             hb_ot_map_feature_flags_t flags,
                                             unsigned int value);

HB_EXTERN void rb_ot_map_builder_enable_feature(rb_ot_map_builder_t *builder,
                                                hb_tag_t tag,
                                                hb_ot_map_feature_flags_t flags,
                                                unsigned int value);

HB_EXTERN void rb_ot_map_builder_add_gsub_pause(rb_ot_map_builder_t *builder, pause_func_t pause_func);

HB_EXTERN hb_tag_t rb_ot_map_builder_chosen_script(rb_ot_map_builder_t *builder, unsigned int table_index);
}
