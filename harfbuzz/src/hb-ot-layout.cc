/*
 * Copyright © 1998-2004  David Turner and Werner Lemberg
 * Copyright © 2006  Behdad Esfahbod
 * Copyright © 2007,2008,2009  Red Hat, Inc.
 * Copyright © 2012,2013  Google, Inc.
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

#ifndef HB_NO_OT_LAYOUT

#ifdef HB_NO_OT_TAG
#error "Cannot compile hb-ot-layout.cc with HB_NO_OT_TAG."
#endif

#include "hb-map.hh"
#include "hb-open-type.hh"
#include "hb-ot-face.hh"
#include "hb-ot-layout.hh"
#include "hb-ot-map.hh"

#include "hb-ot-kern-table.hh"
#include "hb-ot-layout-gdef-table.hh"
#include "hb-ot-layout-gpos-table.hh"
#include "hb-ot-layout-gsub-table.hh"

#include "hb-aat-layout-morx-table.hh"

/**
 * SECTION:hb-ot-layout
 * @title: hb-ot-layout
 * @short_description: OpenType Layout
 * @include: hb-ot.h
 *
 * Functions for querying OpenType Layout features in the font face.
 **/

/*
 * kern
 */

/**
 * hb_ot_layout_has_kerning:
 * @face: The #hb_face_t to work on
 *
 * Tests whether a face includes any kerning data in the 'kern' table.
 * Does NOT test for kerning lookups in the GPOS table.
 *
 * Return value: true if data found, false otherwise
 *
 **/
bool hb_ot_layout_has_kerning(hb_face_t *face)
{
    return face->table.kern->has_data();
}

/**
 * hb_ot_layout_has_machine_kerning:
 * @face: The #hb_face_t to work on
 *
 * Tests whether a face includes any state-machine kerning in the 'kern' table.
 * Does NOT examine the GPOS table.
 *
 * Return value: true if data found, false otherwise
 *
 **/
bool hb_ot_layout_has_machine_kerning(hb_face_t *face)
{
    return face->table.kern->has_state_machine();
}

/**
 * hb_ot_layout_has_cross_kerning:
 * @face: The #hb_face_t to work on
 *
 * Tests whether a face has any cross-stream kerning (i.e., kerns
 * that make adjustments perpendicular to the direction of the text
 * flow: Y adjustments in horizontal text or X adjustments in
 * vertical text) in the 'kern' table.
 *
 * Does NOT examine the GPOS table.
 *
 * Return value: true is data found, false otherwise
 *
 **/
bool hb_ot_layout_has_cross_kerning(hb_face_t *face)
{
    return face->table.kern->has_cross_stream();
}

void hb_ot_layout_kern(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer)
{
    hb_blob_t *blob = font->face->table.kern.get_blob();
    const AAT::kern &kern = *blob->as<AAT::kern>();

    AAT::hb_aat_apply_context_t c(plan, font, buffer, blob);

    kern.apply(&c);
}

/*
 * GDEF
 */

bool OT::GDEF::is_blacklisted(hb_blob_t *blob, hb_face_t *face) const
{
#ifdef HB_NO_OT_LAYOUT_BLACKLIST
    return false;
#endif
    /* The ugly business of blacklisting individual fonts' tables happen here!
     * See this thread for why we finally had to bend in and do this:
     * https://lists.freedesktop.org/archives/harfbuzz/2016-February/005489.html
     *
     * In certain versions of Times New Roman Italic and Bold Italic,
     * ASCII double quotation mark U+0022 has wrong glyph class 3 (mark)
     * in GDEF.  Many versions of Tahoma have bad GDEF tables that
     * incorrectly classify some spacing marks such as certain IPA
     * symbols as glyph class 3. So do older versions of Microsoft
     * Himalaya, and the version of Cantarell shipped by Ubuntu 16.04.
     *
     * Nuke the GDEF tables of to avoid unwanted width-zeroing.
     *
     * See https://bugzilla.mozilla.org/show_bug.cgi?id=1279925
     *     https://bugzilla.mozilla.org/show_bug.cgi?id=1279693
     *     https://bugzilla.mozilla.org/show_bug.cgi?id=1279875
     */
    switch
        HB_CODEPOINT_ENCODE3(blob->length, face->table.GSUB->table.get_length(), face->table.GPOS->table.get_length())
        {
        /* sha1sum:c5ee92f0bca4bfb7d06c4d03e8cf9f9cf75d2e8a Windows 7? timesi.ttf */
        case HB_CODEPOINT_ENCODE3(442, 2874, 42038):
        /* sha1sum:37fc8c16a0894ab7b749e35579856c73c840867b Windows 7? timesbi.ttf */
        case HB_CODEPOINT_ENCODE3(430, 2874, 40662):
        /* sha1sum:19fc45110ea6cd3cdd0a5faca256a3797a069a80 Windows 7 timesi.ttf */
        case HB_CODEPOINT_ENCODE3(442, 2874, 39116):
        /* sha1sum:6d2d3c9ed5b7de87bc84eae0df95ee5232ecde26 Windows 7 timesbi.ttf */
        case HB_CODEPOINT_ENCODE3(430, 2874, 39374):
        /* sha1sum:8583225a8b49667c077b3525333f84af08c6bcd8 OS X 10.11.3 Times New Roman Italic.ttf */
        case HB_CODEPOINT_ENCODE3(490, 3046, 41638):
        /* sha1sum:ec0f5a8751845355b7c3271d11f9918a966cb8c9 OS X 10.11.3 Times New Roman Bold Italic.ttf */
        case HB_CODEPOINT_ENCODE3(478, 3046, 41902):
        /* sha1sum:96eda93f7d33e79962451c6c39a6b51ee893ce8c  tahoma.ttf from Windows 8 */
        case HB_CODEPOINT_ENCODE3(898, 12554, 46470):
        /* sha1sum:20928dc06014e0cd120b6fc942d0c3b1a46ac2bc  tahomabd.ttf from Windows 8 */
        case HB_CODEPOINT_ENCODE3(910, 12566, 47732):
        /* sha1sum:4f95b7e4878f60fa3a39ca269618dfde9721a79e  tahoma.ttf from Windows 8.1 */
        case HB_CODEPOINT_ENCODE3(928, 23298, 59332):
        /* sha1sum:6d400781948517c3c0441ba42acb309584b73033  tahomabd.ttf from Windows 8.1 */
        case HB_CODEPOINT_ENCODE3(940, 23310, 60732):
        /* tahoma.ttf v6.04 from Windows 8.1 x64, see https://bugzilla.mozilla.org/show_bug.cgi?id=1279925 */
        case HB_CODEPOINT_ENCODE3(964, 23836, 60072):
        /* tahomabd.ttf v6.04 from Windows 8.1 x64, see https://bugzilla.mozilla.org/show_bug.cgi?id=1279925 */
        case HB_CODEPOINT_ENCODE3(976, 23832, 61456):
        /* sha1sum:e55fa2dfe957a9f7ec26be516a0e30b0c925f846  tahoma.ttf from Windows 10 */
        case HB_CODEPOINT_ENCODE3(994, 24474, 60336):
        /* sha1sum:7199385abb4c2cc81c83a151a7599b6368e92343  tahomabd.ttf from Windows 10 */
        case HB_CODEPOINT_ENCODE3(1006, 24470, 61740):
        /* tahoma.ttf v6.91 from Windows 10 x64, see https://bugzilla.mozilla.org/show_bug.cgi?id=1279925 */
        case HB_CODEPOINT_ENCODE3(1006, 24576, 61346):
        /* tahomabd.ttf v6.91 from Windows 10 x64, see https://bugzilla.mozilla.org/show_bug.cgi?id=1279925 */
        case HB_CODEPOINT_ENCODE3(1018, 24572, 62828):
        /* sha1sum:b9c84d820c49850d3d27ec498be93955b82772b5  tahoma.ttf from Windows 10 AU */
        case HB_CODEPOINT_ENCODE3(1006, 24576, 61352):
        /* sha1sum:2bdfaab28174bdadd2f3d4200a30a7ae31db79d2  tahomabd.ttf from Windows 10 AU */
        case HB_CODEPOINT_ENCODE3(1018, 24572, 62834):
        /* sha1sum:b0d36cf5a2fbe746a3dd277bffc6756a820807a7  Tahoma.ttf from Mac OS X 10.9 */
        case HB_CODEPOINT_ENCODE3(832, 7324, 47162):
        /* sha1sum:12fc4538e84d461771b30c18b5eb6bd434e30fba  Tahoma Bold.ttf from Mac OS X 10.9 */
        case HB_CODEPOINT_ENCODE3(844, 7302, 45474):
        /* sha1sum:eb8afadd28e9cf963e886b23a30b44ab4fd83acc  himalaya.ttf from Windows 7 */
        case HB_CODEPOINT_ENCODE3(180, 13054, 7254):
        /* sha1sum:73da7f025b238a3f737aa1fde22577a6370f77b0  himalaya.ttf from Windows 8 */
        case HB_CODEPOINT_ENCODE3(192, 12638, 7254):
        /* sha1sum:6e80fd1c0b059bbee49272401583160dc1e6a427  himalaya.ttf from Windows 8.1 */
        case HB_CODEPOINT_ENCODE3(192, 12690, 7254):
        /* 8d9267aea9cd2c852ecfb9f12a6e834bfaeafe44  cantarell-fonts-0.0.21/otf/Cantarell-Regular.otf */
        /* 983988ff7b47439ab79aeaf9a45bd4a2c5b9d371  cantarell-fonts-0.0.21/otf/Cantarell-Oblique.otf */
        case HB_CODEPOINT_ENCODE3(188, 248, 3852):
        /* 2c0c90c6f6087ffbfea76589c93113a9cbb0e75f  cantarell-fonts-0.0.21/otf/Cantarell-Bold.otf */
        /* 55461f5b853c6da88069ffcdf7f4dd3f8d7e3e6b  cantarell-fonts-0.0.21/otf/Cantarell-Bold-Oblique.otf */
        case HB_CODEPOINT_ENCODE3(188, 264, 3426):
        /* d125afa82a77a6475ac0e74e7c207914af84b37a padauk-2.80/Padauk.ttf RHEL 7.2 */
        case HB_CODEPOINT_ENCODE3(1058, 47032, 11818):
        /* 0f7b80437227b90a577cc078c0216160ae61b031 padauk-2.80/Padauk-Bold.ttf RHEL 7.2*/
        case HB_CODEPOINT_ENCODE3(1046, 47030, 12600):
        /* d3dde9aa0a6b7f8f6a89ef1002e9aaa11b882290 padauk-2.80/Padauk.ttf Ubuntu 16.04 */
        case HB_CODEPOINT_ENCODE3(1058, 71796, 16770):
        /* 5f3c98ccccae8a953be2d122c1b3a77fd805093f padauk-2.80/Padauk-Bold.ttf Ubuntu 16.04 */
        case HB_CODEPOINT_ENCODE3(1046, 71790, 17862):
        /* 6c93b63b64e8b2c93f5e824e78caca555dc887c7 padauk-2.80/Padauk-book.ttf */
        case HB_CODEPOINT_ENCODE3(1046, 71788, 17112):
        /* d89b1664058359b8ec82e35d3531931125991fb9 padauk-2.80/Padauk-bookbold.ttf */
        case HB_CODEPOINT_ENCODE3(1058, 71794, 17514):
        /* 824cfd193aaf6234b2b4dc0cf3c6ef576c0d00ef padauk-3.0/Padauk-book.ttf */
        case HB_CODEPOINT_ENCODE3(1330, 109904, 57938):
        /* 91fcc10cf15e012d27571e075b3b4dfe31754a8a padauk-3.0/Padauk-bookbold.ttf */
        case HB_CODEPOINT_ENCODE3(1330, 109904, 58972):
        /* sha1sum: c26e41d567ed821bed997e937bc0c41435689e85  Padauk.ttf
         *  "Padauk Regular" "Version 2.5", see https://crbug.com/681813 */
        case HB_CODEPOINT_ENCODE3(1004, 59092, 14836):
            return true;
        }
    return false;
}

static void _hb_ot_layout_set_glyph_props(hb_font_t *font, rb_buffer_t *buffer)
{
    const OT::GDEF &gdef = *font->face->table.GDEF->table;
    unsigned int count = rb_buffer_get_length(buffer);
    for (unsigned int i = 0; i < count; i++) {
        _hb_glyph_info_set_glyph_props(&rb_buffer_get_info(buffer)[i],
                                       gdef.get_glyph_props(font, rb_buffer_get_info(buffer)[i].codepoint));
        _hb_glyph_info_clear_lig_props(&rb_buffer_get_info(buffer)[i]);
        rb_buffer_get_info(buffer)[i].syllable() = 0;
    }
}

/* Public API */

/*
 * GSUB/GPOS
 */

bool OT::GSUB::is_blacklisted(hb_blob_t *blob HB_UNUSED, hb_face_t *face) const
{
#ifdef HB_NO_OT_LAYOUT_BLACKLIST
    return false;
#endif

    /* Mac OS X prefers morx over GSUB.  It also ships with various Indic fonts,
     * all by 'MUTF' foundry (Tamil MN, Tamil Sangam MN, etc.), that have broken
     * GSUB/GPOS tables.  Some have GSUB with zero scripts, those are ignored by
     * our morx/GSUB preference code.  But if GSUB has non-zero scripts, we tend
     * to prefer it over morx because we want to be consistent with other OpenType
     * shapers.
     *
     * To work around broken Indic Mac system fonts, we ignore GSUB table if
     * OS/2 VendorId is 'MUTF' and font has morx table as well.
     *
     * https://github.com/harfbuzz/harfbuzz/issues/1410
     * https://github.com/harfbuzz/harfbuzz/issues/1348
     * https://github.com/harfbuzz/harfbuzz/issues/1391
     */
    //  if (unlikely (face->table.OS2->achVendID == HB_TAG ('M','U','T','F') &&
    //		face->table.morx->has_data ()))
    //    return true;

    return false;
}

bool OT::GPOS::is_blacklisted(hb_blob_t *blob HB_UNUSED, hb_face_t *face HB_UNUSED) const
{
#ifdef HB_NO_OT_LAYOUT_BLACKLIST
    return false;
#endif
    return false;
}

/*
 * OT::GSUB
 */

/**
 * hb_ot_layout_lookup_would_substitute:
 * @face: #hb_face_t to work upon
 * @lookup_index: The index of the lookup to query
 * @glyphs: The sequence of glyphs to query for substitution
 * @glyphs_length: The length of the glyph sequence
 * @zero_context: #hb_bool_t indicating whether substitutions should be context-free
 *
 * Tests whether a specified lookup in the specified face would
 * trigger a substitution on the given glyph sequence.
 *
 * Return value: true if a substitution would be triggered, false otherwise
 *
 * Since: 0.9.7
 **/
hb_bool_t hb_ot_layout_lookup_would_substitute(hb_face_t *face,
                                               unsigned int lookup_index,
                                               const hb_codepoint_t *glyphs,
                                               unsigned int glyphs_length,
                                               hb_bool_t zero_context)
{
    if (unlikely(lookup_index >= face->table.GSUB->lookup_count))
        return false;
    OT::hb_would_apply_context_t c(face, glyphs, glyphs_length, (bool)zero_context);

    const OT::SubstLookup &l = face->table.GSUB->table->get_lookup(lookup_index);

    return l.would_apply(&c, &face->table.GSUB->accels[lookup_index]);
}

/**
 * hb_ot_layout_substitute_start:
 * @font: #hb_font_t to use
 * @buffer: #hb_buffer_t buffer to work upon
 *
 * Called before substitution lookups are performed, to ensure that glyph
 * class and other properties are set on the glyphs in the buffer.
 *
 **/
void hb_ot_layout_substitute_start(hb_font_t *font, rb_buffer_t *buffer)
{
    _hb_ot_layout_set_glyph_props(font, buffer);
}

void hb_ot_layout_delete_glyphs_inplace(rb_buffer_t *buffer, bool (*filter)(const hb_glyph_info_t *info))
{
    /* Merge clusters and delete filtered glyphs.
     * NOTE! We can't use out-buffer as we have positioning data. */
    unsigned int j = 0;
    unsigned int count = rb_buffer_get_length(buffer);
    hb_glyph_info_t *info = rb_buffer_get_info(buffer);
    hb_glyph_position_t *pos = rb_buffer_get_pos(buffer);
    for (unsigned int i = 0; i < count; i++) {
        if (filter(&info[i])) {
            /* Merge clusters.
             * Same logic as buffer->delete_glyph(), but for in-place removal. */

            unsigned int cluster = info[i].cluster;
            if (i + 1 < count && cluster == info[i + 1].cluster)
                continue; /* Cluster survives; do nothing. */

            if (j) {
                /* Merge cluster backward. */
                if (cluster < info[j - 1].cluster) {
                    unsigned int mask = info[i].mask;
                    unsigned int old_cluster = info[j - 1].cluster;
                    for (unsigned k = j; k && info[k - 1].cluster == old_cluster; k--)
                        rb_buffer_set_cluster(&info[k - 1], cluster, mask);
                }
                continue;
            }

            if (i + 1 < count)
                rb_buffer_merge_clusters(buffer, i, i + 2); /* Merge cluster forward. */

            continue;
        }

        if (j != i) {
            info[j] = info[i];
            pos[j] = pos[i];
        }
        j++;
    }
    rb_buffer_set_length(buffer, j);
}

/*
 * OT::GPOS
 */

/**
 * hb_ot_layout_position_start:
 * @font: #hb_font_t to use
 * @buffer: #hb_buffer_t buffer to work upon
 *
 * Called before positioning lookups are performed, to ensure that glyph
 * attachment types and glyph-attachment chains are set for the glyphs in the buffer.
 *
 **/
void hb_ot_layout_position_start(hb_font_t *font, rb_buffer_t *buffer)
{
    OT::GPOS::position_start(font, buffer);
}

/**
 * hb_ot_layout_position_finish_advances:
 * @font: #hb_font_t to use
 * @buffer: #hb_buffer_t buffer to work upon
 *
 * Called after positioning lookups are performed, to finish glyph advances.
 *
 **/
void hb_ot_layout_position_finish_advances(hb_font_t *font, rb_buffer_t *buffer)
{
    OT::GPOS::position_finish_advances(font, buffer);
}

/**
 * hb_ot_layout_position_finish_offsets:
 * @font: #hb_font_t to use
 * @buffer: #hb_buffer_t buffer to work upon
 *
 * Called after positioning lookups are performed, to finish glyph offsets.
 *
 **/
void hb_ot_layout_position_finish_offsets(hb_font_t *font, rb_buffer_t *buffer)
{
    OT::GPOS::position_finish_offsets(font, buffer);
}

/*
 * Parts of different types are implemented here such that they have direct
 * access to GSUB/GPOS lookups.
 */

static inline bool apply_forward(OT::hb_ot_apply_context_t *c, const OT::hb_ot_layout_lookup_accelerator_t &accel)
{
    bool ret = false;
    rb_buffer_t *buffer = c->buffer;
    while (rb_buffer_get_idx(buffer) < rb_buffer_get_length(buffer)) {
        bool applied = false;
        if (accel.may_have(rb_buffer_get_cur(buffer, 0)->codepoint) &&
            (rb_buffer_get_cur(buffer, 0)->mask & c->lookup_mask) &&
            c->check_glyph_property(rb_buffer_get_cur(buffer, 0), c->lookup_props)) {
            applied = accel.apply(c);
        }

        if (applied)
            ret = true;
        else
            rb_buffer_next_glyph(buffer);
    }
    return ret;
}

static inline bool apply_backward(OT::hb_ot_apply_context_t *c, const OT::hb_ot_layout_lookup_accelerator_t &accel)
{
    bool ret = false;
    rb_buffer_t *buffer = c->buffer;
    do {
        if (accel.may_have(rb_buffer_get_cur(buffer, 0)->codepoint) &&
            (rb_buffer_get_cur(buffer, 0)->mask & c->lookup_mask) &&
            c->check_glyph_property(rb_buffer_get_cur(buffer, 0), c->lookup_props))
            ret |= accel.apply(c);

        /* The reverse lookup doesn't "advance" cursor (for good reason). */
        rb_buffer_set_idx(buffer, rb_buffer_get_idx(buffer) - 1);

    } while ((int)rb_buffer_get_idx(buffer) >= 0);
    return ret;
}

static inline void apply_string_gsub(OT::hb_ot_apply_context_t *c,
                                     const OT::SubstLookup &lookup,
                                     const OT::hb_ot_layout_lookup_accelerator_t &accel)
{
    rb_buffer_t *buffer = c->buffer;

    if (unlikely(!rb_buffer_get_length(buffer) || !c->lookup_mask))
        return;

    c->set_lookup_props(lookup.get_props());

    if (likely(!lookup.is_reverse())) {
        /* in/out forward substitution/positioning */
        rb_buffer_clear_output(buffer);
        rb_buffer_set_idx(buffer, 0);

        bool ret;
        ret = apply_forward(c, accel);
        if (ret) {
            rb_buffer_swap_buffers(buffer);
        }
    } else {
        /* in-place backward substitution/positioning */
        rb_buffer_remove_output(buffer);
        rb_buffer_set_idx(buffer, rb_buffer_get_length(buffer) - 1);

        apply_backward(c, accel);
    }
}

static void apply_gsub(const rb_ot_map_t *map,
                       const OT::GSUB &table,
                       const OT::hb_ot_layout_lookup_accelerator_t *accels,
                       const hb_shape_plan_t *plan,
                       hb_font_t *font,
                       rb_buffer_t *buffer)
{
    const unsigned int table_index = 0;
    unsigned int i = 0;
    OT::hb_ot_apply_context_t c(table_index, font, buffer);
    c.set_recurse_func(OT::SubstLookup::apply_recurse_func);

    for (unsigned int stage_index = 0; stage_index < rb_ot_map_get_stages_length(map, table_index); stage_index++) {
        const auto *stage = rb_ot_map_get_stage(map, table_index, stage_index);
        for (; i < stage->last_lookup; i++) {
            unsigned int lookup_index = rb_ot_map_get_lookup(map, table_index, i)->index;
            c.set_lookup_index(lookup_index);
            c.set_lookup_mask(rb_ot_map_get_lookup(map, table_index, i)->mask);
            c.set_auto_zwj(rb_ot_map_get_lookup(map, table_index, i)->auto_zwj);
            c.set_auto_zwnj(rb_ot_map_get_lookup(map, table_index, i)->auto_zwnj);
            if (rb_ot_map_get_lookup(map, table_index, i)->random) {
                c.set_random(true);
                rb_buffer_unsafe_to_break(buffer, 0, rb_buffer_get_length(buffer));
            }
            apply_string_gsub(&c, table.get_lookup(lookup_index), accels[lookup_index]);
        }

        if (stage->pause_func) {
            rb_buffer_clear_output(buffer);
            stage->pause_func(plan, font, buffer);
        }
    }
}

static inline void apply_string_gpos(OT::hb_ot_apply_context_t *c,
                                     const OT::PosLookup &lookup,
                                     const OT::hb_ot_layout_lookup_accelerator_t &accel)
{
    rb_buffer_t *buffer = c->buffer;

    if (unlikely(!rb_buffer_get_length(buffer) || !c->lookup_mask))
        return;

    c->set_lookup_props(lookup.get_props());

    if (likely(!lookup.is_reverse())) {
        rb_buffer_set_idx(buffer, 0);

        bool ret;
        ret = apply_forward(c, accel);
        if (ret) {
            assert(!rb_buffer_have_separate_output(buffer));
        }
    } else {
        rb_buffer_set_idx(buffer, rb_buffer_get_length(buffer) - 1);

        apply_backward(c, accel);
    }
}

static void apply_gpos(const rb_ot_map_t *map,
                       const OT::GPOS &table,
                       const OT::hb_ot_layout_lookup_accelerator_t *accels,
                       const hb_shape_plan_t *plan,
                       hb_font_t *font,
                       rb_buffer_t *buffer)
{
    const unsigned int table_index = 1;
    unsigned int i = 0;
    OT::hb_ot_apply_context_t c(table_index, font, buffer);
    c.set_recurse_func(OT::PosLookup::apply_recurse_func);

    for (unsigned int stage_index = 0; stage_index < rb_ot_map_get_stages_length(map, table_index); stage_index++) {
        const auto *stage = rb_ot_map_get_stage(map, table_index, stage_index);
        for (; i < stage->last_lookup; i++) {
            unsigned int lookup_index = rb_ot_map_get_lookup(map, table_index, i)->index;
            c.set_lookup_index(lookup_index);
            c.set_lookup_mask(rb_ot_map_get_lookup(map, table_index, i)->mask);
            c.set_auto_zwj(rb_ot_map_get_lookup(map, table_index, i)->auto_zwj);
            c.set_auto_zwnj(rb_ot_map_get_lookup(map, table_index, i)->auto_zwnj);
            if (rb_ot_map_get_lookup(map, table_index, i)->random) {
                c.set_random(true);
                rb_buffer_unsafe_to_break(buffer, 0, rb_buffer_get_length(buffer));
            }
            apply_string_gpos(&c, table.get_lookup(lookup_index), accels[lookup_index]);
        }

        if (stage->pause_func) {
            rb_buffer_clear_output(buffer);
            stage->pause_func(plan, font, buffer);
        }
    }
}

void hb_ot_layout_substitute(const rb_ot_map_t *map, const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer)
{
    apply_gsub(map, *font->face->table.GSUB->table, font->face->table.GSUB->accels, plan, font, buffer);
}

void hb_ot_layout_position(const rb_ot_map_t *map, const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer)
{
    apply_gpos(map, *font->face->table.GPOS->table, font->face->table.GPOS->accels, plan, font, buffer);
}

void hb_ot_layout_substitute_lookup(OT::hb_ot_apply_context_t *c,
                                    const OT::SubstLookup &lookup,
                                    const OT::hb_ot_layout_lookup_accelerator_t &accel)
{
    apply_string_gsub(c, lookup, accel);
}

void hb_layout_clear_syllables(const hb_shape_plan_t *plan, hb_font_t *font, rb_buffer_t *buffer)
{
    _hb_clear_syllables(plan, font, buffer);
}

#endif
