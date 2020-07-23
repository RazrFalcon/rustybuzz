/*
 * Copyright © 2017  Google, Inc.
 * Copyright © 2018  Ebrahim Byagowi
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

#include "hb.hh"

#include "hb-aat-layout.hh"
#include "hb-aat-layout-ankr-table.hh"
#include "hb-aat-layout-feat-table.hh"
#include "hb-aat-layout-kerx-table.hh"
#include "hb-aat-layout-morx-table.hh"
#include "hb-aat-layout-trak-table.hh"

/*
 * rb_aat_apply_context_t
 */

/* Note: This context is used for kerning, even without AAT, hence the condition. */
AAT::rb_aat_apply_context_t::rb_aat_apply_context_t(const rb_ot_shape_plan_t *plan_,
                                                    rb_font_t *font_,
                                                    rb_buffer_t *buffer_,
                                                    rb_blob_t *blob)
    : plan(plan_)
    , font(font_)
    , face(rb_font_get_face(font))
    , buffer(buffer_)
    , sanitizer()
    , ankr_table(&Null(AAT::ankr))
    , lookup_index(0)
{
    sanitizer.init(blob);
    sanitizer.set_num_glyphs(face->get_num_glyphs());
    sanitizer.start_processing();
    sanitizer.set_max_ops(RB_SANITIZE_MAX_OPS_MAX);
}

AAT::rb_aat_apply_context_t::~rb_aat_apply_context_t()
{
    sanitizer.end_processing();
}

void AAT::rb_aat_apply_context_t::set_ankr_table(const AAT::ankr *ankr_table_)
{
    ankr_table = ankr_table_;
}

/**
 * SECTION:hb-aat-layout
 * @title: hb-aat-layout
 * @short_description: Apple Advanced Typography Layout
 * @include: hb-aat.h
 *
 * Functions for querying OpenType Layout features in the font face.
 **/

/* Table data courtesy of Apple.  Converted from mnemonics to integers
 * when moving to this file. */
static const rb_aat_feature_mapping_t feature_mappings[] = {
    {RB_TAG('a', 'f', 'r', 'c'),
     RB_AAT_LAYOUT_FEATURE_TYPE_FRACTIONS,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_VERTICAL_FRACTIONS,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_NO_FRACTIONS},
    {RB_TAG('c', '2', 'p', 'c'),
     RB_AAT_LAYOUT_FEATURE_TYPE_UPPER_CASE,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_UPPER_CASE_PETITE_CAPS,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_DEFAULT_UPPER_CASE},
    {RB_TAG('c', '2', 's', 'c'),
     RB_AAT_LAYOUT_FEATURE_TYPE_UPPER_CASE,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_UPPER_CASE_SMALL_CAPS,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_DEFAULT_UPPER_CASE},
    {RB_TAG('c', 'a', 'l', 't'),
     RB_AAT_LAYOUT_FEATURE_TYPE_CONTEXTUAL_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_CONTEXTUAL_ALTERNATES_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_CONTEXTUAL_ALTERNATES_OFF},
    {RB_TAG('c', 'a', 's', 'e'),
     RB_AAT_LAYOUT_FEATURE_TYPE_CASE_SENSITIVE_LAYOUT,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_CASE_SENSITIVE_LAYOUT_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_CASE_SENSITIVE_LAYOUT_OFF},
    {RB_TAG('c', 'l', 'i', 'g'),
     RB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_CONTEXTUAL_LIGATURES_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_CONTEXTUAL_LIGATURES_OFF},
    {RB_TAG('c', 'p', 's', 'p'),
     RB_AAT_LAYOUT_FEATURE_TYPE_CASE_SENSITIVE_LAYOUT,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_CASE_SENSITIVE_SPACING_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_CASE_SENSITIVE_SPACING_OFF},
    {RB_TAG('c', 's', 'w', 'h'),
     RB_AAT_LAYOUT_FEATURE_TYPE_CONTEXTUAL_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_CONTEXTUAL_SWASH_ALTERNATES_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_CONTEXTUAL_SWASH_ALTERNATES_OFF},
    {RB_TAG('d', 'l', 'i', 'g'),
     RB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_RARE_LIGATURES_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_RARE_LIGATURES_OFF},
    {RB_TAG('e', 'x', 'p', 't'),
     RB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_EXPERT_CHARACTERS,
     (rb_aat_layout_feature_selector_t)16},
    {RB_TAG('f', 'r', 'a', 'c'),
     RB_AAT_LAYOUT_FEATURE_TYPE_FRACTIONS,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_DIAGONAL_FRACTIONS,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_NO_FRACTIONS},
    {RB_TAG('f', 'w', 'i', 'd'),
     RB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_MONOSPACED_TEXT,
     (rb_aat_layout_feature_selector_t)7},
    {RB_TAG('h', 'a', 'l', 't'),
     RB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_ALT_HALF_WIDTH_TEXT,
     (rb_aat_layout_feature_selector_t)7},
    {RB_TAG('h', 'i', 's', 't'),
     RB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_HISTORICAL_LIGATURES_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_HISTORICAL_LIGATURES_OFF},
    {RB_TAG('h', 'k', 'n', 'a'),
     RB_AAT_LAYOUT_FEATURE_TYPE_ALTERNATE_KANA,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_ALTERNATE_HORIZ_KANA_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_ALTERNATE_HORIZ_KANA_OFF},
    {RB_TAG('h', 'l', 'i', 'g'),
     RB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_HISTORICAL_LIGATURES_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_HISTORICAL_LIGATURES_OFF},
    {RB_TAG('h', 'n', 'g', 'l'),
     RB_AAT_LAYOUT_FEATURE_TYPE_TRANSLITERATION,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_HANJA_TO_HANGUL,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_NO_TRANSLITERATION},
    {RB_TAG('h', 'o', 'j', 'o'),
     RB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_HOJO_CHARACTERS,
     (rb_aat_layout_feature_selector_t)16},
    {RB_TAG('h', 'w', 'i', 'd'),
     RB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_HALF_WIDTH_TEXT,
     (rb_aat_layout_feature_selector_t)7},
    {RB_TAG('i', 't', 'a', 'l'),
     RB_AAT_LAYOUT_FEATURE_TYPE_ITALIC_CJK_ROMAN,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_CJK_ITALIC_ROMAN_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_CJK_ITALIC_ROMAN_OFF},
    {RB_TAG('j', 'p', '0', '4'),
     RB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_JIS2004_CHARACTERS,
     (rb_aat_layout_feature_selector_t)16},
    {RB_TAG('j', 'p', '7', '8'),
     RB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_JIS1978_CHARACTERS,
     (rb_aat_layout_feature_selector_t)16},
    {RB_TAG('j', 'p', '8', '3'),
     RB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_JIS1983_CHARACTERS,
     (rb_aat_layout_feature_selector_t)16},
    {RB_TAG('j', 'p', '9', '0'),
     RB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_JIS1990_CHARACTERS,
     (rb_aat_layout_feature_selector_t)16},
    {RB_TAG('l', 'i', 'g', 'a'),
     RB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_COMMON_LIGATURES_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_COMMON_LIGATURES_OFF},
    {RB_TAG('l', 'n', 'u', 'm'),
     RB_AAT_LAYOUT_FEATURE_TYPE_NUMBER_CASE,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_UPPER_CASE_NUMBERS,
     (rb_aat_layout_feature_selector_t)2},
    {RB_TAG('m', 'g', 'r', 'k'),
     RB_AAT_LAYOUT_FEATURE_TYPE_MATHEMATICAL_EXTRAS,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_MATHEMATICAL_GREEK_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_MATHEMATICAL_GREEK_OFF},
    {RB_TAG('n', 'l', 'c', 'k'),
     RB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_NLCCHARACTERS,
     (rb_aat_layout_feature_selector_t)16},
    {RB_TAG('o', 'n', 'u', 'm'),
     RB_AAT_LAYOUT_FEATURE_TYPE_NUMBER_CASE,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_LOWER_CASE_NUMBERS,
     (rb_aat_layout_feature_selector_t)2},
    {RB_TAG('o', 'r', 'd', 'n'),
     RB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_POSITION,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_ORDINALS,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_NORMAL_POSITION},
    {RB_TAG('p', 'a', 'l', 't'),
     RB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_ALT_PROPORTIONAL_TEXT,
     (rb_aat_layout_feature_selector_t)7},
    {RB_TAG('p', 'c', 'a', 'p'),
     RB_AAT_LAYOUT_FEATURE_TYPE_LOWER_CASE,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_LOWER_CASE_PETITE_CAPS,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_DEFAULT_LOWER_CASE},
    {RB_TAG('p', 'k', 'n', 'a'),
     RB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_PROPORTIONAL_TEXT,
     (rb_aat_layout_feature_selector_t)7},
    {RB_TAG('p', 'n', 'u', 'm'),
     RB_AAT_LAYOUT_FEATURE_TYPE_NUMBER_SPACING,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_PROPORTIONAL_NUMBERS,
     (rb_aat_layout_feature_selector_t)4},
    {RB_TAG('p', 'w', 'i', 'd'),
     RB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_PROPORTIONAL_TEXT,
     (rb_aat_layout_feature_selector_t)7},
    {RB_TAG('q', 'w', 'i', 'd'),
     RB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_QUARTER_WIDTH_TEXT,
     (rb_aat_layout_feature_selector_t)7},
    {RB_TAG('r', 'u', 'b', 'y'),
     RB_AAT_LAYOUT_FEATURE_TYPE_RUBY_KANA,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_RUBY_KANA_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_RUBY_KANA_OFF},
    {RB_TAG('s', 'i', 'n', 'f'),
     RB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_POSITION,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_SCIENTIFIC_INFERIORS,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_NORMAL_POSITION},
    {RB_TAG('s', 'm', 'c', 'p'),
     RB_AAT_LAYOUT_FEATURE_TYPE_LOWER_CASE,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_LOWER_CASE_SMALL_CAPS,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_DEFAULT_LOWER_CASE},
    {RB_TAG('s', 'm', 'p', 'l'),
     RB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_SIMPLIFIED_CHARACTERS,
     (rb_aat_layout_feature_selector_t)16},
    {RB_TAG('s', 's', '0', '1'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_ONE_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_ONE_OFF},
    {RB_TAG('s', 's', '0', '2'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_TWO_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_TWO_OFF},
    {RB_TAG('s', 's', '0', '3'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_THREE_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_THREE_OFF},
    {RB_TAG('s', 's', '0', '4'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_FOUR_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_FOUR_OFF},
    {RB_TAG('s', 's', '0', '5'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_FIVE_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_FIVE_OFF},
    {RB_TAG('s', 's', '0', '6'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_SIX_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_SIX_OFF},
    {RB_TAG('s', 's', '0', '7'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_SEVEN_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_SEVEN_OFF},
    {RB_TAG('s', 's', '0', '8'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_EIGHT_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_EIGHT_OFF},
    {RB_TAG('s', 's', '0', '9'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_NINE_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_NINE_OFF},
    {RB_TAG('s', 's', '1', '0'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_TEN_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_TEN_OFF},
    {RB_TAG('s', 's', '1', '1'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_ELEVEN_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_ELEVEN_OFF},
    {RB_TAG('s', 's', '1', '2'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_TWELVE_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_TWELVE_OFF},
    {RB_TAG('s', 's', '1', '3'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_THIRTEEN_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_THIRTEEN_OFF},
    {RB_TAG('s', 's', '1', '4'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_FOURTEEN_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_FOURTEEN_OFF},
    {RB_TAG('s', 's', '1', '5'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_FIFTEEN_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_FIFTEEN_OFF},
    {RB_TAG('s', 's', '1', '6'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_SIXTEEN_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_SIXTEEN_OFF},
    {RB_TAG('s', 's', '1', '7'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_SEVENTEEN_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_SEVENTEEN_OFF},
    {RB_TAG('s', 's', '1', '8'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_EIGHTEEN_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_EIGHTEEN_OFF},
    {RB_TAG('s', 's', '1', '9'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_NINETEEN_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_NINETEEN_OFF},
    {RB_TAG('s', 's', '2', '0'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_TWENTY_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_TWENTY_OFF},
    {RB_TAG('s', 'u', 'b', 's'),
     RB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_POSITION,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_INFERIORS,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_NORMAL_POSITION},
    {RB_TAG('s', 'u', 'p', 's'),
     RB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_POSITION,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_SUPERIORS,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_NORMAL_POSITION},
    {RB_TAG('s', 'w', 's', 'h'),
     RB_AAT_LAYOUT_FEATURE_TYPE_CONTEXTUAL_ALTERNATIVES,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_SWASH_ALTERNATES_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_SWASH_ALTERNATES_OFF},
    {RB_TAG('t', 'i', 't', 'l'),
     RB_AAT_LAYOUT_FEATURE_TYPE_STYLE_OPTIONS,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_TITLING_CAPS,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_NO_STYLE_OPTIONS},
    {RB_TAG('t', 'n', 'a', 'm'),
     RB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_TRADITIONAL_NAMES_CHARACTERS,
     (rb_aat_layout_feature_selector_t)16},
    {RB_TAG('t', 'n', 'u', 'm'),
     RB_AAT_LAYOUT_FEATURE_TYPE_NUMBER_SPACING,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_MONOSPACED_NUMBERS,
     (rb_aat_layout_feature_selector_t)4},
    {RB_TAG('t', 'r', 'a', 'd'),
     RB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_TRADITIONAL_CHARACTERS,
     (rb_aat_layout_feature_selector_t)16},
    {RB_TAG('t', 'w', 'i', 'd'),
     RB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_THIRD_WIDTH_TEXT,
     (rb_aat_layout_feature_selector_t)7},
    {RB_TAG('u', 'n', 'i', 'c'),
     RB_AAT_LAYOUT_FEATURE_TYPE_LETTER_CASE,
     (rb_aat_layout_feature_selector_t)14,
     (rb_aat_layout_feature_selector_t)15},
    {RB_TAG('v', 'a', 'l', 't'),
     RB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_ALT_PROPORTIONAL_TEXT,
     (rb_aat_layout_feature_selector_t)7},
    {RB_TAG('v', 'e', 'r', 't'),
     RB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_SUBSTITUTION,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_SUBSTITUTE_VERTICAL_FORMS_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_SUBSTITUTE_VERTICAL_FORMS_OFF},
    {RB_TAG('v', 'h', 'a', 'l'),
     RB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_ALT_HALF_WIDTH_TEXT,
     (rb_aat_layout_feature_selector_t)7},
    {RB_TAG('v', 'k', 'n', 'a'),
     RB_AAT_LAYOUT_FEATURE_TYPE_ALTERNATE_KANA,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_ALTERNATE_VERT_KANA_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_ALTERNATE_VERT_KANA_OFF},
    {RB_TAG('v', 'p', 'a', 'l'),
     RB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_ALT_PROPORTIONAL_TEXT,
     (rb_aat_layout_feature_selector_t)7},
    {RB_TAG('v', 'r', 't', '2'),
     RB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_SUBSTITUTION,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_SUBSTITUTE_VERTICAL_FORMS_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_SUBSTITUTE_VERTICAL_FORMS_OFF},
    {RB_TAG('z', 'e', 'r', 'o'),
     RB_AAT_LAYOUT_FEATURE_TYPE_TYPOGRAPHIC_EXTRAS,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_SLASHED_ZERO_ON,
     RB_AAT_LAYOUT_FEATURE_SELECTOR_SLASHED_ZERO_OFF},
};

const rb_aat_feature_mapping_t *rb_aat_layout_find_feature_mapping(rb_tag_t tag)
{
    return rb_sorted_array(feature_mappings).bsearch(tag);
}

/*
 * mort/morx/kerx/trak
 */

void rb_aat_layout_compile_map(const rb_aat_map_builder_t *mapper, rb_aat_map_t *map)
{
    const AAT::morx &morx = *mapper->face->table.morx;
    if (morx.has_data()) {
        morx.compile_flags(mapper, map);
        return;
    }

    const AAT::mort &mort = *mapper->face->table.mort;
    if (mort.has_data()) {
        mort.compile_flags(mapper, map);
        return;
    }
}

/*
 * rb_aat_layout_has_substitution:
 * @face:
 *
 * Returns:
 * Since: 2.3.0
 */
rb_bool_t rb_aat_layout_has_substitution(rb_face_t *face)
{
    return face->table.morx->has_data() || face->table.mort->has_data();
}

void rb_aat_layout_substitute(const rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer)
{
    rb_blob_t *morx_blob = rb_font_get_face(font)->table.morx.get_blob();
    const AAT::morx &morx = *morx_blob->as<AAT::morx>();
    if (morx.has_data()) {
        AAT::rb_aat_apply_context_t c(plan, font, buffer, morx_blob);
        morx.apply(&c);
        return;
    }

    rb_blob_t *mort_blob = rb_font_get_face(font)->table.mort.get_blob();
    const AAT::mort &mort = *mort_blob->as<AAT::mort>();
    if (mort.has_data()) {
        AAT::rb_aat_apply_context_t c(plan, font, buffer, mort_blob);
        mort.apply(&c);
        return;
    }
}

void rb_aat_layout_zero_width_deleted_glyphs(rb_buffer_t *buffer)
{
    unsigned int count = rb_buffer_get_length(buffer);
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);
    rb_glyph_position_t *pos = rb_buffer_get_glyph_positions(buffer);
    for (unsigned int i = 0; i < count; i++)
        if (unlikely(info[i].codepoint == AAT::DELETED_GLYPH))
            pos[i].x_advance = pos[i].y_advance = pos[i].x_offset = pos[i].y_offset = 0;
}

static bool is_deleted_glyph(const rb_glyph_info_t *info)
{
    return info->codepoint == AAT::DELETED_GLYPH;
}

void rb_aat_layout_remove_deleted_glyphs(rb_buffer_t *buffer)
{
    rb_ot_layout_delete_glyphs_inplace(buffer, is_deleted_glyph);
}

/*
 * rb_aat_layout_has_positioning:
 * @face:
 *
 * Returns:
 * Since: 2.3.0
 */
rb_bool_t rb_aat_layout_has_positioning(rb_face_t *face)
{
    return face->table.kerx->has_data();
}

void rb_aat_layout_position(const rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer)
{
    rb_blob_t *kerx_blob = rb_font_get_face(font)->table.kerx.get_blob();
    const AAT::kerx &kerx = *kerx_blob->as<AAT::kerx>();

    AAT::rb_aat_apply_context_t c(plan, font, buffer, kerx_blob);
    c.set_ankr_table(rb_font_get_face(font)->table.ankr.get());
    kerx.apply(&c);
}

/*
 * rb_aat_layout_has_tracking:
 * @face:
 *
 * Returns:
 * Since: 2.3.0
 */
rb_bool_t rb_aat_layout_has_tracking(rb_face_t *face)
{
    return face->table.trak->has_data();
}

void rb_aat_layout_track(const rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer)
{
    const AAT::trak &trak = *rb_font_get_face(font)->table.trak;

    AAT::rb_aat_apply_context_t c(plan, font, buffer);
    trak.apply(&c);
}

/**
 * rb_aat_layout_get_feature_types:
 * @face: a face object
 * @start_offset: iteration's start offset
 * @feature_count:(inout) (allow-none): buffer size as input, filled size as output
 * @features: (out caller-allocates) (array length=feature_count): features buffer
 *
 * Return value: Number of all available feature types.
 *
 * Since: 2.2.0
 */
unsigned int rb_aat_layout_get_feature_types(rb_face_t *face,
                                             unsigned int start_offset,
                                             unsigned int *feature_count, /* IN/OUT.  May be NULL. */
                                             rb_aat_layout_feature_type_t *features /* OUT.     May be NULL. */)
{
    return face->table.feat->get_feature_types(start_offset, feature_count, features);
}

/**
 * rb_aat_layout_feature_type_get_name_id:
 * @face: a face object
 * @feature_type: feature id
 *
 * Return value: Name ID index
 *
 * Since: 2.2.0
 */
rb_ot_name_id_t rb_aat_layout_feature_type_get_name_id(rb_face_t *face, rb_aat_layout_feature_type_t feature_type)
{
    return face->table.feat->get_feature_name_id(feature_type);
}

/**
 * rb_aat_layout_feature_type_get_selectors:
 * @face:    a face object
 * @feature_type: feature id
 * @start_offset:    iteration's start offset
 * @selector_count: (inout) (allow-none): buffer size as input, filled size as output
 * @selectors: (out caller-allocates) (array length=selector_count): settings buffer
 * @default_index: (out) (allow-none): index of default selector if any
 *
 * If upon return, @default_index is set to #RB_AAT_LAYOUT_NO_SELECTOR_INDEX, then
 * the feature type is non-exclusive.  Otherwise, @default_index is the index of
 * the selector that is selected by default.
 *
 * Return value: Number of all available feature selectors.
 *
 * Since: 2.2.0
 */
unsigned int rb_aat_layout_feature_type_get_selector_infos(
    rb_face_t *face,
    rb_aat_layout_feature_type_t feature_type,
    unsigned int start_offset,
    unsigned int *selector_count,                     /* IN/OUT.  May be NULL. */
    rb_aat_layout_feature_selector_info_t *selectors, /* OUT.     May be NULL. */
    unsigned int *default_index /* OUT.     May be NULL. */)
{
    return face->table.feat->get_selector_infos(feature_type, start_offset, selector_count, selectors, default_index);
}
