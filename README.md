# rustybuzz
[![Build Status](https://travis-ci.org/RazrFalcon/rustybuzz.svg?branch=master)](https://travis-ci.org/RazrFalcon/rustybuzz)

`rustybuzz` is an attempt to incrementally port [harfbuzz](https://github.com/harfbuzz/harfbuzz) to Rust.

You can use it already, since we simply linking `hardbuzz` statically.
And we're testing `rustybuzz` against `harfbuzz` test suite.

You can find more details about porting status in the [changelog](./CHANGELOG.md).

Embedded `harfbuzz` version: 2.6.4

## Limitations

- harfbuzz configured to not depend on system libraries, like glib, coretext, freetype, icu.
  So it relies only on internal implementation, which should be enough in most cases.
- Subseting is a second class citizen. It's not removed, but not supported yet.

## License

*rustybuzz* is licensed under the **MIT**.

*HarfBuzz* is [licensed](https://github.com/harfbuzz/harfbuzz/blob/master/COPYING) under the **Old MIT**
