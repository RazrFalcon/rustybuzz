# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [0.4.0] - 2021-06-27
### Added
- `Face::from_face`, so you can create `rustybuzz::Face` directly from `ttf_parser::Face`.
  Thanks to [@lain-dono](https://github.com/lain-dono)
- `no_std` support thanks to [@CryZe](https://github.com/CryZe).
- `GlyphInfo::unsafe_to_break` thanks to [@glowcoil](https://github.com/glowcoil).

### Changed
- Sync with harfbuzz 2.7.1
- Rename `GlyphInfo.codepoint` into `GlyphInfo.glyph_id` to remove confusion.

## [0.3.0] - 2020-12-05
### Ported
- Everything! 🎉
- Tables: `GSUB`, `GPOS`, `GDEF`, `ankr`, `feat`, `kern`, `kerx`, `morx`, `trak`.
- Main shaping logic.
- `hb_shape_plan_t` and `hb_ot_shape_plan_t`
- `hb_ot_map_t`
- `hb_ot_complex_shaper_t`
- OpenType layout (GSUB, GPOS).
- AAT layout.
- Normalization.
- Fallback shaper.
- Kerning.

### Changed
- Rename `Font` to `Face`.

Most of the changes in this release were made by [laurmaedje](https://github.com/laurmaedje).

## [0.2.0] - 2020-07-25
### Ported
- All complex shapers.
- Tables: `CBDT`, `CFF`, `CFF2`, `HVAR`, `MVAR`, `OS/2`, `SVG`, `VORG`, `VVAR`,
  `avar`, `cmap`, `fvar`, `glyf`, `gvar`, `hhea`, `hmtx`, `post`, `sbix`, `vhea`, `vmtx`.
- `hb_buffer_t`
- `hb_script_t`
- `hb_feature_t`
- `hb_variation_t`
- `hb_language_t`
- `hb_font_t`
- `hb-ot-metrics`
- Unicode functions and tables.
- Buffer serialization.

### Changed
- Update to HarfBuzz 2.7.0
- Rename `Font::from_data` into `Font::from_slice`.
- Font is parsed via `ttf-parser` first.
  And if the parsing fails, the `Font` will not be created.
  `harfbuzz` allows malformed fonts.

### Removed
- `hb_font_funcs_t`. Only the embedded TrueType implementation is used.
- `hb_unicode_funcs_t`. Only the embedded Unicode implementation is used.
- `Font::set_scale`/`hb_font_set_scale`/`--font-size`. Shaping is always in font units now.
  This simplifies the code quite a lot.
- Shaping plan caching.
- Fallback shaper.
- Unused `hdmx` table.

## [0.1.1] - 2020-07-04
### Fixed
- Compilation with an old XCode.

## 0.1.0 - 2020-07-04
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

[Unreleased]: https://github.com/RazrFalcon/rustybuzz/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/RazrFalcon/rustybuzz/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/RazrFalcon/rustybuzz/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/RazrFalcon/rustybuzz/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/RazrFalcon/rustybuzz/compare/v0.1.0...v0.1.1
