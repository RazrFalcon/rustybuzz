use crate::hb::aat_map::*;
use crate::hb::buffer::hb_buffer_t;
use crate::hb::hb_font_t;
use crate::hb::hb_tag_t;
use crate::hb::shape_plan::hb_ot_shape_plan_t;
use crate::hb::{aat_layout_kerx_table, aat_layout_morx_table, aat_layout_trak_table};

pub fn substitute(plan: &hb_ot_shape_plan_t, face: &hb_font_t, buffer: &mut hb_buffer_t) {
    aat_layout_morx_table::apply(plan, face, buffer);
}

pub fn position(plan: &hb_ot_shape_plan_t, face: &hb_font_t, buffer: &mut hb_buffer_t) {
    aat_layout_kerx_table::apply(plan, face, buffer);
}

pub fn track(plan: &hb_ot_shape_plan_t, face: &hb_font_t, buffer: &mut hb_buffer_t) {
    aat_layout_trak_table::apply(plan, face, buffer);
}

pub fn zero_width_deleted_glyphs(buffer: &mut hb_buffer_t) {
    for i in 0..buffer.len {
        if buffer.info[i].glyph_id == 0xFFFF {
            buffer.pos[i].x_advance = 0;
            buffer.pos[i].y_advance = 0;
            buffer.pos[i].x_offset = 0;
            buffer.pos[i].y_offset = 0;
        }
    }
}

pub fn remove_deleted_glyphs(buffer: &mut hb_buffer_t) {
    buffer.delete_glyphs_inplace(|info| info.glyph_id == 0xFFFF)
}

// FeatureType::Ligatures
pub const COMMON_LIGATURES_ON: u8 = 2;
pub const COMMON_LIGATURES_OFF: u8 = 3;
pub const RARE_LIGATURES_ON: u8 = 4;
pub const RARE_LIGATURES_OFF: u8 = 5;
pub const CONTEXTUAL_LIGATURES_ON: u8 = 18;
pub const CONTEXTUAL_LIGATURES_OFF: u8 = 19;
pub const HISTORICAL_LIGATURES_ON: u8 = 20;
pub const HISTORICAL_LIGATURES_OFF: u8 = 21;

// FeatureType::LetterCase
pub const SMALL_CAPS: u8 = 3; // deprecated

// FeatureType::VerticalSubstitution
pub const SUBSTITUTE_VERTICAL_FORMS_ON: u8 = 0;
pub const SUBSTITUTE_VERTICAL_FORMS_OFF: u8 = 1;

// FeatureType::NumberSpacing
pub const MONOSPACED_NUMBERS: u8 = 0;
pub const PROPORTIONAL_NUMBERS: u8 = 1;

// FeatureType::VerticalPosition
pub const NORMAL_POSITION: u8 = 0;
pub const SUPERIORS: u8 = 1;
pub const INFERIORS: u8 = 2;
pub const ORDINALS: u8 = 3;
pub const SCIENTIFIC_INFERIORS: u8 = 4;

// FeatureType::Fractions
pub const NO_FRACTIONS: u8 = 0;
pub const VERTICAL_FRACTIONS: u8 = 1;
pub const DIAGONAL_FRACTIONS: u8 = 2;

// FeatureType::TypographicExtras
pub const SLASHED_ZERO_ON: u8 = 4;
pub const SLASHED_ZERO_OFF: u8 = 5;

// FeatureType::MathematicalExtras
pub const MATHEMATICAL_GREEK_ON: u8 = 10;
pub const MATHEMATICAL_GREEK_OFF: u8 = 11;

// FeatureType::StyleOptions
pub const NO_STYLE_OPTIONS: u8 = 0;
pub const TITLING_CAPS: u8 = 4;

// FeatureType::CharacterShape
pub const TRADITIONAL_CHARACTERS: u8 = 0;
pub const SIMPLIFIED_CHARACTERS: u8 = 1;
pub const JIS1978_CHARACTERS: u8 = 2;
pub const JIS1983_CHARACTERS: u8 = 3;
pub const JIS1990_CHARACTERS: u8 = 4;
pub const EXPERT_CHARACTERS: u8 = 10;
pub const JIS2004_CHARACTERS: u8 = 11;
pub const HOJO_CHARACTERS: u8 = 12;
pub const NLCCHARACTERS: u8 = 13;
pub const TRADITIONAL_NAMES_CHARACTERS: u8 = 14;

// FeatureType::NumberCase
pub const LOWER_CASE_NUMBERS: u8 = 0;
pub const UPPER_CASE_NUMBERS: u8 = 1;

// FeatureType::TextSpacing
pub const PROPORTIONAL_TEXT: u8 = 0;
pub const MONOSPACED_TEXT: u8 = 1;
pub const HALF_WIDTH_TEXT: u8 = 2;
pub const THIRD_WIDTH_TEXT: u8 = 3;
pub const QUARTER_WIDTH_TEXT: u8 = 4;
pub const ALT_PROPORTIONAL_TEXT: u8 = 5;
pub const ALT_HALF_WIDTH_TEXT: u8 = 6;

// FeatureType::Transliteration
pub const NO_TRANSLITERATION: u8 = 0;
pub const HANJA_TO_HANGUL: u8 = 1;

// FeatureType::RubyKana
pub const RUBY_KANA_ON: u8 = 2;
pub const RUBY_KANA_OFF: u8 = 3;

// FeatureType::ItalicCjkRoman
pub const CJK_ITALIC_ROMAN_ON: u8 = 2;
pub const CJK_ITALIC_ROMAN_OFF: u8 = 3;

// FeatureType::CaseSensitiveLayout
pub const CASE_SENSITIVE_LAYOUT_ON: u8 = 0;
pub const CASE_SENSITIVE_LAYOUT_OFF: u8 = 1;
pub const CASE_SENSITIVE_SPACING_ON: u8 = 2;
pub const CASE_SENSITIVE_SPACING_OFF: u8 = 3;

// FeatureType::AlternateKana
pub const ALTERNATE_HORIZ_KANA_ON: u8 = 0;
pub const ALTERNATE_HORIZ_KANA_OFF: u8 = 1;
pub const ALTERNATE_VERT_KANA_ON: u8 = 2;
pub const ALTERNATE_VERT_KANA_OFF: u8 = 3;

// FeatureType::StylisticAlternatives
pub const STYLISTIC_ALT_ONE_ON: u8 = 2;
pub const STYLISTIC_ALT_ONE_OFF: u8 = 3;
pub const STYLISTIC_ALT_TWO_ON: u8 = 4;
pub const STYLISTIC_ALT_TWO_OFF: u8 = 5;
pub const STYLISTIC_ALT_THREE_ON: u8 = 6;
pub const STYLISTIC_ALT_THREE_OFF: u8 = 7;
pub const STYLISTIC_ALT_FOUR_ON: u8 = 8;
pub const STYLISTIC_ALT_FOUR_OFF: u8 = 9;
pub const STYLISTIC_ALT_FIVE_ON: u8 = 10;
pub const STYLISTIC_ALT_FIVE_OFF: u8 = 11;
pub const STYLISTIC_ALT_SIX_ON: u8 = 12;
pub const STYLISTIC_ALT_SIX_OFF: u8 = 13;
pub const STYLISTIC_ALT_SEVEN_ON: u8 = 14;
pub const STYLISTIC_ALT_SEVEN_OFF: u8 = 15;
pub const STYLISTIC_ALT_EIGHT_ON: u8 = 16;
pub const STYLISTIC_ALT_EIGHT_OFF: u8 = 17;
pub const STYLISTIC_ALT_NINE_ON: u8 = 18;
pub const STYLISTIC_ALT_NINE_OFF: u8 = 19;
pub const STYLISTIC_ALT_TEN_ON: u8 = 20;
pub const STYLISTIC_ALT_TEN_OFF: u8 = 21;
pub const STYLISTIC_ALT_ELEVEN_ON: u8 = 22;
pub const STYLISTIC_ALT_ELEVEN_OFF: u8 = 23;
pub const STYLISTIC_ALT_TWELVE_ON: u8 = 24;
pub const STYLISTIC_ALT_TWELVE_OFF: u8 = 25;
pub const STYLISTIC_ALT_THIRTEEN_ON: u8 = 26;
pub const STYLISTIC_ALT_THIRTEEN_OFF: u8 = 27;
pub const STYLISTIC_ALT_FOURTEEN_ON: u8 = 28;
pub const STYLISTIC_ALT_FOURTEEN_OFF: u8 = 29;
pub const STYLISTIC_ALT_FIFTEEN_ON: u8 = 30;
pub const STYLISTIC_ALT_FIFTEEN_OFF: u8 = 31;
pub const STYLISTIC_ALT_SIXTEEN_ON: u8 = 32;
pub const STYLISTIC_ALT_SIXTEEN_OFF: u8 = 33;
pub const STYLISTIC_ALT_SEVENTEEN_ON: u8 = 34;
pub const STYLISTIC_ALT_SEVENTEEN_OFF: u8 = 35;
pub const STYLISTIC_ALT_EIGHTEEN_ON: u8 = 36;
pub const STYLISTIC_ALT_EIGHTEEN_OFF: u8 = 37;
pub const STYLISTIC_ALT_NINETEEN_ON: u8 = 38;
pub const STYLISTIC_ALT_NINETEEN_OFF: u8 = 39;
pub const STYLISTIC_ALT_TWENTY_ON: u8 = 40;
pub const STYLISTIC_ALT_TWENTY_OFF: u8 = 41;

// FeatureType::ContextualAlternatives
pub const CONTEXTUAL_ALTERNATES_ON: u8 = 0;
pub const CONTEXTUAL_ALTERNATES_OFF: u8 = 1;
pub const SWASH_ALTERNATES_ON: u8 = 2;
pub const SWASH_ALTERNATES_OFF: u8 = 3;
pub const CONTEXTUAL_SWASH_ALTERNATES_ON: u8 = 4;
pub const CONTEXTUAL_SWASH_ALTERNATES_OFF: u8 = 5;

// FeatureType::LowerCase
pub const DEFAULT_LOWER_CASE: u8 = 0;
pub const LOWER_CASE_SMALL_CAPS: u8 = 1;
pub const LOWER_CASE_PETITE_CAPS: u8 = 2;

// FeatureType::UpperCase
pub const DEFAULT_UPPER_CASE: u8 = 0;
pub const UPPER_CASE_SMALL_CAPS: u8 = 1;
pub const UPPER_CASE_PETITE_CAPS: u8 = 2;

pub struct FeatureMapping {
    pub ot_feature_tag: hb_tag_t,
    pub aat_feature_type: FeatureType,
    pub selector_to_enable: u8,
    pub selector_to_disable: u8,
}

impl FeatureMapping {
    const fn new(
        ot_feature_tag: &[u8; 4],
        aat_feature_type: FeatureType,
        selector_to_enable: u8,
        selector_to_disable: u8,
    ) -> Self {
        FeatureMapping {
            ot_feature_tag: hb_tag_t::from_bytes(ot_feature_tag),
            aat_feature_type,
            selector_to_enable,
            selector_to_disable,
        }
    }
}

/// Mapping from OpenType feature tags to AAT feature names and selectors.
///
/// Table data courtesy of Apple.
/// Converted from mnemonics to integers when moving to this file.
#[rustfmt::skip]
pub const FEATURE_MAPPINGS: &[FeatureMapping] = &[
    FeatureMapping::new(b"afrc", FeatureType::Fractions, VERTICAL_FRACTIONS, NO_FRACTIONS),
    FeatureMapping::new(b"c2pc", FeatureType::UpperCase, UPPER_CASE_PETITE_CAPS, DEFAULT_UPPER_CASE),
    FeatureMapping::new(b"c2sc", FeatureType::UpperCase, UPPER_CASE_SMALL_CAPS, DEFAULT_UPPER_CASE),
    FeatureMapping::new(b"calt", FeatureType::ContextualAlternatives, CONTEXTUAL_ALTERNATES_ON, CONTEXTUAL_ALTERNATES_OFF),
    FeatureMapping::new(b"case", FeatureType::CaseSensitiveLayout, CASE_SENSITIVE_LAYOUT_ON, CASE_SENSITIVE_LAYOUT_OFF),
    FeatureMapping::new(b"clig", FeatureType::Ligatures, CONTEXTUAL_LIGATURES_ON, CONTEXTUAL_LIGATURES_OFF),
    FeatureMapping::new(b"cpsp", FeatureType::CaseSensitiveLayout, CASE_SENSITIVE_SPACING_ON, CASE_SENSITIVE_SPACING_OFF),
    FeatureMapping::new(b"cswh", FeatureType::ContextualAlternatives, CONTEXTUAL_SWASH_ALTERNATES_ON, CONTEXTUAL_SWASH_ALTERNATES_OFF),
    FeatureMapping::new(b"dlig", FeatureType::Ligatures, RARE_LIGATURES_ON, RARE_LIGATURES_OFF),
    FeatureMapping::new(b"expt", FeatureType::CharacterShape, EXPERT_CHARACTERS, 16),
    FeatureMapping::new(b"frac", FeatureType::Fractions, DIAGONAL_FRACTIONS, NO_FRACTIONS),
    FeatureMapping::new(b"fwid", FeatureType::TextSpacing, MONOSPACED_TEXT, 7),
    FeatureMapping::new(b"halt", FeatureType::TextSpacing, ALT_HALF_WIDTH_TEXT, 7),
    FeatureMapping::new(b"hist", FeatureType::Dummy, 0, 1),
    FeatureMapping::new(b"hkna", FeatureType::AlternateKana, ALTERNATE_HORIZ_KANA_ON, ALTERNATE_HORIZ_KANA_OFF),
    FeatureMapping::new(b"hlig", FeatureType::Ligatures, HISTORICAL_LIGATURES_ON, HISTORICAL_LIGATURES_OFF),
    FeatureMapping::new(b"hngl", FeatureType::Transliteration, HANJA_TO_HANGUL, NO_TRANSLITERATION),
    FeatureMapping::new(b"hojo", FeatureType::CharacterShape, HOJO_CHARACTERS, 16),
    FeatureMapping::new(b"hwid", FeatureType::TextSpacing, HALF_WIDTH_TEXT, 7),
    FeatureMapping::new(b"ital", FeatureType::ItalicCjkRoman, CJK_ITALIC_ROMAN_ON, CJK_ITALIC_ROMAN_OFF),
    FeatureMapping::new(b"jp04", FeatureType::CharacterShape, JIS2004_CHARACTERS, 16),
    FeatureMapping::new(b"jp78", FeatureType::CharacterShape, JIS1978_CHARACTERS, 16),
    FeatureMapping::new(b"jp83", FeatureType::CharacterShape, JIS1983_CHARACTERS, 16),
    FeatureMapping::new(b"jp90", FeatureType::CharacterShape, JIS1990_CHARACTERS, 16),
    FeatureMapping::new(b"liga", FeatureType::Ligatures, COMMON_LIGATURES_ON, COMMON_LIGATURES_OFF),
    FeatureMapping::new(b"lnum", FeatureType::NumberCase, UPPER_CASE_NUMBERS, 2),
    FeatureMapping::new(b"mgrk", FeatureType::MathematicalExtras, MATHEMATICAL_GREEK_ON, MATHEMATICAL_GREEK_OFF),
    FeatureMapping::new(b"nlck", FeatureType::CharacterShape, NLCCHARACTERS, 16),
    FeatureMapping::new(b"onum", FeatureType::NumberCase, LOWER_CASE_NUMBERS, 2),
    FeatureMapping::new(b"ordn", FeatureType::VerticalPosition, ORDINALS, NORMAL_POSITION),
    FeatureMapping::new(b"palt", FeatureType::TextSpacing, ALT_PROPORTIONAL_TEXT, 7),
    FeatureMapping::new(b"pcap", FeatureType::LowerCase, LOWER_CASE_PETITE_CAPS, DEFAULT_LOWER_CASE),
    FeatureMapping::new(b"pkna", FeatureType::TextSpacing, PROPORTIONAL_TEXT, 7),
    FeatureMapping::new(b"pnum", FeatureType::NumberSpacing, PROPORTIONAL_NUMBERS, 4),
    FeatureMapping::new(b"pwid", FeatureType::TextSpacing, PROPORTIONAL_TEXT, 7),
    FeatureMapping::new(b"qwid", FeatureType::TextSpacing, QUARTER_WIDTH_TEXT, 7),
    FeatureMapping::new(b"ruby", FeatureType::RubyKana, RUBY_KANA_ON, RUBY_KANA_OFF),
    FeatureMapping::new(b"sinf", FeatureType::VerticalPosition, SCIENTIFIC_INFERIORS, NORMAL_POSITION),
    FeatureMapping::new(b"smcp", FeatureType::LowerCase, LOWER_CASE_SMALL_CAPS, DEFAULT_LOWER_CASE),
    FeatureMapping::new(b"smpl", FeatureType::CharacterShape, SIMPLIFIED_CHARACTERS, 16),
    FeatureMapping::new(b"ss01", FeatureType::StylisticAlternatives, STYLISTIC_ALT_ONE_ON, STYLISTIC_ALT_ONE_OFF),
    FeatureMapping::new(b"ss02", FeatureType::StylisticAlternatives, STYLISTIC_ALT_TWO_ON, STYLISTIC_ALT_TWO_OFF),
    FeatureMapping::new(b"ss03", FeatureType::StylisticAlternatives, STYLISTIC_ALT_THREE_ON, STYLISTIC_ALT_THREE_OFF),
    FeatureMapping::new(b"ss04", FeatureType::StylisticAlternatives, STYLISTIC_ALT_FOUR_ON, STYLISTIC_ALT_FOUR_OFF),
    FeatureMapping::new(b"ss05", FeatureType::StylisticAlternatives, STYLISTIC_ALT_FIVE_ON, STYLISTIC_ALT_FIVE_OFF),
    FeatureMapping::new(b"ss06", FeatureType::StylisticAlternatives, STYLISTIC_ALT_SIX_ON, STYLISTIC_ALT_SIX_OFF),
    FeatureMapping::new(b"ss07", FeatureType::StylisticAlternatives, STYLISTIC_ALT_SEVEN_ON, STYLISTIC_ALT_SEVEN_OFF),
    FeatureMapping::new(b"ss08", FeatureType::StylisticAlternatives, STYLISTIC_ALT_EIGHT_ON, STYLISTIC_ALT_EIGHT_OFF),
    FeatureMapping::new(b"ss09", FeatureType::StylisticAlternatives, STYLISTIC_ALT_NINE_ON, STYLISTIC_ALT_NINE_OFF),
    FeatureMapping::new(b"ss10", FeatureType::StylisticAlternatives, STYLISTIC_ALT_TEN_ON, STYLISTIC_ALT_TEN_OFF),
    FeatureMapping::new(b"ss11", FeatureType::StylisticAlternatives, STYLISTIC_ALT_ELEVEN_ON, STYLISTIC_ALT_ELEVEN_OFF),
    FeatureMapping::new(b"ss12", FeatureType::StylisticAlternatives, STYLISTIC_ALT_TWELVE_ON, STYLISTIC_ALT_TWELVE_OFF),
    FeatureMapping::new(b"ss13", FeatureType::StylisticAlternatives, STYLISTIC_ALT_THIRTEEN_ON, STYLISTIC_ALT_THIRTEEN_OFF),
    FeatureMapping::new(b"ss14", FeatureType::StylisticAlternatives, STYLISTIC_ALT_FOURTEEN_ON, STYLISTIC_ALT_FOURTEEN_OFF),
    FeatureMapping::new(b"ss15", FeatureType::StylisticAlternatives, STYLISTIC_ALT_FIFTEEN_ON, STYLISTIC_ALT_FIFTEEN_OFF),
    FeatureMapping::new(b"ss16", FeatureType::StylisticAlternatives, STYLISTIC_ALT_SIXTEEN_ON, STYLISTIC_ALT_SIXTEEN_OFF),
    FeatureMapping::new(b"ss17", FeatureType::StylisticAlternatives, STYLISTIC_ALT_SEVENTEEN_ON, STYLISTIC_ALT_SEVENTEEN_OFF),
    FeatureMapping::new(b"ss18", FeatureType::StylisticAlternatives, STYLISTIC_ALT_EIGHTEEN_ON, STYLISTIC_ALT_EIGHTEEN_OFF),
    FeatureMapping::new(b"ss19", FeatureType::StylisticAlternatives, STYLISTIC_ALT_NINETEEN_ON, STYLISTIC_ALT_NINETEEN_OFF),
    FeatureMapping::new(b"ss20", FeatureType::StylisticAlternatives, STYLISTIC_ALT_TWENTY_ON, STYLISTIC_ALT_TWENTY_OFF),
    FeatureMapping::new(b"subs", FeatureType::VerticalPosition, INFERIORS, NORMAL_POSITION),
    FeatureMapping::new(b"sups", FeatureType::VerticalPosition, SUPERIORS, NORMAL_POSITION),
    FeatureMapping::new(b"swsh", FeatureType::ContextualAlternatives, SWASH_ALTERNATES_ON, SWASH_ALTERNATES_OFF),
    FeatureMapping::new(b"titl", FeatureType::StyleOptions, TITLING_CAPS, NO_STYLE_OPTIONS),
    FeatureMapping::new(b"tnam", FeatureType::CharacterShape, TRADITIONAL_NAMES_CHARACTERS, 16),
    FeatureMapping::new(b"tnum", FeatureType::NumberSpacing, MONOSPACED_NUMBERS, 4),
    FeatureMapping::new(b"trad", FeatureType::CharacterShape, TRADITIONAL_CHARACTERS, 16),
    FeatureMapping::new(b"twid", FeatureType::TextSpacing, THIRD_WIDTH_TEXT, 7),
    FeatureMapping::new(b"unic", FeatureType::LetterCase, 14, 15),
    FeatureMapping::new(b"valt", FeatureType::TextSpacing, ALT_PROPORTIONAL_TEXT, 7),
    FeatureMapping::new(b"vert", FeatureType::VerticalSubstitution, SUBSTITUTE_VERTICAL_FORMS_ON, SUBSTITUTE_VERTICAL_FORMS_OFF),
    FeatureMapping::new(b"vhal", FeatureType::TextSpacing, ALT_HALF_WIDTH_TEXT, 7),
    FeatureMapping::new(b"vkna", FeatureType::AlternateKana, ALTERNATE_VERT_KANA_ON, ALTERNATE_VERT_KANA_OFF),
    FeatureMapping::new(b"vpal", FeatureType::TextSpacing, ALT_PROPORTIONAL_TEXT, 7),
    FeatureMapping::new(b"vrt2", FeatureType::VerticalSubstitution, SUBSTITUTE_VERTICAL_FORMS_ON, SUBSTITUTE_VERTICAL_FORMS_OFF),
    FeatureMapping::new(b"vrtr", FeatureType::VerticalSubstitution, 2, 3),
    FeatureMapping::new(b"zero", FeatureType::TypographicExtras, SLASHED_ZERO_ON, SLASHED_ZERO_OFF),
];
