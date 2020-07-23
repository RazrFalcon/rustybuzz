/*
 * Copyright © 2007,2008,2009  Red Hat, Inc.
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
 * Google Author(s): Behdad Esfahbod
 */

#ifndef RB_H_IN
#error "Include <hb.h> instead."
#endif

#ifndef RB_COMMON_H
#define RB_COMMON_H

#ifndef RB_EXTERN
#define RB_EXTERN extern
#endif

#ifndef RB_BEGIN_DECLS
#ifdef __cplusplus
#define RB_BEGIN_DECLS extern "C" {
#define RB_END_DECLS }
#else /* !__cplusplus */
#define RB_BEGIN_DECLS
#define RB_END_DECLS
#endif /* !__cplusplus */
#endif

#if defined(_SVR4) || defined(SVR4) || defined(__OpenBSD__) || defined(_sgi) || defined(__sun) || defined(sun) ||      \
    defined(__digital__) || defined(__HP_cc)
#include <inttypes.h>
#elif defined(_AIX)
#include <sys/inttypes.h>
#elif defined(_MSC_VER) && _MSC_VER < 1600
/* VS 2010 (_MSC_VER 1600) has stdint.h */
typedef __int8 int8_t;
typedef unsigned __int8 uint8_t;
typedef __int16 int16_t;
typedef unsigned __int16 uint16_t;
typedef __int32 int32_t;
typedef unsigned __int32 uint32_t;
typedef __int64 int64_t;
typedef unsigned __int64 uint64_t;
#elif defined(__KERNEL__)
#include <linux/types.h>
#else
#include <stdint.h>
#endif

#if defined(__GNUC__) && ((__GNUC__ > 3) || (__GNUC__ == 3 && __GNUC_MINOR__ >= 1))
#define RB_DEPRECATED __attribute__((__deprecated__))
#elif defined(_MSC_VER) && (_MSC_VER >= 1300)
#define RB_DEPRECATED __declspec(deprecated)
#else
#define RB_DEPRECATED
#endif

#if defined(__GNUC__) && ((__GNUC__ > 4) || (__GNUC__ == 4 && __GNUC_MINOR__ >= 5))
#define RB_DEPRECATED_FOR(f) __attribute__((__deprecated__("Use '" #f "' instead")))
#elif defined(_MSC_FULL_VER) && (_MSC_FULL_VER > 140050320)
#define RB_DEPRECATED_FOR(f) __declspec(deprecated("is deprecated. Use '" #f "' instead"))
#else
#define RB_DEPRECATED_FOR(f) RB_DEPRECATED
#endif

RB_BEGIN_DECLS

typedef int rb_bool_t;

typedef uint32_t rb_codepoint_t;
typedef int32_t rb_position_t;
typedef uint32_t rb_mask_t;

typedef union _rb_var_int_t {
    uint32_t u32;
    int32_t i32;
    uint16_t u16[2];
    int16_t i16[2];
    uint8_t u8[4];
    int8_t i8[4];
} rb_var_int_t;

/* rb_tag_t */

typedef uint32_t rb_tag_t;

#define RB_TAG(c1, c2, c3, c4)                                                                                         \
    ((rb_tag_t)((((uint32_t)(c1)&0xFF) << 24) | (((uint32_t)(c2)&0xFF) << 16) | (((uint32_t)(c3)&0xFF) << 8) |         \
                ((uint32_t)(c4)&0xFF)))
#define RB_UNTAG(tag)                                                                                                  \
    (uint8_t)(((tag) >> 24) & 0xFF), (uint8_t)(((tag) >> 16) & 0xFF), (uint8_t)(((tag) >> 8) & 0xFF),                  \
        (uint8_t)((tag)&0xFF)

#define RB_TAG_NONE RB_TAG(0, 0, 0, 0)
#define RB_TAG_MAX RB_TAG(0xff, 0xff, 0xff, 0xff)
#define RB_TAG_MAX_SIGNED RB_TAG(0x7f, 0xff, 0xff, 0xff)

/* len=-1 means str is NUL-terminated. */
RB_EXTERN rb_tag_t rb_tag_from_string(const char *str, int len);

/**
 * rb_direction_t:
 * @RB_DIRECTION_INVALID: Initial, unset direction.
 * @RB_DIRECTION_LTR: Text is set horizontally from left to right.
 * @RB_DIRECTION_RTL: Text is set horizontally from right to left.
 * @RB_DIRECTION_TTB: Text is set vertically from top to bottom.
 * @RB_DIRECTION_BTT: Text is set vertically from bottom to top.
 */
typedef enum {
    RB_DIRECTION_INVALID = 0,
    RB_DIRECTION_LTR = 4,
    RB_DIRECTION_RTL,
    RB_DIRECTION_TTB,
    RB_DIRECTION_BTT
} rb_direction_t;

#define RB_DIRECTION_IS_VALID(dir) ((((unsigned int)(dir)) & ~3U) == 4)
/* Direction must be valid for the following */
#define RB_DIRECTION_IS_HORIZONTAL(dir) ((((unsigned int)(dir)) & ~1U) == 4)
#define RB_DIRECTION_IS_VERTICAL(dir) ((((unsigned int)(dir)) & ~1U) == 6)
#define RB_DIRECTION_IS_FORWARD(dir) ((((unsigned int)(dir)) & ~2U) == 4)
#define RB_DIRECTION_IS_BACKWARD(dir) ((((unsigned int)(dir)) & ~2U) == 5)
#define RB_DIRECTION_REVERSE(dir) ((rb_direction_t)(((unsigned int)(dir)) ^ 1))

/* rb_script_t */

/* https://unicode.org/iso15924/ */
/* https://docs.google.com/spreadsheets/d/1Y90M0Ie3MUJ6UVCRDOypOtijlMDLNNyyLk36T6iMu0o */
/* Unicode Character Database property: Script (sc) */
typedef enum {
    /*1.1*/ RB_SCRIPT_COMMON = RB_TAG('Z', 'y', 'y', 'y'),
    /*1.1*/ RB_SCRIPT_INHERITED = RB_TAG('Z', 'i', 'n', 'h'),
    /*5.0*/ RB_SCRIPT_UNKNOWN = RB_TAG('Z', 'z', 'z', 'z'),

    /*1.1*/ RB_SCRIPT_ARABIC = RB_TAG('A', 'r', 'a', 'b'),
    /*1.1*/ RB_SCRIPT_ARMENIAN = RB_TAG('A', 'r', 'm', 'n'),
    /*1.1*/ RB_SCRIPT_BENGALI = RB_TAG('B', 'e', 'n', 'g'),
    /*1.1*/ RB_SCRIPT_CYRILLIC = RB_TAG('C', 'y', 'r', 'l'),
    /*1.1*/ RB_SCRIPT_DEVANAGARI = RB_TAG('D', 'e', 'v', 'a'),
    /*1.1*/ RB_SCRIPT_GEORGIAN = RB_TAG('G', 'e', 'o', 'r'),
    /*1.1*/ RB_SCRIPT_GREEK = RB_TAG('G', 'r', 'e', 'k'),
    /*1.1*/ RB_SCRIPT_GUJARATI = RB_TAG('G', 'u', 'j', 'r'),
    /*1.1*/ RB_SCRIPT_GURMUKHI = RB_TAG('G', 'u', 'r', 'u'),
    /*1.1*/ RB_SCRIPT_HANGUL = RB_TAG('H', 'a', 'n', 'g'),
    /*1.1*/ RB_SCRIPT_HAN = RB_TAG('H', 'a', 'n', 'i'),
    /*1.1*/ RB_SCRIPT_HEBREW = RB_TAG('H', 'e', 'b', 'r'),
    /*1.1*/ RB_SCRIPT_HIRAGANA = RB_TAG('H', 'i', 'r', 'a'),
    /*1.1*/ RB_SCRIPT_KANNADA = RB_TAG('K', 'n', 'd', 'a'),
    /*1.1*/ RB_SCRIPT_KATAKANA = RB_TAG('K', 'a', 'n', 'a'),
    /*1.1*/ RB_SCRIPT_LAO = RB_TAG('L', 'a', 'o', 'o'),
    /*1.1*/ RB_SCRIPT_LATIN = RB_TAG('L', 'a', 't', 'n'),
    /*1.1*/ RB_SCRIPT_MALAYALAM = RB_TAG('M', 'l', 'y', 'm'),
    /*1.1*/ RB_SCRIPT_ORIYA = RB_TAG('O', 'r', 'y', 'a'),
    /*1.1*/ RB_SCRIPT_TAMIL = RB_TAG('T', 'a', 'm', 'l'),
    /*1.1*/ RB_SCRIPT_TELUGU = RB_TAG('T', 'e', 'l', 'u'),
    /*1.1*/ RB_SCRIPT_THAI = RB_TAG('T', 'h', 'a', 'i'),

    /*2.0*/ RB_SCRIPT_TIBETAN = RB_TAG('T', 'i', 'b', 't'),

    /*3.0*/ RB_SCRIPT_BOPOMOFO = RB_TAG('B', 'o', 'p', 'o'),
    /*3.0*/ RB_SCRIPT_BRAILLE = RB_TAG('B', 'r', 'a', 'i'),
    /*3.0*/ RB_SCRIPT_CANADIAN_SYLLABICS = RB_TAG('C', 'a', 'n', 's'),
    /*3.0*/ RB_SCRIPT_CHEROKEE = RB_TAG('C', 'h', 'e', 'r'),
    /*3.0*/ RB_SCRIPT_ETHIOPIC = RB_TAG('E', 't', 'h', 'i'),
    /*3.0*/ RB_SCRIPT_KHMER = RB_TAG('K', 'h', 'm', 'r'),
    /*3.0*/ RB_SCRIPT_MONGOLIAN = RB_TAG('M', 'o', 'n', 'g'),
    /*3.0*/ RB_SCRIPT_MYANMAR = RB_TAG('M', 'y', 'm', 'r'),
    /*3.0*/ RB_SCRIPT_OGHAM = RB_TAG('O', 'g', 'a', 'm'),
    /*3.0*/ RB_SCRIPT_RUNIC = RB_TAG('R', 'u', 'n', 'r'),
    /*3.0*/ RB_SCRIPT_SINHALA = RB_TAG('S', 'i', 'n', 'h'),
    /*3.0*/ RB_SCRIPT_SYRIAC = RB_TAG('S', 'y', 'r', 'c'),
    /*3.0*/ RB_SCRIPT_THAANA = RB_TAG('T', 'h', 'a', 'a'),
    /*3.0*/ RB_SCRIPT_YI = RB_TAG('Y', 'i', 'i', 'i'),

    /*3.1*/ RB_SCRIPT_DESERET = RB_TAG('D', 's', 'r', 't'),
    /*3.1*/ RB_SCRIPT_GOTHIC = RB_TAG('G', 'o', 't', 'h'),
    /*3.1*/ RB_SCRIPT_OLD_ITALIC = RB_TAG('I', 't', 'a', 'l'),

    /*3.2*/ RB_SCRIPT_BUHID = RB_TAG('B', 'u', 'h', 'd'),
    /*3.2*/ RB_SCRIPT_HANUNOO = RB_TAG('H', 'a', 'n', 'o'),
    /*3.2*/ RB_SCRIPT_TAGALOG = RB_TAG('T', 'g', 'l', 'g'),
    /*3.2*/ RB_SCRIPT_TAGBANWA = RB_TAG('T', 'a', 'g', 'b'),

    /*4.0*/ RB_SCRIPT_CYPRIOT = RB_TAG('C', 'p', 'r', 't'),
    /*4.0*/ RB_SCRIPT_LIMBU = RB_TAG('L', 'i', 'm', 'b'),
    /*4.0*/ RB_SCRIPT_LINEAR_B = RB_TAG('L', 'i', 'n', 'b'),
    /*4.0*/ RB_SCRIPT_OSMANYA = RB_TAG('O', 's', 'm', 'a'),
    /*4.0*/ RB_SCRIPT_SHAVIAN = RB_TAG('S', 'h', 'a', 'w'),
    /*4.0*/ RB_SCRIPT_TAI_LE = RB_TAG('T', 'a', 'l', 'e'),
    /*4.0*/ RB_SCRIPT_UGARITIC = RB_TAG('U', 'g', 'a', 'r'),

    /*4.1*/ RB_SCRIPT_BUGINESE = RB_TAG('B', 'u', 'g', 'i'),
    /*4.1*/ RB_SCRIPT_COPTIC = RB_TAG('C', 'o', 'p', 't'),
    /*4.1*/ RB_SCRIPT_GLAGOLITIC = RB_TAG('G', 'l', 'a', 'g'),
    /*4.1*/ RB_SCRIPT_KHAROSHTHI = RB_TAG('K', 'h', 'a', 'r'),
    /*4.1*/ RB_SCRIPT_NEW_TAI_LUE = RB_TAG('T', 'a', 'l', 'u'),
    /*4.1*/ RB_SCRIPT_OLD_PERSIAN = RB_TAG('X', 'p', 'e', 'o'),
    /*4.1*/ RB_SCRIPT_SYLOTI_NAGRI = RB_TAG('S', 'y', 'l', 'o'),
    /*4.1*/ RB_SCRIPT_TIFINAGH = RB_TAG('T', 'f', 'n', 'g'),

    /*5.0*/ RB_SCRIPT_BALINESE = RB_TAG('B', 'a', 'l', 'i'),
    /*5.0*/ RB_SCRIPT_CUNEIFORM = RB_TAG('X', 's', 'u', 'x'),
    /*5.0*/ RB_SCRIPT_NKO = RB_TAG('N', 'k', 'o', 'o'),
    /*5.0*/ RB_SCRIPT_PHAGS_PA = RB_TAG('P', 'h', 'a', 'g'),
    /*5.0*/ RB_SCRIPT_PHOENICIAN = RB_TAG('P', 'h', 'n', 'x'),

    /*5.1*/ RB_SCRIPT_CARIAN = RB_TAG('C', 'a', 'r', 'i'),
    /*5.1*/ RB_SCRIPT_CHAM = RB_TAG('C', 'h', 'a', 'm'),
    /*5.1*/ RB_SCRIPT_KAYAH_LI = RB_TAG('K', 'a', 'l', 'i'),
    /*5.1*/ RB_SCRIPT_LEPCHA = RB_TAG('L', 'e', 'p', 'c'),
    /*5.1*/ RB_SCRIPT_LYCIAN = RB_TAG('L', 'y', 'c', 'i'),
    /*5.1*/ RB_SCRIPT_LYDIAN = RB_TAG('L', 'y', 'd', 'i'),
    /*5.1*/ RB_SCRIPT_OL_CHIKI = RB_TAG('O', 'l', 'c', 'k'),
    /*5.1*/ RB_SCRIPT_REJANG = RB_TAG('R', 'j', 'n', 'g'),
    /*5.1*/ RB_SCRIPT_SAURASHTRA = RB_TAG('S', 'a', 'u', 'r'),
    /*5.1*/ RB_SCRIPT_SUNDANESE = RB_TAG('S', 'u', 'n', 'd'),
    /*5.1*/ RB_SCRIPT_VAI = RB_TAG('V', 'a', 'i', 'i'),

    /*5.2*/ RB_SCRIPT_AVESTAN = RB_TAG('A', 'v', 's', 't'),
    /*5.2*/ RB_SCRIPT_BAMUM = RB_TAG('B', 'a', 'm', 'u'),
    /*5.2*/ RB_SCRIPT_EGYPTIAN_HIEROGLYPHS = RB_TAG('E', 'g', 'y', 'p'),
    /*5.2*/ RB_SCRIPT_IMPERIAL_ARAMAIC = RB_TAG('A', 'r', 'm', 'i'),
    /*5.2*/ RB_SCRIPT_INSCRIPTIONAL_PAHLAVI = RB_TAG('P', 'h', 'l', 'i'),
    /*5.2*/ RB_SCRIPT_INSCRIPTIONAL_PARTHIAN = RB_TAG('P', 'r', 't', 'i'),
    /*5.2*/ RB_SCRIPT_JAVANESE = RB_TAG('J', 'a', 'v', 'a'),
    /*5.2*/ RB_SCRIPT_KAITHI = RB_TAG('K', 't', 'h', 'i'),
    /*5.2*/ RB_SCRIPT_LISU = RB_TAG('L', 'i', 's', 'u'),
    /*5.2*/ RB_SCRIPT_MEETEI_MAYEK = RB_TAG('M', 't', 'e', 'i'),
    /*5.2*/ RB_SCRIPT_OLD_SOUTH_ARABIAN = RB_TAG('S', 'a', 'r', 'b'),
    /*5.2*/ RB_SCRIPT_OLD_TURKIC = RB_TAG('O', 'r', 'k', 'h'),
    /*5.2*/ RB_SCRIPT_SAMARITAN = RB_TAG('S', 'a', 'm', 'r'),
    /*5.2*/ RB_SCRIPT_TAI_THAM = RB_TAG('L', 'a', 'n', 'a'),
    /*5.2*/ RB_SCRIPT_TAI_VIET = RB_TAG('T', 'a', 'v', 't'),

    /*6.0*/ RB_SCRIPT_BATAK = RB_TAG('B', 'a', 't', 'k'),
    /*6.0*/ RB_SCRIPT_BRAHMI = RB_TAG('B', 'r', 'a', 'h'),
    /*6.0*/ RB_SCRIPT_MANDAIC = RB_TAG('M', 'a', 'n', 'd'),

    /*6.1*/ RB_SCRIPT_CHAKMA = RB_TAG('C', 'a', 'k', 'm'),
    /*6.1*/ RB_SCRIPT_MEROITIC_CURSIVE = RB_TAG('M', 'e', 'r', 'c'),
    /*6.1*/ RB_SCRIPT_MEROITIC_HIEROGLYPHS = RB_TAG('M', 'e', 'r', 'o'),
    /*6.1*/ RB_SCRIPT_MIAO = RB_TAG('P', 'l', 'r', 'd'),
    /*6.1*/ RB_SCRIPT_SHARADA = RB_TAG('S', 'h', 'r', 'd'),
    /*6.1*/ RB_SCRIPT_SORA_SOMPENG = RB_TAG('S', 'o', 'r', 'a'),
    /*6.1*/ RB_SCRIPT_TAKRI = RB_TAG('T', 'a', 'k', 'r'),

    /*
     * Since: 0.9.30
     */
    /*7.0*/ RB_SCRIPT_BASSA_VAH = RB_TAG('B', 'a', 's', 's'),
    /*7.0*/ RB_SCRIPT_CAUCASIAN_ALBANIAN = RB_TAG('A', 'g', 'h', 'b'),
    /*7.0*/ RB_SCRIPT_DUPLOYAN = RB_TAG('D', 'u', 'p', 'l'),
    /*7.0*/ RB_SCRIPT_ELBASAN = RB_TAG('E', 'l', 'b', 'a'),
    /*7.0*/ RB_SCRIPT_GRANTHA = RB_TAG('G', 'r', 'a', 'n'),
    /*7.0*/ RB_SCRIPT_KHOJKI = RB_TAG('K', 'h', 'o', 'j'),
    /*7.0*/ RB_SCRIPT_KHUDAWADI = RB_TAG('S', 'i', 'n', 'd'),
    /*7.0*/ RB_SCRIPT_LINEAR_A = RB_TAG('L', 'i', 'n', 'a'),
    /*7.0*/ RB_SCRIPT_MAHAJANI = RB_TAG('M', 'a', 'h', 'j'),
    /*7.0*/ RB_SCRIPT_MANICHAEAN = RB_TAG('M', 'a', 'n', 'i'),
    /*7.0*/ RB_SCRIPT_MENDE_KIKAKUI = RB_TAG('M', 'e', 'n', 'd'),
    /*7.0*/ RB_SCRIPT_MODI = RB_TAG('M', 'o', 'd', 'i'),
    /*7.0*/ RB_SCRIPT_MRO = RB_TAG('M', 'r', 'o', 'o'),
    /*7.0*/ RB_SCRIPT_NABATAEAN = RB_TAG('N', 'b', 'a', 't'),
    /*7.0*/ RB_SCRIPT_OLD_NORTH_ARABIAN = RB_TAG('N', 'a', 'r', 'b'),
    /*7.0*/ RB_SCRIPT_OLD_PERMIC = RB_TAG('P', 'e', 'r', 'm'),
    /*7.0*/ RB_SCRIPT_PAHAWH_HMONG = RB_TAG('H', 'm', 'n', 'g'),
    /*7.0*/ RB_SCRIPT_PALMYRENE = RB_TAG('P', 'a', 'l', 'm'),
    /*7.0*/ RB_SCRIPT_PAU_CIN_HAU = RB_TAG('P', 'a', 'u', 'c'),
    /*7.0*/ RB_SCRIPT_PSALTER_PAHLAVI = RB_TAG('P', 'h', 'l', 'p'),
    /*7.0*/ RB_SCRIPT_SIDDHAM = RB_TAG('S', 'i', 'd', 'd'),
    /*7.0*/ RB_SCRIPT_TIRHUTA = RB_TAG('T', 'i', 'r', 'h'),
    /*7.0*/ RB_SCRIPT_WARANG_CITI = RB_TAG('W', 'a', 'r', 'a'),

    /*8.0*/ RB_SCRIPT_AHOM = RB_TAG('A', 'h', 'o', 'm'),
    /*8.0*/ RB_SCRIPT_ANATOLIAN_HIEROGLYPHS = RB_TAG('H', 'l', 'u', 'w'),
    /*8.0*/ RB_SCRIPT_HATRAN = RB_TAG('H', 'a', 't', 'r'),
    /*8.0*/ RB_SCRIPT_MULTANI = RB_TAG('M', 'u', 'l', 't'),
    /*8.0*/ RB_SCRIPT_OLD_HUNGARIAN = RB_TAG('H', 'u', 'n', 'g'),
    /*8.0*/ RB_SCRIPT_SIGNWRITING = RB_TAG('S', 'g', 'n', 'w'),

    /*
     * Since 1.3.0
     */
    /*9.0*/ RB_SCRIPT_ADLAM = RB_TAG('A', 'd', 'l', 'm'),
    /*9.0*/ RB_SCRIPT_BHAIKSUKI = RB_TAG('B', 'h', 'k', 's'),
    /*9.0*/ RB_SCRIPT_MARCHEN = RB_TAG('M', 'a', 'r', 'c'),
    /*9.0*/ RB_SCRIPT_OSAGE = RB_TAG('O', 's', 'g', 'e'),
    /*9.0*/ RB_SCRIPT_TANGUT = RB_TAG('T', 'a', 'n', 'g'),
    /*9.0*/ RB_SCRIPT_NEWA = RB_TAG('N', 'e', 'w', 'a'),

    /*
     * Since 1.6.0
     */
    /*10.0*/ RB_SCRIPT_MASARAM_GONDI = RB_TAG('G', 'o', 'n', 'm'),
    /*10.0*/ RB_SCRIPT_NUSHU = RB_TAG('N', 's', 'h', 'u'),
    /*10.0*/ RB_SCRIPT_SOYOMBO = RB_TAG('S', 'o', 'y', 'o'),
    /*10.0*/ RB_SCRIPT_ZANABAZAR_SQUARE = RB_TAG('Z', 'a', 'n', 'b'),

    /*
     * Since 1.8.0
     */
    /*11.0*/ RB_SCRIPT_DOGRA = RB_TAG('D', 'o', 'g', 'r'),
    /*11.0*/ RB_SCRIPT_GUNJALA_GONDI = RB_TAG('G', 'o', 'n', 'g'),
    /*11.0*/ RB_SCRIPT_HANIFI_ROHINGYA = RB_TAG('R', 'o', 'h', 'g'),
    /*11.0*/ RB_SCRIPT_MAKASAR = RB_TAG('M', 'a', 'k', 'a'),
    /*11.0*/ RB_SCRIPT_MEDEFAIDRIN = RB_TAG('M', 'e', 'd', 'f'),
    /*11.0*/ RB_SCRIPT_OLD_SOGDIAN = RB_TAG('S', 'o', 'g', 'o'),
    /*11.0*/ RB_SCRIPT_SOGDIAN = RB_TAG('S', 'o', 'g', 'd'),

    /*
     * Since 2.4.0
     */
    /*12.0*/ RB_SCRIPT_ELYMAIC = RB_TAG('E', 'l', 'y', 'm'),
    /*12.0*/ RB_SCRIPT_NANDINAGARI = RB_TAG('N', 'a', 'n', 'd'),
    /*12.0*/ RB_SCRIPT_NYIAKENG_PUACHUE_HMONG = RB_TAG('H', 'm', 'n', 'p'),
    /*12.0*/ RB_SCRIPT_WANCHO = RB_TAG('W', 'c', 'h', 'o'),

    /*
     * Since 2.6.7
     */
    /*13.0*/ RB_SCRIPT_CHORASMIAN = RB_TAG('C', 'h', 'r', 's'),
    /*13.0*/ RB_SCRIPT_DIVES_AKURU = RB_TAG('D', 'i', 'a', 'k'),
    /*13.0*/ RB_SCRIPT_KHITAN_SMALL_SCRIPT = RB_TAG('K', 'i', 't', 's'),
    /*13.0*/ RB_SCRIPT_YEZIDI = RB_TAG('Y', 'e', 'z', 'i'),

    /* No script set. */
    RB_SCRIPT_INVALID = RB_TAG_NONE,

    /* Dummy values to ensure any rb_tag_t value can be passed/stored as rb_script_t
     * without risking undefined behavior.  We have two, for historical reasons.
     * RB_TAG_MAX used to be unsigned, but that was invalid Ansi C, so was changed
     * to _RB_SCRIPT_MAX_VALUE to be equal to RB_TAG_MAX_SIGNED as well.
     *
     * See this thread for technicalities:
     *
     *   https://lists.freedesktop.org/archives/harfbuzz/2014-March/004150.html
     */
    _RB_SCRIPT_MAX_VALUE = RB_TAG_MAX_SIGNED,       /*< skip >*/
    _RB_SCRIPT_MAX_VALUE_SIGNED = RB_TAG_MAX_SIGNED /*< skip >*/

} rb_script_t;

/* Script functions */

extern "C" {
rb_direction_t rb_script_get_horizontal_direction(rb_script_t script);
}

typedef void (*rb_destroy_func_t)(void *user_data);

/* Font features and variations. */

/**
 * RB_FEATURE_GLOBAL_START
 *
 * Since: 2.0.0
 */
#define RB_FEATURE_GLOBAL_START 0
/**
 * RB_FEATURE_GLOBAL_END
 *
 * Since: 2.0.0
 */
#define RB_FEATURE_GLOBAL_END ((unsigned int)-1)

/**
 * rb_feature_t:
 * @tag: a feature tag
 * @value: 0 disables the feature, non-zero (usually 1) enables the feature.
 * For features implemented as lookup type 3 (like 'salt') the @value is a one
 * based index into the alternates.
 * @start: the cluster to start applying this feature setting (inclusive).
 * @end: the cluster to end applying this feature setting (exclusive).
 *
 * The #rb_feature_t is the structure that holds information about requested
 * feature application. The feature will be applied with the given value to all
 * glyphs which are in clusters between @start (inclusive) and @end (exclusive).
 * Setting start to @RB_FEATURE_GLOBAL_START and end to @RB_FEATURE_GLOBAL_END
 * specifies that the feature always applies to the entire buffer.
 */
typedef struct rb_feature_t
{
    rb_tag_t tag;
    uint32_t value;
    unsigned int start;
    unsigned int end;
} rb_feature_t;

/**
 * rb_variation_t:
 *
 * Since: 1.4.2
 */
typedef struct rb_variation_t
{
    rb_tag_t tag;
    float value;
} rb_variation_t;

RB_END_DECLS

#endif /* RB_COMMON_H */
