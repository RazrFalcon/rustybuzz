# rustybuzz
![Build Status](https://github.com/RazrFalcon/rustybuzz/workflows/Rust/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/rustybuzz.svg)](https://crates.io/crates/rustybuzz)
[![Documentation](https://docs.rs/rustybuzz/badge.svg)](https://docs.rs/rustybuzz)

`rustybuzz` is a [harfbuzz](https://github.com/harfbuzz/harfbuzz)'s shaping algorithm port to Rust.

Matches `harfbuzz` v2.7.0

## Major changes

- Subsetting removed.
- TrueType parsing has been implemented from scratch, mostly on the
  [ttf_parser](https://github.com/RazrFalcon/ttf_parser) side.
  And while the parsing algorithm is very different, it's not better or worse, just different.
- Malformed fonts will cause an error. HarfBuzz uses fallback/dummy shaper in this case.
- Most of the TrueType and Unicode handling code was moved into separate crates.
- rustybuzz doesn't interact with any system libraries and must produce exactly the same
  results on all OS'es and targets.
- `mort` table is not supported, since it's deprecated by Apple.
- No Arabic fallback shaper, since it requires subsetting.
- No `graphite` library support.

## Performance

At the moment, performance isn't that great. We're 1.5-2x slower than harfbuzz.
Also, rustybuzz doesn't support shaping plan caching at the moment.

See [benches/README.md](./benches/README.md) for details.

## License

`rustybuzz` is licensed under the **MIT**.

`harfbuzz` is [licensed](https://github.com/harfbuzz/harfbuzz/blob/master/COPYING) under the **Old MIT**
