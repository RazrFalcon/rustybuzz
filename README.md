# rustybuzz
![Build Status](https://github.com/RazrFalcon/rustybuzz/workflows/Rust/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/rustybuzz.svg)](https://crates.io/crates/rustybuzz)
[![Documentation](https://docs.rs/rustybuzz/badge.svg)](https://docs.rs/rustybuzz)

`rustybuzz` is an attempt to incrementally port [harfbuzz](https://github.com/harfbuzz/harfbuzz)'s
shaping algorithm to Rust.
But while `harfbuzz` does a lot of things (shaping, subsetting, font properties querying, etc.),
`rustybuzz` is *strictly* a shaper.

You can use it already, since we are simply linking the `harfbuzz` statically.
And we're testing `rustybuzz` against the `harfbuzz`'s test suite.

`rustybuzz` must produce exactly the same results as `harfbuzz`.

The current progress can be found at [CHANGELOG.md](./CHANGELOG.md)

Embedded `harfbuzz` version: 2.7.0

## Major changes

- Subsetting removed.
- Malformed fonts will cause an error. HarfBuzz uses fallback/dummy shaper in this case.
- Most of the TrueType and Unicode handling code moved into separate crates.
- rustybuzz doesn't interact with any system libraries and must produce exactly the same
  results on all OS'es and targets.

## Roadmap

- [x] Port OpenType tables:
  - [x] `GDEF`
  - [x] `GPOS`
  - [x] `GSUB`
- [x] Port text normalization (`hb-ot-shape-normalize.cc`, easy).
- [x] Port fallback shaper (`hb-ot-shape-fallback.cc`, easy).
- [x] Port shaper logic (`hb-ot-shape.cc`).
- [ ] Port Apple tables:
  - [ ] Make macos tests OS-independent (ultra hard).
  - [x] `ankr`
  - [x] `feat`
  - [x] `kern`
  - [x] `kerx`
  - [ ] `mort`
  - [ ] `morx`
  - [x] `trak`
- [ ] Remove C++ sources.
- [ ] Remove `unsafe`.

## Performance

At the moment, performance isn't that great. We're 1.5-2x slower than harfbuzz.
Mainly because we have a lot of FFI glue code, which doesn't work well with inlining.
And we have to constanly convert between C++ and Rust types (like `char` for example).
And also, because rustybuzz doesn't support shaping plan caching at the moment.

See [benches/README.md](./benches/README.md) for details.

## License

`rustybuzz` is licensed under the **MIT**.

`harfbuzz` is [licensed](https://github.com/harfbuzz/harfbuzz/blob/master/COPYING) under the **Old MIT**
