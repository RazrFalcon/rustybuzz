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
#include "hb-aat-layout-feat-table.hh"

void rb_aat_map_builder_t::add_feature(rb_tag_t tag, unsigned value)
{
    if (!face->table.feat->has_data())
        return;

    if (tag == RB_TAG('a', 'a', 'l', 't')) {
        if (!face->table.feat->exposes_feature(RB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_ALTERNATIVES))
            return;
        feature_info_t *info = features.push();
        info->type = RB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_ALTERNATIVES;
        info->setting = (rb_aat_layout_feature_selector_t)value;
        info->seq = features.length;
        info->is_exclusive = true;
        return;
    }

    const rb_aat_feature_mapping_t *mapping = rb_aat_layout_find_feature_mapping(tag);
    if (!mapping)
        return;

    const AAT::FeatureName *feature = &face->table.feat->get_feature(mapping->aatFeatureType);
    if (!feature->has_data()) {
        /* Special case: Chain::compile_flags will fall back to the deprecated version of
         * small-caps if necessary, so we need to check for that possibility.
         * https://github.com/harfbuzz/harfbuzz/issues/2307 */
        if (mapping->aatFeatureType == RB_AAT_LAYOUT_FEATURE_TYPE_LOWER_CASE &&
            mapping->selectorToEnable == RB_AAT_LAYOUT_FEATURE_SELECTOR_LOWER_CASE_SMALL_CAPS) {
            feature = &face->table.feat->get_feature(RB_AAT_LAYOUT_FEATURE_TYPE_LETTER_CASE);
            if (!feature->has_data())
                return;
        } else
            return;
    }

    feature_info_t *info = features.push();
    info->type = mapping->aatFeatureType;
    info->setting = value ? mapping->selectorToEnable : mapping->selectorToDisable;
    info->seq = features.length;
    info->is_exclusive = feature->is_exclusive();
}

void rb_aat_map_builder_t::compile(rb_aat_map_t &m)
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

    rb_aat_layout_compile_map(this, &m);
}
