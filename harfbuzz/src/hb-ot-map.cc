/*
 * Copyright © 2009,2010  Red Hat, Inc.
 * Copyright © 2010,2011,2013  Google, Inc.
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

#include "hb.hh"

#include "hb-ot-map.hh"
#include "hb-ot-shape.hh"
#include "hb-ot-layout.hh"

rb_mask_t rb_ot_map_get_1_mask(const rb_ot_map_t *map, rb_tag_t tag)
{
    return map->get_1_mask(tag);
}

rb_mask_t rb_ot_map_global_mask(const rb_ot_map_t *map)
{
    return map->get_global_mask();
}

bool rb_ot_map_get_found_script(const rb_ot_map_t *map, unsigned int index)
{
    return map->found_script[index];
}

rb_tag_t rb_ot_map_get_chosen_script(const rb_ot_map_t *map, unsigned int index)
{
    return map->chosen_script[index];
}

unsigned int rb_ot_map_get_feature_stage(const rb_ot_map_t *map, unsigned int table_index, rb_tag_t feature_tag)
{
    return map->get_feature_stage(table_index, feature_tag);
}

void rb_ot_map_get_stages(const rb_ot_map_t *map,
                          unsigned int table_index,
                          const struct rb_ot_map_stage_map_t **pstages,
                          unsigned int *stage_count)
{
    map->get_stages(table_index, pstages, stage_count);
}

void rb_ot_map_get_stage_lookups(const rb_ot_map_t *map,
                                 unsigned int table_index,
                                 unsigned int stage,
                                 const struct rb_ot_map_lookup_map_t **plookups,
                                 unsigned int *lookup_count)
{
    map->get_stage_lookups(table_index, stage, plookups, lookup_count);
}

rb_ot_map_builder_t::rb_ot_map_builder_t(rb_face_t *face_, const rb_segment_properties_t *props_)
{
    memset(this, 0, sizeof(*this));

    feature_infos.init();
    for (unsigned int table_index = 0; table_index < 2; table_index++)
        stages[table_index].init();

    face = face_;
    props = *props_;

    /* Fetch script/language indices for GSUB/GPOS.  We need these later to skip
     * features not available in either table and not waste precious bits for them. */

    unsigned int script_count = RB_OT_MAX_TAGS_PER_SCRIPT;
    unsigned int language_count = RB_OT_MAX_TAGS_PER_LANGUAGE;
    rb_tag_t script_tags[RB_OT_MAX_TAGS_PER_SCRIPT];
    rb_tag_t language_tags[RB_OT_MAX_TAGS_PER_LANGUAGE];

    rb_ot_tags_from_script_and_language(
        props.script, props.language, &script_count, script_tags, &language_count, language_tags);

    for (unsigned int table_index = 0; table_index < 2; table_index++) {
        rb_tag_t table_tag = table_tags[table_index];
        found_script[table_index] = (bool)rb_ot_layout_table_select_script(
            face, table_tag, script_count, script_tags, &script_index[table_index], &chosen_script[table_index]);
        rb_ot_layout_script_select_language(
            face, table_tag, script_index[table_index], language_count, language_tags, &language_index[table_index]);
    }
}

rb_ot_map_builder_t::~rb_ot_map_builder_t()
{
    feature_infos.fini();
    for (unsigned int table_index = 0; table_index < 2; table_index++)
        stages[table_index].fini();
}

void rb_ot_map_builder_t::add_feature(rb_tag_t tag, rb_ot_map_feature_flags_t flags, unsigned int value)
{
    if (unlikely(!tag))
        return;
    feature_info_t *info = feature_infos.push();
    info->tag = tag;
    info->seq = feature_infos.length;
    info->max_value = value;
    info->flags = flags;
    info->default_value = (flags & F_GLOBAL) ? value : 0;
    info->stage[0] = current_stage[0];
    info->stage[1] = current_stage[1];
}

void rb_ot_map_builder_add_feature(rb_ot_map_builder_t *builder,
                                   rb_tag_t tag,
                                   rb_ot_map_feature_flags_t flags,
                                   unsigned int value)
{
    builder->add_feature(tag, flags, value);
}

void rb_ot_map_builder_t::add_lookups(rb_ot_map_t &m,
                                      unsigned int table_index,
                                      unsigned int feature_index,
                                      unsigned int variations_index,
                                      rb_mask_t mask,
                                      bool auto_zwnj,
                                      bool auto_zwj,
                                      bool random)
{
    unsigned int lookup_indices[32];
    unsigned int offset, len;
    unsigned int table_lookup_count;

    table_lookup_count = rb_ot_layout_table_get_lookup_count(face, table_tags[table_index]);

    offset = 0;
    do {
        len = ARRAY_LENGTH(lookup_indices);
        rb_ot_layout_feature_with_variations_get_lookups(
            face, table_tags[table_index], feature_index, variations_index, offset, &len, lookup_indices);

        for (unsigned int i = 0; i < len; i++) {
            if (lookup_indices[i] >= table_lookup_count)
                continue;
            rb_ot_map_lookup_map_t *lookup = m.lookups[table_index].push();
            lookup->mask = mask;
            lookup->index = lookup_indices[i];
            lookup->auto_zwnj = auto_zwnj;
            lookup->auto_zwj = auto_zwj;
            lookup->random = random;
        }

        offset += len;
    } while (len == ARRAY_LENGTH(lookup_indices));
}

void rb_ot_map_builder_t::add_pause(unsigned int table_index, rb_ot_pause_func_t pause_func)
{
    stage_info_t *s = stages[table_index].push();
    s->index = current_stage[table_index];
    s->pause_func = pause_func;

    current_stage[table_index]++;
}

void rb_ot_map_builder_add_gsub_pause(rb_ot_map_builder_t *builder, rb_ot_pause_func_t pause_func)
{
    builder->add_gsub_pause(pause_func);
}

void rb_ot_map_builder_add_gpos_pause(rb_ot_map_builder_t *builder, rb_ot_pause_func_t pause_func)
{
    builder->add_gpos_pause(pause_func);
}

void rb_ot_map_builder_t::compile(rb_ot_map_t &m, unsigned int *variations_index)
{
    static_assert((!(RB_GLYPH_FLAG_DEFINED & (RB_GLYPH_FLAG_DEFINED + 1))), "");
    unsigned int global_bit_mask = RB_GLYPH_FLAG_DEFINED + 1;
    unsigned int global_bit_shift = rb_popcount(RB_GLYPH_FLAG_DEFINED);

    m.global_mask = global_bit_mask;

    unsigned int required_feature_index[2];
    rb_tag_t required_feature_tag[2];
    /* We default to applying required feature in stage 0.  If the required
     * feature has a tag that is known to the shaper, we apply required feature
     * in the stage for that tag.
     */
    unsigned int required_feature_stage[2] = {0, 0};

    for (unsigned int table_index = 0; table_index < 2; table_index++) {
        m.chosen_script[table_index] = chosen_script[table_index];
        m.found_script[table_index] = found_script[table_index];

        rb_ot_layout_language_get_required_feature(face,
                                                   table_tags[table_index],
                                                   script_index[table_index],
                                                   language_index[table_index],
                                                   &required_feature_index[table_index],
                                                   &required_feature_tag[table_index]);
    }

    /* Sort features and merge duplicates */
    if (feature_infos.length) {
        feature_infos.qsort();
        unsigned int j = 0;
        for (unsigned int i = 1; i < feature_infos.length; i++)
            if (feature_infos[i].tag != feature_infos[j].tag)
                feature_infos[++j] = feature_infos[i];
            else {
                if (feature_infos[i].flags & F_GLOBAL) {
                    feature_infos[j].flags |= F_GLOBAL;
                    feature_infos[j].max_value = feature_infos[i].max_value;
                    feature_infos[j].default_value = feature_infos[i].default_value;
                } else {
                    if (feature_infos[j].flags & F_GLOBAL)
                        feature_infos[j].flags ^= F_GLOBAL;
                    feature_infos[j].max_value = rb_max(feature_infos[j].max_value, feature_infos[i].max_value);
                    /* Inherit default_value from j */
                }
                feature_infos[j].flags |= (feature_infos[i].flags & F_HAS_FALLBACK);
                feature_infos[j].stage[0] = rb_min(feature_infos[j].stage[0], feature_infos[i].stage[0]);
                feature_infos[j].stage[1] = rb_min(feature_infos[j].stage[1], feature_infos[i].stage[1]);
            }
        feature_infos.shrink(j + 1);
    }

    /* Allocate bits now */
    unsigned int next_bit = global_bit_shift + 1;

    for (unsigned int i = 0; i < feature_infos.length; i++) {
        const feature_info_t *info = &feature_infos[i];

        unsigned int bits_needed;

        if ((info->flags & F_GLOBAL) && info->max_value == 1)
            /* Uses the global bit */
            bits_needed = 0;
        else
            /* Limit bits per feature. */
            bits_needed = rb_min(RB_OT_MAP_MAX_BITS, rb_bit_storage(info->max_value));

        if (!info->max_value || next_bit + bits_needed > 8 * sizeof(rb_mask_t))
            continue; /* Feature disabled, or not enough bits. */

        bool found = false;
        unsigned int feature_index[2];
        for (unsigned int table_index = 0; table_index < 2; table_index++) {
            if (required_feature_tag[table_index] == info->tag)
                required_feature_stage[table_index] = info->stage[table_index];

            found |= (bool)rb_ot_layout_language_find_feature(face,
                                                              table_tags[table_index],
                                                              script_index[table_index],
                                                              language_index[table_index],
                                                              info->tag,
                                                              &feature_index[table_index]);
        }
        if (!found && (info->flags & F_GLOBAL_SEARCH)) {
            for (unsigned int table_index = 0; table_index < 2; table_index++) {
                found |= (bool)rb_ot_layout_table_find_feature(
                    face, table_tags[table_index], info->tag, &feature_index[table_index]);
            }
        }
        if (!found && !(info->flags & F_HAS_FALLBACK))
            continue;

        rb_ot_map_t::feature_map_t *map = m.features.push();

        map->tag = info->tag;
        map->index[0] = feature_index[0];
        map->index[1] = feature_index[1];
        map->stage[0] = info->stage[0];
        map->stage[1] = info->stage[1];
        map->auto_zwnj = !(info->flags & F_MANUAL_ZWNJ);
        map->auto_zwj = !(info->flags & F_MANUAL_ZWJ);
        map->random = !!(info->flags & F_RANDOM);
        if ((info->flags & F_GLOBAL) && info->max_value == 1) {
            /* Uses the global bit */
            map->shift = global_bit_shift;
            map->mask = global_bit_mask;
        } else {
            map->shift = next_bit;
            map->mask = (1u << (next_bit + bits_needed)) - (1u << next_bit);
            next_bit += bits_needed;
            m.global_mask |= (info->default_value << map->shift) & map->mask;
        }
        map->_1_mask = (1u << map->shift) & map->mask;
        map->needs_fallback = !found;
    }
    feature_infos.shrink(0); /* Done with these */

    add_gsub_pause(nullptr);
    add_gpos_pause(nullptr);

    for (unsigned int table_index = 0; table_index < 2; table_index++) {
        /* Collect lookup indices for features */

        unsigned int stage_index = 0;
        unsigned int last_num_lookups = 0;
        for (unsigned stage = 0; stage < current_stage[table_index]; stage++) {
            if (required_feature_index[table_index] != RB_OT_LAYOUT_NO_FEATURE_INDEX &&
                required_feature_stage[table_index] == stage)
                add_lookups(m,
                            table_index,
                            required_feature_index[table_index],
                            variations_index[table_index],
                            global_bit_mask);

            for (unsigned i = 0; i < m.features.length; i++)
                if (m.features[i].stage[table_index] == stage)
                    add_lookups(m,
                                table_index,
                                m.features[i].index[table_index],
                                variations_index[table_index],
                                m.features[i].mask,
                                m.features[i].auto_zwnj,
                                m.features[i].auto_zwj,
                                m.features[i].random);

            /* Sort lookups and merge duplicates */
            if (last_num_lookups < m.lookups[table_index].length) {
                m.lookups[table_index].qsort(last_num_lookups, m.lookups[table_index].length);

                unsigned int j = last_num_lookups;
                for (unsigned int i = j + 1; i < m.lookups[table_index].length; i++)
                    if (m.lookups[table_index][i].index != m.lookups[table_index][j].index)
                        m.lookups[table_index][++j] = m.lookups[table_index][i];
                    else {
                        m.lookups[table_index][j].mask |= m.lookups[table_index][i].mask;
                        m.lookups[table_index][j].auto_zwnj &= m.lookups[table_index][i].auto_zwnj;
                        m.lookups[table_index][j].auto_zwj &= m.lookups[table_index][i].auto_zwj;
                    }
                m.lookups[table_index].shrink(j + 1);
            }

            last_num_lookups = m.lookups[table_index].length;

            if (stage_index < stages[table_index].length && stages[table_index][stage_index].index == stage) {
                rb_ot_map_stage_map_t *stage_map = m.stages[table_index].push();
                stage_map->last_lookup = last_num_lookups;
                stage_map->pause_func = stages[table_index][stage_index].pause_func;

                stage_index++;
            }
        }
    }
}
