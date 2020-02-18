# rustybuzz
[![Build Status](https://travis-ci.org/RazrFalcon/rustybuzz.svg?branch=master)](https://travis-ci.org/RazrFalcon/rustybuzz)

`rustybuzz` is an attempt to incrementally port [harfbuzz](https://github.com/harfbuzz/harfbuzz) to Rust.
But while `harfbuzz` does a lot of things (shaping, subseting, font querying, etc.),
`rustybuzz` is *strictly* an OpenType shaper.

You can use it already, since we simply linking `harfbuzz` statically.
And we're testing `rustybuzz` against `harfbuzz` test suite.

Embedded `harfbuzz` version: 2.6.4

## Changes

- Subsetting is out of scope and removed.
- Malformed font is an error now.
  `harfbuzz` accepts malformed fonts, but doesn't do shaping in this case.
- `harfbuzz` configured to not depend on system libraries, like glib, coretext, freetype, icu.
  So it relies only on internal implementation, which should be enough in most cases.

## Notes about the port

Thanks to Cargo, `rustybuzz` is pretty modular, unlike `harfbuzz`, which is basically a monolith.
And it can be roughly split into these modules: shaping, subsetting, font parsing, Unicode functions,
containers and utilities.
While `rustybuzz` implements only the *shaping*. Font parsing is handled by the
[ttf-parser](https://github.com/RazrFalcon/ttf-parser), which is not based on `harfbuzz` and
has its own architecture. Unicode functions also handled by external crates.
And most of the containers and utilities were already implemented in the Rust std.

## License

*rustybuzz* is licensed under the **MIT**.

`harfbuzz` is [licensed](https://github.com/harfbuzz/harfbuzz/blob/master/COPYING) under the **Old MIT**
