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

#include "hb-aat-map.hh"
#include "hb-aat-layout.hh"

void rb_aat_map_builder_t::compile(rb_aat_map_t *m)
{
    /* Sort features and merge duplicates */
    if (features.length) {
        features.qsort();
        unsigned int j = 0;
        for (unsigned int i = 1; i < features.length; i++)
            if (features[i].type != features[j].type ||
                /* Nonexclusive feature selectors come in even/odd pairs to turn a setting on/off
                 * respectively, so we mask out the low-order bit when checking for "duplicates"
                 * (selectors referring to the same feature setting) here. */
                (!features[i].is_exclusive && ((features[i].setting & ~1) != (features[j].setting & ~1))))
                features[++j] = features[i];
        features.shrink(j + 1);
    }

    rb_aat_layout_compile_map(this, m);
}

void rb_aat_map_init(rb_aat_map_t *map)
{
    map->init();
}

void rb_aat_map_fini(rb_aat_map_t *map)
{
    map->fini();
}

void rb_aat_map_builder_init(rb_aat_map_builder_t *builder, rb_face_t *face)
{
    builder->init(face);
}

void rb_aat_map_builder_fini(rb_aat_map_builder_t *builder)
{
    builder->fini();
}

void rb_aat_map_builder_add_feature(rb_aat_map_builder_t *builder, int type, int setting, bool is_exclusive)
{
    rb_aat_map_builder_t::feature_info_t *info = builder->features.push();
    info->type = (rb_aat_layout_feature_type_t)type;
    info->setting = (rb_aat_layout_feature_selector_t)setting;
    info->seq = builder->features.length;
    info->is_exclusive = is_exclusive;
}

void rb_aat_map_builder_compile(rb_aat_map_builder_t *builder, rb_aat_map_t *map)
{
    builder->compile(map);
}
