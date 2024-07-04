// WARNING: this file was generated by ../scripts/gen-shaping-tests.py

use crate::shape;

#[test]
fn glyph_flags_001() {
    assert_eq!(
        shape(
            "tests/fonts/aots/gpos_chaining1_boundary_f1.otf",
            "\u{0000}\u{0014}\u{0015}\u{0016}\u{0017}\u{0000}",
            "--show-flags --features=\"test\"",
        ),
        ".notdef=0+1500|\
         g20=1+1500|\
         g21=2+1500#1|\
         g22=3+1500#1|\
         g23=4+1500#1|\
         .notdef=5+1500"
    );
}
