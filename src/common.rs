use crate::ffi;

/// A type to represent 4-byte SFNT tags.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Tag(pub(crate) ffi::hb_tag_t);

impl Tag {
    /// Creates a `Tag` from bytes.
    pub const fn from_bytes(bytes: &[u8; 4]) -> Self {
        Tag(((bytes[0] as u32) << 24) | ((bytes[1] as u32) << 16) |
            ((bytes[2] as u32) << 8) | (bytes[3] as u32))
    }

    /// Creates a `Tag` from bytes.
    ///
    /// In case of empty data will return `Tag` set to 0.
    ///
    /// When `bytes` are shorter than 4, will set missing bytes to ` `.
    ///
    /// Data after first 4 bytes is ignored.
    pub fn from_bytes_lossy(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return Tag::from_bytes(&[0, 0, 0, 0]);
        }

        let mut iter = bytes.iter().cloned().chain(std::iter::repeat(b' '));
        Tag::from_bytes(&[
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
        ])
    }

    /// Returns tag as 4-element byte array.
    pub const fn to_bytes(self) -> [u8; 4] {
        [
            (self.0 >> 24 & 0xff) as u8,
            (self.0 >> 16 & 0xff) as u8,
            (self.0 >> 8 & 0xff) as u8,
            (self.0 >> 0 & 0xff) as u8,
        ]
    }

    /// Returns tag as 4-element byte array.
    pub const fn to_chars(self) -> [char; 4] {
        [
            (self.0 >> 24 & 0xff) as u8 as char,
            (self.0 >> 16 & 0xff) as u8 as char,
            (self.0 >> 8 & 0xff) as u8 as char,
            (self.0 >> 0 & 0xff) as u8 as char,
        ]
    }

    /// Returns tag for a default script.
    pub const fn default_script() -> Self {
        Tag::from_bytes(b"DFLT")
    }

    /// Returns tag for a default language.
    pub const fn default_language() -> Self {
        Tag::from_bytes(b"dflt")
    }

    /// Checks if tag is null / `[0, 0, 0, 0]`.
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    /// Returns tag value as `u32` number.
    pub const fn as_u32(&self) -> u32 {
        self.0
    }

    /// Converts tag to lowercase.
    pub fn to_lowercase(&self) -> Self {
        let b = self.to_bytes();
        Tag::from_bytes(&[
            b[0].to_ascii_lowercase(),
            b[1].to_ascii_lowercase(),
            b[2].to_ascii_lowercase(),
            b[3].to_ascii_lowercase(),
        ])
    }

    /// Converts tag to uppercase.
    pub fn to_uppercase(&self) -> Self {
        let b = self.to_bytes();
        Tag::from_bytes(&[
            b[0].to_ascii_uppercase(),
            b[1].to_ascii_uppercase(),
            b[2].to_ascii_uppercase(),
            b[3].to_ascii_uppercase(),
        ])
    }
}

impl std::fmt::Debug for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let b = self.to_chars();
        write!(
            f,
            "Tag({}{}{}{})",
            b.get(0).unwrap_or(&' '),
            b.get(1).unwrap_or(&' '),
            b.get(2).unwrap_or(&' '),
            b.get(3).unwrap_or(&' ')
        )
    }
}

/// Defines the direction in which text is to be read.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Initial, unset direction.
    Invalid,
    /// Text is set horizontally from left to right.
    LeftToRight,
    /// Text is set horizontally from right to left.
    RightToLeft,
    /// Text is set vertically from top to bottom.
    TopToBottom,
    /// Text is set vertically from bottom to top.
    BottomToTop,
}

impl Direction {
    pub(crate) fn to_raw(self) -> ffi::hb_direction_t {
        match self {
            Direction::Invalid     => ffi::HB_DIRECTION_INVALID,
            Direction::LeftToRight => ffi::HB_DIRECTION_LTR,
            Direction::RightToLeft => ffi::HB_DIRECTION_RTL,
            Direction::TopToBottom => ffi::HB_DIRECTION_TTB,
            Direction::BottomToTop => ffi::HB_DIRECTION_BTT,
        }
    }

    pub(crate) fn from_raw(dir: ffi::hb_direction_t) -> Self {
        match dir {
            ffi::HB_DIRECTION_LTR => Direction::LeftToRight,
            ffi::HB_DIRECTION_RTL => Direction::RightToLeft,
            ffi::HB_DIRECTION_TTB => Direction::TopToBottom,
            ffi::HB_DIRECTION_BTT => Direction::BottomToTop,
            _ => Direction::Invalid,
        }
    }
}

impl std::str::FromStr for Direction {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("invalid direction");
        }

        // harfbuzz also matches only the first letter.
        match s.as_bytes()[0].to_ascii_lowercase() {
            b'l' => Ok(Direction::LeftToRight),
            b'r' => Ok(Direction::RightToLeft),
            b't' => Ok(Direction::TopToBottom),
            b'b' => Ok(Direction::BottomToTop),
            _ => Err("invalid direction"),
        }
    }
}


/// A text language.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Language(pub ffi::hb_language_t);

impl Default for Language {
    fn default() -> Language {
        Language(unsafe { ffi::hb_language_get_default() })
    }
}

impl std::fmt::Debug for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Language(\"{}\")", self)
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = unsafe {
            let char_ptr = ffi::hb_language_to_string(self.0);
            if char_ptr.is_null() {
                return Err(std::fmt::Error);
            }
            std::ffi::CStr::from_ptr(char_ptr)
                .to_str()
                .expect("String representation of language is not valid utf8.")
        };
        write!(f, "{}", string)
    }
}

impl std::str::FromStr for Language {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let len = std::cmp::min(s.len(), std::i32::MAX as _) as i32;
        let lang = unsafe { ffi::hb_language_from_string(s.as_ptr() as *mut _, len) };
        if lang.is_null() {
            Err("invalid language")
        } else {
            Ok(Language(lang))
        }
    }
}

// In harfbuzz, despite having `hb_script_t`, script can actually have any tag.
// So we're doing the same.
// The only difference is that `Script` cannot be set to `HB_SCRIPT_INVALID`.
/// A text script.
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Script(pub(crate) Tag);

impl Script {
    pub(crate) const fn from_bytes(bytes: &[u8; 4]) -> Self {
        Script(Tag::from_bytes(bytes))
    }

    /// Converts an ISO 15924 script tag to a corresponding `Script`.
    pub fn from_iso15924_tag(tag: Tag) -> Option<Script> {
        if tag.is_null() {
            return None;
        }

        // Be lenient, adjust case (one capital letter followed by three small letters).
        let tag = Tag((tag.as_u32() & 0xDFDFDFDF) | 0x00202020);

        match &tag.to_bytes() {
            // These graduated from the 'Q' private-area codes, but
            // the old code is still aliased by Unicode, and the Qaai
            // one in use by ICU.
            b"Qaai" => return Some(script::INHERITED),
            b"Qaac" => return Some(script::COPTIC),

            // Script variants from https://unicode.org/iso15924/
            b"Cyrs" => return Some(script::CYRILLIC),
            b"Latf" | b"Latg" => return Some(script::LATIN),
            b"Syre" | b"Syrj" | b"Syrn" => return Some(script::SYRIAC),

            _ => {}
        }

        if tag.as_u32() & 0xE0E0E0E0 == 0x40606060 {
            Some(Script(tag))
        } else {
            Some(script::UNKNOWN)
        }
    }

    /// Returns script's tag.
    pub fn tag(&self) -> Tag {
        self.0
    }
}

impl std::str::FromStr for Script {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tag = Tag::from_bytes_lossy(s.as_bytes());
        Script::from_iso15924_tag(tag).ok_or("invalid script")
    }
}

/// Predefined scripts.
pub mod script {
    #![allow(missing_docs)]

    use crate::Script;

    // Since 1.1
    pub const COMMON: Script                    = Script::from_bytes(b"Zyyy");
    pub const INHERITED: Script                 = Script::from_bytes(b"Zinh");
    pub const ARABIC: Script                    = Script::from_bytes(b"Arab");
    pub const ARMENIAN: Script                  = Script::from_bytes(b"Armn");
    pub const BENGALI: Script                   = Script::from_bytes(b"Beng");
    pub const CYRILLIC: Script                  = Script::from_bytes(b"Cyrl");
    pub const DEVANAGARI: Script                = Script::from_bytes(b"Deva");
    pub const GEORGIAN: Script                  = Script::from_bytes(b"Geor");
    pub const GREEK: Script                     = Script::from_bytes(b"Grek");
    pub const GUJARATI: Script                  = Script::from_bytes(b"Gujr");
    pub const GURMUKHI: Script                  = Script::from_bytes(b"Guru");
    pub const HANGUL: Script                    = Script::from_bytes(b"Hang");
    pub const HAN: Script                       = Script::from_bytes(b"Hani");
    pub const HEBREW: Script                    = Script::from_bytes(b"Hebr");
    pub const HIRAGANA: Script                  = Script::from_bytes(b"Hira");
    pub const KANNADA: Script                   = Script::from_bytes(b"Knda");
    pub const KATAKANA: Script                  = Script::from_bytes(b"Kana");
    pub const LAO: Script                       = Script::from_bytes(b"Laoo");
    pub const LATIN: Script                     = Script::from_bytes(b"Latn");
    pub const MALAYALAM: Script                 = Script::from_bytes(b"Mlym");
    pub const ORIYA: Script                     = Script::from_bytes(b"Orya");
    pub const TAMIL: Script                     = Script::from_bytes(b"Taml");
    pub const TELUGU: Script                    = Script::from_bytes(b"Telu");
    pub const THAI: Script                      = Script::from_bytes(b"Thai");
    // Since 2.0
    pub const TIBETAN: Script                   = Script::from_bytes(b"Tibt");
    // Since 3.0
    pub const BOPOMOFO: Script                  = Script::from_bytes(b"Bopo");
    pub const BRAILLE: Script                   = Script::from_bytes(b"Brai");
    pub const CANADIAN_SYLLABICS: Script        = Script::from_bytes(b"Cans");
    pub const CHEROKEE: Script                  = Script::from_bytes(b"Cher");
    pub const ETHIOPIC: Script                  = Script::from_bytes(b"Ethi");
    pub const KHMER: Script                     = Script::from_bytes(b"Khmr");
    pub const MONGOLIAN: Script                 = Script::from_bytes(b"Mong");
    pub const MYANMAR: Script                   = Script::from_bytes(b"Mymr");
    pub const OGHAM: Script                     = Script::from_bytes(b"Ogam");
    pub const RUNIC: Script                     = Script::from_bytes(b"Runr");
    pub const SINHALA: Script                   = Script::from_bytes(b"Sinh");
    pub const SYRIAC: Script                    = Script::from_bytes(b"Syrc");
    pub const THAANA: Script                    = Script::from_bytes(b"Thaa");
    pub const YI: Script                        = Script::from_bytes(b"Yiii");
    // Since 3.1
    pub const DESERET: Script                   = Script::from_bytes(b"Dsrt");
    pub const GOTHIC: Script                    = Script::from_bytes(b"Goth");
    pub const OLD_ITALIC: Script                = Script::from_bytes(b"Ital");
    // Since 3.2
    pub const BUHID: Script                     = Script::from_bytes(b"Buhd");
    pub const HANUNOO: Script                   = Script::from_bytes(b"Hano");
    pub const TAGALOG: Script                   = Script::from_bytes(b"Tglg");
    pub const TAGBANWA: Script                  = Script::from_bytes(b"Tagb");
    // Since 4.0
    pub const CYPRIOT: Script                   = Script::from_bytes(b"Cprt");
    pub const LIMBU: Script                     = Script::from_bytes(b"Limb");
    pub const LINEAR_B: Script                  = Script::from_bytes(b"Linb");
    pub const OSMANYA: Script                   = Script::from_bytes(b"Osma");
    pub const SHAVIAN: Script                   = Script::from_bytes(b"Shaw");
    pub const TAI_LE: Script                    = Script::from_bytes(b"Tale");
    pub const UGARITIC: Script                  = Script::from_bytes(b"Ugar");
    // Since 4.1
    pub const BUGINESE: Script                  = Script::from_bytes(b"Bugi");
    pub const COPTIC: Script                    = Script::from_bytes(b"Copt");
    pub const GLAGOLITIC: Script                = Script::from_bytes(b"Glag");
    pub const KHAROSHTHI: Script                = Script::from_bytes(b"Khar");
    pub const NEW_TAI_LUE: Script               = Script::from_bytes(b"Talu");
    pub const OLD_PERSIAN: Script               = Script::from_bytes(b"Xpeo");
    pub const SYLOTI_NAGRI: Script              = Script::from_bytes(b"Sylo");
    pub const TIFINAGH: Script                  = Script::from_bytes(b"Tfng");
    // Since 5.0
    pub const UNKNOWN: Script                   = Script::from_bytes(b"Zzzz"); // Script can be Unknown, but not Invalid.
    pub const BALINESE: Script                  = Script::from_bytes(b"Bali");
    pub const CUNEIFORM: Script                 = Script::from_bytes(b"Xsux");
    pub const NKO: Script                       = Script::from_bytes(b"Nkoo");
    pub const PHAGS_PA: Script                  = Script::from_bytes(b"Phag");
    pub const PHOENICIAN: Script                = Script::from_bytes(b"Phnx");
    // Since 5.1
    pub const CARIAN: Script                    = Script::from_bytes(b"Cari");
    pub const CHAM: Script                      = Script::from_bytes(b"Cham");
    pub const KAYAH_LI: Script                  = Script::from_bytes(b"Kali");
    pub const LEPCHA: Script                    = Script::from_bytes(b"Lepc");
    pub const LYCIAN: Script                    = Script::from_bytes(b"Lyci");
    pub const LYDIAN: Script                    = Script::from_bytes(b"Lydi");
    pub const OL_CHIKI: Script                  = Script::from_bytes(b"Olck");
    pub const REJANG: Script                    = Script::from_bytes(b"Rjng");
    pub const SAURASHTRA: Script                = Script::from_bytes(b"Saur");
    pub const SUNDANESE: Script                 = Script::from_bytes(b"Sund");
    pub const VAI: Script                       = Script::from_bytes(b"Vaii");
    // Since 5.2
    pub const AVESTAN: Script                   = Script::from_bytes(b"Avst");
    pub const BAMUM: Script                     = Script::from_bytes(b"Bamu");
    pub const EGYPTIAN_HIEROGLYPHS: Script      = Script::from_bytes(b"Egyp");
    pub const IMPERIAL_ARAMAIC: Script          = Script::from_bytes(b"Armi");
    pub const INSCRIPTIONAL_PAHLAVI: Script     = Script::from_bytes(b"Phli");
    pub const INSCRIPTIONAL_PARTHIAN: Script    = Script::from_bytes(b"Prti");
    pub const JAVANESE: Script                  = Script::from_bytes(b"Java");
    pub const KAITHI: Script                    = Script::from_bytes(b"Kthi");
    pub const LISU: Script                      = Script::from_bytes(b"Lisu");
    pub const MEETEI_MAYEK: Script              = Script::from_bytes(b"Mtei");
    pub const OLD_SOUTH_ARABIAN: Script         = Script::from_bytes(b"Sarb");
    pub const OLD_TURKIC: Script                = Script::from_bytes(b"Orkh");
    pub const SAMARITAN: Script                 = Script::from_bytes(b"Samr");
    pub const TAI_THAM: Script                  = Script::from_bytes(b"Lana");
    pub const TAI_VIET: Script                  = Script::from_bytes(b"Tavt");
    // Since 6.0
    pub const BATAK: Script                     = Script::from_bytes(b"Batk");
    pub const BRAHMI: Script                    = Script::from_bytes(b"Brah");
    pub const MANDAIC: Script                   = Script::from_bytes(b"Mand");
    // Since 6.1
    pub const CHAKMA: Script                    = Script::from_bytes(b"Cakm");
    pub const MEROITIC_CURSIVE: Script          = Script::from_bytes(b"Merc");
    pub const MEROITIC_HIEROGLYPHS: Script      = Script::from_bytes(b"Mero");
    pub const MIAO: Script                      = Script::from_bytes(b"Plrd");
    pub const SHARADA: Script                   = Script::from_bytes(b"Shrd");
    pub const SORA_SOMPENG: Script              = Script::from_bytes(b"Sora");
    pub const TAKRI: Script                     = Script::from_bytes(b"Takr");
    // Since 7.0
    pub const BASSA_VAH: Script                 = Script::from_bytes(b"Bass");
    pub const CAUCASIAN_ALBANIAN: Script        = Script::from_bytes(b"Aghb");
    pub const DUPLOYAN: Script                  = Script::from_bytes(b"Dupl");
    pub const ELBASAN: Script                   = Script::from_bytes(b"Elba");
    pub const GRANTHA: Script                   = Script::from_bytes(b"Gran");
    pub const KHOJKI: Script                    = Script::from_bytes(b"Khoj");
    pub const KHUDAWADI: Script                 = Script::from_bytes(b"Sind");
    pub const LINEAR_A: Script                  = Script::from_bytes(b"Lina");
    pub const MAHAJANI: Script                  = Script::from_bytes(b"Mahj");
    pub const MANICHAEAN: Script                = Script::from_bytes(b"Mani");
    pub const MENDE_KIKAKUI: Script             = Script::from_bytes(b"Mend");
    pub const MODI: Script                      = Script::from_bytes(b"Modi");
    pub const MRO: Script                       = Script::from_bytes(b"Mroo");
    pub const NABATAEAN: Script                 = Script::from_bytes(b"Nbat");
    pub const OLD_NORTH_ARABIAN: Script         = Script::from_bytes(b"Narb");
    pub const OLD_PERMIC: Script                = Script::from_bytes(b"Perm");
    pub const PAHAWH_HMONG: Script              = Script::from_bytes(b"Hmng");
    pub const PALMYRENE: Script                 = Script::from_bytes(b"Palm");
    pub const PAU_CIN_HAU: Script               = Script::from_bytes(b"Pauc");
    pub const PSALTER_PAHLAVI: Script           = Script::from_bytes(b"Phlp");
    pub const SIDDHAM: Script                   = Script::from_bytes(b"Sidd");
    pub const TIRHUTA: Script                   = Script::from_bytes(b"Tirh");
    pub const WARANG_CITI: Script               = Script::from_bytes(b"Wara");
    // Since 8.0
    pub const AHOM: Script                      = Script::from_bytes(b"Ahom");
    pub const ANATOLIAN_HIEROGLYPHS: Script     = Script::from_bytes(b"Hluw");
    pub const HATRAN: Script                    = Script::from_bytes(b"Hatr");
    pub const MULTANI: Script                   = Script::from_bytes(b"Mult");
    pub const OLD_HUNGARIAN: Script             = Script::from_bytes(b"Hung");
    pub const SIGNWRITING: Script               = Script::from_bytes(b"Sgnw");
    // Since 9.0
    pub const ADLAM: Script                     = Script::from_bytes(b"Adlm");
    pub const BHAIKSUKI: Script                 = Script::from_bytes(b"Bhks");
    pub const MARCHEN: Script                   = Script::from_bytes(b"Marc");
    pub const OSAGE: Script                     = Script::from_bytes(b"Osge");
    pub const TANGUT: Script                    = Script::from_bytes(b"Tang");
    pub const NEWA: Script                      = Script::from_bytes(b"Newa");
    // Since 10.0
    pub const MASARAM_GONDI: Script             = Script::from_bytes(b"Gonm");
    pub const NUSHU: Script                     = Script::from_bytes(b"Nshu");
    pub const SOYOMBO: Script                   = Script::from_bytes(b"Soyo");
    pub const ZANABAZAR_SQUARE: Script          = Script::from_bytes(b"Zanb");
    // Since 11.0
    pub const DOGRA: Script                     = Script::from_bytes(b"Dogr");
    pub const GUNJALA_GONDI: Script             = Script::from_bytes(b"Gong");
    pub const HANIFI_ROHINGYA: Script           = Script::from_bytes(b"Rohg");
    pub const MAKASAR: Script                   = Script::from_bytes(b"Maka");
    pub const MEDEFAIDRIN: Script               = Script::from_bytes(b"Medf");
    pub const OLD_SOGDIAN: Script               = Script::from_bytes(b"Sogo");
    pub const SOGDIAN: Script                   = Script::from_bytes(b"Sogd");
    // Since 12.0
    pub const ELYMAIC: Script                   = Script::from_bytes(b"Elym");
    pub const NANDINAGARI: Script               = Script::from_bytes(b"Nand");
    pub const NYIAKENG_PUACHUE_HMONG: Script    = Script::from_bytes(b"Hmnp");
    pub const WANCHO: Script                    = Script::from_bytes(b"Wcho");
}
