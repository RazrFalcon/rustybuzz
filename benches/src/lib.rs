#![feature(test)]

extern crate test;

#[bench]
fn shape_latin_rb(bencher: &mut test::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    bencher.iter(|| {
        test::black_box({
            let font = rustybuzz::Font::from_slice(&font_data, 0).unwrap();
            let mut buffer = rustybuzz::UnicodeBuffer::new();
            buffer.push_str("Some dummy text.");
            buffer.reset_clusters();
             rustybuzz::shape(&font, &[], buffer)
         });
    })
}

#[bench]
fn shape_latin_hb(bencher: &mut test::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    bencher.iter(|| {
        test::black_box({
            // Yes, we are creating face and font objects each time intentionally,
            // to prevent shape plan caching, which was removed in rustybuzz.
            let face = harfbuzz_rs::Face::from_bytes(&font_data, 0);
            let font = harfbuzz_rs::Font::new(face);
            let buffer = harfbuzz_rs::UnicodeBuffer::new().add_str("Some dummy text.");
            harfbuzz_rs::shape(&font, buffer, &[])
        });
    })
}

#[bench]
fn shape_zalgo_rb(bencher: &mut test::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let text = std::fs::read_to_string("zalgo.txt").unwrap().trim().to_string();
    bencher.iter(|| {
        test::black_box({
            let font = rustybuzz::Font::from_slice(&font_data, 0).unwrap();
            let mut buffer = rustybuzz::UnicodeBuffer::new();
            buffer.push_str(&text);
            buffer.reset_clusters();
            rustybuzz::shape(&font, &[], buffer)
        });
    })
}

#[bench]
fn shape_zalgo_hb(bencher: &mut test::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let text = std::fs::read_to_string("zalgo.txt").unwrap().trim().to_string();
    bencher.iter(|| {
        test::black_box({
            let face = harfbuzz_rs::Face::from_bytes(&font_data, 0);
            let font = harfbuzz_rs::Font::new(face);
            let buffer = harfbuzz_rs::UnicodeBuffer::new().add_str(&text);
            harfbuzz_rs::shape(&font, buffer, &[])
        });
    })
}

#[bench]
fn shape_arabic_rb(bencher: &mut test::Bencher) {
    let font_data = std::fs::read("fonts/Amiri-Regular.ttf").unwrap();
    let text = std::fs::read_to_string("arabic.txt").unwrap().trim().to_string();
    bencher.iter(|| {
        test::black_box({
            let font = rustybuzz::Font::from_slice(&font_data, 0).unwrap();
            let mut buffer = rustybuzz::UnicodeBuffer::new();
            buffer.push_str(&text);
            buffer.reset_clusters();
            rustybuzz::shape(&font, &[], buffer)
        });
    })
}

#[bench]
fn shape_arabic_hb(bencher: &mut test::Bencher) {
    let font_data = std::fs::read("fonts/Amiri-Regular.ttf").unwrap();
    let text = std::fs::read_to_string("arabic.txt").unwrap().trim().to_string();
    bencher.iter(|| {
        test::black_box({
            let face = harfbuzz_rs::Face::from_bytes(&font_data, 0);
            let font = harfbuzz_rs::Font::new(face);
            let buffer = harfbuzz_rs::UnicodeBuffer::new().add_str(&text);
            harfbuzz_rs::shape(&font, buffer, &[])
        });
    })
}
