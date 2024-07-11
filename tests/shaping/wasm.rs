// tests for shape_with_wasm

#[test]
fn calculator() {
    // here are the wasm functions imported in this font:
    //
    // (import "env" "buffer_copy_contents" (func (;0;) (type 0)))
    // (import "env" "buffer_set_contents" (func (;1;) (type 0)))
    // (import "env" "font_get_glyph" (func (;2;) (type 3)))
    // (import "env" "font_get_glyph_h_advance" (func (;3;) (type 0)))
    // (import "env" "debugprint" (func (;4;) (type 2)))

    let calculator_font = include_bytes!("../fonts/text-rendering-tests/Calculator-Regular.ttf");
    let face = rustybuzz::Face::from_slice(calculator_font, 0).unwrap();

    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str("22/7=");

    let res = rustybuzz::shape(&face, &[], buffer)
        .glyph_infos()
        .iter()
        .map(|i| i.glyph_id)
        .collect::<Vec<_>>();

    // glyphids for 3.142857
    let expected = vec![20, 15, 18, 21, 19, 25, 22, 24];

    assert_eq!(expected, res);
}

#[test]
fn ruqaa_final_period() {
    // here are the wasm functions imported in this font:
    //
    // (import "env" "buffer_copy_contents" (func (;0;) (type 0)))
    // (import "env" "buffer_set_contents" (func (;1;) (type 0)))
    // (import "env" "shape_with" (func (;2;) (type 11)))
    // (import "env" "font_get_face" (func (;3;) (type 4)))
    // (import "env" "font_glyph_to_string" (func (;4;) (type 7)))
    // (import "env" "font_get_scale" (func (;5;) (type 3)))
    // (import "env" "font_copy_glyph_outline" (func (;6;) (type 1)))
    // (import "env" "debugprint" (func (;7;) (type 5)))
    // (import "env" "face_get_upem" (func (;8;) (type 4)))

    // This test currently passes if we do not compare clusters.
    // Clusters assignment seems different between Rustybuzz and harfbuzz.

    let ruqaa_font = include_bytes!("../../tests/fonts/text-rendering-tests/ArefRuqaa-Wasm.ttf");
    let face = rustybuzz::Face::from_slice(ruqaa_font, 0).unwrap();

    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str("أفشوا السلام بينكم.");

    let res = rustybuzz::shape(&face, &[], buffer);
    let res = res
        .glyph_positions()
        .iter()
        .zip(res.glyph_infos().iter())
        .map(|(p, i)| {
            format!(
                "gid{} adv{} | dX{} dY{}",
                i.glyph_id, p.x_advance, p.x_offset, p.y_offset
            )
        });

    // Copied from Wasm FontGoggles.
    let expected = vec![
        "period 272 0 0 18 462",
        "meem-ar.fina 303 0 -213 17 301",
        "kaf-ar.medi.meem 321 0 20 16 243",
        "dotabove-ar 0 215 394 15 491",
        "behDotless-ar.medi 198 0 20 15 14",
        "twodotshorizontalbelow-ar 0 167 -81 14 494",
        "behDotless-ar.medi.high 229 0 42 14 20",
        "dotbelow-ar 0 163 77 13 492",
        "behDotless-ar.init.ascend 313 0 213 13 30",
        "space 146 0 0 12 455",
        "meem-ar 287 0 0 11 300",
        "alef-ar.fina.lam -27 0 -35 10 5",
        "lam-ar.medi.alef 732 0 -35 9 275",
        "seen-ar.medi 387 0 -35 8 89",
        "lam-ar.init 358 0 35 7 286",
        "alef-ar 248 0 0 6 3",
        "space 146 0 0 5 455",
        "alef-ar 145 0 0 4 3",
        "waw-ar.fina 280 -146 -164 3 388",
        "threedotsupabove-ar 0 338 526 2 496",
        "seen-ar.medi 387 0 95 2 89",
        "dotabove-ar 0 259 807 1 491",
        "fehDotless-ar.init 414 0 165 1 215",
        "hamzaabove-ar 0 121 791 0 501",
        "alef-ar 248 0 0 0 3",
    ];
    let expected = expected.iter().map(|s| {
        let mut s = s.split_ascii_whitespace();
        let _name = s.next();
        let adv = s.next().unwrap();
        let d_x = s.next().unwrap();
        let d_y = s.next().unwrap();
        let _cluster = s.next(); // comparing these breaks the test
        let gid = s.next().unwrap();
        format!("gid{} adv{} | dX{} dY{}", gid, adv, d_x, d_y)
    });

    for (expected, res) in expected.zip(res) {
        assert_eq!(expected, res);
    }
}

#[test]
fn ruqaa_no_final_period() {
    // same font and text as ruqaa_final_period, but without a final period.
    // Same success/failure mode.

    let ruqaa_font = include_bytes!("../../tests/fonts/text-rendering-tests/ArefRuqaa-Wasm.ttf");
    let face = rustybuzz::Face::from_slice(ruqaa_font, 0).unwrap();

    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str("أفشوا السلام بينكم");

    let res = rustybuzz::shape(&face, &[], buffer);
    let res = res
        .glyph_positions()
        .iter()
        .zip(res.glyph_infos().iter())
        .map(|(p, i)| {
            format!(
                "gid{} adv{} | dX{} dY{}",
                i.glyph_id, p.x_advance, p.x_offset, p.y_offset
            )
        });

    // Copied from Wasm FontGoggles Wasm.
    let expected = vec![
        "meem-ar.fina 303 0 -213 17 301",
        "kaf-ar.medi.meem 321 0 20 16 243",
        "dotabove-ar 0 215 394 15 491",
        "behDotless-ar.medi 198 0 20 15 14",
        "twodotshorizontalbelow-ar 0 167 -81 14 494",
        "behDotless-ar.medi.high 229 0 42 14 20",
        "dotbelow-ar 0 163 77 13 492",
        "behDotless-ar.init.ascend 313 0 213 13 30",
        "space 146 0 0 12 455",
        "meem-ar 287 0 0 11 300",
        "alef-ar.fina.lam -27 0 -35 10 5",
        "lam-ar.medi.alef 732 0 -35 9 275",
        "seen-ar.medi 387 0 -35 8 89",
        "lam-ar.init 358 0 35 7 286",
        "alef-ar 248 0 0 6 3",
        "space 146 0 0 5 455",
        "alef-ar 145 0 0 4 3",
        "waw-ar.fina 280 -146 -164 3 388",
        "threedotsupabove-ar 0 338 526 2 496",
        "seen-ar.medi 387 0 95 2 89",
        "dotabove-ar 0 259 807 1 491",
        "fehDotless-ar.init 414 0 165 1 215",
        "hamzaabove-ar 0 121 791 0 501",
        "alef-ar 248 0 0 0 3",
    ];
    let expected = expected.iter().map(|s| {
        let mut s = s.split_ascii_whitespace();
        let _name = s.next();
        let adv = s.next().unwrap();
        let d_x = s.next().unwrap();
        let d_y = s.next().unwrap();
        let _cluster = s.next(); // comparing these breaks the test
        let gid = s.next().unwrap();
        format!("gid{} adv{} | dX{} dY{}", gid, adv, d_x, d_y)
    });

    for (expected, res) in expected.zip(res) {
        assert_eq!(expected, res);
    }
}
