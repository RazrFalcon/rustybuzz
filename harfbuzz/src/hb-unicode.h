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

#ifndef RB_H_IN
#error "Include <hb.h> instead."
#endif

#ifndef RB_UNICODE_H
#define RB_UNICODE_H

#include "hb-common.h"

RB_BEGIN_DECLS

/**
 * RB_UNICODE_MAX
 *
 * Since: 1.9.0
 **/
#define RB_UNICODE_MAX 0x10FFFFu

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
#define RB_MODIFIED_COMBINING_CLASS_CCC10 22 /* sheva */
#define RB_MODIFIED_COMBINING_CLASS_CCC11 15 /* hataf segol */
#define RB_MODIFIED_COMBINING_CLASS_CCC12 16 /* hataf patah */
#define RB_MODIFIED_COMBINING_CLASS_CCC13 17 /* hataf qamats */
#define RB_MODIFIED_COMBINING_CLASS_CCC14 23 /* hiriq */
#define RB_MODIFIED_COMBINING_CLASS_CCC15 18 /* tsere */
#define RB_MODIFIED_COMBINING_CLASS_CCC16 19 /* segol */
#define RB_MODIFIED_COMBINING_CLASS_CCC17 20 /* patah */
#define RB_MODIFIED_COMBINING_CLASS_CCC18 21 /* qamats */
#define RB_MODIFIED_COMBINING_CLASS_CCC19 14 /* holam */
#define RB_MODIFIED_COMBINING_CLASS_CCC20 24 /* qubuts */
#define RB_MODIFIED_COMBINING_CLASS_CCC21 12 /* dagesh */
#define RB_MODIFIED_COMBINING_CLASS_CCC22 25 /* meteg */
#define RB_MODIFIED_COMBINING_CLASS_CCC23 13 /* rafe */
#define RB_MODIFIED_COMBINING_CLASS_CCC24 10 /* shin dot */
#define RB_MODIFIED_COMBINING_CLASS_CCC25 11 /* sin dot */
#define RB_MODIFIED_COMBINING_CLASS_CCC26 26 /* point varika */

/*
 * Arabic
 *
 * Modify to move Shadda (ccc=33) before other marks.  See:
 * https://unicode.org/faq/normalization.html#8
 * https://unicode.org/faq/normalization.html#9
 */
#define RB_MODIFIED_COMBINING_CLASS_CCC27 28 /* fathatan */
#define RB_MODIFIED_COMBINING_CLASS_CCC28 29 /* dammatan */
#define RB_MODIFIED_COMBINING_CLASS_CCC29 30 /* kasratan */
#define RB_MODIFIED_COMBINING_CLASS_CCC30 31 /* fatha */
#define RB_MODIFIED_COMBINING_CLASS_CCC31 32 /* damma */
#define RB_MODIFIED_COMBINING_CLASS_CCC32 33 /* kasra */
#define RB_MODIFIED_COMBINING_CLASS_CCC33 27 /* shadda */
#define RB_MODIFIED_COMBINING_CLASS_CCC34 34 /* sukun */
#define RB_MODIFIED_COMBINING_CLASS_CCC35 35 /* superscript alef */

/* Syriac */
#define RB_MODIFIED_COMBINING_CLASS_CCC36 36 /* superscript alaph */

/* Telugu
 *
 * Modify Telugu length marks (ccc=84, ccc=91).
 * These are the only matras in the main Indic scripts range that have
 * a non-zero ccc.  That makes them reorder with the Halant (ccc=9).
 * Assign 4 and 5, which are otherwise unassigned.
 */
#define RB_MODIFIED_COMBINING_CLASS_CCC84 4 /* length mark */
#define RB_MODIFIED_COMBINING_CLASS_CCC91 5 /* ai length mark */

/* Thai
 *
 * Modify U+0E38 and U+0E39 (ccc=103) to be reordered before U+0E3A (ccc=9).
 * Assign 3, which is unassigned otherwise.
 * Uniscribe does this reordering too.
 */
#define RB_MODIFIED_COMBINING_CLASS_CCC103 3   /* sara u / sara uu */
#define RB_MODIFIED_COMBINING_CLASS_CCC107 107 /* mai * */

/* Lao */
#define RB_MODIFIED_COMBINING_CLASS_CCC118 118 /* sign u / sign uu */
#define RB_MODIFIED_COMBINING_CLASS_CCC122 122 /* mai * */

/* Tibetan
 *
 * In case of multiple vowel-signs, use u first (but after achung)
 * this allows Dzongkha multi-vowel shortcuts to render correctly
 */
#define RB_MODIFIED_COMBINING_CLASS_CCC129 129 /* sign aa */
#define RB_MODIFIED_COMBINING_CLASS_CCC130 132 /* sign i */
#define RB_MODIFIED_COMBINING_CLASS_CCC132 131 /* sign u */

/* Misc */

#define RB_UNICODE_GENERAL_CATEGORY_IS_MARK(gen_cat)                                                                   \
    (FLAG_UNSAFE(gen_cat) &                                                                                            \
     (FLAG(RB_UNICODE_GENERAL_CATEGORY_SPACING_MARK) | FLAG(RB_UNICODE_GENERAL_CATEGORY_ENCLOSING_MARK) |              \
      FLAG(RB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK)))

/* rb_unicode_general_category_t */

/* Unicode Character Database property: General_Category (gc) */
typedef enum {
    RB_UNICODE_GENERAL_CATEGORY_CONTROL,             /* Cc */
    RB_UNICODE_GENERAL_CATEGORY_FORMAT,              /* Cf */
    RB_UNICODE_GENERAL_CATEGORY_UNASSIGNED,          /* Cn */
    RB_UNICODE_GENERAL_CATEGORY_PRIVATE_USE,         /* Co */
    RB_UNICODE_GENERAL_CATEGORY_SURROGATE,           /* Cs */
    RB_UNICODE_GENERAL_CATEGORY_LOWERCASE_LETTER,    /* Ll */
    RB_UNICODE_GENERAL_CATEGORY_MODIFIER_LETTER,     /* Lm */
    RB_UNICODE_GENERAL_CATEGORY_OTHER_LETTER,        /* Lo */
    RB_UNICODE_GENERAL_CATEGORY_TITLECASE_LETTER,    /* Lt */
    RB_UNICODE_GENERAL_CATEGORY_UPPERCASE_LETTER,    /* Lu */
    RB_UNICODE_GENERAL_CATEGORY_SPACING_MARK,        /* Mc */
    RB_UNICODE_GENERAL_CATEGORY_ENCLOSING_MARK,      /* Me */
    RB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK,    /* Mn */
    RB_UNICODE_GENERAL_CATEGORY_DECIMAL_NUMBER,      /* Nd */
    RB_UNICODE_GENERAL_CATEGORY_LETTER_NUMBER,       /* Nl */
    RB_UNICODE_GENERAL_CATEGORY_OTHER_NUMBER,        /* No */
    RB_UNICODE_GENERAL_CATEGORY_CONNECT_PUNCTUATION, /* Pc */
    RB_UNICODE_GENERAL_CATEGORY_DASH_PUNCTUATION,    /* Pd */
    RB_UNICODE_GENERAL_CATEGORY_CLOSE_PUNCTUATION,   /* Pe */
    RB_UNICODE_GENERAL_CATEGORY_FINAL_PUNCTUATION,   /* Pf */
    RB_UNICODE_GENERAL_CATEGORY_INITIAL_PUNCTUATION, /* Pi */
    RB_UNICODE_GENERAL_CATEGORY_OTHER_PUNCTUATION,   /* Po */
    RB_UNICODE_GENERAL_CATEGORY_OPEN_PUNCTUATION,    /* Ps */
    RB_UNICODE_GENERAL_CATEGORY_CURRENCY_SYMBOL,     /* Sc */
    RB_UNICODE_GENERAL_CATEGORY_MODIFIER_SYMBOL,     /* Sk */
    RB_UNICODE_GENERAL_CATEGORY_MATH_SYMBOL,         /* Sm */
    RB_UNICODE_GENERAL_CATEGORY_OTHER_SYMBOL,        /* So */
    RB_UNICODE_GENERAL_CATEGORY_LINE_SEPARATOR,      /* Zl */
    RB_UNICODE_GENERAL_CATEGORY_PARAGRAPH_SEPARATOR, /* Zp */
    RB_UNICODE_GENERAL_CATEGORY_SPACE_SEPARATOR      /* Zs */
} rb_unicode_general_category_t;

/* rb_unicode_combining_class_t */

/* Note: newer versions of Unicode may add new values.  Clients should be ready to handle
 * any value in the 0..254 range being returned from rb_unicode_combining_class().
 */

/* Unicode Character Database property: Canonical_Combining_Class (ccc) */
typedef enum {
    RB_UNICODE_COMBINING_CLASS_NOT_REORDERED = 0,
    RB_UNICODE_COMBINING_CLASS_OVERLAY = 1,
    RB_UNICODE_COMBINING_CLASS_NUKTA = 7,
    RB_UNICODE_COMBINING_CLASS_KANA_VOICING = 8,
    RB_UNICODE_COMBINING_CLASS_VIRAMA = 9,

    /* Hebrew */
    RB_UNICODE_COMBINING_CLASS_CCC10 = 10,
    RB_UNICODE_COMBINING_CLASS_CCC11 = 11,
    RB_UNICODE_COMBINING_CLASS_CCC12 = 12,
    RB_UNICODE_COMBINING_CLASS_CCC13 = 13,
    RB_UNICODE_COMBINING_CLASS_CCC14 = 14,
    RB_UNICODE_COMBINING_CLASS_CCC15 = 15,
    RB_UNICODE_COMBINING_CLASS_CCC16 = 16,
    RB_UNICODE_COMBINING_CLASS_CCC17 = 17,
    RB_UNICODE_COMBINING_CLASS_CCC18 = 18,
    RB_UNICODE_COMBINING_CLASS_CCC19 = 19,
    RB_UNICODE_COMBINING_CLASS_CCC20 = 20,
    RB_UNICODE_COMBINING_CLASS_CCC21 = 21,
    RB_UNICODE_COMBINING_CLASS_CCC22 = 22,
    RB_UNICODE_COMBINING_CLASS_CCC23 = 23,
    RB_UNICODE_COMBINING_CLASS_CCC24 = 24,
    RB_UNICODE_COMBINING_CLASS_CCC25 = 25,
    RB_UNICODE_COMBINING_CLASS_CCC26 = 26,

    /* Arabic */
    RB_UNICODE_COMBINING_CLASS_CCC27 = 27,
    RB_UNICODE_COMBINING_CLASS_CCC28 = 28,
    RB_UNICODE_COMBINING_CLASS_CCC29 = 29,
    RB_UNICODE_COMBINING_CLASS_CCC30 = 30,
    RB_UNICODE_COMBINING_CLASS_CCC31 = 31,
    RB_UNICODE_COMBINING_CLASS_CCC32 = 32,
    RB_UNICODE_COMBINING_CLASS_CCC33 = 33,
    RB_UNICODE_COMBINING_CLASS_CCC34 = 34,
    RB_UNICODE_COMBINING_CLASS_CCC35 = 35,

    /* Syriac */
    RB_UNICODE_COMBINING_CLASS_CCC36 = 36,

    /* Telugu */
    RB_UNICODE_COMBINING_CLASS_CCC84 = 84,
    RB_UNICODE_COMBINING_CLASS_CCC91 = 91,

    /* Thai */
    RB_UNICODE_COMBINING_CLASS_CCC103 = 103,
    RB_UNICODE_COMBINING_CLASS_CCC107 = 107,

    /* Lao */
    RB_UNICODE_COMBINING_CLASS_CCC118 = 118,
    RB_UNICODE_COMBINING_CLASS_CCC122 = 122,

    /* Tibetan */
    RB_UNICODE_COMBINING_CLASS_CCC129 = 129,
    RB_UNICODE_COMBINING_CLASS_CCC130 = 130,
    RB_UNICODE_COMBINING_CLASS_CCC133 = 132,

    RB_UNICODE_COMBINING_CLASS_ATTACHED_BELOW_LEFT = 200,
    RB_UNICODE_COMBINING_CLASS_ATTACHED_BELOW = 202,
    RB_UNICODE_COMBINING_CLASS_ATTACHED_ABOVE = 214,
    RB_UNICODE_COMBINING_CLASS_ATTACHED_ABOVE_RIGHT = 216,
    RB_UNICODE_COMBINING_CLASS_BELOW_LEFT = 218,
    RB_UNICODE_COMBINING_CLASS_BELOW = 220,
    RB_UNICODE_COMBINING_CLASS_BELOW_RIGHT = 222,
    RB_UNICODE_COMBINING_CLASS_LEFT = 224,
    RB_UNICODE_COMBINING_CLASS_RIGHT = 226,
    RB_UNICODE_COMBINING_CLASS_ABOVE_LEFT = 228,
    RB_UNICODE_COMBINING_CLASS_ABOVE = 230,
    RB_UNICODE_COMBINING_CLASS_ABOVE_RIGHT = 232,
    RB_UNICODE_COMBINING_CLASS_DOUBLE_BELOW = 233,
    RB_UNICODE_COMBINING_CLASS_DOUBLE_ABOVE = 234,

    RB_UNICODE_COMBINING_CLASS_IOTA_SUBSCRIPT = 240,

    RB_UNICODE_COMBINING_CLASS_INVALID = 255
} rb_unicode_combining_class_t;

/* Space estimates based on:
 * https://unicode.org/charts/PDF/U2000.pdf
 * https://docs.microsoft.com/en-us/typography/develop/character-design-standards/whitespace
 */
typedef enum {
    RB_SPACE_NOT_SPACE = 0,
    RB_SPACE_EM = 1,
    RB_SPACE_EM_2 = 2,
    RB_SPACE_EM_3 = 3,
    RB_SPACE_EM_4 = 4,
    RB_SPACE_EM_5 = 5,
    RB_SPACE_EM_6 = 6,
    RB_SPACE_EM_16 = 16,
    RB_SPACE_4_EM_18, /* 4/18th of an EM! */
    RB_SPACE,
    RB_SPACE_FIGURE,
    RB_SPACE_PUNCTUATION,
    RB_SPACE_NARROW,
} rb_space_t;

RB_EXTERN rb_bool_t rb_ucd_is_default_ignorable(rb_codepoint_t cp);
RB_EXTERN rb_script_t rb_ucd_script(rb_codepoint_t cp);
RB_EXTERN rb_unicode_combining_class_t rb_ucd_combining_class(rb_codepoint_t cp);
RB_EXTERN rb_unicode_general_category_t rb_ucd_general_category(rb_codepoint_t cp);
RB_EXTERN rb_codepoint_t rb_ucd_mirroring(rb_codepoint_t cp);
RB_EXTERN unsigned int rb_ucd_modified_combining_class(rb_codepoint_t u);
RB_EXTERN rb_bool_t rb_ucd_compose(rb_codepoint_t a, rb_codepoint_t b, rb_codepoint_t *ab);
RB_EXTERN rb_bool_t rb_ucd_decompose(rb_codepoint_t ab, rb_codepoint_t *a, rb_codepoint_t *b);
RB_EXTERN rb_space_t rb_ucd_space_fallback_type(rb_codepoint_t cp);
RB_EXTERN rb_bool_t rb_ucd_is_variation_selector(rb_codepoint_t cp);
RB_EXTERN rb_bool_t rb_ucd_is_emoji_extended_pictographic(rb_codepoint_t cp);

RB_END_DECLS

#endif /* RB_UNICODE_H */
