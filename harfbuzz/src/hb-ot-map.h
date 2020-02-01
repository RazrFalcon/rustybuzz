#pragma once

#include "hb.h"

typedef struct hb_ot_map_builder_t hb_ot_map_builder_t;
typedef struct hb_ot_map_t hb_ot_map_t;
typedef struct lookup_map_t lookup_map_t;

typedef void (*pause_func_t)(const struct hb_ot_shape_plan_t *plan, hb_font_t *font, hb_buffer_t *buffer);

enum hb_ot_map_feature_flags_t {
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

struct lookup_map_t
{
    unsigned short index;
    bool auto_zwnj;
    bool auto_zwj;
    bool random;
    hb_mask_t mask;
};

struct stage_map_t
{
    unsigned int last_lookup; /* Cumulative */
    pause_func_t pause_func;
};

struct hb_ot_map_feature_t
{
    hb_tag_t tag;
    hb_ot_map_feature_flags_t flags;
};

// Map

hb_ot_map_t *hb_ot_map_init();

void hb_ot_map_fini(hb_ot_map_t *map);

hb_mask_t hb_ot_map_get_global_mask(const hb_ot_map_t *map);

hb_mask_t hb_ot_map_get_mask(const hb_ot_map_t *map, hb_tag_t feature_tag, unsigned int *shift);

bool hb_ot_map_needs_fallback(const hb_ot_map_t *map, hb_tag_t feature_tag);

hb_mask_t hb_ot_map_get_1_mask(const hb_ot_map_t *map, hb_tag_t feature_tag);

unsigned int hb_ot_map_get_feature_index(const hb_ot_map_t *map, unsigned int table_index, hb_tag_t feature_tag);

unsigned int hb_ot_map_get_feature_stage(const hb_ot_map_t *map, unsigned int table_index, hb_tag_t feature_tag);

hb_tag_t hb_ot_map_get_chosen_script(const hb_ot_map_t *map, unsigned int table_index);

bool hb_ot_map_has_found_script(const hb_ot_map_t *map, unsigned int table_index);

const lookup_map_t *hb_ot_map_get_lookup(const hb_ot_map_t *map, unsigned int table_index, unsigned int i);

unsigned int hb_ot_map_get_stages_length(const hb_ot_map_t *map, unsigned int table_index);

const stage_map_t *hb_ot_map_get_stage(const hb_ot_map_t *map, unsigned int table_index, unsigned int i);

void hb_ot_map_get_stage_lookups(const hb_ot_map_t *map,
                                 unsigned int table_index,
                                 unsigned int stage,
                                 const struct lookup_map_t **plookups,
                                 unsigned int *lookup_count);

void hb_ot_map_substitute(const hb_ot_map_t *map, const hb_ot_shape_plan_t *plan, hb_font_t *font, hb_buffer_t *buffer);

void hb_ot_map_position(const hb_ot_map_t *map, const hb_ot_shape_plan_t *plan, hb_font_t *font, hb_buffer_t *buffer);

// Builder

hb_ot_map_builder_t *hb_ot_map_builder_init(hb_face_t *face_, const hb_segment_properties_t *props_);

void hb_ot_map_builder_fini(hb_ot_map_builder_t *builder);

void hb_ot_map_builder_compile(hb_ot_map_builder_t *builder, hb_ot_map_t *m, unsigned int *variations_index);

void hb_ot_map_builder_add_feature(hb_ot_map_builder_t *builder, const hb_ot_map_feature_t *feat);

void hb_ot_map_builder_add_feature_full(hb_ot_map_builder_t *builder,
                                        hb_tag_t tag,
                                        hb_ot_map_feature_flags_t flags,
                                        unsigned int value);

void hb_ot_map_builder_enable_feature(hb_ot_map_builder_t *builder,
                                      hb_tag_t tag,
                                      hb_ot_map_feature_flags_t flags,
                                      unsigned int value);

void hb_ot_map_builder_disable_feature(hb_ot_map_builder_t *builder, hb_tag_t tag);

void hb_ot_map_builder_add_gsub_pause(hb_ot_map_builder_t *builder, pause_func_t pause_func);

hb_tag_t hb_ot_map_builder_chosen_script(hb_ot_map_builder_t *builder, unsigned int table_index);
