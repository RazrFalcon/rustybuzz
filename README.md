# rustybuzz
[![Build Status](https://travis-ci.org/RazrFalcon/rustybuzz.svg?branch=master)](https://travis-ci.org/RazrFalcon/rustybuzz)
[![Crates.io](https://img.shields.io/crates/v/rustybuzz.svg)](https://crates.io/crates/rustybuzz)
[![Documentation](https://docs.rs/rustybuzz/badge.svg)](https://docs.rs/rustybuzz)

`rustybuzz` is an attempt to incrementally port [harfbuzz](https://github.com/harfbuzz/harfbuzz)'s
shaping algorithm to Rust.
But while `harfbuzz` does a lot of things (shaping, subsetting, font properties querying, etc.),
`rustybuzz` is *strictly* a shaper.

You can use it already, since we are simply linking the `harfbuzz` statically.
And we're testing `rustybuzz` against the `harfbuzz`'s test suite.

The final goal is to produce exactly the same results as harfbuzz.

The current progress can be found at [CHANGELOG.md](./CHANGELOG.md)

Embedded `harfbuzz` version: 2.6.8

## Major changes

- Subsetting removed.
- Malformed fonts will cause an error. HarfBuzz uses fallback/dummy shaper in this case.
- Most of the TrueType and Unicode handling code moved into separate crates.
- rustybuzz doesn't interact with any system libraries and must produce exactly the same
  results on all OS'es and targets.

## Prior work

This is mine yet another attempt to port harfbuzz to Rust.
The previous attempt can be found at [rustybuzz-old](https://github.com/RazrFalcon/rustybuzz-old).

This time I'm focusing on delivering a well-tested bindings for a stripped-down harfbuzz fork
with some parts reimplemented in Rust.

The problem is that harfbuzz code is very interconnected and it's hard to swap out random parts.
Also, despite having almost 1800 tests, there are still a lot of white spots which I will
try to fill first. Since it's way too easy to miss some important edge-case.

## License

`rustybuzz` is licensed under the **MIT**.

`harfbuzz` is [licensed](https://github.com/harfbuzz/harfbuzz/blob/master/COPYING) under the **Old MIT**
