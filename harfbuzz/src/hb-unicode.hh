/*
 * Copyright © 2009  Red Hat, Inc.
 * Copyright © 2011  Codethink Limited
 * Copyright © 2010,2011,2012  Google, Inc.
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
 * Codethink Author(s): Ryan Lortie
 * Google Author(s): Behdad Esfahbod
 */

#pragma once

#include "hb.hh"
#include "hb-ucd.hh"


extern HB_INTERNAL const uint8_t _hb_modified_combining_class[256];

extern "C" {
  hb_bool_t rb_is_default_ignorable (hb_codepoint_t ch);
}

inline unsigned int
hb_ucd_modified_combining_class (hb_codepoint_t u)
{
  /* XXX This hack belongs to the USE shaper (for Tai Tham):
   * Reorder SAKOT to ensure it comes after any tone marks. */
  if (unlikely (u == 0x1A60u)) return 254;

  /* XXX This hack belongs to the Tibetan shaper:
   * Reorder PADMA to ensure it comes after any vowel marks. */
  if (unlikely (u == 0x0FC6u)) return 254;
  /* Reorder TSA -PHRU to reorder before U+0F74 */
  if (unlikely (u == 0x0F39u)) return 127;

  return _hb_modified_combining_class[hb_ucd_combining_class (u)];
}

inline hb_bool_t
hb_ucd_is_variation_selector (hb_codepoint_t unicode)
{
  /* U+180B..180D MONGOLIAN FREE VARIATION SELECTORs are handled in the
   * Arabic shaper.  No need to match them here. */
  return unlikely (hb_in_ranges<hb_codepoint_t> (unicode,
  					   0xFE00u, 0xFE0Fu, /* VARIATION SELECTOR-1..16 */
  					   0xE0100u, 0xE01EFu));  /* VARIATION SELECTOR-17..256 */
}

inline hb_bool_t
hb_ucd_is_default_ignorable (hb_codepoint_t ch)
{
  return rb_is_default_ignorable (ch);
}

/* Space estimates based on:
 * https://unicode.org/charts/PDF/U2000.pdf
 * https://docs.microsoft.com/en-us/typography/develop/character-design-standards/whitespace
 */
enum space_t {
  NOT_SPACE = 0,
  SPACE_EM   = 1,
  SPACE_EM_2 = 2,
  SPACE_EM_3 = 3,
  SPACE_EM_4 = 4,
  SPACE_EM_5 = 5,
  SPACE_EM_6 = 6,
  SPACE_EM_16 = 16,
  SPACE_4_EM_18,	/* 4/18th of an EM! */
  SPACE,
  SPACE_FIGURE,
  SPACE_PUNCTUATION,
  SPACE_NARROW,
};
inline space_t
hb_ucd_space_fallback_type (hb_codepoint_t u)
{
  switch (u)
  {
    /* All GC=Zs chars that can use a fallback. */
    default:	    return NOT_SPACE;	/* U+1680 OGHAM SPACE MARK */
    case 0x0020u: return SPACE;	/* U+0020 SPACE */
    case 0x00A0u: return SPACE;	/* U+00A0 NO-BREAK SPACE */
    case 0x2000u: return SPACE_EM_2;	/* U+2000 EN QUAD */
    case 0x2001u: return SPACE_EM;	/* U+2001 EM QUAD */
    case 0x2002u: return SPACE_EM_2;	/* U+2002 EN SPACE */
    case 0x2003u: return SPACE_EM;	/* U+2003 EM SPACE */
    case 0x2004u: return SPACE_EM_3;	/* U+2004 THREE-PER-EM SPACE */
    case 0x2005u: return SPACE_EM_4;	/* U+2005 FOUR-PER-EM SPACE */
    case 0x2006u: return SPACE_EM_6;	/* U+2006 SIX-PER-EM SPACE */
    case 0x2007u: return SPACE_FIGURE;	/* U+2007 FIGURE SPACE */
    case 0x2008u: return SPACE_PUNCTUATION;	/* U+2008 PUNCTUATION SPACE */
    case 0x2009u: return SPACE_EM_5;		/* U+2009 THIN SPACE */
    case 0x200Au: return SPACE_EM_16;		/* U+200A HAIR SPACE */
    case 0x202Fu: return SPACE_NARROW;	/* U+202F NARROW NO-BREAK SPACE */
    case 0x205Fu: return SPACE_4_EM_18;	/* U+205F MEDIUM MATHEMATICAL SPACE */
    case 0x3000u: return SPACE_EM;		/* U+3000 IDEOGRAPHIC SPACE */
  }
}

/*
 * Modified combining marks
 */

/* Hebrew
 *
 * We permute the "fixed-position" classes 10-26 into the order
 * described in the SBL Hebrew manual:
 *
 * https://www.sbl-site.org/Fonts/SBLHebrewUserManual1.5x.pdf
 *
 * (as recommended by:
 *  https://forum.fontlab.com/archive-old-microsoft-volt-group/vista-and-diacritic-ordering/msg22823/)
 *
 * More details here:
 * https://bugzilla.mozilla.org/show_bug.cgi?id=662055
 */
#define HB_MODIFIED_COMBINING_CLASS_CCC10 22 /* sheva */
#define HB_MODIFIED_COMBINING_CLASS_CCC11 15 /* hataf segol */
#define HB_MODIFIED_COMBINING_CLASS_CCC12 16 /* hataf patah */
#define HB_MODIFIED_COMBINING_CLASS_CCC13 17 /* hataf qamats */
#define HB_MODIFIED_COMBINING_CLASS_CCC14 23 /* hiriq */
#define HB_MODIFIED_COMBINING_CLASS_CCC15 18 /* tsere */
#define HB_MODIFIED_COMBINING_CLASS_CCC16 19 /* segol */
#define HB_MODIFIED_COMBINING_CLASS_CCC17 20 /* patah */
#define HB_MODIFIED_COMBINING_CLASS_CCC18 21 /* qamats */
#define HB_MODIFIED_COMBINING_CLASS_CCC19 14 /* holam */
#define HB_MODIFIED_COMBINING_CLASS_CCC20 24 /* qubuts */
#define HB_MODIFIED_COMBINING_CLASS_CCC21 12 /* dagesh */
#define HB_MODIFIED_COMBINING_CLASS_CCC22 25 /* meteg */
#define HB_MODIFIED_COMBINING_CLASS_CCC23 13 /* rafe */
#define HB_MODIFIED_COMBINING_CLASS_CCC24 10 /* shin dot */
#define HB_MODIFIED_COMBINING_CLASS_CCC25 11 /* sin dot */
#define HB_MODIFIED_COMBINING_CLASS_CCC26 26 /* point varika */

/*
 * Arabic
 *
 * Modify to move Shadda (ccc=33) before other marks.  See:
 * https://unicode.org/faq/normalization.html#8
 * https://unicode.org/faq/normalization.html#9
 */
#define HB_MODIFIED_COMBINING_CLASS_CCC27 28 /* fathatan */
#define HB_MODIFIED_COMBINING_CLASS_CCC28 29 /* dammatan */
#define HB_MODIFIED_COMBINING_CLASS_CCC29 30 /* kasratan */
#define HB_MODIFIED_COMBINING_CLASS_CCC30 31 /* fatha */
#define HB_MODIFIED_COMBINING_CLASS_CCC31 32 /* damma */
#define HB_MODIFIED_COMBINING_CLASS_CCC32 33 /* kasra */
#define HB_MODIFIED_COMBINING_CLASS_CCC33 27 /* shadda */
#define HB_MODIFIED_COMBINING_CLASS_CCC34 34 /* sukun */
#define HB_MODIFIED_COMBINING_CLASS_CCC35 35 /* superscript alef */

/* Syriac */
#define HB_MODIFIED_COMBINING_CLASS_CCC36 36 /* superscript alaph */

/* Telugu
 *
 * Modify Telugu length marks (ccc=84, ccc=91).
 * These are the only matras in the main Indic scripts range that have
 * a non-zero ccc.  That makes them reorder with the Halant (ccc=9).
 * Assign 5 and 6, which are otherwise unassigned.
 */
#define HB_MODIFIED_COMBINING_CLASS_CCC84 5 /* length mark */
#define HB_MODIFIED_COMBINING_CLASS_CCC91 6 /* ai length mark */

/* Thai
 *
 * Modify U+0E38 and U+0E39 (ccc=103) to be reordered before U+0E3A (ccc=9).
 * Assign 3, which is unassigned otherwise.
 * Uniscribe does this reordering too.
 */
#define HB_MODIFIED_COMBINING_CLASS_CCC103 3 /* sara u / sara uu */
#define HB_MODIFIED_COMBINING_CLASS_CCC107 107 /* mai * */

/* Lao */
#define HB_MODIFIED_COMBINING_CLASS_CCC118 118 /* sign u / sign uu */
#define HB_MODIFIED_COMBINING_CLASS_CCC122 122 /* mai * */

/* Tibetan
 *
 * In case of multiple vowel-signs, use u first (but after achung)
 * this allows Dzongkha multi-vowel shortcuts to render correctly
 */
#define HB_MODIFIED_COMBINING_CLASS_CCC129 129 /* sign aa */
#define HB_MODIFIED_COMBINING_CLASS_CCC130 132 /* sign i */
#define HB_MODIFIED_COMBINING_CLASS_CCC132 131 /* sign u */

/* Misc */

#define HB_UNICODE_GENERAL_CATEGORY_IS_MARK(gen_cat) \
	(FLAG_UNSAFE (gen_cat) & \
	 (FLAG (HB_UNICODE_GENERAL_CATEGORY_SPACING_MARK) | \
	  FLAG (HB_UNICODE_GENERAL_CATEGORY_ENCLOSING_MARK) | \
	  FLAG (HB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK)))


/*
 * Ranges, used for bsearch tables.
 */

struct hb_unicode_range_t
{
  static int
  cmp (const void *_key, const void *_item)
  {
    hb_codepoint_t cp = *((hb_codepoint_t *) _key);
    const hb_unicode_range_t *range = (hb_unicode_range_t *) _item;

    if (cp < range->start)
      return -1;
    else if (cp <= range->end)
      return 0;
    else
      return +1;
  }

  hb_codepoint_t start;
  hb_codepoint_t end;
};

/*
 * Emoji.
 */

HB_INTERNAL bool
_hb_unicode_is_emoji_Extended_Pictographic (hb_codepoint_t cp);
