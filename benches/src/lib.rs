#![feature(test)]
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate test;

use rustybuzz::ttf_parser::Tag;

struct CustomVariation {
    tag: rustybuzz::ttf_parser::Tag,
    value: f32,
}

impl Into<rustybuzz::Variation> for CustomVariation {
    fn into(self) -> rustybuzz::Variation {
        rustybuzz::Variation { tag: self.tag, value: self.value }
    }
}

impl Into<harfbuzz_rs::Variation> for CustomVariation {
    fn into(self) -> harfbuzz_rs::Variation {
        harfbuzz_rs::Variation::new(harfbuzz_rs::Tag(self.tag.0), self.value)
    }
}

macro_rules! simple_bench {
    ($name:ident, $font_path:expr, $text_path:expr) => {
        mod $name {
            use super::*;
            use test::Bencher;

            #[bench]
            fn rb(bencher: &mut Bencher) {
                let font_data = std::fs::read($font_path).unwrap();
                let text = std::fs::read_to_string($text_path).unwrap().trim().to_string();
                bencher.iter(|| {
                    test::black_box({
                        let face = rustybuzz::Face::from_slice(&font_data, 0).unwrap();
                        let mut buffer = rustybuzz::UnicodeBuffer::new();
                        buffer.push_str(&text);
                        buffer.reset_clusters();
                        rustybuzz::shape(&face, &[], buffer);
                    });
                })
            }

            #[cfg(feature = "hb")]
            #[bench]
            fn hb(bencher: &mut Bencher) {
                let font_data = std::fs::read($font_path).unwrap();
                let text = std::fs::read_to_string($text_path).unwrap().trim().to_string();
                bencher.iter(|| {
                    test::black_box({
                        let face = harfbuzz_rs::Face::from_bytes(&font_data, 0);
                        let font = harfbuzz_rs::Font::new(face);
                        let buffer = harfbuzz_rs::UnicodeBuffer::new().add_str(&text);
                        harfbuzz_rs::shape(&font, buffer, &[])
                    });
                })
            }
        }
    };

    // Keep in sync with above.
    ($name:ident, $font_path:expr, $text_path:expr, $variations:expr) => {
        mod $name {
            use super::*;
            use test::Bencher;

            #[bench]
            fn rb(bencher: &mut Bencher) {
                let font_data = std::fs::read($font_path).unwrap();
                let text = std::fs::read_to_string($text_path).unwrap().trim().to_string();
                bencher.iter(|| {
                    test::black_box({
                        let mut face = rustybuzz::Face::from_slice(&font_data, 0).unwrap();
                        face.set_variations($variations);
                        let mut buffer = rustybuzz::UnicodeBuffer::new();
                        buffer.push_str(&text);
                        buffer.reset_clusters();
                        rustybuzz::shape(&face, &[], buffer);
                    });
                })
            }

            #[cfg(feature = "hb")]
            #[bench]
            fn hb(bencher: &mut Bencher) {
                let font_data = std::fs::read($font_path).unwrap();
                let text = std::fs::read_to_string($text_path).unwrap().trim().to_string();
                bencher.iter(|| {
                    test::black_box({
                        let face = harfbuzz_rs::Face::from_bytes(&font_data, 0);
                        let mut font = harfbuzz_rs::Font::new(face);
                        font.set_variations($variations);
                        let buffer = harfbuzz_rs::UnicodeBuffer::new().add_str(&text);
                        harfbuzz_rs::shape(&font, buffer, &[])
                    });
                })
            }
        }
    };
}

mod english {
    use super::*;

    simple_bench!(short_zalgo, "fonts/NotoSans-Regular.ttf", "texts/english/short_zalgo.txt");
    simple_bench!(long_zalgo, "fonts/NotoSans-Regular.ttf", "texts/english/long_zalgo.txt");
    simple_bench!(word_1, "fonts/NotoSans-Regular.ttf", "texts/english/word_1.txt");
    simple_bench!(word_2, "fonts/NotoSans-Regular.ttf", "texts/english/word_2.txt");
    simple_bench!(word_3, "fonts/NotoSans-Regular.ttf", "texts/english/word_3.txt");
    simple_bench!(word_4, "fonts/NotoSans-Regular.ttf", "texts/english/word_4.txt");
    simple_bench!(sentence_1, "fonts/NotoSans-Regular.ttf", "texts/english/sentence_1.txt");
    simple_bench!(sentence_2, "fonts/NotoSans-Regular.ttf", "texts/english/sentence_2.txt");
    simple_bench!(paragraph_short, "fonts/NotoSans-Regular.ttf", "texts/english/paragraph_short.txt");
    simple_bench!(paragraph_medium, "fonts/NotoSans-Regular.ttf", "texts/english/paragraph_medium.txt");
    simple_bench!(paragraph_long, "fonts/NotoSans-Regular.ttf", "texts/english/paragraph_long.txt");

    simple_bench!(sentence_mono, "fonts/RobotoMono-Regular.ttf", "texts/english/sentence_1.txt");
    simple_bench!(paragraph_long_mono, "fonts/RobotoMono-Regular.ttf", "texts/english/paragraph_long.txt");

    use crate::CustomVariation;

    const WIDTH_VAR: CustomVariation = CustomVariation {
        tag: Tag::from_bytes(b"wdth"),
        value: 50.0,
    };

    const WIDTH_VAR_DEFAULT: CustomVariation = CustomVariation {
        tag: Tag::from_bytes(b"wdth"),
        value: 100.0,
    };

    simple_bench!(variations, "fonts/NotoSans-VariableFont.ttf", "texts/english/paragraph_long.txt", &[WIDTH_VAR.into()]);
    simple_bench!(variations_default, "fonts/NotoSans-VariableFont.ttf", "texts/english/paragraph_long.txt", &[WIDTH_VAR_DEFAULT.into()]);

    #[cfg(target_os = "macos")]
    simple_bench!(aat_word_1, "/System/Library/Fonts/Supplemental/Zapfino.ttf", "texts/english/word_1.txt");
    #[cfg(target_os = "macos")]
    simple_bench!(aat_sentence_1, "/System/Library/Fonts/Supplemental/Zapfino.ttf", "texts/english/sentence_1.txt");
    #[cfg(target_os = "macos")]
    simple_bench!(aat_paragraph_long, "/System/Library/Fonts/Supplemental/Zapfino.ttf", "texts/english/paragraph_long.txt");
}

mod arabic {
    use super::*;

    simple_bench!(word_1, "fonts/NotoSansArabic-Regular.ttf", "texts/arabic/word_1.txt");
    simple_bench!(word_2, "fonts/NotoSansArabic-Regular.ttf", "texts/arabic/word_2.txt");
    simple_bench!(word_3, "fonts/NotoSansArabic-Regular.ttf", "texts/arabic/word_3.txt");
    simple_bench!(sentence_1, "fonts/NotoSansArabic-Regular.ttf", "texts/arabic/sentence_1.txt");
    simple_bench!(sentence_2, "fonts/NotoSansArabic-Regular.ttf", "texts/arabic/sentence_2.txt");
    simple_bench!(paragraph_short, "fonts/NotoSansArabic-Regular.ttf", "texts/arabic/paragraph_short.txt");
    simple_bench!(paragraph_medium, "fonts/NotoSansArabic-Regular.ttf", "texts/arabic/paragraph_medium.txt");
    simple_bench!(paragraph_long, "fonts/NotoSansArabic-Regular.ttf", "texts/arabic/paragraph_long.txt");
}

mod khmer {
    use super::*;

    simple_bench!(word_1, "fonts/NotoSansKhmer-Regular.ttf", "texts/khmer/word_1.txt");
    simple_bench!(word_2, "fonts/NotoSansKhmer-Regular.ttf", "texts/khmer/word_2.txt");
    simple_bench!(word_3, "fonts/NotoSansKhmer-Regular.ttf", "texts/khmer/word_3.txt");
    simple_bench!(sentence_1, "fonts/NotoSansKhmer-Regular.ttf", "texts/khmer/sentence_1.txt");
    simple_bench!(sentence_2, "fonts/NotoSansKhmer-Regular.ttf", "texts/khmer/sentence_2.txt");
    simple_bench!(paragraph_medium, "fonts/NotoSansKhmer-Regular.ttf", "texts/khmer/paragraph_medium.txt");
    simple_bench!(paragraph_long_1, "fonts/NotoSansKhmer-Regular.ttf", "texts/khmer/paragraph_long_1.txt");
    simple_bench!(paragraph_long_2, "fonts/NotoSansKhmer-Regular.ttf", "texts/khmer/paragraph_long_2.txt");

    #[cfg(target_os = "macos")]
    simple_bench!(aat_word_1, "/System/Library/Fonts/Supplemental/Khmer MN.ttc", "texts/khmer/word_1.txt");
    #[cfg(target_os = "macos")]
    simple_bench!(aat_sentence_1, "/System/Library/Fonts/Supplemental/Khmer MN.ttc", "texts/khmer/sentence_1.txt");
    #[cfg(target_os = "macos")]
    simple_bench!(aat_paragraph_long_1, "/System/Library/Fonts/Supplemental/Khmer MN.ttc", "texts/khmer/paragraph_long_1.txt");
}

mod hebrew {
    use super::*;

    simple_bench!(word_1, "fonts/NotoSansHebrew-Regular.ttf", "texts/hebrew/word_1.txt");
    simple_bench!(word_2, "fonts/NotoSansHebrew-Regular.ttf", "texts/hebrew/word_2.txt");
    simple_bench!(sentence_1, "fonts/NotoSansHebrew-Regular.ttf", "texts/hebrew/sentence_1.txt");
    simple_bench!(sentence_2, "fonts/NotoSansHebrew-Regular.ttf", "texts/hebrew/sentence_2.txt");
    simple_bench!(paragraph_medium, "fonts/NotoSansHebrew-Regular.ttf", "texts/hebrew/paragraph_medium.txt");
    simple_bench!(paragraph_long_1, "fonts/NotoSansHebrew-Regular.ttf", "texts/hebrew/paragraph_long_1.txt");
    simple_bench!(paragraph_long_2, "fonts/NotoSansHebrew-Regular.ttf", "texts/hebrew/paragraph_long_2.txt");
}

mod myanmar {
    use super::*;

    simple_bench!(word_1, "fonts/NotoSansMyanmar-Regular.ttf", "texts/myanmar/word_1.txt");
    simple_bench!(word_2, "fonts/NotoSansMyanmar-Regular.ttf", "texts/myanmar/word_2.txt");
    simple_bench!(sentence_1, "fonts/NotoSansMyanmar-Regular.ttf", "texts/myanmar/sentence_1.txt");
    simple_bench!(sentence_2, "fonts/NotoSansMyanmar-Regular.ttf", "texts/myanmar/sentence_2.txt");
    simple_bench!(paragraph_short, "fonts/NotoSansMyanmar-Regular.ttf", "texts/myanmar/paragraph_short.txt");
    simple_bench!(paragraph_medium, "fonts/NotoSansMyanmar-Regular.ttf", "texts/myanmar/paragraph_medium.txt");
    simple_bench!(paragraph_long, "fonts/NotoSansMyanmar-Regular.ttf", "texts/myanmar/paragraph_long.txt");

    #[cfg(target_os = "macos")]
    simple_bench!(aat_word_1, "/System/Library/Fonts/Supplemental/Myanmar MN.ttc", "texts/myanmar/word_1.txt");
    #[cfg(target_os = "macos")]
    simple_bench!(aat_sentence_1, "/System/Library/Fonts/Supplemental/Myanmar MN.ttc", "texts/myanmar/sentence_1.txt");
    #[cfg(target_os = "macos")]
    simple_bench!(aat_paragraph_long, "/System/Library/Fonts/Supplemental/Myanmar MN.ttc", "texts/myanmar/paragraph_long.txt");
}

mod hindi {
    use super::*;

    simple_bench!(word_1, "fonts/NotoSansDevanagari-Regular.ttf", "texts/hindi/word_1.txt");
    simple_bench!(word_2, "fonts/NotoSansDevanagari-Regular.ttf", "texts/hindi/word_2.txt");
    simple_bench!(sentence_1, "fonts/NotoSansDevanagari-Regular.ttf", "texts/hindi/sentence_1.txt");
    simple_bench!(sentence_2, "fonts/NotoSansDevanagari-Regular.ttf", "texts/hindi/sentence_2.txt");
    simple_bench!(paragraph_short, "fonts/NotoSansDevanagari-Regular.ttf", "texts/hindi/paragraph_short.txt");
    simple_bench!(paragraph_medium, "fonts/NotoSansDevanagari-Regular.ttf", "texts/hindi/paragraph_medium.txt");
    simple_bench!(paragraph_long, "fonts/NotoSansDevanagari-Regular.ttf", "texts/hindi/paragraph_long.txt");

    #[cfg(target_os = "macos")]
    simple_bench!(aat_word, "/System/Library/Fonts/Supplemental/Devanagari Sangam MN.ttc", "texts/hindi/word_1.txt");
    #[cfg(target_os = "macos")]
    simple_bench!(aat_sentence, "/System/Library/Fonts/Supplemental/Devanagari Sangam MN.ttc", "texts/hindi/sentence_1.txt");
    #[cfg(target_os = "macos")]
    simple_bench!(aat_paragraph_long, "/System/Library/Fonts/Supplemental/Devanagari Sangam MN.ttc", "texts/hindi/paragraph_long.txt");
}

mod thai {
    use super::*;

    simple_bench!(word_1, "fonts/NotoSansThai-Regular.ttf", "texts/thai/word_1.txt");
    simple_bench!(word_2, "fonts/NotoSansThai-Regular.ttf", "texts/thai/word_2.txt");
    simple_bench!(sentence_1, "fonts/NotoSansThai-Regular.ttf", "texts/thai/sentence_1.txt");
    simple_bench!(paragraph_short, "fonts/NotoSansThai-Regular.ttf", "texts/thai/paragraph_short.txt");
    simple_bench!(paragraph_medium, "fonts/NotoSansThai-Regular.ttf", "texts/thai/paragraph_medium.txt");
    simple_bench!(paragraph_long, "fonts/NotoSansThai-Regular.ttf", "texts/thai/paragraph_long.txt");
}
