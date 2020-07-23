/*
 * Copyright © 2009  Red Hat, Inc.
 * Copyright © 2011  Codethink Limited
 * Copyright © 2011,2012  Google, Inc.
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

#ifndef HB_H_IN
#error "Include <hb.h> instead."
#endif

#ifndef HB_UNICODE_H
#define HB_UNICODE_H

#include "hb-common.h"

HB_BEGIN_DECLS

/**
 * HB_UNICODE_MAX
 *
 * Since: 1.9.0
 **/
#define HB_UNICODE_MAX 0x10FFFFu

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
 * Assign 4 and 5, which are otherwise unassigned.
 */
#define HB_MODIFIED_COMBINING_CLASS_CCC84 4 /* length mark */
#define HB_MODIFIED_COMBINING_CLASS_CCC91 5 /* ai length mark */

/* Thai
 *
 * Modify U+0E38 and U+0E39 (ccc=103) to be reordered before U+0E3A (ccc=9).
 * Assign 3, which is unassigned otherwise.
 * Uniscribe does this reordering too.
 */
#define HB_MODIFIED_COMBINING_CLASS_CCC103 3   /* sara u / sara uu */
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

#define HB_UNICODE_GENERAL_CATEGORY_IS_MARK(gen_cat)                                                                   \
    (FLAG_UNSAFE(gen_cat) &                                                                                            \
     (FLAG(HB_UNICODE_GENERAL_CATEGORY_SPACING_MARK) | FLAG(HB_UNICODE_GENERAL_CATEGORY_ENCLOSING_MARK) |              \
      FLAG(HB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK)))

/* hb_unicode_general_category_t */

/* Unicode Character Database property: General_Category (gc) */
typedef enum {
    HB_UNICODE_GENERAL_CATEGORY_CONTROL,             /* Cc */
    HB_UNICODE_GENERAL_CATEGORY_FORMAT,              /* Cf */
    HB_UNICODE_GENERAL_CATEGORY_UNASSIGNED,          /* Cn */
    HB_UNICODE_GENERAL_CATEGORY_PRIVATE_USE,         /* Co */
    HB_UNICODE_GENERAL_CATEGORY_SURROGATE,           /* Cs */
    HB_UNICODE_GENERAL_CATEGORY_LOWERCASE_LETTER,    /* Ll */
    HB_UNICODE_GENERAL_CATEGORY_MODIFIER_LETTER,     /* Lm */
    HB_UNICODE_GENERAL_CATEGORY_OTHER_LETTER,        /* Lo */
    HB_UNICODE_GENERAL_CATEGORY_TITLECASE_LETTER,    /* Lt */
    HB_UNICODE_GENERAL_CATEGORY_UPPERCASE_LETTER,    /* Lu */
    HB_UNICODE_GENERAL_CATEGORY_SPACING_MARK,        /* Mc */
    HB_UNICODE_GENERAL_CATEGORY_ENCLOSING_MARK,      /* Me */
    HB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK,    /* Mn */
    HB_UNICODE_GENERAL_CATEGORY_DECIMAL_NUMBER,      /* Nd */
    HB_UNICODE_GENERAL_CATEGORY_LETTER_NUMBER,       /* Nl */
    HB_UNICODE_GENERAL_CATEGORY_OTHER_NUMBER,        /* No */
    HB_UNICODE_GENERAL_CATEGORY_CONNECT_PUNCTUATION, /* Pc */
    HB_UNICODE_GENERAL_CATEGORY_DASH_PUNCTUATION,    /* Pd */
    HB_UNICODE_GENERAL_CATEGORY_CLOSE_PUNCTUATION,   /* Pe */
    HB_UNICODE_GENERAL_CATEGORY_FINAL_PUNCTUATION,   /* Pf */
    HB_UNICODE_GENERAL_CATEGORY_INITIAL_PUNCTUATION, /* Pi */
    HB_UNICODE_GENERAL_CATEGORY_OTHER_PUNCTUATION,   /* Po */
    HB_UNICODE_GENERAL_CATEGORY_OPEN_PUNCTUATION,    /* Ps */
    HB_UNICODE_GENERAL_CATEGORY_CURRENCY_SYMBOL,     /* Sc */
    HB_UNICODE_GENERAL_CATEGORY_MODIFIER_SYMBOL,     /* Sk */
    HB_UNICODE_GENERAL_CATEGORY_MATH_SYMBOL,         /* Sm */
    HB_UNICODE_GENERAL_CATEGORY_OTHER_SYMBOL,        /* So */
    HB_UNICODE_GENERAL_CATEGORY_LINE_SEPARATOR,      /* Zl */
    HB_UNICODE_GENERAL_CATEGORY_PARAGRAPH_SEPARATOR, /* Zp */
    HB_UNICODE_GENERAL_CATEGORY_SPACE_SEPARATOR      /* Zs */
} hb_unicode_general_category_t;

/* hb_unicode_combining_class_t */

/* Note: newer versions of Unicode may add new values.  Clients should be ready to handle
 * any value in the 0..254 range being returned from hb_unicode_combining_class().
 */

/* Unicode Character Database property: Canonical_Combining_Class (ccc) */
typedef enum {
    HB_UNICODE_COMBINING_CLASS_NOT_REORDERED = 0,
    HB_UNICODE_COMBINING_CLASS_OVERLAY = 1,
    HB_UNICODE_COMBINING_CLASS_NUKTA = 7,
    HB_UNICODE_COMBINING_CLASS_KANA_VOICING = 8,
    HB_UNICODE_COMBINING_CLASS_VIRAMA = 9,

    /* Hebrew */
    HB_UNICODE_COMBINING_CLASS_CCC10 = 10,
    HB_UNICODE_COMBINING_CLASS_CCC11 = 11,
    HB_UNICODE_COMBINING_CLASS_CCC12 = 12,
    HB_UNICODE_COMBINING_CLASS_CCC13 = 13,
    HB_UNICODE_COMBINING_CLASS_CCC14 = 14,
    HB_UNICODE_COMBINING_CLASS_CCC15 = 15,
    HB_UNICODE_COMBINING_CLASS_CCC16 = 16,
    HB_UNICODE_COMBINING_CLASS_CCC17 = 17,
    HB_UNICODE_COMBINING_CLASS_CCC18 = 18,
    HB_UNICODE_COMBINING_CLASS_CCC19 = 19,
    HB_UNICODE_COMBINING_CLASS_CCC20 = 20,
    HB_UNICODE_COMBINING_CLASS_CCC21 = 21,
    HB_UNICODE_COMBINING_CLASS_CCC22 = 22,
    HB_UNICODE_COMBINING_CLASS_CCC23 = 23,
    HB_UNICODE_COMBINING_CLASS_CCC24 = 24,
    HB_UNICODE_COMBINING_CLASS_CCC25 = 25,
    HB_UNICODE_COMBINING_CLASS_CCC26 = 26,

    /* Arabic */
    HB_UNICODE_COMBINING_CLASS_CCC27 = 27,
    HB_UNICODE_COMBINING_CLASS_CCC28 = 28,
    HB_UNICODE_COMBINING_CLASS_CCC29 = 29,
    HB_UNICODE_COMBINING_CLASS_CCC30 = 30,
    HB_UNICODE_COMBINING_CLASS_CCC31 = 31,
    HB_UNICODE_COMBINING_CLASS_CCC32 = 32,
    HB_UNICODE_COMBINING_CLASS_CCC33 = 33,
    HB_UNICODE_COMBINING_CLASS_CCC34 = 34,
    HB_UNICODE_COMBINING_CLASS_CCC35 = 35,

    /* Syriac */
    HB_UNICODE_COMBINING_CLASS_CCC36 = 36,

    /* Telugu */
    HB_UNICODE_COMBINING_CLASS_CCC84 = 84,
    HB_UNICODE_COMBINING_CLASS_CCC91 = 91,

    /* Thai */
    HB_UNICODE_COMBINING_CLASS_CCC103 = 103,
    HB_UNICODE_COMBINING_CLASS_CCC107 = 107,

    /* Lao */
    HB_UNICODE_COMBINING_CLASS_CCC118 = 118,
    HB_UNICODE_COMBINING_CLASS_CCC122 = 122,

    /* Tibetan */
    HB_UNICODE_COMBINING_CLASS_CCC129 = 129,
    HB_UNICODE_COMBINING_CLASS_CCC130 = 130,
    HB_UNICODE_COMBINING_CLASS_CCC133 = 132,

    HB_UNICODE_COMBINING_CLASS_ATTACHED_BELOW_LEFT = 200,
    HB_UNICODE_COMBINING_CLASS_ATTACHED_BELOW = 202,
    HB_UNICODE_COMBINING_CLASS_ATTACHED_ABOVE = 214,
    HB_UNICODE_COMBINING_CLASS_ATTACHED_ABOVE_RIGHT = 216,
    HB_UNICODE_COMBINING_CLASS_BELOW_LEFT = 218,
    HB_UNICODE_COMBINING_CLASS_BELOW = 220,
    HB_UNICODE_COMBINING_CLASS_BELOW_RIGHT = 222,
    HB_UNICODE_COMBINING_CLASS_LEFT = 224,
    HB_UNICODE_COMBINING_CLASS_RIGHT = 226,
    HB_UNICODE_COMBINING_CLASS_ABOVE_LEFT = 228,
    HB_UNICODE_COMBINING_CLASS_ABOVE = 230,
    HB_UNICODE_COMBINING_CLASS_ABOVE_RIGHT = 232,
    HB_UNICODE_COMBINING_CLASS_DOUBLE_BELOW = 233,
    HB_UNICODE_COMBINING_CLASS_DOUBLE_ABOVE = 234,

    HB_UNICODE_COMBINING_CLASS_IOTA_SUBSCRIPT = 240,

    HB_UNICODE_COMBINING_CLASS_INVALID = 255
} hb_unicode_combining_class_t;

/* Space estimates based on:
 * https://unicode.org/charts/PDF/U2000.pdf
 * https://docs.microsoft.com/en-us/typography/develop/character-design-standards/whitespace
 */
typedef enum {
    HB_SPACE_NOT_SPACE = 0,
    HB_SPACE_EM = 1,
    HB_SPACE_EM_2 = 2,
    HB_SPACE_EM_3 = 3,
    HB_SPACE_EM_4 = 4,
    HB_SPACE_EM_5 = 5,
    HB_SPACE_EM_6 = 6,
    HB_SPACE_EM_16 = 16,
    HB_SPACE_4_EM_18, /* 4/18th of an EM! */
    HB_SPACE,
    HB_SPACE_FIGURE,
    HB_SPACE_PUNCTUATION,
    HB_SPACE_NARROW,
} hb_space_t;

HB_EXTERN hb_bool_t hb_ucd_is_default_ignorable(hb_codepoint_t cp);
HB_EXTERN hb_script_t hb_ucd_script(hb_codepoint_t cp);
HB_EXTERN hb_unicode_combining_class_t hb_ucd_combining_class(hb_codepoint_t cp);
HB_EXTERN hb_unicode_general_category_t hb_ucd_general_category(hb_codepoint_t cp);
HB_EXTERN hb_codepoint_t hb_ucd_mirroring(hb_codepoint_t cp);
HB_EXTERN unsigned int hb_ucd_modified_combining_class(hb_codepoint_t u);
HB_EXTERN hb_bool_t hb_ucd_compose(hb_codepoint_t a, hb_codepoint_t b, hb_codepoint_t *ab);
HB_EXTERN hb_bool_t hb_ucd_decompose(hb_codepoint_t ab, hb_codepoint_t *a, hb_codepoint_t *b);
HB_EXTERN hb_space_t hb_ucd_space_fallback_type(hb_codepoint_t cp);
HB_EXTERN hb_bool_t hb_ucd_is_variation_selector(hb_codepoint_t cp);
HB_EXTERN hb_bool_t hb_ucd_is_emoji_extended_pictographic(hb_codepoint_t cp);

HB_END_DECLS

#endif /* HB_UNICODE_H */
