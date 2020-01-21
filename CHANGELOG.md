# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- A minimal Rust API. Roughly based on [harfbuzz_rs](https://github.com/manuel-rhdt/harfbuzz_rs).

### Ported
- Malformed font is an error now.
- Most of the shaping tests. We store only font files.
- `hb-shape` executable. Most flags are supported.
- `cmap`, `CFF`, `CFF2`, `post`, `GDEF`(mostly), `avar`, `fvar`, `VORG`, `MVAR`,
  `HVAR`, `VVAR`, `hmtx`, `vmtx`, `maxp`, `head` tables.
- `is_default_ignorable`
- `hb_script_from_iso15924_tag`
- `hb_script_to_iso15924_tag`
- `hb_script_from_string`
- `hb_ucd_script`
- `hb_ucd_compose`
- `hb_ucd_decompose`
- `hb_ucd_general_category`
- `hb_direction_from_string`
- `hb_tag_from_string`
- `hb_feature_from_string`
- `hb_variation_from_string`
- `hb_ot_tags_from_script_and_language`
- `hb_font_set_variations`

### Changed
- Replace some header guards with `#pragma once`, otherwise Qt Creator unable to parse them.

### Removed
- autotools/CMake dependency. HarfBuzz will be built via `cc` crate.
- All external dependencies. No icu, glib, freetype, etc.
- Subsetting.
- `hb_shape_list_shapers`, since we only have `ot`.
- Deprecated methods.
- `hb_unicode_funcs_t`, since we are using only internal Unicode tables.
- `hb_font_funcs_t`, since we are using internal OpenType parser.
- Most of the `hb_font_t` query API.
- `hb_buffer_deserialize_glyphs`
- `name`, `meta`, `STAT`, `BASE`, `JSTF`, `OPBD`, `math` and `gasp` tables. Unused.
- `hb_ot_metrics_*`. Unused.
- `hb_face_collect_*`. Unused.
- OpenType tables serialization. Unused.
- `hb-ot-color`. Unused.
- User data support.
