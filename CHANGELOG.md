# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Ported
- `hb_feature_t`

### Removed
- `hb_font_funcs_t`. Only the embedded TrueType implementation is used.
- `hb_unicode_funcs_t`. Only the embedded Unicode implementation is used.
- Shaping plan caching.
- Fallback shaper.
- Unused `hdmx` table.

## [0.1.1] - 2020-07-04
### Fixed
- Compilation with an old XCode.

## [0.1.0] - 2020-07-04
At this point, this is just a simple Rust bindings to a stripped down harfbuzz.

### Added
- An absolute minimum Rust API.
- harfbuzz's shaping test suite had been ported to Rust.

### Changed
- harfbuzz source code was reformatted using clang-format.

### Removed
- Subsetting. This is probably a bit controversial, but I want to port only the shaper for now.
  This is also removes around 7000 LOC.
- Arabic fallback shaper. Since it requires subsetting.
- Unused TrueType tables: BASE, COLR, CPAL, JSTF, MATH, STAT, bsln, fdsc, gasp, just, lcar, ltag, meta, name, opbd.
- All external dependencies: coretext, directwrite, freetype, gdi, glib, gobject, graphite, icu, uniscribe.
  Embedded harfbuzz relies only on internal TrueType implementation.
- Most of the non-shaping harfbuzz API.

[Unreleased]: https://github.com/RazrFalcon/rustybuzz/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/RazrFalcon/resvg/compare/v0.1.0...v0.1.1
