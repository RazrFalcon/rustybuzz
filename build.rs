fn main() {
    let mut build = cc::Build::new();
    build.cpp(true);

    if build.get_compiler().is_like_msvc() {
        // From harfbuzz/meson.build
        build.flag("/wd4018"); // implicit signed/unsigned conversion
        build.flag("/wd4146"); // unary minus on unsigned (beware INT_MIN)
        build.flag("/wd4244"); // lossy type conversion (e.g. double -> int)
        build.flag("/wd4305"); // truncating type conversion (e.g. double -> float)

        // Required by hb-algs.hh
        build.flag("/std:c++14");
    } else {
        build.flag("-std=c++11");
        build.flag_if_supported("-fno-rtti");
        build.flag_if_supported("-fno-exceptions");
        build.flag_if_supported("-fno-threadsafe-statics");
        build.flag_if_supported("-fvisibility-inlines-hidden");
    }

    build.file("harfbuzz/src/hb-aat-layout.cc");
    build.file("harfbuzz/src/hb-aat-map.cc");
    build.file("harfbuzz/src/hb-blob.cc");
    build.file("harfbuzz/src/hb-buffer.cc");
    build.file("harfbuzz/src/hb-common.cc");
    build.file("harfbuzz/src/hb-face.cc");
    build.file("harfbuzz/src/hb-font.cc");
    build.file("harfbuzz/src/hb-map.cc");
    build.file("harfbuzz/src/hb-ot-face.cc");
    build.file("harfbuzz/src/hb-ot-font.cc");
    build.file("harfbuzz/src/hb-ot-layout.cc");
    build.file("harfbuzz/src/hb-ot-map.cc");
    build.file("harfbuzz/src/hb-ot-shape.cc");
    build.file("harfbuzz/src/hb-ot-shape-complex-default.cc");
    build.file("harfbuzz/src/hb-ot-shape-complex-indic-table.cc");
    build.file("harfbuzz/src/hb-ot-shape-complex-myanmar.cc");
    build.file("harfbuzz/src/hb-ot-shape-complex-use.cc");
    build.file("harfbuzz/src/hb-ot-shape-complex-use-table.cc");
    build.file("harfbuzz/src/hb-ot-shape-complex-vowel-constraints.cc");
    build.file("harfbuzz/src/hb-ot-shape-fallback.cc");
    build.file("harfbuzz/src/hb-ot-shape-normalize.cc");
    build.file("harfbuzz/src/hb-ot-tag.cc");
    build.file("harfbuzz/src/hb-set.cc");
    build.file("harfbuzz/src/hb-shape.cc");
    build.file("harfbuzz/src/hb-shape-plan.cc");
    build.file("harfbuzz/src/hb-static.cc");
    build.include("harfbuzz/src");
    build.compile("librustybuzz.a");
}
