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

#include "hb-open-type.hh"
#include "hb-ot-layout.hh"
#include "hb-ot-face.hh"
#include "hb-ot-map.hh"
#include "hb-map.hh"

#include "hb-ot-kern-table.hh"
#include "hb-ot-layout-gdef-table.hh"
#include "hb-ot-layout-gsub-table.hh"
#include "hb-ot-layout-gpos-table.hh"
#include "hb-ot-layout-gsubgpos.hh"

#include "hb-aat-layout-morx-table.hh"

extern "C" {
unsigned int   rb_would_apply_context_get_len(const OT::rb_would_apply_context_t *c) { return c->len; }
rb_codepoint_t rb_would_apply_context_get_glyph(const OT::rb_would_apply_context_t *c, unsigned int index) { return c->glyphs[index]; }
rb_bool_t      rb_would_apply_context_get_zero_context(const OT::rb_would_apply_context_t *c) { return (rb_bool_t)c->zero_context; }
rb_buffer_t   *rb_ot_apply_context_get_buffer(const OT::rb_ot_apply_context_t *c) { return c->buffer; }
rb_mask_t      rb_ot_apply_context_get_lookup_mask(const OT::rb_ot_apply_context_t *c) { return c->lookup_mask; }
unsigned int   rb_ot_apply_context_get_table_index(const OT::rb_ot_apply_context_t *c) { return c->table_index; }
unsigned int   rb_ot_apply_context_get_lookup_index(const OT::rb_ot_apply_context_t *c) { return c->lookup_index; }
unsigned int   rb_ot_apply_context_get_lookup_props(const OT::rb_ot_apply_context_t *c) { return c->lookup_props; }
rb_bool_t      rb_ot_apply_context_get_has_glyph_classes(const OT::rb_ot_apply_context_t *c) { return c->has_glyph_classes; }
rb_bool_t      rb_ot_apply_context_get_auto_zwnj(const OT::rb_ot_apply_context_t *c) { return c->auto_zwnj; }
rb_bool_t      rb_ot_apply_context_get_auto_zwj(const OT::rb_ot_apply_context_t *c) { return c->auto_zwj; }
rb_bool_t      rb_ot_apply_context_get_random(const OT::rb_ot_apply_context_t *c) { return (rb_bool_t)c->random; }
rb_bool_t      rb_ot_apply_context_gdef_mark_set_covers(const OT::rb_ot_apply_context_t *c, unsigned int set_index, rb_codepoint_t glyph_id) { return (rb_bool_t)c->gdef.mark_set_covers(set_index, glyph_id); }
unsigned int   rb_ot_apply_context_gdef_get_glyph_props(const OT::rb_ot_apply_context_t *c, rb_codepoint_t glyph_id) { return (rb_bool_t)c->gdef.get_glyph_props(glyph_id); }
uint32_t       rb_ot_apply_context_random_number(OT::rb_ot_apply_context_t *c) { return c->random_number(); }
rb_bool_t      rb_ot_apply_context_recurse(OT::rb_ot_apply_context_t *c, unsigned int sub_lookup_index) { return (rb_bool_t)c->recurse(sub_lookup_index); }
}

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
 * rb_ot_layout_has_kerning:
 * @face: The #rb_face_t to work on
 *
 * Tests whether a face includes any kerning data in the 'kern' table.
 * Does NOT test for kerning lookups in the GPOS table.
 *
 * Return value: true if data found, false otherwise
 *
 **/
bool rb_ot_layout_has_kerning(rb_face_t *face)
{
    return face->table.kern->has_data();
}

/**
 * rb_ot_layout_has_machine_kerning:
 * @face: The #rb_face_t to work on
 *
 * Tests whether a face includes any state-machine kerning in the 'kern' table.
 * Does NOT examine the GPOS table.
 *
 * Return value: true if data found, false otherwise
 *
 **/
bool rb_ot_layout_has_machine_kerning(rb_face_t *face)
{
    return face->table.kern->has_state_machine();
}

/**
 * rb_ot_layout_has_cross_kerning:
 * @face: The #rb_face_t to work on
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
bool rb_ot_layout_has_cross_kerning(rb_face_t *face)
{
    return face->table.kern->has_cross_stream();
}

void rb_ot_layout_kern(const rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer)
{
    rb_blob_t *blob = rb_font_get_face(font)->table.kern.get_blob();
    const AAT::kern &kern = *blob->as<AAT::kern>();

    AAT::rb_aat_apply_context_t c(plan, font, buffer, blob);

    kern.apply(&c);
}

/*
 * GDEF
 */

bool OT::GDEF::is_blocklisted(rb_blob_t *blob, rb_face_t *face) const
{
    /* The ugly business of blocklisting individual fonts' tables happen here!
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
        RB_CODEPOINT_ENCODE3(blob->length, face->table.GSUB->table.get_length(), face->table.GPOS->table.get_length())
        {
        /* sha1sum:c5ee92f0bca4bfb7d06c4d03e8cf9f9cf75d2e8a Windows 7? timesi.ttf */
        case RB_CODEPOINT_ENCODE3(442, 2874, 42038):
        /* sha1sum:37fc8c16a0894ab7b749e35579856c73c840867b Windows 7? timesbi.ttf */
        case RB_CODEPOINT_ENCODE3(430, 2874, 40662):
        /* sha1sum:19fc45110ea6cd3cdd0a5faca256a3797a069a80 Windows 7 timesi.ttf */
        case RB_CODEPOINT_ENCODE3(442, 2874, 39116):
        /* sha1sum:6d2d3c9ed5b7de87bc84eae0df95ee5232ecde26 Windows 7 timesbi.ttf */
        case RB_CODEPOINT_ENCODE3(430, 2874, 39374):
        /* sha1sum:8583225a8b49667c077b3525333f84af08c6bcd8 OS X 10.11.3 Times New Roman Italic.ttf */
        case RB_CODEPOINT_ENCODE3(490, 3046, 41638):
        /* sha1sum:ec0f5a8751845355b7c3271d11f9918a966cb8c9 OS X 10.11.3 Times New Roman Bold Italic.ttf */
        case RB_CODEPOINT_ENCODE3(478, 3046, 41902):
        /* sha1sum:96eda93f7d33e79962451c6c39a6b51ee893ce8c  tahoma.ttf from Windows 8 */
        case RB_CODEPOINT_ENCODE3(898, 12554, 46470):
        /* sha1sum:20928dc06014e0cd120b6fc942d0c3b1a46ac2bc  tahomabd.ttf from Windows 8 */
        case RB_CODEPOINT_ENCODE3(910, 12566, 47732):
        /* sha1sum:4f95b7e4878f60fa3a39ca269618dfde9721a79e  tahoma.ttf from Windows 8.1 */
        case RB_CODEPOINT_ENCODE3(928, 23298, 59332):
        /* sha1sum:6d400781948517c3c0441ba42acb309584b73033  tahomabd.ttf from Windows 8.1 */
        case RB_CODEPOINT_ENCODE3(940, 23310, 60732):
        /* tahoma.ttf v6.04 from Windows 8.1 x64, see https://bugzilla.mozilla.org/show_bug.cgi?id=1279925 */
        case RB_CODEPOINT_ENCODE3(964, 23836, 60072):
        /* tahomabd.ttf v6.04 from Windows 8.1 x64, see https://bugzilla.mozilla.org/show_bug.cgi?id=1279925 */
        case RB_CODEPOINT_ENCODE3(976, 23832, 61456):
        /* sha1sum:e55fa2dfe957a9f7ec26be516a0e30b0c925f846  tahoma.ttf from Windows 10 */
        case RB_CODEPOINT_ENCODE3(994, 24474, 60336):
        /* sha1sum:7199385abb4c2cc81c83a151a7599b6368e92343  tahomabd.ttf from Windows 10 */
        case RB_CODEPOINT_ENCODE3(1006, 24470, 61740):
        /* tahoma.ttf v6.91 from Windows 10 x64, see https://bugzilla.mozilla.org/show_bug.cgi?id=1279925 */
        case RB_CODEPOINT_ENCODE3(1006, 24576, 61346):
        /* tahomabd.ttf v6.91 from Windows 10 x64, see https://bugzilla.mozilla.org/show_bug.cgi?id=1279925 */
        case RB_CODEPOINT_ENCODE3(1018, 24572, 62828):
        /* sha1sum:b9c84d820c49850d3d27ec498be93955b82772b5  tahoma.ttf from Windows 10 AU */
        case RB_CODEPOINT_ENCODE3(1006, 24576, 61352):
        /* sha1sum:2bdfaab28174bdadd2f3d4200a30a7ae31db79d2  tahomabd.ttf from Windows 10 AU */
        case RB_CODEPOINT_ENCODE3(1018, 24572, 62834):
        /* sha1sum:b0d36cf5a2fbe746a3dd277bffc6756a820807a7  Tahoma.ttf from Mac OS X 10.9 */
        case RB_CODEPOINT_ENCODE3(832, 7324, 47162):
        /* sha1sum:12fc4538e84d461771b30c18b5eb6bd434e30fba  Tahoma Bold.ttf from Mac OS X 10.9 */
        case RB_CODEPOINT_ENCODE3(844, 7302, 45474):
        /* sha1sum:eb8afadd28e9cf963e886b23a30b44ab4fd83acc  himalaya.ttf from Windows 7 */
        case RB_CODEPOINT_ENCODE3(180, 13054, 7254):
        /* sha1sum:73da7f025b238a3f737aa1fde22577a6370f77b0  himalaya.ttf from Windows 8 */
        case RB_CODEPOINT_ENCODE3(192, 12638, 7254):
        /* sha1sum:6e80fd1c0b059bbee49272401583160dc1e6a427  himalaya.ttf from Windows 8.1 */
        case RB_CODEPOINT_ENCODE3(192, 12690, 7254):
        /* 8d9267aea9cd2c852ecfb9f12a6e834bfaeafe44  cantarell-fonts-0.0.21/otf/Cantarell-Regular.otf */
        /* 983988ff7b47439ab79aeaf9a45bd4a2c5b9d371  cantarell-fonts-0.0.21/otf/Cantarell-Oblique.otf */
        case RB_CODEPOINT_ENCODE3(188, 248, 3852):
        /* 2c0c90c6f6087ffbfea76589c93113a9cbb0e75f  cantarell-fonts-0.0.21/otf/Cantarell-Bold.otf */
        /* 55461f5b853c6da88069ffcdf7f4dd3f8d7e3e6b  cantarell-fonts-0.0.21/otf/Cantarell-Bold-Oblique.otf */
        case RB_CODEPOINT_ENCODE3(188, 264, 3426):
        /* d125afa82a77a6475ac0e74e7c207914af84b37a padauk-2.80/Padauk.ttf RHEL 7.2 */
        case RB_CODEPOINT_ENCODE3(1058, 47032, 11818):
        /* 0f7b80437227b90a577cc078c0216160ae61b031 padauk-2.80/Padauk-Bold.ttf RHEL 7.2*/
        case RB_CODEPOINT_ENCODE3(1046, 47030, 12600):
        /* d3dde9aa0a6b7f8f6a89ef1002e9aaa11b882290 padauk-2.80/Padauk.ttf Ubuntu 16.04 */
        case RB_CODEPOINT_ENCODE3(1058, 71796, 16770):
        /* 5f3c98ccccae8a953be2d122c1b3a77fd805093f padauk-2.80/Padauk-Bold.ttf Ubuntu 16.04 */
        case RB_CODEPOINT_ENCODE3(1046, 71790, 17862):
        /* 6c93b63b64e8b2c93f5e824e78caca555dc887c7 padauk-2.80/Padauk-book.ttf */
        case RB_CODEPOINT_ENCODE3(1046, 71788, 17112):
        /* d89b1664058359b8ec82e35d3531931125991fb9 padauk-2.80/Padauk-bookbold.ttf */
        case RB_CODEPOINT_ENCODE3(1058, 71794, 17514):
        /* 824cfd193aaf6234b2b4dc0cf3c6ef576c0d00ef padauk-3.0/Padauk-book.ttf */
        case RB_CODEPOINT_ENCODE3(1330, 109904, 57938):
        /* 91fcc10cf15e012d27571e075b3b4dfe31754a8a padauk-3.0/Padauk-bookbold.ttf */
        case RB_CODEPOINT_ENCODE3(1330, 109904, 58972):
        /* sha1sum: c26e41d567ed821bed997e937bc0c41435689e85  Padauk.ttf
         *  "Padauk Regular" "Version 2.5", see https://crbug.com/681813 */
        case RB_CODEPOINT_ENCODE3(1004, 59092, 14836):
            return true;
        }
    return false;
}

static void _rb_ot_layout_set_glyph_props(rb_font_t *font, rb_buffer_t *buffer)
{
    const OT::GDEF &gdef = *rb_font_get_face(font)->table.GDEF->table;
    unsigned int count = rb_buffer_get_length(buffer);
    for (unsigned int i = 0; i < count; i++) {
        _rb_glyph_info_set_glyph_props(&rb_buffer_get_glyph_infos(buffer)[i],
                                       gdef.get_glyph_props(rb_buffer_get_glyph_infos(buffer)[i].codepoint));
        _rb_glyph_info_clear_lig_props(&rb_buffer_get_glyph_infos(buffer)[i]);
        rb_buffer_get_glyph_infos(buffer)[i].syllable() = 0;
    }
}

/* Public API */

/**
 * rb_ot_layout_has_glyph_classes:
 * @face: #rb_face_t to work upon
 *
 * Tests whether a face has any glyph classes defined in its GDEF table.
 *
 * Return value: true if data found, false otherwise
 *
 **/
rb_bool_t rb_ot_layout_has_glyph_classes(rb_face_t *face)
{
    return face->table.GDEF->table->has_glyph_classes();
}

/*
 * GSUB/GPOS
 */

bool OT::GSUB::is_blocklisted(rb_blob_t *blob RB_UNUSED, rb_face_t *face) const
{
    return false;
}

bool OT::GPOS::is_blocklisted(rb_blob_t *blob RB_UNUSED, rb_face_t *face RB_UNUSED) const
{
    return false;
}

static const OT::GSUBGPOS &get_gsubgpos_table(rb_face_t *face, rb_tag_t table_tag)
{
    switch (table_tag) {
    case RB_OT_TAG_GSUB:
        return *face->table.GSUB->table;
    case RB_OT_TAG_GPOS:
        return *face->table.GPOS->table;
    default:
        return Null(OT::GSUBGPOS);
    }
}

#define RB_OT_TAG_LATIN_SCRIPT RB_TAG('l', 'a', 't', 'n')

/**
 * rb_ot_layout_table_select_script:
 * @face: #rb_face_t to work upon
 * @table_tag: RB_OT_TAG_GSUB or RB_OT_TAG_GPOS
 * @script_count: Number of script tags in the array
 * @script_tags: Array of #rb_tag_t script tags
 * @script_index: (out): The index of the requested script
 * @chosen_script: (out): #rb_tag_t of the requested script
 *
 * Since: 2.0.0
 **/
rb_bool_t rb_ot_layout_table_select_script(rb_face_t *face,
                                           rb_tag_t table_tag,
                                           unsigned int script_count,
                                           const rb_tag_t *script_tags,
                                           unsigned int *script_index /* OUT */,
                                           rb_tag_t *chosen_script /* OUT */)
{
    static_assert((OT::Index::NOT_FOUND_INDEX == RB_OT_LAYOUT_NO_SCRIPT_INDEX), "");
    const OT::GSUBGPOS &g = get_gsubgpos_table(face, table_tag);
    unsigned int i;

    for (i = 0; i < script_count; i++) {
        if (g.find_script_index(script_tags[i], script_index)) {
            if (chosen_script)
                *chosen_script = script_tags[i];
            return true;
        }
    }

    /* try finding 'DFLT' */
    if (g.find_script_index(RB_OT_TAG_DEFAULT_SCRIPT, script_index)) {
        if (chosen_script)
            *chosen_script = RB_OT_TAG_DEFAULT_SCRIPT;
        return false;
    }

    /* try with 'dflt'; MS site has had typos and many fonts use it now :( */
    if (g.find_script_index(RB_OT_TAG_DEFAULT_LANGUAGE, script_index)) {
        if (chosen_script)
            *chosen_script = RB_OT_TAG_DEFAULT_LANGUAGE;
        return false;
    }

    /* try with 'latn'; some old fonts put their features there even though
       they're really trying to support Thai, for example :( */
    if (g.find_script_index(RB_OT_TAG_LATIN_SCRIPT, script_index)) {
        if (chosen_script)
            *chosen_script = RB_OT_TAG_LATIN_SCRIPT;
        return false;
    }

    if (script_index)
        *script_index = RB_OT_LAYOUT_NO_SCRIPT_INDEX;
    if (chosen_script)
        *chosen_script = RB_OT_LAYOUT_NO_SCRIPT_INDEX;
    return false;
}

/**
 * rb_ot_layout_table_find_feature:
 * @face: #rb_face_t to work upon
 * @table_tag: RB_OT_TAG_GSUB or RB_OT_TAG_GPOS
 * @feature_tag: The #rb_tag_t og the requested feature tag
 * @feature_index: (out): The index of the requested feature
 *
 * Fetches the index for a given feature tag in the specified face's GSUB table
 * or GPOS table.
 *
 * Return value: true if the feature is found, false otherwise
 **/
bool rb_ot_layout_table_find_feature(rb_face_t *face,
                                     rb_tag_t table_tag,
                                     rb_tag_t feature_tag,
                                     unsigned int *feature_index /* OUT */)
{
    static_assert((OT::Index::NOT_FOUND_INDEX == RB_OT_LAYOUT_NO_FEATURE_INDEX), "");
    const OT::GSUBGPOS &g = get_gsubgpos_table(face, table_tag);

    unsigned int num_features = g.get_feature_count();
    for (unsigned int i = 0; i < num_features; i++) {
        if (feature_tag == g.get_feature_tag(i)) {
            if (feature_index)
                *feature_index = i;
            return true;
        }
    }

    if (feature_index)
        *feature_index = RB_OT_LAYOUT_NO_FEATURE_INDEX;
    return false;
}

/**
 * rb_ot_layout_script_select_language:
 * @face: #rb_face_t to work upon
 * @table_tag: RB_OT_TAG_GSUB or RB_OT_TAG_GPOS
 * @script_index: The index of the requested script tag
 * @language_count: The number of languages in the specified script
 * @language_tags: The array of language tags
 * @language_index: (out): The index of the requested language
 *
 * Fetches the index of a given language tag in the specified face's GSUB table
 * or GPOS table, underneath the specified script index.
 *
 * Return value: true if the language tag is found, false otherwise
 *
 * Since: 2.0.0
 **/
rb_bool_t rb_ot_layout_script_select_language(rb_face_t *face,
                                              rb_tag_t table_tag,
                                              unsigned int script_index,
                                              unsigned int language_count,
                                              const rb_tag_t *language_tags,
                                              unsigned int *language_index /* OUT */)
{
    static_assert((OT::Index::NOT_FOUND_INDEX == RB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX), "");
    const OT::Script &s = get_gsubgpos_table(face, table_tag).get_script(script_index);
    unsigned int i;

    for (i = 0; i < language_count; i++) {
        if (s.find_lang_sys_index(language_tags[i], language_index))
            return true;
    }

    /* try finding 'dflt' */
    if (s.find_lang_sys_index(RB_OT_TAG_DEFAULT_LANGUAGE, language_index))
        return false;

    if (language_index)
        *language_index = RB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX;
    return false;
}

/**
 * rb_ot_layout_language_get_required_feature:
 * @face: #rb_face_t to work upon
 * @table_tag: RB_OT_TAG_GSUB or RB_OT_TAG_GPOS
 * @script_index: The index of the requested script tag
 * @language_index: The index of the requested language tag
 * @feature_index: (out): The index of the requested feature
 * @feature_tag: (out): The #rb_tag_t of the requested feature
 *
 * Fetches the tag of a requested feature index in the given face's GSUB or GPOS table,
 * underneath the specified script and language.
 *
 * Return value: true if the feature is found, false otherwise
 *
 * Since: 0.9.30
 **/
rb_bool_t rb_ot_layout_language_get_required_feature(rb_face_t *face,
                                                     rb_tag_t table_tag,
                                                     unsigned int script_index,
                                                     unsigned int language_index,
                                                     unsigned int *feature_index /* OUT */,
                                                     rb_tag_t *feature_tag /* OUT */)
{
    const OT::GSUBGPOS &g = get_gsubgpos_table(face, table_tag);
    const OT::LangSys &l = g.get_script(script_index).get_lang_sys(language_index);

    unsigned int index = l.get_required_feature_index();
    if (feature_index)
        *feature_index = index;
    if (feature_tag)
        *feature_tag = g.get_feature_tag(index);

    return l.has_required_feature();
}

/**
 * rb_ot_layout_language_find_feature:
 * @face: #rb_face_t to work upon
 * @table_tag: RB_OT_TAG_GSUB or RB_OT_TAG_GPOS
 * @script_index: The index of the requested script tag
 * @language_index: The index of the requested language tag
 * @feature_tag: #rb_tag_t of the feature tag requested
 * @feature_index: (out): The index of the requested feature
 *
 * Fetches the index of a given feature tag in the specified face's GSUB table
 * or GPOS table, underneath the specified script and language.
 *
 * Return value: true if the feature is found, false otherwise
 *
 **/
rb_bool_t rb_ot_layout_language_find_feature(rb_face_t *face,
                                             rb_tag_t table_tag,
                                             unsigned int script_index,
                                             unsigned int language_index,
                                             rb_tag_t feature_tag,
                                             unsigned int *feature_index /* OUT */)
{
    static_assert((OT::Index::NOT_FOUND_INDEX == RB_OT_LAYOUT_NO_FEATURE_INDEX), "");
    const OT::GSUBGPOS &g = get_gsubgpos_table(face, table_tag);
    const OT::LangSys &l = g.get_script(script_index).get_lang_sys(language_index);

    unsigned int num_features = l.get_feature_count();
    for (unsigned int i = 0; i < num_features; i++) {
        unsigned int f_index = l.get_feature_index(i);

        if (feature_tag == g.get_feature_tag(f_index)) {
            if (feature_index)
                *feature_index = f_index;
            return true;
        }
    }

    if (feature_index)
        *feature_index = RB_OT_LAYOUT_NO_FEATURE_INDEX;
    return false;
}

/**
 * rb_ot_layout_table_get_lookup_count:
 * @face: #rb_face_t to work upon
 * @table_tag: RB_OT_TAG_GSUB or RB_OT_TAG_GPOS
 *
 * Fetches the total number of lookups enumerated in the specified
 * face's GSUB table or GPOS table.
 *
 * Since: 0.9.22
 **/
unsigned int rb_ot_layout_table_get_lookup_count(rb_face_t *face, rb_tag_t table_tag)
{
    return get_gsubgpos_table(face, table_tag).get_lookup_count();
}

/* Variations support */

/**
 * rb_ot_layout_table_find_feature_variations:
 * @face: #rb_face_t to work upon
 * @table_tag: RB_OT_TAG_GSUB or RB_OT_TAG_GPOS
 * @coords: The variation coordinates to query
 * @num_coords: The number of variation coorinates
 * @variations_index: (out): The array of feature variations found for the query
 *
 * Fetches a list of feature variations in the specified face's GSUB table
 * or GPOS table, at the specified variation coordinates.
 *
 **/
rb_bool_t rb_ot_layout_table_find_feature_variations(rb_face_t *face,
                                                     rb_tag_t table_tag,
                                                     const int *coords,
                                                     unsigned int num_coords,
                                                     unsigned int *variations_index /* out */)
{
    const OT::GSUBGPOS &g = get_gsubgpos_table(face, table_tag);

    return g.find_variations_index(coords, num_coords, variations_index);
}

/**
 * rb_ot_layout_feature_with_variations_get_lookups:
 * @face: #rb_face_t to work upon
 * @table_tag: RB_OT_TAG_GSUB or RB_OT_TAG_GPOS
 * @feature_index: The index of the feature to query
 * @variations_index: The index of the feature variation to query
 * @start_offset: offset of the first lookup to retrieve
 * @lookup_count: (inout) (allow-none): Input = the maximum number of lookups to return;
 *                Output = the actual number of lookups returned (may be zero)
 * @lookup_indexes: (out) (array length=lookup_count): The array of lookups found for the query
 *
 * Fetches a list of all lookups enumerated for the specified feature, in
 * the specified face's GSUB table or GPOS table, enabled at the specified
 * variations index. The list returned will begin at the offset provided.
 *
 **/
unsigned int rb_ot_layout_feature_with_variations_get_lookups(rb_face_t *face,
                                                              rb_tag_t table_tag,
                                                              unsigned int feature_index,
                                                              unsigned int variations_index,
                                                              unsigned int start_offset,
                                                              unsigned int *lookup_count /* IN/OUT */,
                                                              unsigned int *lookup_indexes /* OUT */)
{
    static_assert((OT::FeatureVariations::NOT_FOUND_INDEX == RB_OT_LAYOUT_NO_VARIATIONS_INDEX), "");
    const OT::GSUBGPOS &g = get_gsubgpos_table(face, table_tag);

    const OT::Feature &f = g.get_feature_variation(feature_index, variations_index);

    return f.get_lookup_indexes(start_offset, lookup_count, lookup_indexes);
}

/*
 * OT::GSUB
 */

/**
 * rb_ot_layout_has_substitution:
 * @face: #rb_face_t to work upon
 *
 * Tests whether the specified face includes any GSUB substitutions.
 *
 * Return value: true if data found, false otherwise
 *
 **/
rb_bool_t rb_ot_layout_has_substitution(rb_face_t *face)
{
    return face->table.GSUB->table->has_data();
}

/**
 * rb_ot_layout_lookup_would_substitute:
 * @face: #rb_face_t to work upon
 * @lookup_index: The index of the lookup to query
 * @glyphs: The sequence of glyphs to query for substitution
 * @glyphs_length: The length of the glyph sequence
 * @zero_context: #rb_bool_t indicating whether substitutions should be context-free
 *
 * Tests whether a specified lookup in the specified face would
 * trigger a substitution on the given glyph sequence.
 *
 * Return value: true if a substitution would be triggered, false otherwise
 *
 * Since: 0.9.7
 **/
rb_bool_t rb_ot_layout_lookup_would_substitute(rb_face_t *face,
                                               unsigned int lookup_index,
                                               const rb_codepoint_t *glyphs,
                                               unsigned int glyphs_length,
                                               rb_bool_t zero_context)
{
    if (unlikely(lookup_index >= face->table.GSUB->lookup_count))
        return false;
    OT::rb_would_apply_context_t c(face, glyphs, glyphs_length, (bool)zero_context);

    const OT::SubstLookup &l = face->table.GSUB->table->get_lookup(lookup_index);
    return l.would_apply(&c, &face->table.GSUB->accels[lookup_index]);
}

/**
 * rb_ot_layout_substitute_start:
 * @font: #rb_font_t to use
 * @buffer: #rb_buffer_t buffer to work upon
 *
 * Called before substitution lookups are performed, to ensure that glyph
 * class and other properties are set on the glyphs in the buffer.
 *
 **/
void rb_ot_layout_substitute_start(rb_font_t *font, rb_buffer_t *buffer)
{
    _rb_ot_layout_set_glyph_props(font, buffer);
}

void rb_ot_layout_delete_glyphs_inplace(rb_buffer_t *buffer, bool (*filter)(const rb_glyph_info_t *info))
{
    /* Merge clusters and delete filtered glyphs.
     * NOTE! We can't use out-buffer as we have positioning data. */
    unsigned int j = 0;
    unsigned int count = rb_buffer_get_length(buffer);
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);
    rb_glyph_position_t *pos = rb_buffer_get_glyph_positions(buffer);
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
                        rb_buffer_set_cluster(buffer, &info[k - 1], cluster, mask);
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
 * rb_ot_layout_has_positioning:
 * @face: #rb_face_t to work upon
 *
 * Return value: true if the face has GPOS data, false otherwise
 *
 **/
rb_bool_t rb_ot_layout_has_positioning(rb_face_t *face)
{
    return face->table.GPOS->table->has_data();
}

/**
 * rb_ot_layout_position_start:
 * @font: #rb_font_t to use
 * @buffer: #rb_buffer_t buffer to work upon
 *
 * Called before positioning lookups are performed, to ensure that glyph
 * attachment types and glyph-attachment chains are set for the glyphs in the buffer.
 *
 **/
void rb_ot_layout_position_start(rb_font_t *font, rb_buffer_t *buffer)
{
    OT::GPOS::position_start(font, buffer);
}

/**
 * rb_ot_layout_position_finish_advances:
 * @font: #rb_font_t to use
 * @buffer: #rb_buffer_t buffer to work upon
 *
 * Called after positioning lookups are performed, to finish glyph advances.
 *
 **/
void rb_ot_layout_position_finish_advances(rb_font_t *font, rb_buffer_t *buffer)
{
    OT::GPOS::position_finish_advances(font, buffer);
}

/**
 * rb_ot_layout_position_finish_offsets:
 * @font: #rb_font_t to use
 * @buffer: #rb_buffer_t buffer to work upon
 *
 * Called after positioning lookups are performed, to finish glyph offsets.
 *
 **/
void rb_ot_layout_position_finish_offsets(rb_font_t *font, rb_buffer_t *buffer)
{
    OT::GPOS::position_finish_offsets(font, buffer);
}

/*
 * Parts of different types are implemented here such that they have direct
 * access to GSUB/GPOS lookups.
 */

struct GSUBProxy
{
    static constexpr unsigned table_index = 0u;
    static constexpr bool inplace = false;
    typedef OT::SubstLookup Lookup;

    GSUBProxy(rb_face_t *face)
        : table(*face->table.GSUB->table)
        , accels(face->table.GSUB->accels)
    {
    }

    const OT::GSUB &table;
    const OT::rb_ot_layout_lookup_accelerator_t *accels;
};

struct GPOSProxy
{
    static constexpr unsigned table_index = 1u;
    static constexpr bool inplace = true;
    typedef OT::PosLookup Lookup;

    GPOSProxy(rb_face_t *face)
        : table(*face->table.GPOS->table)
        , accels(face->table.GPOS->accels)
    {
    }

    const OT::GPOS &table;
    const OT::rb_ot_layout_lookup_accelerator_t *accels;
};

static inline bool apply_forward(OT::rb_ot_apply_context_t *c, const OT::rb_ot_layout_lookup_accelerator_t &accel)
{
    bool ret = false;
    rb_buffer_t *buffer = c->buffer;
    while (rb_buffer_get_index(buffer) < rb_buffer_get_length(buffer) && rb_buffer_is_allocation_successful(buffer)) {
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

static inline bool apply_backward(OT::rb_ot_apply_context_t *c, const OT::rb_ot_layout_lookup_accelerator_t &accel)
{
    bool ret = false;
    rb_buffer_t *buffer = c->buffer;
    do {
        if (accel.may_have(rb_buffer_get_cur(buffer, 0)->codepoint) &&
            (rb_buffer_get_cur(buffer, 0)->mask & c->lookup_mask) &&
            c->check_glyph_property(rb_buffer_get_cur(buffer, 0), c->lookup_props))
            ret |= accel.apply(c);

        /* The reverse lookup doesn't "advance" cursor (for good reason). */
        rb_buffer_set_index(buffer, rb_buffer_get_index(buffer) - 1);
    } while ((int)rb_buffer_get_index(buffer) >= 0);
    return ret;
}

template <typename Proxy>
static inline void apply_string(OT::rb_ot_apply_context_t *c,
                                const typename Proxy::Lookup &lookup,
                                const OT::rb_ot_layout_lookup_accelerator_t &accel)
{
    rb_buffer_t *buffer = c->buffer;

    if (unlikely(!rb_buffer_get_length(buffer) || !c->lookup_mask))
        return;

    c->set_lookup_props(lookup.get_props());

    if (likely(!lookup.is_reverse())) {
        /* in/out forward substitution/positioning */
        if (Proxy::table_index == 0u)
            rb_buffer_clear_output(buffer);
        rb_buffer_set_index(buffer, 0);

        bool ret;
        ret = apply_forward(c, accel);
        if (ret) {
            if (!Proxy::inplace)
                rb_buffer_swap_buffers(buffer);
            else
                assert(!rb_buffer_has_separate_output(buffer));
        }
    } else {
        /* in-place backward substitution/positioning */
        if (Proxy::table_index == 0u)
            rb_buffer_remove_output(buffer);
        rb_buffer_set_index(buffer, rb_buffer_get_length(buffer) - 1);

        apply_backward(c, accel);
    }
}

template <typename Proxy>
inline void
rb_ot_map_t::apply(const Proxy &proxy, const rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer) const
{
    const unsigned int table_index = proxy.table_index;
    unsigned int i = 0;
    OT::rb_ot_apply_context_t c(table_index, font, buffer);
    c.set_recurse_func(Proxy::Lookup::apply_recurse_func);

    for (unsigned int stage_index = 0; stage_index < stages[table_index].length; stage_index++) {
        const stage_map_t *stage = &stages[table_index][stage_index];
        for (; i < stage->last_lookup; i++) {
            unsigned int lookup_index = lookups[table_index][i].index;
            c.set_lookup_index(lookup_index);
            c.set_lookup_mask(lookups[table_index][i].mask);
            c.set_auto_zwj(lookups[table_index][i].auto_zwj);
            c.set_auto_zwnj(lookups[table_index][i].auto_zwnj);
            if (lookups[table_index][i].random) {
                c.set_random(true);
                rb_buffer_unsafe_to_break_all(buffer);
            }
            apply_string<Proxy>(&c, proxy.table.get_lookup(lookup_index), proxy.accels[lookup_index]);
        }

        if (stage->pause_func) {
            rb_buffer_clear_output(buffer);
            stage->pause_func(plan, font, buffer);
        }
    }
}

void rb_ot_map_t::substitute(const rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer) const
{
    GSUBProxy proxy(rb_font_get_face(font));
    apply(proxy, plan, font, buffer);
}

void rb_ot_map_t::position(const rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer) const
{
    GPOSProxy proxy(rb_font_get_face(font));
    apply(proxy, plan, font, buffer);
}

void rb_ot_layout_substitute_lookup(OT::rb_ot_apply_context_t *c,
                                    const OT::SubstLookup &lookup,
                                    const OT::rb_ot_layout_lookup_accelerator_t &accel)
{
    apply_string<GSUBProxy>(c, lookup, accel);
}

unsigned int rb_layout_next_syllable(rb_buffer_t *buffer, unsigned int start)
{
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);
    unsigned int count = rb_buffer_get_length(buffer);

    unsigned int syllable = info[start].syllable();
    while (++start < count && syllable == info[start].syllable())
        ;

    return start;
}

void rb_layout_clear_syllables(const rb_ot_shape_plan_t *plan RB_UNUSED, rb_font_t *font RB_UNUSED, rb_buffer_t *buffer)
{
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);
    unsigned int count = rb_buffer_get_length(buffer);
    for (unsigned int i = 0; i < count; i++)
        info[i].syllable() = 0;
}
