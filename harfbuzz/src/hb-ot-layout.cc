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

#include "hb-open-type.hh"
#include "hb-ot-layout.hh"
#include "hb-ot-face.hh"
#include "hb-ot-map.hh"
#include "hb-map.hh"

#include "hb-ot-kern-table.hh"
#include "hb-ot-layout-gdef-table.hh"
#include "hb-ot-layout-gsub-table.hh"
#include "hb-ot-layout-gpos-table.hh"

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
bool
hb_ot_layout_has_kerning (hb_face_t *face)
{
  return face->table.kern->has_data ();
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
bool
hb_ot_layout_has_machine_kerning (hb_face_t *face)
{
  return face->table.kern->has_state_machine ();
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
bool
hb_ot_layout_has_cross_kerning (hb_face_t *face)
{
  return face->table.kern->has_cross_stream ();
}

void
hb_ot_layout_kern (const hb_ot_shape_plan_t *plan,
		   hb_font_t *font,
		   hb_buffer_t  *buffer)
{
  hb_blob_t *blob = font->face->table.kern.get_blob ();
  const AAT::kern& kern = *blob->as<AAT::kern> ();

  AAT::hb_aat_apply_context_t c (plan, font, buffer, blob);

  kern.apply (&c);
}


/*
 * GDEF
 */

bool
OT::GDEF::is_blacklisted (hb_blob_t *blob,
			  hb_face_t *face) const
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
  switch HB_CODEPOINT_ENCODE3(blob->length,
			      face->table.GSUB->table.get_length (),
			      face->table.GPOS->table.get_length ())
  {
    /* sha1sum:c5ee92f0bca4bfb7d06c4d03e8cf9f9cf75d2e8a Windows 7? timesi.ttf */
    case HB_CODEPOINT_ENCODE3 (442, 2874, 42038):
    /* sha1sum:37fc8c16a0894ab7b749e35579856c73c840867b Windows 7? timesbi.ttf */
    case HB_CODEPOINT_ENCODE3 (430, 2874, 40662):
    /* sha1sum:19fc45110ea6cd3cdd0a5faca256a3797a069a80 Windows 7 timesi.ttf */
    case HB_CODEPOINT_ENCODE3 (442, 2874, 39116):
    /* sha1sum:6d2d3c9ed5b7de87bc84eae0df95ee5232ecde26 Windows 7 timesbi.ttf */
    case HB_CODEPOINT_ENCODE3 (430, 2874, 39374):
    /* sha1sum:8583225a8b49667c077b3525333f84af08c6bcd8 OS X 10.11.3 Times New Roman Italic.ttf */
    case HB_CODEPOINT_ENCODE3 (490, 3046, 41638):
    /* sha1sum:ec0f5a8751845355b7c3271d11f9918a966cb8c9 OS X 10.11.3 Times New Roman Bold Italic.ttf */
    case HB_CODEPOINT_ENCODE3 (478, 3046, 41902):
    /* sha1sum:96eda93f7d33e79962451c6c39a6b51ee893ce8c  tahoma.ttf from Windows 8 */
    case HB_CODEPOINT_ENCODE3 (898, 12554, 46470):
    /* sha1sum:20928dc06014e0cd120b6fc942d0c3b1a46ac2bc  tahomabd.ttf from Windows 8 */
    case HB_CODEPOINT_ENCODE3 (910, 12566, 47732):
    /* sha1sum:4f95b7e4878f60fa3a39ca269618dfde9721a79e  tahoma.ttf from Windows 8.1 */
    case HB_CODEPOINT_ENCODE3 (928, 23298, 59332):
    /* sha1sum:6d400781948517c3c0441ba42acb309584b73033  tahomabd.ttf from Windows 8.1 */
    case HB_CODEPOINT_ENCODE3 (940, 23310, 60732):
    /* tahoma.ttf v6.04 from Windows 8.1 x64, see https://bugzilla.mozilla.org/show_bug.cgi?id=1279925 */
    case HB_CODEPOINT_ENCODE3 (964, 23836, 60072):
    /* tahomabd.ttf v6.04 from Windows 8.1 x64, see https://bugzilla.mozilla.org/show_bug.cgi?id=1279925 */
    case HB_CODEPOINT_ENCODE3 (976, 23832, 61456):
    /* sha1sum:e55fa2dfe957a9f7ec26be516a0e30b0c925f846  tahoma.ttf from Windows 10 */
    case HB_CODEPOINT_ENCODE3 (994, 24474, 60336):
    /* sha1sum:7199385abb4c2cc81c83a151a7599b6368e92343  tahomabd.ttf from Windows 10 */
    case HB_CODEPOINT_ENCODE3 (1006, 24470, 61740):
    /* tahoma.ttf v6.91 from Windows 10 x64, see https://bugzilla.mozilla.org/show_bug.cgi?id=1279925 */
    case HB_CODEPOINT_ENCODE3 (1006, 24576, 61346):
    /* tahomabd.ttf v6.91 from Windows 10 x64, see https://bugzilla.mozilla.org/show_bug.cgi?id=1279925 */
    case HB_CODEPOINT_ENCODE3 (1018, 24572, 62828):
    /* sha1sum:b9c84d820c49850d3d27ec498be93955b82772b5  tahoma.ttf from Windows 10 AU */
    case HB_CODEPOINT_ENCODE3 (1006, 24576, 61352):
    /* sha1sum:2bdfaab28174bdadd2f3d4200a30a7ae31db79d2  tahomabd.ttf from Windows 10 AU */
    case HB_CODEPOINT_ENCODE3 (1018, 24572, 62834):
    /* sha1sum:b0d36cf5a2fbe746a3dd277bffc6756a820807a7  Tahoma.ttf from Mac OS X 10.9 */
    case HB_CODEPOINT_ENCODE3 (832, 7324, 47162):
    /* sha1sum:12fc4538e84d461771b30c18b5eb6bd434e30fba  Tahoma Bold.ttf from Mac OS X 10.9 */
    case HB_CODEPOINT_ENCODE3 (844, 7302, 45474):
    /* sha1sum:eb8afadd28e9cf963e886b23a30b44ab4fd83acc  himalaya.ttf from Windows 7 */
    case HB_CODEPOINT_ENCODE3 (180, 13054, 7254):
    /* sha1sum:73da7f025b238a3f737aa1fde22577a6370f77b0  himalaya.ttf from Windows 8 */
    case HB_CODEPOINT_ENCODE3 (192, 12638, 7254):
    /* sha1sum:6e80fd1c0b059bbee49272401583160dc1e6a427  himalaya.ttf from Windows 8.1 */
    case HB_CODEPOINT_ENCODE3 (192, 12690, 7254):
    /* 8d9267aea9cd2c852ecfb9f12a6e834bfaeafe44  cantarell-fonts-0.0.21/otf/Cantarell-Regular.otf */
    /* 983988ff7b47439ab79aeaf9a45bd4a2c5b9d371  cantarell-fonts-0.0.21/otf/Cantarell-Oblique.otf */
    case HB_CODEPOINT_ENCODE3 (188, 248, 3852):
    /* 2c0c90c6f6087ffbfea76589c93113a9cbb0e75f  cantarell-fonts-0.0.21/otf/Cantarell-Bold.otf */
    /* 55461f5b853c6da88069ffcdf7f4dd3f8d7e3e6b  cantarell-fonts-0.0.21/otf/Cantarell-Bold-Oblique.otf */
    case HB_CODEPOINT_ENCODE3 (188, 264, 3426):
    /* d125afa82a77a6475ac0e74e7c207914af84b37a padauk-2.80/Padauk.ttf RHEL 7.2 */
    case HB_CODEPOINT_ENCODE3 (1058, 47032, 11818):
    /* 0f7b80437227b90a577cc078c0216160ae61b031 padauk-2.80/Padauk-Bold.ttf RHEL 7.2*/
    case HB_CODEPOINT_ENCODE3 (1046, 47030, 12600):
    /* d3dde9aa0a6b7f8f6a89ef1002e9aaa11b882290 padauk-2.80/Padauk.ttf Ubuntu 16.04 */
    case HB_CODEPOINT_ENCODE3 (1058, 71796, 16770):
    /* 5f3c98ccccae8a953be2d122c1b3a77fd805093f padauk-2.80/Padauk-Bold.ttf Ubuntu 16.04 */
    case HB_CODEPOINT_ENCODE3 (1046, 71790, 17862):
    /* 6c93b63b64e8b2c93f5e824e78caca555dc887c7 padauk-2.80/Padauk-book.ttf */
    case HB_CODEPOINT_ENCODE3 (1046, 71788, 17112):
    /* d89b1664058359b8ec82e35d3531931125991fb9 padauk-2.80/Padauk-bookbold.ttf */
    case HB_CODEPOINT_ENCODE3 (1058, 71794, 17514):
    /* 824cfd193aaf6234b2b4dc0cf3c6ef576c0d00ef padauk-3.0/Padauk-book.ttf */
    case HB_CODEPOINT_ENCODE3 (1330, 109904, 57938):
    /* 91fcc10cf15e012d27571e075b3b4dfe31754a8a padauk-3.0/Padauk-bookbold.ttf */
    case HB_CODEPOINT_ENCODE3 (1330, 109904, 58972):
    /* sha1sum: c26e41d567ed821bed997e937bc0c41435689e85  Padauk.ttf
     *  "Padauk Regular" "Version 2.5", see https://crbug.com/681813 */
    case HB_CODEPOINT_ENCODE3 (1004, 59092, 14836):
      return true;
  }
  return false;
}

static void
_hb_ot_layout_set_glyph_props (hb_font_t *font,
			       hb_buffer_t *buffer)
{
  const OT::GDEF &gdef = *font->face->table.GDEF->table;
  unsigned int count = buffer->len();
  for (unsigned int i = 0; i < count; i++)
  {
    _hb_glyph_info_set_glyph_props (&buffer->info[i], gdef.get_glyph_props (font, buffer->info[i].codepoint));
    _hb_glyph_info_clear_lig_props (&buffer->info[i]);
    buffer->info[i].syllable() = 0;
  }
}

/* Public API */

/**
 * hb_ot_layout_has_glyph_classes:
 * @face: #hb_face_t to work upon
 *
 * Tests whether a face has any glyph classes defined in its GDEF table.
 *
 * Return value: true if data found, false otherwise
 *
 **/
hb_bool_t
hb_ot_layout_has_glyph_classes (hb_face_t *face)
{
  return rb_ot_has_glyph_classes (face->rust_data);
}

/*
 * GSUB/GPOS
 */

bool
OT::GSUB::is_blacklisted (hb_blob_t *blob HB_UNUSED,
			  hb_face_t *face) const
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

bool
OT::GPOS::is_blacklisted (hb_blob_t *blob HB_UNUSED,
			  hb_face_t *face HB_UNUSED) const
{
#ifdef HB_NO_OT_LAYOUT_BLACKLIST
  return false;
#endif
  return false;
}

static const OT::GSUBGPOS&
get_gsubgpos_table (hb_face_t *face,
		    hb_tag_t   table_tag)
{
  switch (table_tag) {
    case HB_OT_TAG_GSUB: return *face->table.GSUB->table;
    case HB_OT_TAG_GPOS: return *face->table.GPOS->table;
    default:             return Null(OT::GSUBGPOS);
  }
}


/**
 * hb_ot_layout_table_get_script_tags:
 * @face: #hb_face_t to work upon
 * @table_tag: HB_OT_TAG_GSUB or HB_OT_TAG_GPOS
 * @start_offset: offset of the first script tag to retrieve
 * @script_count: (inout) (allow-none): Input = the maximum number of script tags to return;
 *                Output = the actual number of script tags returned (may be zero)
 * @script_tags: (out) (array length=script_count): The array of #hb_tag_t script tags found for the query
 *
 * Fetches a list of all scripts enumerated in the specified face's GSUB table
 * or GPOS table. The list returned will begin at the offset provided.
 *
 **/
unsigned int
hb_ot_layout_table_get_script_tags (hb_face_t    *face,
				    hb_tag_t      table_tag,
				    unsigned int  start_offset,
				    unsigned int *script_count /* IN/OUT */,
				    hb_tag_t     *script_tags  /* OUT */)
{
  const OT::GSUBGPOS &g = get_gsubgpos_table (face, table_tag);

  return g.get_script_tags (start_offset, script_count, script_tags);
}

#define HB_OT_TAG_LATIN_SCRIPT		HB_TAG ('l', 'a', 't', 'n')

/**
 * hb_ot_layout_table_select_script:
 * @face: #hb_face_t to work upon
 * @table_tag: HB_OT_TAG_GSUB or HB_OT_TAG_GPOS
 * @script_count: Number of script tags in the array
 * @script_tags: Array of #hb_tag_t script tags
 * @script_index: (out): The index of the requested script
 * @chosen_script: (out): #hb_tag_t of the requested script
 *
 * Since: 2.0.0
 **/
hb_bool_t
hb_ot_layout_table_select_script (hb_face_t      *face,
				  hb_tag_t        table_tag,
				  unsigned int    script_count,
				  const hb_tag_t *script_tags,
				  unsigned int   *script_index  /* OUT */,
				  hb_tag_t       *chosen_script /* OUT */)
{
  static_assert ((OT::Index::NOT_FOUND_INDEX == HB_OT_LAYOUT_NO_SCRIPT_INDEX), "");
  const OT::GSUBGPOS &g = get_gsubgpos_table (face, table_tag);
  unsigned int i;

  for (i = 0; i < script_count; i++)
  {
    if (g.find_script_index (script_tags[i], script_index))
    {
      if (chosen_script)
	*chosen_script = script_tags[i];
      return true;
    }
  }

  /* try finding 'DFLT' */
  if (g.find_script_index (HB_OT_TAG_DEFAULT_SCRIPT, script_index)) {
    if (chosen_script)
      *chosen_script = HB_OT_TAG_DEFAULT_SCRIPT;
    return false;
  }

  /* try with 'dflt'; MS site has had typos and many fonts use it now :( */
  if (g.find_script_index (HB_OT_TAG_DEFAULT_LANGUAGE, script_index)) {
    if (chosen_script)
      *chosen_script = HB_OT_TAG_DEFAULT_LANGUAGE;
    return false;
  }

  /* try with 'latn'; some old fonts put their features there even though
     they're really trying to support Thai, for example :( */
  if (g.find_script_index (HB_OT_TAG_LATIN_SCRIPT, script_index)) {
    if (chosen_script)
      *chosen_script = HB_OT_TAG_LATIN_SCRIPT;
    return false;
  }

  if (script_index) *script_index = HB_OT_LAYOUT_NO_SCRIPT_INDEX;
  if (chosen_script)
    *chosen_script = HB_OT_LAYOUT_NO_SCRIPT_INDEX;
  return false;
}

/**
 * hb_ot_layout_table_find_feature:
 * @face: #hb_face_t to work upon
 * @table_tag: HB_OT_TAG_GSUB or HB_OT_TAG_GPOS
 * @feature_tag: The #hb_tag_t og the requested feature tag
 * @feature_index: (out): The index of the requested feature
 *
 * Fetches the index for a given feature tag in the specified face's GSUB table
 * or GPOS table.
 *
 * Return value: true if the feature is found, false otherwise
 **/
bool
hb_ot_layout_table_find_feature (hb_face_t    *face,
				 hb_tag_t      table_tag,
				 hb_tag_t      feature_tag,
				 unsigned int *feature_index /* OUT */)
{
  static_assert ((OT::Index::NOT_FOUND_INDEX == HB_OT_LAYOUT_NO_FEATURE_INDEX), "");
  const OT::GSUBGPOS &g = get_gsubgpos_table (face, table_tag);

  unsigned int num_features = g.get_feature_count ();
  for (unsigned int i = 0; i < num_features; i++)
  {
    if (feature_tag == g.get_feature_tag (i)) {
      if (feature_index) *feature_index = i;
      return true;
    }
  }

  if (feature_index) *feature_index = HB_OT_LAYOUT_NO_FEATURE_INDEX;
  return false;
}

/**
 * hb_ot_layout_script_select_language:
 * @face: #hb_face_t to work upon
 * @table_tag: HB_OT_TAG_GSUB or HB_OT_TAG_GPOS
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
hb_bool_t
hb_ot_layout_script_select_language (hb_face_t      *face,
				     hb_tag_t        table_tag,
				     unsigned int    script_index,
				     unsigned int    language_count,
				     const hb_tag_t *language_tags,
				     unsigned int   *language_index /* OUT */)
{
  static_assert ((OT::Index::NOT_FOUND_INDEX == HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX), "");
  const OT::Script &s = get_gsubgpos_table (face, table_tag).get_script (script_index);
  unsigned int i;

  for (i = 0; i < language_count; i++)
  {
    if (s.find_lang_sys_index (language_tags[i], language_index))
      return true;
  }

  /* try finding 'dflt' */
  if (s.find_lang_sys_index (HB_OT_TAG_DEFAULT_LANGUAGE, language_index))
    return false;

  if (language_index) *language_index = HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX;
  return false;
}


/**
 * hb_ot_layout_language_get_required_feature:
 * @face: #hb_face_t to work upon
 * @table_tag: HB_OT_TAG_GSUB or HB_OT_TAG_GPOS
 * @script_index: The index of the requested script tag
 * @language_index: The index of the requested language tag
 * @feature_index: The index of the requested feature
 * @feature_tag: (out): The #hb_tag_t of the requested feature
 *
 * Fetches the tag of a requested feature index in the given face's GSUB or GPOS table,
 * underneath the specified script and language.
 *
 * Return value: true if the feature is found, false otherwise
 *
 * Since: 0.9.30
 **/
hb_bool_t
hb_ot_layout_language_get_required_feature (hb_face_t    *face,
					    hb_tag_t      table_tag,
					    unsigned int  script_index,
					    unsigned int  language_index,
					    unsigned int *feature_index,
					    hb_tag_t     *feature_tag)
{
  const OT::GSUBGPOS &g = get_gsubgpos_table (face, table_tag);
  const OT::LangSys &l = g.get_script (script_index).get_lang_sys (language_index);

  unsigned int index = l.get_required_feature_index ();
  if (feature_index) *feature_index = index;
  if (feature_tag) *feature_tag = g.get_feature_tag (index);

  return l.has_required_feature ();
}


/**
 * hb_ot_layout_language_find_feature:
 * @face: #hb_face_t to work upon
 * @table_tag: HB_OT_TAG_GSUB or HB_OT_TAG_GPOS
 * @script_index: The index of the requested script tag
 * @language_index: The index of the requested language tag
 * @feature_tag: #hb_tag_t of the feature tag requested
 * @feature_index: (out): The index of the requested feature
 *
 * Fetches the index of a given feature tag in the specified face's GSUB table
 * or GPOS table, underneath the specified script and language.
 *
 * Return value: true if the feature is found, false otherwise
 *
 **/
hb_bool_t
hb_ot_layout_language_find_feature (hb_face_t    *face,
				    hb_tag_t      table_tag,
				    unsigned int  script_index,
				    unsigned int  language_index,
				    hb_tag_t      feature_tag,
				    unsigned int *feature_index /* OUT */)
{
  static_assert ((OT::Index::NOT_FOUND_INDEX == HB_OT_LAYOUT_NO_FEATURE_INDEX), "");
  const OT::GSUBGPOS &g = get_gsubgpos_table (face, table_tag);
  const OT::LangSys &l = g.get_script (script_index).get_lang_sys (language_index);

  unsigned int num_features = l.get_feature_count ();
  for (unsigned int i = 0; i < num_features; i++) {
    unsigned int f_index = l.get_feature_index (i);

    if (feature_tag == g.get_feature_tag (f_index)) {
      if (feature_index) *feature_index = f_index;
      return true;
    }
  }

  if (feature_index) *feature_index = HB_OT_LAYOUT_NO_FEATURE_INDEX;
  return false;
}


/**
 * hb_ot_layout_table_get_lookup_count:
 * @face: #hb_face_t to work upon
 * @table_tag: HB_OT_TAG_GSUB or HB_OT_TAG_GPOS
 *
 * Fetches the total number of lookups enumerated in the specified
 * face's GSUB table or GPOS table.
 *
 * Since: 0.9.22
 **/
unsigned int
hb_ot_layout_table_get_lookup_count (hb_face_t    *face,
				     hb_tag_t      table_tag)
{
  return get_gsubgpos_table (face, table_tag).get_lookup_count ();
}

/* Variations support */


/**
 * hb_ot_layout_table_find_feature_variations:
 * @face: #hb_face_t to work upon
 * @table_tag: HB_OT_TAG_GSUB or HB_OT_TAG_GPOS
 * @coords: The variation coordinates to query
 * @num_coords: The number of variation coorinates
 * @variations_index: (out): The array of feature variations found for the query
 *
 * Fetches a list of feature variations in the specified face's GSUB table
 * or GPOS table, at the specified variation coordinates.
 *
 **/
hb_bool_t
hb_ot_layout_table_find_feature_variations (hb_face_t    *face,
					    hb_tag_t      table_tag,
					    const int    *coords,
					    unsigned int  num_coords,
					    unsigned int *variations_index /* out */)
{
  const OT::GSUBGPOS &g = get_gsubgpos_table (face, table_tag);

  return g.find_variations_index (coords, num_coords, variations_index);
}


/**
 * hb_ot_layout_feature_with_variations_get_lookups:
 * @face: #hb_face_t to work upon
 * @table_tag: HB_OT_TAG_GSUB or HB_OT_TAG_GPOS
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
unsigned int
hb_ot_layout_feature_with_variations_get_lookups (hb_face_t    *face,
						  hb_tag_t      table_tag,
						  unsigned int  feature_index,
						  unsigned int  variations_index,
						  unsigned int  start_offset,
						  unsigned int *lookup_count /* IN/OUT */,
						  unsigned int *lookup_indexes /* OUT */)
{
  static_assert ((OT::FeatureVariations::NOT_FOUND_INDEX == HB_OT_LAYOUT_NO_VARIATIONS_INDEX), "");
  const OT::GSUBGPOS &g = get_gsubgpos_table (face, table_tag);

  const OT::Feature &f = g.get_feature_variation (feature_index, variations_index);

  return f.get_lookup_indexes (start_offset, lookup_count, lookup_indexes);
}


/*
 * OT::GSUB
 */


/**
 * hb_ot_layout_has_substitution:
 * @face: #hb_face_t to work upon
 *
 * Tests whether the specified face includes any GSUB substitutions.
 *
 * Return value: true if data found, false otherwise
 *
 **/
hb_bool_t
hb_ot_layout_has_substitution (hb_face_t *face)
{
  return face->table.GSUB->table->has_data ();
}


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
hb_bool_t
hb_ot_layout_lookup_would_substitute (hb_face_t            *face,
				      unsigned int          lookup_index,
				      const hb_codepoint_t *glyphs,
				      unsigned int          glyphs_length,
				      hb_bool_t             zero_context)
{
  if (unlikely (lookup_index >= face->table.GSUB->lookup_count)) return false;
  OT::hb_would_apply_context_t c (face, glyphs, glyphs_length, (bool) zero_context);

  const OT::SubstLookup& l = face->table.GSUB->table->get_lookup (lookup_index);

  return l.would_apply (&c, &face->table.GSUB->accels[lookup_index]);
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
void
hb_ot_layout_substitute_start (hb_font_t    *font,
			       hb_buffer_t  *buffer)
{
_hb_ot_layout_set_glyph_props (font, buffer);
}

void
hb_ot_layout_delete_glyphs_inplace (hb_buffer_t *buffer,
				    bool (*filter) (const hb_glyph_info_t *info))
{
  /* Merge clusters and delete filtered glyphs.
   * NOTE! We can't use out-buffer as we have positioning data. */
  unsigned int j = 0;
  unsigned int count = buffer->len();
  hb_glyph_info_t *info = buffer->info;
  hb_glyph_position_t *pos = buffer->pos;
  for (unsigned int i = 0; i < count; i++)
  {
    if (filter (&info[i]))
    {
      /* Merge clusters.
       * Same logic as buffer->delete_glyph(), but for in-place removal. */

      unsigned int cluster = info[i].cluster;
      if (i + 1 < count && cluster == info[i + 1].cluster)
	continue; /* Cluster survives; do nothing. */

      if (j)
      {
	/* Merge cluster backward. */
	if (cluster < info[j - 1].cluster)
	{
	  unsigned int mask = info[i].mask;
	  unsigned int old_cluster = info[j - 1].cluster;
	  for (unsigned k = j; k && info[k - 1].cluster == old_cluster; k--)
	    buffer->set_cluster (info[k - 1], cluster, mask);
	}
	continue;
      }

      if (i + 1 < count)
	buffer->merge_clusters (i, i + 2); /* Merge cluster forward. */

      continue;
    }

    if (j != i)
    {
      info[j] = info[i];
      pos[j] = pos[i];
    }
    j++;
  }
  buffer->info_vec.length = j;
}

/**
 * hb_ot_layout_lookups_substitute_closure:
 * @face: #hb_face_t to work upon
 * @lookups: The set of lookups to query
 * @glyphs: (out): Array of glyphs comprising the transitive closure of the lookups
 *
 * Compute the transitive closure of glyphs needed for all of the
 * provided lookups.
 *
 * Since: 1.8.1
 **/
void
hb_ot_layout_lookups_substitute_closure (hb_face_t      *face,
					 const hb_set_t *lookups,
					 hb_set_t       *glyphs /* OUT */)
{
  hb_map_t done_lookups;
  OT::hb_closure_context_t c (face, glyphs, &done_lookups);
  const OT::GSUB& gsub = *face->table.GSUB->table;

  unsigned int iteration_count = 0;
  unsigned int glyphs_length;
  do
  {
    glyphs_length = glyphs->get_population ();
    if (lookups != nullptr)
    {
      for (hb_codepoint_t lookup_index = HB_SET_VALUE_INVALID; hb_set_next (lookups, &lookup_index);)
	gsub.get_lookup (lookup_index).closure (&c, lookup_index);
    }
    else
    {
      for (unsigned int i = 0; i < gsub.get_lookup_count (); i++)
	gsub.get_lookup (i).closure (&c, i);
    }
  } while (iteration_count++ <= HB_CLOSURE_MAX_STAGES &&
	   glyphs_length != glyphs->get_population ());
}

/*
 * OT::GPOS
 */


/**
 * hb_ot_layout_has_positioning:
 * @face: #hb_face_t to work upon
 *
 * Return value: true if the face has GPOS data, false otherwise
 *
 **/
hb_bool_t
hb_ot_layout_has_positioning (hb_face_t *face)
{
  return face->table.GPOS->table->has_data ();
}

/**
 * hb_ot_layout_position_start:
 * @font: #hb_font_t to use
 * @buffer: #hb_buffer_t buffer to work upon
 *
 * Called before positioning lookups are performed, to ensure that glyph
 * attachment types and glyph-attachment chains are set for the glyphs in the buffer.
 *
 **/
void
hb_ot_layout_position_start (hb_font_t *font, hb_buffer_t *buffer)
{
  OT::GPOS::position_start (font, buffer);
}


/**
 * hb_ot_layout_position_finish_advances:
 * @font: #hb_font_t to use
 * @buffer: #hb_buffer_t buffer to work upon
 *
 * Called after positioning lookups are performed, to finish glyph advances.
 *
 **/
void
hb_ot_layout_position_finish_advances (hb_font_t *font, hb_buffer_t *buffer)
{
  OT::GPOS::position_finish_advances (font, buffer);
}

/**
 * hb_ot_layout_position_finish_offsets:
 * @font: #hb_font_t to use
 * @buffer: #hb_buffer_t buffer to work upon
 *
 * Called after positioning lookups are performed, to finish glyph offsets.
 *
 **/
void
hb_ot_layout_position_finish_offsets (hb_font_t *font, hb_buffer_t *buffer)
{
  OT::GPOS::position_finish_offsets (font, buffer);
}


/*
 * Parts of different types are implemented here such that they have direct
 * access to GSUB/GPOS lookups.
 */

static inline bool
apply_forward (OT::hb_ot_apply_context_t *c,
	       const OT::hb_ot_layout_lookup_accelerator_t &accel)
{
  bool ret = false;
  hb_buffer_t *buffer = c->buffer;
  while (buffer->idx < buffer->len())
  {
    bool applied = false;
    if (accel.may_have (buffer->cur().codepoint) &&
	(buffer->cur().mask & c->lookup_mask) &&
	c->check_glyph_property (&buffer->cur(), c->lookup_props))
     {
       applied = accel.apply (c);
     }

    if (applied)
      ret = true;
    else
      buffer->next_glyph ();
  }
  return ret;
}

static inline bool
apply_backward (OT::hb_ot_apply_context_t *c,
	        const OT::hb_ot_layout_lookup_accelerator_t &accel)
{
  bool ret = false;
  hb_buffer_t *buffer = c->buffer;
  do
  {
    if (accel.may_have (buffer->cur().codepoint) &&
	(buffer->cur().mask & c->lookup_mask) &&
	c->check_glyph_property (&buffer->cur(), c->lookup_props))
     ret |= accel.apply (c);

    /* The reverse lookup doesn't "advance" cursor (for good reason). */
    buffer->idx--;

  }
  while ((int) buffer->idx >= 0);
  return ret;
}

static inline void
apply_string_gsub (OT::hb_ot_apply_context_t *c,
		   const OT::SubstLookup &lookup,
		   const OT::hb_ot_layout_lookup_accelerator_t &accel)
{
  hb_buffer_t *buffer = c->buffer;

  if (unlikely (!buffer->len() || !c->lookup_mask))
    return;

  c->set_lookup_props (lookup.get_props ());

  if (likely (!lookup.is_reverse ()))
  {
    /* in/out forward substitution/positioning */
    buffer->clear_output ();
    buffer->idx = 0;

    bool ret;
    ret = apply_forward (c, accel);
    if (ret)
    {
      buffer->swap_buffers ();
    }
  }
  else
  {
    /* in-place backward substitution/positioning */
    buffer->remove_output ();
    buffer->idx = buffer->len() - 1;

    apply_backward (c, accel);
  }
}

inline void hb_ot_map_t::apply_gsub (const OT::GSUB &table,
				     const OT::hb_ot_layout_lookup_accelerator_t *accels,
				     const hb_ot_shape_plan_t *plan,
				     hb_font_t *font,
				     hb_buffer_t *buffer) const
{
  const unsigned int table_index = 0;
  unsigned int i = 0;
  OT::hb_ot_apply_context_t c (table_index, font, buffer);
  c.set_recurse_func (OT::SubstLookup::apply_recurse_func);

  for (unsigned int stage_index = 0; stage_index < stages[table_index].length; stage_index++) {
    const stage_map_t *stage = &stages[table_index][stage_index];
    for (; i < stage->last_lookup; i++)
    {
      unsigned int lookup_index = lookups[table_index][i].index;
      c.set_lookup_index (lookup_index);
      c.set_lookup_mask (lookups[table_index][i].mask);
      c.set_auto_zwj (lookups[table_index][i].auto_zwj);
      c.set_auto_zwnj (lookups[table_index][i].auto_zwnj);
      if (lookups[table_index][i].random)
      {
	c.set_random (true);
	buffer->unsafe_to_break_all ();
      }
      apply_string_gsub (&c,
			 table.get_lookup (lookup_index),
			 accels[lookup_index]);
    }

    if (stage->pause_func)
    {
      buffer->clear_output ();
      stage->pause_func (plan, font, buffer);
    }
  }
}

static inline void
apply_string_gpos (OT::hb_ot_apply_context_t *c,
		   const OT::PosLookup &lookup,
		   const OT::hb_ot_layout_lookup_accelerator_t &accel)
{
  hb_buffer_t *buffer = c->buffer;

  if (unlikely (!buffer->len() || !c->lookup_mask))
    return;

  c->set_lookup_props (lookup.get_props ());

  if (likely (!lookup.is_reverse ()))
  {
    buffer->idx = 0;

    bool ret;
    ret = apply_forward (c, accel);
    if (ret)
    {
	assert (!buffer->has_separate_output);
    }
  }
  else
  {
    buffer->idx = buffer->len() - 1;

    apply_backward (c, accel);
  }
}

inline void hb_ot_map_t::apply_gpos (const OT::GPOS &table,
				     const OT::hb_ot_layout_lookup_accelerator_t *accels,
				     const hb_ot_shape_plan_t *plan,
				     hb_font_t *font,
				     hb_buffer_t *buffer) const
{
  const unsigned int table_index = 1;
  unsigned int i = 0;
  OT::hb_ot_apply_context_t c (table_index, font, buffer);
  c.set_recurse_func (OT::PosLookup::apply_recurse_func);

  for (unsigned int stage_index = 0; stage_index < stages[table_index].length; stage_index++) {
    const stage_map_t *stage = &stages[table_index][stage_index];
    for (; i < stage->last_lookup; i++)
    {
      unsigned int lookup_index = lookups[table_index][i].index;
      c.set_lookup_index (lookup_index);
      c.set_lookup_mask (lookups[table_index][i].mask);
      c.set_auto_zwj (lookups[table_index][i].auto_zwj);
      c.set_auto_zwnj (lookups[table_index][i].auto_zwnj);
      if (lookups[table_index][i].random)
      {
	c.set_random (true);
	buffer->unsafe_to_break_all ();
      }
      apply_string_gpos (&c,
			 table.get_lookup (lookup_index),
			 accels[lookup_index]);
    }

    if (stage->pause_func)
    {
      buffer->clear_output ();
      stage->pause_func (plan, font, buffer);
    }
  }
}

void hb_ot_map_t::substitute (const hb_ot_shape_plan_t *plan, hb_font_t *font, hb_buffer_t *buffer) const
{
  apply_gsub (*font->face->table.GSUB->table, font->face->table.GSUB->accels, plan, font, buffer);
}

void hb_ot_map_t::position (const hb_ot_shape_plan_t *plan, hb_font_t *font, hb_buffer_t *buffer) const
{
  apply_gpos (*font->face->table.GPOS->table, font->face->table.GPOS->accels, plan, font, buffer);
}

void
hb_ot_layout_substitute_lookup (OT::hb_ot_apply_context_t *c,
				const OT::SubstLookup &lookup,
				const OT::hb_ot_layout_lookup_accelerator_t &accel)
{
  apply_string_gsub(c, lookup, accel);
}

#endif
