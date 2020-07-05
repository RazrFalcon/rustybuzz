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

#include "hb.hh"

#include "hb-ucd-table.hh"

/* See hb-unicode.hh for details. */
static const uint8_t _hb_modified_combining_class[256] = {
    0, /* HB_UNICODE_COMBINING_CLASS_NOT_REORDERED */
    1, /* HB_UNICODE_COMBINING_CLASS_OVERLAY */
    2,
    3,
    4,
    5,
    6,
    7, /* HB_UNICODE_COMBINING_CLASS_NUKTA */
    8, /* HB_UNICODE_COMBINING_CLASS_KANA_VOICING */
    9, /* HB_UNICODE_COMBINING_CLASS_VIRAMA */

    /* Hebrew */
    HB_MODIFIED_COMBINING_CLASS_CCC10,
    HB_MODIFIED_COMBINING_CLASS_CCC11,
    HB_MODIFIED_COMBINING_CLASS_CCC12,
    HB_MODIFIED_COMBINING_CLASS_CCC13,
    HB_MODIFIED_COMBINING_CLASS_CCC14,
    HB_MODIFIED_COMBINING_CLASS_CCC15,
    HB_MODIFIED_COMBINING_CLASS_CCC16,
    HB_MODIFIED_COMBINING_CLASS_CCC17,
    HB_MODIFIED_COMBINING_CLASS_CCC18,
    HB_MODIFIED_COMBINING_CLASS_CCC19,
    HB_MODIFIED_COMBINING_CLASS_CCC20,
    HB_MODIFIED_COMBINING_CLASS_CCC21,
    HB_MODIFIED_COMBINING_CLASS_CCC22,
    HB_MODIFIED_COMBINING_CLASS_CCC23,
    HB_MODIFIED_COMBINING_CLASS_CCC24,
    HB_MODIFIED_COMBINING_CLASS_CCC25,
    HB_MODIFIED_COMBINING_CLASS_CCC26,

    /* Arabic */
    HB_MODIFIED_COMBINING_CLASS_CCC27,
    HB_MODIFIED_COMBINING_CLASS_CCC28,
    HB_MODIFIED_COMBINING_CLASS_CCC29,
    HB_MODIFIED_COMBINING_CLASS_CCC30,
    HB_MODIFIED_COMBINING_CLASS_CCC31,
    HB_MODIFIED_COMBINING_CLASS_CCC32,
    HB_MODIFIED_COMBINING_CLASS_CCC33,
    HB_MODIFIED_COMBINING_CLASS_CCC34,
    HB_MODIFIED_COMBINING_CLASS_CCC35,

    /* Syriac */
    HB_MODIFIED_COMBINING_CLASS_CCC36,

    37,
    38,
    39,
    40,
    41,
    42,
    43,
    44,
    45,
    46,
    47,
    48,
    49,
    50,
    51,
    52,
    53,
    54,
    55,
    56,
    57,
    58,
    59,
    60,
    61,
    62,
    63,
    64,
    65,
    66,
    67,
    68,
    69,
    70,
    71,
    72,
    73,
    74,
    75,
    76,
    77,
    78,
    79,
    80,
    81,
    82,
    83,

    /* Telugu */
    HB_MODIFIED_COMBINING_CLASS_CCC84,
    85,
    86,
    87,
    88,
    89,
    90,
    HB_MODIFIED_COMBINING_CLASS_CCC91,
    92,
    93,
    94,
    95,
    96,
    97,
    98,
    99,
    100,
    101,
    102,

    /* Thai */
    HB_MODIFIED_COMBINING_CLASS_CCC103,
    104,
    105,
    106,
    HB_MODIFIED_COMBINING_CLASS_CCC107,
    108,
    109,
    110,
    111,
    112,
    113,
    114,
    115,
    116,
    117,

    /* Lao */
    HB_MODIFIED_COMBINING_CLASS_CCC118,
    119,
    120,
    121,
    HB_MODIFIED_COMBINING_CLASS_CCC122,
    123,
    124,
    125,
    126,
    127,
    128,

    /* Tibetan */
    HB_MODIFIED_COMBINING_CLASS_CCC129,
    HB_MODIFIED_COMBINING_CLASS_CCC130,
    131,
    HB_MODIFIED_COMBINING_CLASS_CCC132,
    133,
    134,
    135,
    136,
    137,
    138,
    139,

    140,
    141,
    142,
    143,
    144,
    145,
    146,
    147,
    148,
    149,
    150,
    151,
    152,
    153,
    154,
    155,
    156,
    157,
    158,
    159,
    160,
    161,
    162,
    163,
    164,
    165,
    166,
    167,
    168,
    169,
    170,
    171,
    172,
    173,
    174,
    175,
    176,
    177,
    178,
    179,
    180,
    181,
    182,
    183,
    184,
    185,
    186,
    187,
    188,
    189,
    190,
    191,
    192,
    193,
    194,
    195,
    196,
    197,
    198,
    199,

    200, /* HB_UNICODE_COMBINING_CLASS_ATTACHED_BELOW_LEFT */
    201,
    202, /* HB_UNICODE_COMBINING_CLASS_ATTACHED_BELOW */
    203,
    204,
    205,
    206,
    207,
    208,
    209,
    210,
    211,
    212,
    213,
    214, /* HB_UNICODE_COMBINING_CLASS_ATTACHED_ABOVE */
    215,
    216, /* HB_UNICODE_COMBINING_CLASS_ATTACHED_ABOVE_RIGHT */
    217,
    218, /* HB_UNICODE_COMBINING_CLASS_BELOW_LEFT */
    219,
    220, /* HB_UNICODE_COMBINING_CLASS_BELOW */
    221,
    222, /* HB_UNICODE_COMBINING_CLASS_BELOW_RIGHT */
    223,
    224, /* HB_UNICODE_COMBINING_CLASS_LEFT */
    225,
    226, /* HB_UNICODE_COMBINING_CLASS_RIGHT */
    227,
    228, /* HB_UNICODE_COMBINING_CLASS_ABOVE_LEFT */
    229,
    230, /* HB_UNICODE_COMBINING_CLASS_ABOVE */
    231,
    232, /* HB_UNICODE_COMBINING_CLASS_ABOVE_RIGHT */
    233, /* HB_UNICODE_COMBINING_CLASS_DOUBLE_BELOW */
    234, /* HB_UNICODE_COMBINING_CLASS_DOUBLE_ABOVE */
    235,
    236,
    237,
    238,
    239,
    240, /* HB_UNICODE_COMBINING_CLASS_IOTA_SUBSCRIPT */
    241,
    242,
    243,
    244,
    245,
    246,
    247,
    248,
    249,
    250,
    251,
    252,
    253,
    254,
    255, /* HB_UNICODE_COMBINING_CLASS_INVALID */
};

/* Default_Ignorable codepoints:
 *
 * Note: While U+115F, U+1160, U+3164 and U+FFA0 are Default_Ignorable,
 * we do NOT want to hide them, as the way Uniscribe has implemented them
 * is with regular spacing glyphs, and that's the way fonts are made to work.
 * As such, we make exceptions for those four.
 * Also ignoring U+1BCA0..1BCA3. https://github.com/harfbuzz/harfbuzz/issues/503
 *
 * Unicode 7.0:
 * $ grep '; Default_Ignorable_Code_Point ' DerivedCoreProperties.txt | sed 's/;.*#/#/'
 * 00AD          # Cf       SOFT HYPHEN
 * 034F          # Mn       COMBINING GRAPHEME JOINER
 * 061C          # Cf       ARABIC LETTER MARK
 * 115F..1160    # Lo   [2] HANGUL CHOSEONG FILLER..HANGUL JUNGSEONG FILLER
 * 17B4..17B5    # Mn   [2] KHMER VOWEL INHERENT AQ..KHMER VOWEL INHERENT AA
 * 180B..180D    # Mn   [3] MONGOLIAN FREE VARIATION SELECTOR ONE..MONGOLIAN FREE VARIATION SELECTOR THREE
 * 180E          # Cf       MONGOLIAN VOWEL SEPARATOR
 * 200B..200F    # Cf   [5] ZERO WIDTH SPACE..RIGHT-TO-LEFT MARK
 * 202A..202E    # Cf   [5] LEFT-TO-RIGHT EMBEDDING..RIGHT-TO-LEFT OVERRIDE
 * 2060..2064    # Cf   [5] WORD JOINER..INVISIBLE PLUS
 * 2065          # Cn       <reserved-2065>
 * 2066..206F    # Cf  [10] LEFT-TO-RIGHT ISOLATE..NOMINAL DIGIT SHAPES
 * 3164          # Lo       HANGUL FILLER
 * FE00..FE0F    # Mn  [16] VARIATION SELECTOR-1..VARIATION SELECTOR-16
 * FEFF          # Cf       ZERO WIDTH NO-BREAK SPACE
 * FFA0          # Lo       HALFWIDTH HANGUL FILLER
 * FFF0..FFF8    # Cn   [9] <reserved-FFF0>..<reserved-FFF8>
 * 1BCA0..1BCA3  # Cf   [4] SHORTHAND FORMAT LETTER OVERLAP..SHORTHAND FORMAT UP STEP
 * 1D173..1D17A  # Cf   [8] MUSICAL SYMBOL BEGIN BEAM..MUSICAL SYMBOL END PHRASE
 * E0000         # Cn       <reserved-E0000>
 * E0001         # Cf       LANGUAGE TAG
 * E0002..E001F  # Cn  [30] <reserved-E0002>..<reserved-E001F>
 * E0020..E007F  # Cf  [96] TAG SPACE..CANCEL TAG
 * E0080..E00FF  # Cn [128] <reserved-E0080>..<reserved-E00FF>
 * E0100..E01EF  # Mn [240] VARIATION SELECTOR-17..VARIATION SELECTOR-256
 * E01F0..E0FFF  # Cn [3600] <reserved-E01F0>..<reserved-E0FFF>
 */
hb_bool_t hb_ucd_is_default_ignorable(hb_codepoint_t cp)
{
    hb_codepoint_t plane = cp >> 16;
    if (likely(plane == 0)) {
        /* BMP */
        hb_codepoint_t page = cp >> 8;
        switch (page) {
        case 0x00:
            return unlikely(cp == 0x00ADu);
        case 0x03:
            return unlikely(cp == 0x034Fu);
        case 0x06:
            return unlikely(cp == 0x061Cu);
        case 0x17:
            return hb_in_range<hb_codepoint_t>(cp, 0x17B4u, 0x17B5u);
        case 0x18:
            return hb_in_range<hb_codepoint_t>(cp, 0x180Bu, 0x180Eu);
        case 0x20:
            return hb_in_ranges<hb_codepoint_t>(cp, 0x200Bu, 0x200Fu, 0x202Au, 0x202Eu, 0x2060u, 0x206Fu);
        case 0xFE:
            return hb_in_range<hb_codepoint_t>(cp, 0xFE00u, 0xFE0Fu) || cp == 0xFEFFu;
        case 0xFF:
            return hb_in_range<hb_codepoint_t>(cp, 0xFFF0u, 0xFFF8u);
        default:
            return false;
        }
    } else {
        /* Other planes */
        switch (plane) {
        case 0x01:
            return hb_in_range<hb_codepoint_t>(cp, 0x1D173u, 0x1D17Au);
        case 0x0E:
            return hb_in_range<hb_codepoint_t>(cp, 0xE0000u, 0xE0FFFu);
        default:
            return false;
        }
    }
}

hb_script_t hb_ucd_script(hb_codepoint_t cp)
{
    return _hb_ucd_sc_map[_hb_ucd_sc(cp)];
}

hb_unicode_combining_class_t hb_ucd_combining_class(hb_codepoint_t cp)
{
    return (hb_unicode_combining_class_t)_hb_ucd_ccc(cp);
}

hb_unicode_general_category_t hb_ucd_general_category(hb_codepoint_t cp)
{
    return (hb_unicode_general_category_t)_hb_ucd_gc(cp);
}

hb_codepoint_t hb_ucd_mirroring(hb_codepoint_t cp)
{
    return cp + _hb_ucd_bmg(cp);
}

unsigned int hb_ucd_modified_combining_class(hb_codepoint_t u)
{
    /* XXX This hack belongs to the USE shaper (for Tai Tham):
     * Reorder SAKOT to ensure it comes after any tone marks. */
    if (unlikely(u == 0x1A60u))
        return 254;

    /* XXX This hack belongs to the Tibetan shaper:
     * Reorder PADMA to ensure it comes after any vowel marks. */
    if (unlikely(u == 0x0FC6u))
        return 254;
    /* Reorder TSA -PHRU to reorder before U+0F74 */
    if (unlikely(u == 0x0F39u))
        return 127;

    return _hb_modified_combining_class[hb_ucd_combining_class(u)];
}

#define SBASE 0xAC00u
#define LBASE 0x1100u
#define VBASE 0x1161u
#define TBASE 0x11A7u
#define SCOUNT 11172u
#define LCOUNT 19u
#define VCOUNT 21u
#define TCOUNT 28u
#define NCOUNT (VCOUNT * TCOUNT)

static inline bool _hb_ucd_decompose_hangul(hb_codepoint_t ab, hb_codepoint_t *a, hb_codepoint_t *b)
{
    unsigned si = ab - SBASE;

    if (si >= SCOUNT)
        return false;

    if (si % TCOUNT) {
        /* LV,T */
        *a = SBASE + (si / TCOUNT) * TCOUNT;
        *b = TBASE + (si % TCOUNT);
        return true;
    } else {
        /* L,V */
        *a = LBASE + (si / NCOUNT);
        *b = VBASE + (si % NCOUNT) / TCOUNT;
        return true;
    }
}

static inline bool _hb_ucd_compose_hangul(hb_codepoint_t a, hb_codepoint_t b, hb_codepoint_t *ab)
{
    if (a >= SBASE && a < (SBASE + SCOUNT) && b > TBASE && b < (TBASE + TCOUNT) && !((a - SBASE) % TCOUNT)) {
        /* LV,T */
        *ab = a + (b - TBASE);
        return true;
    } else if (a >= LBASE && a < (LBASE + LCOUNT) && b >= VBASE && b < (VBASE + VCOUNT)) {
        /* L,V */
        int li = a - LBASE;
        int vi = b - VBASE;
        *ab = SBASE + li * NCOUNT + vi * TCOUNT;
        return true;
    } else
        return false;
}

static int _cmp_pair(const void *_key, const void *_item)
{
    uint64_t &a = *(uint64_t *)_key;
    uint64_t b = (*(uint64_t *)_item) & HB_CODEPOINT_ENCODE3(0x1FFFFFu, 0x1FFFFFu, 0);

    return a < b ? -1 : a > b ? +1 : 0;
}
static int _cmp_pair_11_7_14(const void *_key, const void *_item)
{
    uint32_t &a = *(uint32_t *)_key;
    uint32_t b = (*(uint32_t *)_item) & HB_CODEPOINT_ENCODE3_11_7_14(0x1FFFFFu, 0x1FFFFFu, 0);

    return a < b ? -1 : a > b ? +1 : 0;
}

hb_bool_t hb_ucd_compose(hb_codepoint_t a, hb_codepoint_t b, hb_codepoint_t *ab)
{
    *ab = 0;
    if (unlikely(!a || !b))
        return false;

    if (_hb_ucd_compose_hangul(a, b, ab))
        return true;

    hb_codepoint_t u = 0;

    if ((a & 0xFFFFF800u) == 0x0000u && (b & 0xFFFFFF80) == 0x0300u) {
        uint32_t k = HB_CODEPOINT_ENCODE3_11_7_14(a, b, 0);
        const uint32_t *v = hb_bsearch(
            k, _hb_ucd_dm2_u32_map, ARRAY_LENGTH(_hb_ucd_dm2_u32_map), sizeof(*_hb_ucd_dm2_u32_map), _cmp_pair_11_7_14);
        if (likely(!v))
            return false;
        u = HB_CODEPOINT_DECODE3_11_7_14_3(*v);
    } else {
        uint64_t k = HB_CODEPOINT_ENCODE3(a, b, 0);
        const uint64_t *v = hb_bsearch(
            k, _hb_ucd_dm2_u64_map, ARRAY_LENGTH(_hb_ucd_dm2_u64_map), sizeof(*_hb_ucd_dm2_u64_map), _cmp_pair);
        if (likely(!v))
            return false;
        u = HB_CODEPOINT_DECODE3_3(*v);
    }

    if (unlikely(!u))
        return false;
    *ab = u;
    return true;
}

hb_bool_t hb_ucd_decompose(hb_codepoint_t ab, hb_codepoint_t *a, hb_codepoint_t *b)
{
    *a = ab;
    *b = 0;

    if (_hb_ucd_decompose_hangul(ab, a, b))
        return true;

    unsigned i = _hb_ucd_dm(ab);

    if (likely(!i))
        return false;
    i--;

    if (i < ARRAY_LENGTH(_hb_ucd_dm1_p0_map) + ARRAY_LENGTH(_hb_ucd_dm1_p2_map)) {
        if (i < ARRAY_LENGTH(_hb_ucd_dm1_p0_map))
            *a = _hb_ucd_dm1_p0_map[i];
        else {
            i -= ARRAY_LENGTH(_hb_ucd_dm1_p0_map);
            *a = 0x20000 | _hb_ucd_dm1_p2_map[i];
        }
        *b = 0;
        return true;
    }
    i -= ARRAY_LENGTH(_hb_ucd_dm1_p0_map) + ARRAY_LENGTH(_hb_ucd_dm1_p2_map);

    if (i < ARRAY_LENGTH(_hb_ucd_dm2_u32_map)) {
        uint32_t v = _hb_ucd_dm2_u32_map[i];
        *a = HB_CODEPOINT_DECODE3_11_7_14_1(v);
        *b = HB_CODEPOINT_DECODE3_11_7_14_2(v);
        return true;
    }
    i -= ARRAY_LENGTH(_hb_ucd_dm2_u32_map);

    uint64_t v = _hb_ucd_dm2_u64_map[i];
    *a = HB_CODEPOINT_DECODE3_1(v);
    *b = HB_CODEPOINT_DECODE3_2(v);
    return true;
}

hb_space_t hb_ucd_space_fallback_type(hb_codepoint_t cp)
{
    switch (cp) {
    /* All GC=Zs chars that can use a fallback. */
    default:
        return HB_SPACE_NOT_SPACE; /* U+1680 OGHAM SPACE MARK */
    case 0x0020u:
        return HB_SPACE; /* U+0020 SPACE */
    case 0x00A0u:
        return HB_SPACE; /* U+00A0 NO-BREAK SPACE */
    case 0x2000u:
        return HB_SPACE_EM_2; /* U+2000 EN QUAD */
    case 0x2001u:
        return HB_SPACE_EM; /* U+2001 EM QUAD */
    case 0x2002u:
        return HB_SPACE_EM_2; /* U+2002 EN SPACE */
    case 0x2003u:
        return HB_SPACE_EM; /* U+2003 EM SPACE */
    case 0x2004u:
        return HB_SPACE_EM_3; /* U+2004 THREE-PER-EM SPACE */
    case 0x2005u:
        return HB_SPACE_EM_4; /* U+2005 FOUR-PER-EM SPACE */
    case 0x2006u:
        return HB_SPACE_EM_6; /* U+2006 SIX-PER-EM SPACE */
    case 0x2007u:
        return HB_SPACE_FIGURE; /* U+2007 FIGURE SPACE */
    case 0x2008u:
        return HB_SPACE_PUNCTUATION; /* U+2008 PUNCTUATION SPACE */
    case 0x2009u:
        return HB_SPACE_EM_5; /* U+2009 THIN SPACE */
    case 0x200Au:
        return HB_SPACE_EM_16; /* U+200A HAIR SPACE */
    case 0x202Fu:
        return HB_SPACE_NARROW; /* U+202F NARROW NO-BREAK SPACE */
    case 0x205Fu:
        return HB_SPACE_4_EM_18; /* U+205F MEDIUM MATHEMATICAL SPACE */
    case 0x3000u:
        return HB_SPACE_EM; /* U+3000 IDEOGRAPHIC SPACE */
    }
}

hb_bool_t hb_ucd_is_variation_selector(hb_codepoint_t cp)
{
    /* U+180B..180D MONGOLIAN FREE VARIATION SELECTORs are handled in the
     * Arabic shaper.  No need to match them here. */
    return unlikely(hb_in_ranges<hb_codepoint_t>(cp,
                                                 0xFE00u,
                                                 0xFE0Fu, /* VARIATION SELECTOR-1..16 */
                                                 0xE0100u,
                                                 0xE01EFu)); /* VARIATION SELECTOR-17..256 */
}

/*
 * Emoji
 */

#include "hb-unicode-emoji-table.hh"

hb_bool_t hb_unicode_is_emoji_extended_pictographic(hb_codepoint_t cp)
{
    return _hb_emoji_is_Extended_Pictographic(cp);
}
