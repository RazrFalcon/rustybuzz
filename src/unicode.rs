use crate::CodePoint;

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
