use std::convert::TryFrom;

pub use unicode_general_category::GeneralCategory;

use crate::{Script, CodePoint};

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
#[allow(missing_docs)]
pub extern "C" fn rb_is_default_ignorable(ch: u32) -> i32 {
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
#[allow(missing_docs)]
pub extern "C" fn rb_ucd_script(ch: u32) -> u32 {
    script_from_char(char::try_from(ch).unwrap()).tag().as_u32()
}

#[no_mangle]
#[allow(missing_docs)]
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
#[allow(missing_docs)]
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
#[allow(missing_docs)]
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
}

impl CharExt for char {
    fn general_category(&self) -> GeneralCategory {
        unicode_general_category::get_general_category(*self)
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
