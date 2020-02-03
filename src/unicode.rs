use std::convert::TryFrom;

pub use unicode_general_category::GeneralCategory;
pub use unicode_ccc::CanonicalCombiningClass;

use crate::{Script, CodePoint};

// Space estimates based on:
// https://unicode.org/charts/PDF/U2000.pdf
// https://docs.microsoft.com/en-us/typography/develop/character-design-standards/whitespace
#[derive(Clone, Copy, Debug)]
pub enum Space {
    SpaceEm  = 1,
    SpaceEm2 = 2,
    SpaceEm3 = 3,
    SpaceEm4 = 4,
    SpaceEm5 = 5,
    SpaceEm6 = 6,
    SpaceEm16 = 16,
    Space4Em18, // 4/18th of an EM!
    Space,
    SpaceFigure,
    SpacePunctuation,
    SpaceNarrow,
}

const MODIFIED_COMBINING_CLASS: &[u8; 256] = &[
    CanonicalCombiningClass::NotReordered as u8,
    CanonicalCombiningClass::Overlay as u8,
    2, 3, 4, 5, 6,
    CanonicalCombiningClass::Nukta as u8,
    CanonicalCombiningClass::KanaVoicing as u8,
    CanonicalCombiningClass::Virama as u8,

    // Hebrew
    modified_combining_class::CCC10,
    modified_combining_class::CCC11,
    modified_combining_class::CCC12,
    modified_combining_class::CCC13,
    modified_combining_class::CCC14,
    modified_combining_class::CCC15,
    modified_combining_class::CCC16,
    modified_combining_class::CCC17,
    modified_combining_class::CCC18,
    modified_combining_class::CCC19,
    modified_combining_class::CCC20,
    modified_combining_class::CCC21,
    modified_combining_class::CCC22,
    modified_combining_class::CCC23,
    modified_combining_class::CCC24,
    modified_combining_class::CCC25,
    modified_combining_class::CCC26,

    // Arabic
    modified_combining_class::CCC27,
    modified_combining_class::CCC28,
    modified_combining_class::CCC29,
    modified_combining_class::CCC30,
    modified_combining_class::CCC31,
    modified_combining_class::CCC32,
    modified_combining_class::CCC33,
    modified_combining_class::CCC34,
    modified_combining_class::CCC35,

    // Syriac
    modified_combining_class::CCC36,

    37, 38, 39,
    40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59,
    60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79,
    80, 81, 82, 83,

    // Telugu
    modified_combining_class::CCC84,
    85, 86, 87, 88, 89, 90,
    modified_combining_class::CCC91,
    92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102,

    // Thai
    modified_combining_class::CCC103,
    104, 105, 106,
    modified_combining_class::CCC107,
    108, 109, 110, 111, 112, 113, 114, 115, 116, 117,

    // Lao
    modified_combining_class::CCC118,
    119, 120, 121,
    modified_combining_class::CCC122,
    123, 124, 125, 126, 127, 128,

    // Tibetan
    modified_combining_class::CCC129,
    modified_combining_class::CCC130,
    131,
    modified_combining_class::CCC132,
    133, 134, 135, 136, 137, 138, 139,


    140, 141, 142, 143, 144, 145, 146, 147, 148, 149,
    150, 151, 152, 153, 154, 155, 156, 157, 158, 159,
    160, 161, 162, 163, 164, 165, 166, 167, 168, 169,
    170, 171, 172, 173, 174, 175, 176, 177, 178, 179,
    180, 181, 182, 183, 184, 185, 186, 187, 188, 189,
    190, 191, 192, 193, 194, 195, 196, 197, 198, 199,

    CanonicalCombiningClass::AttachedBelowLeft as u8,
    201,
    CanonicalCombiningClass::AttachedBelow as u8,
    203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213,
    CanonicalCombiningClass::AttachedAbove as u8,
    215,
    CanonicalCombiningClass::AttachedAboveRight as u8,
    217,
    CanonicalCombiningClass::BelowLeft as u8,
    219,
    CanonicalCombiningClass::Below as u8,
    221,
    CanonicalCombiningClass::BelowRight as u8,
    223,
    CanonicalCombiningClass::Left as u8,
    225,
    CanonicalCombiningClass::Right as u8,
    227,
    CanonicalCombiningClass::AboveLeft as u8,
    229,
    CanonicalCombiningClass::Above as u8,
    231,
    CanonicalCombiningClass::AboveRight as u8,
    CanonicalCombiningClass::DoubleBelow as u8,
    CanonicalCombiningClass::DoubleAbove as u8,
    235, 236, 237, 238, 239,
    CanonicalCombiningClass::IotaSubscript as u8,
    241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254,
    255, /* HB_UNICODE_COMBINING_CLASS_INVALID */
];

// Default_Ignorable codepoints:
//
// Note: While U+115F, U+1160, U+3164 and U+FFA0 are Default_Ignorable,
// we do NOT want to hide them, as the way Uniscribe has implemented them
// is with regular spacing glyphs, and that's the way fonts are made to work.
// As such, we make exceptions for those four.
// Also ignoring U+1BCA0..1BCA3. https://github.com/harfbuzz/harfbuzz/issues/503
//
// Unicode 7.0:
// $ grep '; Default_Ignorable_Code_Point ' DerivedCoreProperties.txt | sed 's/;.*#/#/'
// 00AD          # Cf       SOFT HYPHEN
// 034F          # Mn       COMBINING GRAPHEME JOINER
// 061C          # Cf       ARABIC LETTER MARK
// 115F..1160    # Lo   [2] HANGUL CHOSEONG FILLER..HANGUL JUNGSEONG FILLER
// 17B4..17B5    # Mn   [2] KHMER VOWEL INHERENT AQ..KHMER VOWEL INHERENT AA
// 180B..180D    # Mn   [3] MONGOLIAN FREE VARIATION SELECTOR ONE..MONGOLIAN FREE VARIATION SELECTOR THREE
// 180E          # Cf       MONGOLIAN VOWEL SEPARATOR
// 200B..200F    # Cf   [5] ZERO WIDTH SPACE..RIGHT-TO-LEFT MARK
// 202A..202E    # Cf   [5] LEFT-TO-RIGHT EMBEDDING..RIGHT-TO-LEFT OVERRIDE
// 2060..2064    # Cf   [5] WORD JOINER..INVISIBLE PLUS
// 2065          # Cn       <reserved-2065>
// 2066..206F    # Cf  [10] LEFT-TO-RIGHT ISOLATE..NOMINAL DIGIT SHAPES
// 3164          # Lo       HANGUL FILLER
// FE00..FE0F    # Mn  [16] VARIATION SELECTOR-1..VARIATION SELECTOR-16
// FEFF          # Cf       ZERO WIDTH NO-BREAK SPACE
// FFA0          # Lo       HALFWIDTH HANGUL FILLER
// FFF0..FFF8    # Cn   [9] <reserved-FFF0>..<reserved-FFF8>
// 1BCA0..1BCA3  # Cf   [4] SHORTHAND FORMAT LETTER OVERLAP..SHORTHAND FORMAT UP STEP
// 1D173..1D17A  # Cf   [8] MUSICAL SYMBOL BEGIN BEAM..MUSICAL SYMBOL END PHRASE
// E0000         # Cn       <reserved-E0000>
// E0001         # Cf       LANGUAGE TAG
// E0002..E001F  # Cn  [30] <reserved-E0002>..<reserved-E001F>
// E0020..E007F  # Cf  [96] TAG SPACE..CANCEL TAG
// E0080..E00FF  # Cn [128] <reserved-E0080>..<reserved-E00FF>
// E0100..E01EF  # Mn [240] VARIATION SELECTOR-17..VARIATION SELECTOR-256
// E01F0..E0FFF  # Cn [3600] <reserved-E01F0>..<reserved-E0FFF>
fn is_default_ignorable(ch: CodePoint) -> bool {
    let plane = ch >> 16;
    if plane == 0 {
        // BMP
        let page = ch >> 8;
        match page {
            0x00 => ch == 0x00AD,
            0x03 => ch == 0x034F,
            0x06 => ch == 0x061C,
            0x17 => (0x17B4..=0x17B5).contains(&ch),
            0x18 => (0x180B..=0x180E).contains(&ch),
            0x20 => (0x200B..=0x200F).contains(&ch) ||
                    (0x202A..=0x202E).contains(&ch) ||
                    (0x2060..=0x206F).contains(&ch),
            0xFE => (0xFE00..=0xFE0F).contains(&ch) || ch == 0xFEFF,
            0xFF => (0xFFF0..=0xFFF8).contains(&ch),
            _ => false,
        }
    } else {
        // Other planes
        match plane {
            0x01 => (0x1D173..=0x1D17A).contains(&ch),
            0x0E => (0xE0000..=0xE0FFF).contains(&ch),
            _ => false,
        }
    }
}

#[no_mangle]
pub extern "C" fn rb_ucd_is_default_ignorable(ch: u32) -> i32 {
    is_default_ignorable(ch) as i32
}

pub fn script_from_char(c: char) -> Script {
    use unicode_script as us;
    use crate::script;

    match us::Script::from(c) {
        us::Script::Common => script::COMMON,
        us::Script::Inherited => script::INHERITED,
        us::Script::Adlam => script::ADLAM,
        us::Script::Caucasian_Albanian => script::CAUCASIAN_ALBANIAN,
        us::Script::Ahom => script::AHOM,
        us::Script::Arabic => script::ARABIC,
        us::Script::Imperial_Aramaic => script::IMPERIAL_ARAMAIC,
        us::Script::Armenian => script::ARMENIAN,
        us::Script::Avestan => script::AVESTAN,
        us::Script::Balinese => script::BALINESE,
        us::Script::Bamum => script::BAMUM,
        us::Script::Bassa_Vah => script::BASSA_VAH,
        us::Script::Batak => script::BATAK,
        us::Script::Bengali => script::BENGALI,
        us::Script::Bhaiksuki => script::BHAIKSUKI,
        us::Script::Bopomofo => script::BOPOMOFO,
        us::Script::Brahmi => script::BRAHMI,
        us::Script::Braille => script::BRAILLE,
        us::Script::Buginese => script::BUGINESE,
        us::Script::Buhid => script::BUHID,
        us::Script::Chakma => script::CHAKMA,
        us::Script::Canadian_Aboriginal => script::CANADIAN_SYLLABICS,
        us::Script::Carian => script::CARIAN,
        us::Script::Cham => script::CHAM,
        us::Script::Cherokee => script::CHEROKEE,
        us::Script::Coptic => script::COPTIC,
        us::Script::Cypriot => script::CYPRIOT,
        us::Script::Cyrillic => script::CYRILLIC,
        us::Script::Devanagari => script::DEVANAGARI,
        us::Script::Dogra => script::DOGRA,
        us::Script::Deseret => script::DESERET,
        us::Script::Duployan => script::DUPLOYAN,
        us::Script::Egyptian_Hieroglyphs => script::EGYPTIAN_HIEROGLYPHS,
        us::Script::Elbasan => script::ELBASAN,
        us::Script::Elymaic => script::ELYMAIC,
        us::Script::Ethiopic => script::ETHIOPIC,
        us::Script::Georgian => script::GEORGIAN,
        us::Script::Glagolitic => script::GLAGOLITIC,
        us::Script::Gunjala_Gondi => script::GUNJALA_GONDI,
        us::Script::Masaram_Gondi => script::MASARAM_GONDI,
        us::Script::Gothic => script::GOTHIC,
        us::Script::Grantha => script::GRANTHA,
        us::Script::Greek => script::GREEK,
        us::Script::Gujarati => script::GUJARATI,
        us::Script::Gurmukhi => script::GURMUKHI,
        us::Script::Hangul => script::HANGUL,
        us::Script::Han => script::HAN,
        us::Script::Hanunoo => script::HANUNOO,
        us::Script::Hatran => script::HATRAN,
        us::Script::Hebrew => script::HEBREW,
        us::Script::Hiragana => script::HIRAGANA,
        us::Script::Anatolian_Hieroglyphs => script::ANATOLIAN_HIEROGLYPHS,
        us::Script::Pahawh_Hmong => script::PAHAWH_HMONG,
        us::Script::Nyiakeng_Puachue_Hmong => script::NYIAKENG_PUACHUE_HMONG,
        us::Script::Old_Hungarian => script::OLD_HUNGARIAN,
        us::Script::Old_Italic => script::OLD_ITALIC,
        us::Script::Javanese => script::JAVANESE,
        us::Script::Kayah_Li => script::KAYAH_LI,
        us::Script::Katakana => script::KATAKANA,
        us::Script::Kharoshthi => script::KHAROSHTHI,
        us::Script::Khmer => script::KHMER,
        us::Script::Khojki => script::KHOJKI,
        us::Script::Kannada => script::KANNADA,
        us::Script::Kaithi => script::KAITHI,
        us::Script::Tai_Tham => script::TAI_THAM,
        us::Script::Lao => script::LAO,
        us::Script::Latin => script::LATIN,
        us::Script::Lepcha => script::LEPCHA,
        us::Script::Limbu => script::LIMBU,
        us::Script::Linear_A => script::LINEAR_A,
        us::Script::Linear_B => script::LINEAR_B,
        us::Script::Lisu => script::LISU,
        us::Script::Lycian => script::LYCIAN,
        us::Script::Lydian => script::LYDIAN,
        us::Script::Mahajani => script::MAHAJANI,
        us::Script::Makasar => script::MAKASAR,
        us::Script::Mandaic => script::MANDAIC,
        us::Script::Manichaean => script::MANICHAEAN,
        us::Script::Marchen => script::MARCHEN,
        us::Script::Medefaidrin => script::MEDEFAIDRIN,
        us::Script::Mende_Kikakui => script::MENDE_KIKAKUI,
        us::Script::Meroitic_Cursive => script::MEROITIC_CURSIVE,
        us::Script::Meroitic_Hieroglyphs => script::MEROITIC_HIEROGLYPHS,
        us::Script::Malayalam => script::MALAYALAM,
        us::Script::Modi => script::MODI,
        us::Script::Mongolian => script::MONGOLIAN,
        us::Script::Mro => script::MRO,
        us::Script::Meetei_Mayek => script::MEETEI_MAYEK,
        us::Script::Multani => script::MULTANI,
        us::Script::Myanmar => script::MYANMAR,
        us::Script::Nandinagari => script::NANDINAGARI,
        us::Script::Old_North_Arabian => script::OLD_NORTH_ARABIAN,
        us::Script::Nabataean => script::NABATAEAN,
        us::Script::Newa => script::NEWA,
        us::Script::Nko => script::NKO,
        us::Script::Nushu => script::NUSHU,
        us::Script::Ogham => script::OGHAM,
        us::Script::Ol_Chiki => script::OL_CHIKI,
        us::Script::Old_Turkic => script::OLD_TURKIC,
        us::Script::Oriya => script::ORIYA,
        us::Script::Osage => script::OSAGE,
        us::Script::Osmanya => script::OSMANYA,
        us::Script::Palmyrene => script::PALMYRENE,
        us::Script::Pau_Cin_Hau => script::PAU_CIN_HAU,
        us::Script::Old_Permic => script::OLD_PERMIC,
        us::Script::Phags_Pa => script::PHAGS_PA,
        us::Script::Inscriptional_Pahlavi => script::INSCRIPTIONAL_PAHLAVI,
        us::Script::Psalter_Pahlavi => script::PSALTER_PAHLAVI,
        us::Script::Phoenician => script::PHOENICIAN,
        us::Script::Miao => script::MIAO,
        us::Script::Inscriptional_Parthian => script::INSCRIPTIONAL_PARTHIAN,
        us::Script::Rejang => script::REJANG,
        us::Script::Hanifi_Rohingya => script::HANIFI_ROHINGYA,
        us::Script::Runic => script::RUNIC,
        us::Script::Samaritan => script::SAMARITAN,
        us::Script::Old_South_Arabian => script::OLD_SOUTH_ARABIAN,
        us::Script::Saurashtra => script::SAURASHTRA,
        us::Script::SignWriting => script::SIGNWRITING,
        us::Script::Shavian => script::SHAVIAN,
        us::Script::Sharada => script::SHARADA,
        us::Script::Siddham => script::SIDDHAM,
        us::Script::Khudawadi => script::KHUDAWADI,
        us::Script::Sinhala => script::SINHALA,
        us::Script::Sogdian => script::SOGDIAN,
        us::Script::Old_Sogdian => script::OLD_SOGDIAN,
        us::Script::Sora_Sompeng => script::SORA_SOMPENG,
        us::Script::Soyombo => script::SOYOMBO,
        us::Script::Sundanese => script::SUNDANESE,
        us::Script::Syloti_Nagri => script::SYLOTI_NAGRI,
        us::Script::Syriac => script::SYRIAC,
        us::Script::Tagbanwa => script::TAGBANWA,
        us::Script::Takri => script::TAKRI,
        us::Script::Tai_Le => script::TAI_LE,
        us::Script::New_Tai_Lue => script::NEW_TAI_LUE,
        us::Script::Tamil => script::TAMIL,
        us::Script::Tangut => script::TANGUT,
        us::Script::Tai_Viet => script::TAI_VIET,
        us::Script::Telugu => script::TELUGU,
        us::Script::Tifinagh => script::TIFINAGH,
        us::Script::Tagalog => script::TAGALOG,
        us::Script::Thaana => script::THAANA,
        us::Script::Thai => script::THAI,
        us::Script::Tibetan => script::TIBETAN,
        us::Script::Tirhuta => script::TIRHUTA,
        us::Script::Ugaritic => script::UGARITIC,
        us::Script::Vai => script::VAI,
        us::Script::Warang_Citi => script::WARANG_CITI,
        us::Script::Wancho => script::WANCHO,
        us::Script::Old_Persian => script::OLD_PERSIAN,
        us::Script::Cuneiform => script::CUNEIFORM,
        us::Script::Yi => script::YI,
        us::Script::Zanabazar_Square => script::ZANABAZAR_SQUARE,
        _ => script::UNKNOWN,
    }
}

#[no_mangle]
pub extern "C" fn rb_ucd_script(ch: u32) -> u32 {
    script_from_char(char::try_from(ch).unwrap()).tag().as_u32()
}

#[no_mangle]
pub extern "C" fn rb_ucd_compose(a: u32, b: u32, ab: *mut u32) -> i32 {
    unsafe {
        let new = unicode_normalization::char::compose(
            char::try_from(a).unwrap(),
            char::try_from(b).unwrap(),
        );

        if let Some(c) = new {
            *ab = c as u32;
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn rb_ucd_decompose(ab: u32, a: *mut u32, b: *mut u32) -> i32 {
    unsafe {
        let mut is_a_set = false;
        unicode_normalization::char::decompose_canonical(
            char::try_from(ab).unwrap(),
            |c| {
                if is_a_set {
                    *b = c as u32;
                } else {
                    is_a_set = true;
                    *a = c as u32;
                    *b = 0;
                }
            }
        );

        (*a != ab) as i32
    }
}

#[no_mangle]
pub extern "C" fn rb_ucd_general_category(c: u32) -> u32 {
    let cat = unicode_general_category::get_general_category(char::try_from(c).unwrap());
    cat.to_hb()
}

pub trait GeneralCategoryExt {
    fn to_hb(&self) -> u32;
    fn from_hb(gc: u32) -> Self;
    fn is_unicode_mark(&self) -> bool;
}

impl GeneralCategoryExt for GeneralCategory {
    fn to_hb(&self) -> u32 {
        use crate::ffi;
        match *self {
            GeneralCategory::ClosePunctuation => ffi::HB_UNICODE_GENERAL_CATEGORY_CLOSE_PUNCTUATION,
            GeneralCategory::ConnectorPunctuation => ffi::HB_UNICODE_GENERAL_CATEGORY_CONNECT_PUNCTUATION,
            GeneralCategory::Control => ffi::HB_UNICODE_GENERAL_CATEGORY_CONTROL,
            GeneralCategory::CurrencySymbol => ffi::HB_UNICODE_GENERAL_CATEGORY_CURRENCY_SYMBOL,
            GeneralCategory::DashPunctuation => ffi::HB_UNICODE_GENERAL_CATEGORY_DASH_PUNCTUATION,
            GeneralCategory::DecimalNumber => ffi::HB_UNICODE_GENERAL_CATEGORY_DECIMAL_NUMBER,
            GeneralCategory::EnclosingMark => ffi::HB_UNICODE_GENERAL_CATEGORY_ENCLOSING_MARK,
            GeneralCategory::FinalPunctuation => ffi::HB_UNICODE_GENERAL_CATEGORY_FINAL_PUNCTUATION,
            GeneralCategory::Format => ffi::HB_UNICODE_GENERAL_CATEGORY_FORMAT,
            GeneralCategory::InitialPunctuation => ffi::HB_UNICODE_GENERAL_CATEGORY_INITIAL_PUNCTUATION,
            GeneralCategory::LetterNumber => ffi::HB_UNICODE_GENERAL_CATEGORY_LETTER_NUMBER,
            GeneralCategory::LineSeparator => ffi::HB_UNICODE_GENERAL_CATEGORY_LINE_SEPARATOR,
            GeneralCategory::LowercaseLetter => ffi::HB_UNICODE_GENERAL_CATEGORY_LOWERCASE_LETTER,
            GeneralCategory::MathSymbol => ffi::HB_UNICODE_GENERAL_CATEGORY_MATH_SYMBOL,
            GeneralCategory::ModifierLetter => ffi::HB_UNICODE_GENERAL_CATEGORY_MODIFIER_LETTER,
            GeneralCategory::ModifierSymbol => ffi::HB_UNICODE_GENERAL_CATEGORY_MODIFIER_SYMBOL,
            GeneralCategory::NonspacingMark => ffi::HB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK,
            GeneralCategory::OpenPunctuation => ffi::HB_UNICODE_GENERAL_CATEGORY_OPEN_PUNCTUATION,
            GeneralCategory::OtherLetter => ffi::HB_UNICODE_GENERAL_CATEGORY_OTHER_LETTER,
            GeneralCategory::OtherNumber => ffi::HB_UNICODE_GENERAL_CATEGORY_OTHER_NUMBER,
            GeneralCategory::OtherPunctuation => ffi::HB_UNICODE_GENERAL_CATEGORY_OTHER_PUNCTUATION,
            GeneralCategory::OtherSymbol => ffi::HB_UNICODE_GENERAL_CATEGORY_OTHER_SYMBOL,
            GeneralCategory::ParagraphSeparator => ffi::HB_UNICODE_GENERAL_CATEGORY_PARAGRAPH_SEPARATOR,
            GeneralCategory::PrivateUse => ffi::HB_UNICODE_GENERAL_CATEGORY_PRIVATE_USE,
            GeneralCategory::SpaceSeparator => ffi::HB_UNICODE_GENERAL_CATEGORY_SPACE_SEPARATOR,
            GeneralCategory::SpacingMark => ffi::HB_UNICODE_GENERAL_CATEGORY_SPACING_MARK,
            GeneralCategory::Surrogate => ffi::HB_UNICODE_GENERAL_CATEGORY_SURROGATE,
            GeneralCategory::TitlecaseLetter => ffi::HB_UNICODE_GENERAL_CATEGORY_TITLECASE_LETTER,
            GeneralCategory::Unassigned => ffi::HB_UNICODE_GENERAL_CATEGORY_UNASSIGNED,
            GeneralCategory::UppercaseLetter => ffi::HB_UNICODE_GENERAL_CATEGORY_UPPERCASE_LETTER,
        }
    }

    fn from_hb(gc: u32) -> Self {
        use crate::ffi;
        match gc {
            ffi::HB_UNICODE_GENERAL_CATEGORY_CLOSE_PUNCTUATION => GeneralCategory::ClosePunctuation,
            ffi::HB_UNICODE_GENERAL_CATEGORY_CONNECT_PUNCTUATION => GeneralCategory::ConnectorPunctuation,
            ffi::HB_UNICODE_GENERAL_CATEGORY_CONTROL => GeneralCategory::Control,
            ffi::HB_UNICODE_GENERAL_CATEGORY_CURRENCY_SYMBOL => GeneralCategory::CurrencySymbol,
            ffi::HB_UNICODE_GENERAL_CATEGORY_DASH_PUNCTUATION => GeneralCategory::DashPunctuation,
            ffi::HB_UNICODE_GENERAL_CATEGORY_DECIMAL_NUMBER => GeneralCategory::DecimalNumber,
            ffi::HB_UNICODE_GENERAL_CATEGORY_ENCLOSING_MARK => GeneralCategory::EnclosingMark,
            ffi::HB_UNICODE_GENERAL_CATEGORY_FINAL_PUNCTUATION => GeneralCategory::FinalPunctuation,
            ffi::HB_UNICODE_GENERAL_CATEGORY_FORMAT => GeneralCategory::Format,
            ffi::HB_UNICODE_GENERAL_CATEGORY_INITIAL_PUNCTUATION => GeneralCategory::InitialPunctuation,
            ffi::HB_UNICODE_GENERAL_CATEGORY_LETTER_NUMBER => GeneralCategory::LetterNumber,
            ffi::HB_UNICODE_GENERAL_CATEGORY_LINE_SEPARATOR => GeneralCategory::LineSeparator,
            ffi::HB_UNICODE_GENERAL_CATEGORY_LOWERCASE_LETTER => GeneralCategory::LowercaseLetter,
            ffi::HB_UNICODE_GENERAL_CATEGORY_MATH_SYMBOL => GeneralCategory::MathSymbol,
            ffi::HB_UNICODE_GENERAL_CATEGORY_MODIFIER_LETTER => GeneralCategory::ModifierLetter,
            ffi::HB_UNICODE_GENERAL_CATEGORY_MODIFIER_SYMBOL => GeneralCategory::ModifierSymbol,
            ffi::HB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK => GeneralCategory::NonspacingMark,
            ffi::HB_UNICODE_GENERAL_CATEGORY_OPEN_PUNCTUATION => GeneralCategory::OpenPunctuation,
            ffi::HB_UNICODE_GENERAL_CATEGORY_OTHER_LETTER => GeneralCategory::OtherLetter,
            ffi::HB_UNICODE_GENERAL_CATEGORY_OTHER_NUMBER => GeneralCategory::OtherNumber,
            ffi::HB_UNICODE_GENERAL_CATEGORY_OTHER_PUNCTUATION => GeneralCategory::OtherPunctuation,
            ffi::HB_UNICODE_GENERAL_CATEGORY_OTHER_SYMBOL => GeneralCategory::OtherSymbol,
            ffi::HB_UNICODE_GENERAL_CATEGORY_PARAGRAPH_SEPARATOR => GeneralCategory::ParagraphSeparator,
            ffi::HB_UNICODE_GENERAL_CATEGORY_PRIVATE_USE => GeneralCategory::PrivateUse,
            ffi::HB_UNICODE_GENERAL_CATEGORY_SPACE_SEPARATOR => GeneralCategory::SpaceSeparator,
            ffi::HB_UNICODE_GENERAL_CATEGORY_SPACING_MARK => GeneralCategory::SpacingMark,
            ffi::HB_UNICODE_GENERAL_CATEGORY_SURROGATE => GeneralCategory::Surrogate,
            ffi::HB_UNICODE_GENERAL_CATEGORY_TITLECASE_LETTER => GeneralCategory::TitlecaseLetter,
            ffi::HB_UNICODE_GENERAL_CATEGORY_UNASSIGNED => GeneralCategory::Unassigned,
            ffi::HB_UNICODE_GENERAL_CATEGORY_UPPERCASE_LETTER => GeneralCategory::UppercaseLetter,
            _ => unreachable!(),
        }
    }

    fn is_unicode_mark(&self) -> bool {
        match *self {
            GeneralCategory::SpacingMark |
            GeneralCategory::EnclosingMark |
            GeneralCategory::NonspacingMark => true,
            _ => false,
        }
    }
}

pub trait CharExt {
    fn general_category(&self) -> GeneralCategory;
    fn space_fallback(&self) -> Option<Space>;
    fn modified_combining_class(&self) -> u8;
    fn mirrored(&self) -> Option<char>;
    fn is_emoji_extended_pictographic(&self) -> bool;
}

impl CharExt for char {
    fn general_category(&self) -> GeneralCategory {
        unicode_general_category::get_general_category(*self)
    }

    fn space_fallback(&self) -> Option<Space> {
        // All GC=Zs chars that can use a fallback.
        match self {
            '\u{0020}' => Some(Space::Space),               // SPACE
            '\u{00A0}' => Some(Space::Space),               // NO-BREAK SPACE
            '\u{2000}' => Some(Space::SpaceEm2),            // EN QUAD
            '\u{2001}' => Some(Space::SpaceEm),             // EM QUAD
            '\u{2002}' => Some(Space::SpaceEm2),            // EN SPACE
            '\u{2003}' => Some(Space::SpaceEm),             // EM SPACE
            '\u{2004}' => Some(Space::SpaceEm3),            // THREE-PER-EM SPACE
            '\u{2005}' => Some(Space::SpaceEm4),            // FOUR-PER-EM SPACE
            '\u{2006}' => Some(Space::SpaceEm6),            // SIX-PER-EM SPACE
            '\u{2007}' => Some(Space::SpaceFigure),         // FIGURE SPACE
            '\u{2008}' => Some(Space::SpacePunctuation),    // PUNCTUATION SPACE
            '\u{2009}' => Some(Space::SpaceEm5),            // THIN SPACE
            '\u{200A}' => Some(Space::SpaceEm16),           // HAIR SPACE
            '\u{202F}' => Some(Space::SpaceNarrow),         // NARROW NO-BREAK SPACE
            '\u{205F}' => Some(Space::Space4Em18),          // MEDIUM MATHEMATICAL SPACE
            '\u{3000}' => Some(Space::SpaceEm),             // IDEOGRAPHIC SPACE
            _ => None,                                      // OGHAM SPACE MARK
        }
    }

    fn modified_combining_class(&self) -> u8 {
        let mut u = *self;

        // XXX This hack belongs to the Myanmar shaper.
        if u == '\u{1037}'{
            u = '\u{103A}';
        }

        // XXX This hack belongs to the USE shaper (for Tai Tham):
        // Reorder SAKOT to ensure it comes after any tone marks.
        if u == '\u{1A60}' {
            return 254;
        }

        // XXX This hack belongs to the Tibetan shaper:
        // Reorder PADMA to ensure it comes after any vowel marks.
        if u == '\u{0FC6}' {
            return 254;
        }

        // Reorder TSA -PHRU to reorder before U+0F74
        if u == '\u{0F39}' {
            return 127;
        }

        let k = unicode_ccc::get_canonical_combining_class(u);
        MODIFIED_COMBINING_CLASS[k as usize]
    }

    fn mirrored(&self) -> Option<char> {
        unicode_bidi_mirroring::get_mirrored(*self)
    }

    fn is_emoji_extended_pictographic(&self) -> bool {
        is_emoji_extended_pictographic_impl(*self)
    }
}

#[allow(dead_code)]
pub mod modified_combining_class {
    // Hebrew
    //
    // We permute the "fixed-position" classes 10-26 into the order
    // described in the SBL Hebrew manual:
    //
    // https://www.sbl-site.org/Fonts/SBLHebrewUserManual1.5x.pdf
    //
    // (as recommended by:
    //  https://forum.fontlab.com/archive-old-microsoft-volt-group/vista-and-diacritic-ordering/msg22823/)
    //
    // More details here:
    // https://bugzilla.mozilla.org/show_bug.cgi?id=662055
    pub const CCC10: u8 = 22; // sheva
    pub const CCC11: u8 = 15; // hataf segol
    pub const CCC12: u8 = 16; // hataf patah
    pub const CCC13: u8 = 17; // hataf qamats
    pub const CCC14: u8 = 23; // hiriq
    pub const CCC15: u8 = 18; // tsere
    pub const CCC16: u8 = 19; // segol
    pub const CCC17: u8 = 20; // patah
    pub const CCC18: u8 = 21; // qamats
    pub const CCC19: u8 = 14; // holam
    pub const CCC20: u8 = 24; // qubuts
    pub const CCC21: u8 = 12; // dagesh
    pub const CCC22: u8 = 25; // meteg
    pub const CCC23: u8 = 13; // rafe
    pub const CCC24: u8 = 10; // shin dot
    pub const CCC25: u8 = 11; // sin dot
    pub const CCC26: u8 = 26; // point varika

    // Arabic
    //
    // Modify to move Shadda (ccc=33) before other marks.  See:
    // https://unicode.org/faq/normalization.html#8
    // https://unicode.org/faq/normalization.html#9
    pub const CCC27: u8 = 28; // fathatan
    pub const CCC28: u8 = 29; // dammatan
    pub const CCC29: u8 = 30; // kasratan
    pub const CCC30: u8 = 31; // fatha
    pub const CCC31: u8 = 32; // damma
    pub const CCC32: u8 = 33; // kasra
    pub const CCC33: u8 = 27; // shadda
    pub const CCC34: u8 = 34; // sukun
    pub const CCC35: u8 = 35; // superscript alef

    // Syriac
    pub const CCC36: u8 = 36; // superscript alaph

    // Telugu
    //
    // Modify Telugu length marks (ccc=84, ccc=91).
    // These are the only matras in the main Indic scripts range that have
    // a non-zero ccc.  That makes them reorder with the Halant that is
    // ccc=9.  Just zero them, we don't need them in our Indic shaper.
    pub const CCC84: u8 = 0; // length mark
    pub const CCC91: u8 = 0; // ai length mark

    // Thai
    //
    // Modify U+0E38 and U+0E39 (ccc=103) to be reordered before U+0E3A (ccc=9).
    // Assign 3, which is unassigned otherwise.
    // Uniscribe does this reordering too.
    pub const CCC103: u8 = 3;   // sara u / sara uu
    pub const CCC107: u8 = 107; // mai *

    // Lao
    pub const CCC118: u8 = 118; // sign u / sign uu
    pub const CCC122: u8 = 122; // mai *

    // Tibetan
    //
    // In case of multiple vowel-signs, use u first (but after achung)
    // this allows Dzongkha multi-vowel shortcuts to render correctly
    pub const CCC129: u8 = 129; // sign aa
    pub const CCC130: u8 = 132; // sign i
    pub const CCC132: u8 = 131; // sign u
}

#[no_mangle]
pub extern "C" fn rb_ucd_space_fallback_type(u: u32) -> i32 {
    match char::try_from(u).unwrap().space_fallback() {
        Some(space) => space as i32,
        None => 0,
    }
}

#[no_mangle]
pub extern "C" fn rb_ucd_modified_combining_class(u: u32) -> u32 {
    char::try_from(u).unwrap().modified_combining_class() as u32
}

#[no_mangle]
pub extern "C" fn rb_ucd_mirroring(u: u32) -> u32 {
    match char::try_from(u).unwrap().mirrored() {
        Some(c) => c as u32,
        None => u,
    }
}

fn is_emoji_extended_pictographic_impl(c: char) -> bool {
    // Generated by scripts/gen-unicode-is-emoji-ext-pict.py
    match c as u32 {
        0x00A9 => true,
        0x00AE => true,
        0x203C => true,
        0x2049 => true,
        0x2122 => true,
        0x2139 => true,
        0x2194..=0x2199 => true,
        0x21A9..=0x21AA => true,
        0x231A..=0x231B => true,
        0x2328 => true,
        0x2388 => true,
        0x23CF => true,
        0x23E9..=0x23F3 => true,
        0x23F8..=0x23FA => true,
        0x24C2 => true,
        0x25AA..=0x25AB => true,
        0x25B6 => true,
        0x25C0 => true,
        0x25FB..=0x25FE => true,
        0x2600..=0x2605 => true,
        0x2607..=0x2612 => true,
        0x2614..=0x2685 => true,
        0x2690..=0x2705 => true,
        0x2708..=0x2712 => true,
        0x2714 => true,
        0x2716 => true,
        0x271D => true,
        0x2721 => true,
        0x2728 => true,
        0x2733..=0x2734 => true,
        0x2744 => true,
        0x2747 => true,
        0x274C => true,
        0x274E => true,
        0x2753..=0x2755 => true,
        0x2757 => true,
        0x2763..=0x2767 => true,
        0x2795..=0x2797 => true,
        0x27A1 => true,
        0x27B0 => true,
        0x27BF => true,
        0x2934..=0x2935 => true,
        0x2B05..=0x2B07 => true,
        0x2B1B..=0x2B1C => true,
        0x2B50 => true,
        0x2B55 => true,
        0x3030 => true,
        0x303D => true,
        0x3297 => true,
        0x3299 => true,
        0x1F000..=0x1F0FF => true,
        0x1F10D..=0x1F10F => true,
        0x1F12F => true,
        0x1F16C..=0x1F171 => true,
        0x1F17E..=0x1F17F => true,
        0x1F18E => true,
        0x1F191..=0x1F19A => true,
        0x1F1AD..=0x1F1E5 => true,
        0x1F201..=0x1F20F => true,
        0x1F21A => true,
        0x1F22F => true,
        0x1F232..=0x1F23A => true,
        0x1F23C..=0x1F23F => true,
        0x1F249..=0x1F3FA => true,
        0x1F400..=0x1F53D => true,
        0x1F546..=0x1F64F => true,
        0x1F680..=0x1F6FF => true,
        0x1F774..=0x1F77F => true,
        0x1F7D5..=0x1F7FF => true,
        0x1F80C..=0x1F80F => true,
        0x1F848..=0x1F84F => true,
        0x1F85A..=0x1F85F => true,
        0x1F888..=0x1F88F => true,
        0x1F8AE..=0x1F8FF => true,
        0x1F90C..=0x1F93A => true,
        0x1F93C..=0x1F945 => true,
        0x1F947..=0x1FFFD => true,
        _ => false,
    }
}

#[no_mangle]
pub extern "C" fn rb_unicode_is_emoji_extended_pictographic(u: u32) -> bool {
    char::try_from(u).unwrap().is_emoji_extended_pictographic()
}
