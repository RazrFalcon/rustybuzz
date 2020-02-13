/*
 * Copyright © 2012  Google, Inc.
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
 * Google Author(s): Behdad Esfahbod
 */

#pragma once

#include "hb.hh"

#include "hb-ot-shape-complex.hh"


/* buffer var allocations */
#define indic_category() complex_var_u8_0() /* indic_category_t */
#define indic_position() complex_var_u8_1() /* indic_position_t */


#define INDIC_TABLE_ELEMENT_TYPE uint16_t

extern "C" {
    uint16_t rb_indic_get_categories(hb_codepoint_t u);
}

/* Cateories used in the OpenType spec:
 * https://docs.microsoft.com/en-us/typography/script-development/devanagari
 */
/* Note: This enum is duplicated in the -machine.rl source file.
 * Not sure how to avoid duplication. */
enum indic_category_t {
  OT_X = 0,
  OT_C = 1,
  OT_V = 2,
  OT_N = 3,
  OT_H = 4,
  OT_ZWNJ = 5,
  OT_ZWJ = 6,
  OT_M = 7,
  OT_SM = 8,
  /* OT_VD = 9, UNUSED; we use OT_A instead. */
  OT_A = 10,
  OT_PLACEHOLDER = 11,
  OT_DOTTEDCIRCLE = 12,
  OT_RS = 13, /* Register Shifter, used in Khmer OT spec. */
  OT_Coeng = 14, /* Khmer-style Virama. */
  OT_Repha = 15, /* Atomically-encoded logical or visual repha. */
  OT_Ra = 16,
  OT_CM = 17,  /* Consonant-Medial. */
  OT_Symbol = 18, /* Avagraha, etc that take marks (SM,A,VD). */
  OT_CS = 19,

  /* The following are used by Khmer & Myanmar shapers.  Defined
   * here for them to share. */
  OT_VAbv    = 26,
  OT_VBlw    = 27,
  OT_VPre    = 28,
  OT_VPst    = 29,
};

#define MEDIAL_FLAGS (FLAG (OT_CM))

/* Note:
 *
 * We treat Vowels and placeholders as if they were consonants.  This is safe because Vowels
 * cannot happen in a consonant syllable.  The plus side however is, we can call the
 * consonant syllable logic from the vowel syllable function and get it all right! */
#define CONSONANT_FLAGS (FLAG (OT_C) | FLAG (OT_CS) | FLAG (OT_Ra) | MEDIAL_FLAGS | FLAG (OT_V) | FLAG (OT_PLACEHOLDER) | FLAG (OT_DOTTEDCIRCLE))
#define JOINER_FLAGS (FLAG (OT_ZWJ) | FLAG (OT_ZWNJ))


/* Visual positions in a syllable from left to right. */
enum indic_position_t {
  POS_START = 0,

  POS_RA_TO_BECOME_REPH = 1,
  POS_PRE_M = 2,
  POS_PRE_C = 3,

  POS_BASE_C = 4,
  POS_AFTER_MAIN = 5,

  POS_ABOVE_C = 6,

  POS_BEFORE_SUB = 7,
  POS_BELOW_C = 8,
  POS_AFTER_SUB = 9,

  POS_BEFORE_POST = 10,
  POS_POST_C = 11,
  POS_AFTER_POST = 12,

  POS_FINAL_C = 13,
  POS_SMVD = 14,

  POS_END = 15
};

/* Categories used in IndicSyllabicCategory.txt from UCD. */
enum indic_syllabic_category_t {
  INDIC_SYLLABIC_CATEGORY_OTHER				= OT_X,

  INDIC_SYLLABIC_CATEGORY_AVAGRAHA			= OT_Symbol,
  INDIC_SYLLABIC_CATEGORY_BINDU				= OT_SM,
  INDIC_SYLLABIC_CATEGORY_BRAHMI_JOINING_NUMBER		= OT_PLACEHOLDER, /* Don't care. */
  INDIC_SYLLABIC_CATEGORY_CANTILLATION_MARK		= OT_A,
  INDIC_SYLLABIC_CATEGORY_CONSONANT			= OT_C,
  INDIC_SYLLABIC_CATEGORY_CONSONANT_DEAD		= OT_C,
  INDIC_SYLLABIC_CATEGORY_CONSONANT_FINAL		= OT_CM,
  INDIC_SYLLABIC_CATEGORY_CONSONANT_HEAD_LETTER		= OT_C,
  INDIC_SYLLABIC_CATEGORY_CONSONANT_KILLER		= OT_M, /* U+17CD only. */
  INDIC_SYLLABIC_CATEGORY_CONSONANT_MEDIAL		= OT_CM,
  INDIC_SYLLABIC_CATEGORY_CONSONANT_PLACEHOLDER		= OT_PLACEHOLDER,
  INDIC_SYLLABIC_CATEGORY_CONSONANT_PRECEDING_REPHA	= OT_Repha,
  INDIC_SYLLABIC_CATEGORY_CONSONANT_PREFIXED		= OT_X, /* Don't care. */
  INDIC_SYLLABIC_CATEGORY_CONSONANT_SUBJOINED		= OT_CM,
  INDIC_SYLLABIC_CATEGORY_CONSONANT_SUCCEEDING_REPHA	= OT_CM,
  INDIC_SYLLABIC_CATEGORY_CONSONANT_WITH_STACKER	= OT_CS,
  INDIC_SYLLABIC_CATEGORY_GEMINATION_MARK		= OT_SM, /* https://github.com/harfbuzz/harfbuzz/issues/552 */
  INDIC_SYLLABIC_CATEGORY_INVISIBLE_STACKER		= OT_Coeng,
  INDIC_SYLLABIC_CATEGORY_JOINER			= OT_ZWJ,
  INDIC_SYLLABIC_CATEGORY_MODIFYING_LETTER		= OT_X,
  INDIC_SYLLABIC_CATEGORY_NON_JOINER			= OT_ZWNJ,
  INDIC_SYLLABIC_CATEGORY_NUKTA				= OT_N,
  INDIC_SYLLABIC_CATEGORY_NUMBER			= OT_PLACEHOLDER,
  INDIC_SYLLABIC_CATEGORY_NUMBER_JOINER			= OT_PLACEHOLDER, /* Don't care. */
  INDIC_SYLLABIC_CATEGORY_PURE_KILLER			= OT_M, /* Is like a vowel matra. */
  INDIC_SYLLABIC_CATEGORY_REGISTER_SHIFTER		= OT_RS,
  INDIC_SYLLABIC_CATEGORY_SYLLABLE_MODIFIER		= OT_SM,
  INDIC_SYLLABIC_CATEGORY_TONE_LETTER			= OT_X,
  INDIC_SYLLABIC_CATEGORY_TONE_MARK			= OT_N,
  INDIC_SYLLABIC_CATEGORY_VIRAMA			= OT_H,
  INDIC_SYLLABIC_CATEGORY_VISARGA			= OT_SM,
  INDIC_SYLLABIC_CATEGORY_VOWEL				= OT_V,
  INDIC_SYLLABIC_CATEGORY_VOWEL_DEPENDENT		= OT_M,
  INDIC_SYLLABIC_CATEGORY_VOWEL_INDEPENDENT		= OT_V
};

/* Categories used in IndicSMatraCategory.txt from UCD */
enum indic_matra_category_t {
  INDIC_MATRA_CATEGORY_NOT_APPLICABLE			= POS_END,

  INDIC_MATRA_CATEGORY_LEFT				= POS_PRE_C,
  INDIC_MATRA_CATEGORY_TOP				= POS_ABOVE_C,
  INDIC_MATRA_CATEGORY_BOTTOM				= POS_BELOW_C,
  INDIC_MATRA_CATEGORY_RIGHT				= POS_POST_C,

  /* These should resolve to the position of the last part of the split sequence. */
  INDIC_MATRA_CATEGORY_BOTTOM_AND_RIGHT			= INDIC_MATRA_CATEGORY_RIGHT,
  INDIC_MATRA_CATEGORY_LEFT_AND_RIGHT			= INDIC_MATRA_CATEGORY_RIGHT,
  INDIC_MATRA_CATEGORY_TOP_AND_BOTTOM			= INDIC_MATRA_CATEGORY_BOTTOM,
  INDIC_MATRA_CATEGORY_TOP_AND_BOTTOM_AND_RIGHT		= INDIC_MATRA_CATEGORY_RIGHT,
  INDIC_MATRA_CATEGORY_TOP_AND_LEFT			= INDIC_MATRA_CATEGORY_TOP,
  INDIC_MATRA_CATEGORY_TOP_AND_LEFT_AND_RIGHT		= INDIC_MATRA_CATEGORY_RIGHT,
  INDIC_MATRA_CATEGORY_TOP_AND_RIGHT			= INDIC_MATRA_CATEGORY_RIGHT,

  INDIC_MATRA_CATEGORY_OVERSTRUCK			= POS_AFTER_MAIN,
  INDIC_MATRA_CATEGORY_VISUAL_ORDER_LEFT		= POS_PRE_M
};

static inline bool
is_one_of (const hb_glyph_info_t &info, unsigned int flags)
{
  /* If it ligated, all bets are off. */
  if (_hb_glyph_info_ligated (&info)) return false;
  return !!(FLAG_UNSAFE (info.indic_category()) & flags);
}

static inline bool
is_consonant (const hb_glyph_info_t &info)
{
  return is_one_of (info, CONSONANT_FLAGS);
}

struct hb_indic_would_substitute_feature_t
{
  void init (const rb_ot_map_t *map, hb_tag_t feature_tag, bool zero_context_)
  {
    zero_context = zero_context_;
    rb_ot_map_get_stage_lookups(map, 0/*GSUB*/,
                rb_ot_map_get_feature_stage(map, 0/*GSUB*/, feature_tag),
                &lookups, &count);
  }

  bool would_substitute (const hb_codepoint_t *glyphs,
             unsigned int          glyphs_count,
             hb_face_t            *face) const
  {
    for (unsigned int i = 0; i < count; i++)
      if (hb_ot_layout_lookup_would_substitute (face, lookups[i].index, glyphs, glyphs_count, zero_context))
        return true;
    return false;
  }

  private:
  const lookup_map_t *lookups;
  unsigned int count;
  bool zero_context;
};
