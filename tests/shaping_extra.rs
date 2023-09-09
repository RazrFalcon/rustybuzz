mod shaping_impl;
use shaping_impl::shape;

// Some tests are known to have a different, but still valid output
// and we gather them here.

#[test]
fn fallback_positioning_001() {
    assert_eq!(
        shape(
            "tests/fonts/in-house/8228d035fcd65d62ec9728fb34f42c63be93a5d3.ttf",
            "\u{0078}\u{0301}\u{0058}\u{0301}",
            "",
        ),
        "x=0+1030|\
         acutecomb=0@-20,-27+0|\
         X=2+1295|\
         acutecomb=2@-152,321+0"
    );
}
