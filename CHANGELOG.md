# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- A minimal Rust API. Roughly based on [harfbuzz_rs](https://github.com/manuel-rhdt/harfbuzz_rs).

### Ported
- Most of the shaping tests. We store only font files.
- `hb-shape` executable. Most flags are supported.
- `hb_shape_list_shapers`, which returns a static, fixed list now.
- `is_default_ignorable`
- `hb_script_from_iso15924_tag`
- `hb_script_to_iso15924_tag`
- `hb_script_from_string`
- `hb_ucd_script`
- `hb_ucd_compose`
- `hb_ucd_decompose`
- `hb_ucd_general_category`

### Changed
- Replace some header guards with `#pragma once`, otherwise Qt Creator unable to parse them.

### Removed
- autotools/CMake dependency. HarfBuzz will be built via `cc` crate.
- All external dependencies. No icu, glib, freetype, etc.
- `hb_shape_list_shapers`, since we only have `ot` and `fallback`.
- Deprecated methods.
- `hb_unicode_funcs_t`, since we are using only internal Unicode tables.
