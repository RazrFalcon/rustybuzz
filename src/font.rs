use std::convert::TryFrom;
use std::os::raw::{c_void, c_char};
use std::marker::PhantomData;

use ttf_parser::{
    GlyphId, GlyphPosSubTable, ScriptIndex, LanguageIndex, FeatureIndex,
    FeatureVariationIndex,
};

use crate::ffi;
use crate::common::f32_bound;
use crate::{Tag, Variation};


#[derive(Debug)]
struct Blob<'a> {
    ptr: *mut ffi::hb_blob_t,
    marker: PhantomData<&'a [u8]>,
}

impl<'a> Blob<'a> {
    fn with_bytes(bytes: &'a [u8]) -> Blob<'a> {
        unsafe {
            let hb_blob = ffi::hb_blob_create(
                bytes.as_ptr() as *const _,
                bytes.len() as u32,
                ffi::HB_MEMORY_MODE_READONLY,
                std::ptr::null_mut(),
                None,
            );

            Blob {
                ptr: hb_blob,
                marker: PhantomData,
            }
        }
    }

    fn as_ptr(&self) -> *mut ffi::hb_blob_t {
        self.ptr
    }
}

impl<'a> Drop for Blob<'a> {
    fn drop(&mut self) {
        unsafe { ffi::hb_blob_destroy(self.ptr); }
    }
}


/// A wrapper around `hb_face_t`.
///
/// Font face is objects represent a single face in a font family. More
/// exactly, a font face represents a single face in a binary font file. Font
/// faces are typically built from a binary blob and a face index. Font faces
/// are used to create fonts.
#[derive(Debug)]
pub struct Face<'a> {
    ptr: *mut ffi::hb_face_t,
    blob: Blob<'a>,
    ttf: *const ttf_parser::Font<'a>,
}

impl<'a> Face<'a> {
    /// Creates a new `Face` from the data.
    pub fn new(data: &'a [u8], index: u32) -> Result<Face<'a>, ttf_parser::Error> {
        unsafe {
            let ttf = Box::new(ttf_parser::Font::from_data(data, index)?);
            let ttf = Box::into_raw(ttf);
            let blob = Blob::with_bytes(data);
            Ok(Face {
                ptr: ffi::hb_face_create(blob.as_ptr(), ttf as *const _, index),
                blob,
                ttf,
            })
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut ffi::hb_face_t {
        self.ptr
    }

    /// Returns face's UPEM.
    pub fn upem(&self) -> u32 {
        unsafe { ffi::hb_face_get_upem(self.ptr) }
    }

    /// Sets face's UPEM.
    pub fn set_upem(&mut self, upem: u32) {
        unsafe { ffi::hb_face_set_upem(self.ptr, upem) };
    }
}

impl<'a> Drop for Face<'a> {
    fn drop(&mut self) {
        unsafe {
            Box::from_raw(self.ttf as *mut ttf_parser::Font<'a>);
            ffi::hb_face_destroy(self.ptr);
        }
    }
}


/// A type representing a single font (i.e. a specific combination of typeface and typesize).
#[derive(Debug)]
pub struct Font<'a> {
    ptr: *mut ffi::hb_font_t,
    face: Face<'a>,
}

impl<'a> Font<'a> {
    /// Creates a new font from the specified `Face`.
    pub fn new(face: Face<'a>) -> Self {
        unsafe {
            Font {
                ptr: ffi::hb_font_create(face.as_ptr(), face.ttf as *const _),
                face,
            }
        }
    }

    pub(crate) fn font(&self) -> &ttf_parser::Font {
        unsafe { &*(self.face.ttf as *const ttf_parser::Font) }
    }

    pub(crate) fn font_ptr(&self) -> *const ttf_parser::Font {
        self.face.ttf
    }

    pub(crate) fn from_ptr(font: *const ffi::hb_font_t) -> &'static Font<'static> {
        unsafe { &*(font as *const Font) }
    }

    pub(crate) fn as_ptr(&self) -> *mut ffi::hb_font_t {
        self.ptr
    }

    /// Returns the EM scale of the font.
    pub fn scale(&self) -> (i32, i32) {
        let mut result = (0i32, 0i32);
        unsafe { ffi::hb_font_get_scale(self.ptr, &mut result.0, &mut result.1) };
        result
    }

    /// Sets the EM scale of the font.
    pub fn set_scale(&mut self, x: i32, y: i32) {
        unsafe { ffi::hb_font_set_scale(self.ptr, x, y) };
    }

    /// Returns font's PPEM.
    pub fn ppem(&self) -> (u32, u32) {
        let mut result = (0u32, 0u32);
        unsafe { ffi::hb_font_get_ppem(self.ptr, &mut result.0, &mut result.1) };
        result
    }

    /// Set font's PPEM.
    pub fn set_ppem(&mut self, x: u32, y: u32) {
        unsafe { ffi::hb_font_set_ppem(self.ptr, x, y) };
    }

    /// Sets *point size* of the font.
    ///
    /// Set to 0 to unset.
    ///
    /// There are 72 points in an inch.
    pub fn set_ptem(&mut self, ptem: f32) {
        unsafe { ffi::hb_font_set_ptem(self.ptr, ptem) };
    }

    /// Sets a font variations.
    pub fn set_variations(&mut self, variations: &[Variation]) {
        let ttf = unsafe { &*self.face.ttf };
        let coords_len = try_opt!(ttf.variation_axes_count()) as usize;
        let mut coords = vec![0; coords_len];

        for variation in variations {
            if let Ok(Some(axis)) = ttf.variation_axis(variation.tag) {
                let mut v = f32_bound(axis.min_value, variation.value, axis.max_value);

                if v == axis.default_value {
                    v = 0.0;
                } else if v < axis.default_value {
                    v = (v - axis.default_value) / (axis.default_value - axis.min_value);
                } else {
                    v = (v - axis.default_value) / (axis.max_value - axis.default_value)
                }

                coords[axis.index as usize] = (v * 16384.0).round() as i32;
            }
        }

        let _ = ttf.map_variation_coordinates(&mut coords);

        unsafe {
            ffi::hb_font_set_variations(
                self.ptr,
                coords.as_ptr() as *mut _,
                coords.len() as u32,
            )
        }
    }
}

impl<'a> Drop for Font<'a> {
    fn drop(&mut self) {
        unsafe { ffi::hb_font_destroy(self.ptr); }
    }
}


fn font_from_raw(font_data: *const c_void) -> &'static ttf_parser::Font<'static> {
    unsafe { &*(font_data as *const ttf_parser::Font) }
}

#[no_mangle]
pub extern "C" fn rb_ot_get_nominal_glyph(font_data: *const c_void, c: u32, glyph: *mut u32) -> i32 {
    match font_from_raw(font_data).glyph_index(char::try_from(c).unwrap()) {
        Ok(Some(g)) => unsafe { *glyph = g.0 as u32; 1 }
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_get_variation_glyph(font_data: *const c_void, c: u32, variant: u32, glyph: *mut u32) -> i32 {
    let font = font_from_raw(font_data);
    match font.glyph_variation_index(char::try_from(c).unwrap(), char::try_from(variant).unwrap()) {
        Ok(Some(g)) => unsafe { *glyph = g.0 as u32; 1 }
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_get_glyph_bbox(font_data: *const c_void, glyph: u32, extents: *mut ffi::hb_glyph_bbox_t) -> i32 {
    let font = font_from_raw(font_data);
    match font.glyph_bounding_box(GlyphId(u16::try_from(glyph).unwrap())) {
        Ok(Some(bbox)) => unsafe {
            (*extents).x_min = bbox.x_min;
            (*extents).y_min = bbox.y_min;
            (*extents).x_max = bbox.x_max;
            (*extents).y_max = bbox.y_max;
            1
        }
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_get_glyph_name(font_data: *const c_void, glyph: u32, mut raw_name: *mut c_char, len: u32) -> i32 {
    assert_ne!(len, 0);

    let font = font_from_raw(font_data);
    match font.glyph_name(GlyphId(u16::try_from(glyph).unwrap())) {
        Ok(Some(name)) => unsafe {
            let len = std::cmp::min(name.len(), len as usize - 1);

            for b in &name.as_bytes()[0..len] {
                *raw_name = *b as c_char;
                raw_name = raw_name.offset(1);
            }

            *raw_name = b'\0' as c_char;

            1
        }
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_layout_has_glyph_classes(font_data: *const c_void) -> i32 {
    font_from_raw(font_data).has_glyph_classes() as i32
}

#[no_mangle]
pub extern "C" fn rb_ot_get_glyph_class(font_data: *const c_void, glyph: u32) -> u32 {
    match font_from_raw(font_data).glyph_class(GlyphId(u16::try_from(glyph).unwrap())) {
        Ok(Some(c)) => c as u32,
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_get_mark_attachment_class(font_data: *const c_void, glyph: u32) -> u32 {
    let font = font_from_raw(font_data);
    match font.glyph_mark_attachment_class(GlyphId(u16::try_from(glyph).unwrap())) {
        Ok(c) => c.0 as u32,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_is_mark_glyph(font_data: *const c_void, set_index: u32, glyph: u32) -> i32 {
    let font = font_from_raw(font_data);
    match font.is_mark_glyph(GlyphId(u16::try_from(glyph).unwrap()), Some(set_index as u16)) {
        Ok(c) => c as i32,
        Err(_) => 0,
    }
}

const GSUB_TABLE_TAG: Tag = Tag::from_bytes(b"GSUB");
const GPOS_TABLE_TAG: Tag = Tag::from_bytes(b"GPOS");

fn has_table(font: &ttf_parser::Font, tag: Tag) -> bool {
    match tag {
        GSUB_TABLE_TAG => font.has_table(ttf_parser::TableName::GlyphSubstitution),
        GPOS_TABLE_TAG => font.has_table(ttf_parser::TableName::GlyphPositioning),
        _ => false,
    }
}

fn with_table<T, F>(font: &ttf_parser::Font, tag: Tag, f: F) -> T
    where F: FnOnce(&dyn GlyphPosSubTable) -> T
{
    match tag {
        GSUB_TABLE_TAG => f(&font.substitution_table().unwrap()),
        GPOS_TABLE_TAG => f(&font.positioning_table().unwrap()),
        _ => unreachable!(),
    }
}

#[no_mangle]
pub extern "C" fn rb_ot_layout_table_get_script_count(
    font_data: *const c_void,
    table_tag: Tag,
) -> u32 {
    let font = font_from_raw(font_data);

    if !has_table(font, table_tag) {
        return 0;
    }

    with_table(font, table_tag, |table| {
        table.scripts().map(|iter| iter.count()).unwrap_or(0) as u32
    })
}

#[no_mangle]
pub extern "C" fn rb_ot_layout_table_select_script(
    font_data: *const c_void,
    table_tag: Tag,
    script_count: u32,
    script_tags: *const Tag,
    script_index: *mut u32,
    chosen_script: *mut Tag,
) -> ffi::hb_bool_t {
    let font = font_from_raw(font_data);
    let scripts = unsafe { std::slice::from_raw_parts(script_tags as *const _, script_count as usize) };

    unsafe {
        *script_index = 0xFFFF;
        *chosen_script = Tag(0xFFFF);
    }

    if !has_table(font, table_tag) {
        return 0;
    }

    with_table(font, table_tag, |table| {
        let script_by_tag = |tag| {
            table.scripts().ok().and_then(|s| s.filter_map(Result::ok).position(|s| s.tag() == tag))
        };

        for script in scripts {
            if let Some(idx) = script_by_tag(*script) {
                unsafe {
                    *script_index = idx as u32;
                    *chosen_script = *script;
                }
                return 1;
            }
        }

        // try finding 'DFLT'
        if let Some(idx) = script_by_tag(Tag::default_script()) {
            unsafe {
                *script_index = idx as u32;
                *chosen_script = Tag::default_script();
            }
            return 0;
        }

        // try with 'dflt'; MS site has had typos and many fonts use it now :(
        if let Some(idx) = script_by_tag(Tag::default_language()) {
            unsafe {
                *script_index = idx as u32;
                *chosen_script = Tag::default_language();
            }
            return 0;
        }

        // try with 'latn'; some old fonts put their features there even though
        // they're really trying to support Thai, for example :(
        if let Some(idx) = script_by_tag(Tag::from_bytes(b"latn")) {
            unsafe {
                *script_index = idx as u32;
                *chosen_script = Tag::from_bytes(b"latn");
            }
            return 0;
        }

        0
    })
}

#[no_mangle]
pub extern "C" fn rb_ot_layout_table_find_feature(
    font_data: *const c_void,
    table_tag: Tag,
    feature_tag: Tag,
    feature_index: *mut u32,
) -> ffi::hb_bool_t {
    let font = font_from_raw(font_data);

    unsafe { *feature_index = 0xFFFF; }

    if !has_table(font, table_tag) {
        return 0;
    }

    with_table(font, table_tag, |table| {
        if let Some(idx) = try_ok_or!(table.features(), 0).filter_map(Result::ok).position(|f| f.tag == feature_tag) {
            unsafe { *feature_index = idx as u32; };
            1
        } else {
            0
        }
    })
}

#[no_mangle]
pub extern "C" fn rb_ot_layout_script_select_language(
    font_data: *const c_void,
    table_tag: Tag,
    script_index: u32,
    language_count: u32,
    language_tags: *mut Tag,
    language_index: *mut u32,
) -> ffi::hb_bool_t {
    let font = font_from_raw(font_data);
    let languages = unsafe { std::slice::from_raw_parts(language_tags as *const _, language_count as usize) };

    unsafe { *language_index = 0xFFFF; }

    if !has_table(font, table_tag) {
        return 0;
    }

    with_table(font, table_tag, |table| {
        let script = try_res_opt_or!(table.script_at(ScriptIndex(script_index as u16)), 0);

        for lang in languages {
            if let Some((idx, _)) = script.language_by_tag(*lang) {
                unsafe { *language_index = idx.0 as u32; }
                return 1;
            }
        }

        // try finding 'dflt'
        if let Some((idx, _)) = script.language_by_tag(Tag::default_language()) {
            unsafe { *language_index = idx.0 as u32; }
            return 0;
        }

        0
    })
}

#[no_mangle]
pub extern "C" fn rb_ot_layout_language_get_required_feature(
    font_data: *const c_void,
    table_tag: Tag,
    script_index: u32,
    language_index: u32,
    feature_index: *mut u32,
    feature_tag: *mut Tag,
) -> ffi::hb_bool_t {
    let font = font_from_raw(font_data);

    unsafe {
        *feature_index = 0xFFFF;
        *feature_tag = Tag(0);
    }

    if !has_table(font, table_tag) {
        return 0;
    }

    with_table(font, table_tag, |table| {
        let script = try_res_opt_or!(table.script_at(ScriptIndex(script_index as u16)), 0);
        let lang = try_opt_or!(script.language_at(LanguageIndex(language_index as u16)), 0);
        if let Some(idx) = lang.required_feature_index {
            if let Ok(Some(f)) = table.feature_at(idx) {
                unsafe {
                    *feature_index = idx.0 as u32;
                    *feature_tag = f.tag;
                }

                return 1;
            }
        }

        0
    })
}

#[no_mangle]
pub extern "C" fn rb_ot_layout_language_find_feature(
    font_data: *const c_void,
    table_tag: Tag,
    script_index: u32,
    language_index: u32,
    feature_tag: Tag,
    feature_index: *mut u32,
) -> ffi::hb_bool_t {
    let font = font_from_raw(font_data);

    unsafe { *feature_index = 0xFFFF; }

    if !has_table(font, table_tag) {
        return 0;
    }

    with_table(font, table_tag, |table| {
        let script = try_res_opt_or!(table.script_at(ScriptIndex(script_index as u16)), 0);
        let lang = if language_index != 0xFFFF {
            try_opt_or!(script.language_at(LanguageIndex(language_index as u16)), 0)
        } else {
            try_opt_or!(script.default_language(), 0)
        };

        for idx in lang.feature_indices {
            if let Ok(Some(feature)) = table.feature_at(idx) {
                if feature.tag == feature_tag {
                    unsafe { *feature_index = idx.0 as u32; }
                    return 1;
                }
            }
        }

        0
    })
}

#[no_mangle]
pub extern "C" fn rb_ot_layout_table_get_lookup_count(
    font_data: *const c_void,
    table_tag: Tag,
) -> u32 {
    let font = font_from_raw(font_data);

    if !has_table(font, table_tag) {
        return 0;
    }

    with_table(font, table_tag, |table| {
        try_ok_or!(table.lookups(), 0).count() as u32
    })
}

#[no_mangle]
pub extern "C" fn rb_ot_layout_table_find_feature_variations(
    font_data: *const c_void,
    table_tag: Tag,
    coords: *const i32,
    num_coords: u32,
    variations_index: *mut u32,
) -> ffi::hb_bool_t {
    let font = font_from_raw(font_data);
    let coords = unsafe { std::slice::from_raw_parts(coords as *const _, num_coords as usize) };

    unsafe { *variations_index = 0xFFFF_FFFF; }

    if !has_table(font, table_tag) {
        return 0;
    }

    with_table(font, table_tag, |table| {
        let variations = try_ok_or!(table.feature_variations(), 0);
        for (i, var) in variations.enumerate() {
            if let Ok(var) = var {
                if var.evaluate(coords) {
                    unsafe { *variations_index = i as u32; }
                    return 1;
                }
            }
        }

        0
    })
}

#[no_mangle]
pub extern "C" fn rb_ot_layout_feature_with_variations_get_lookups(
    font_data: *const c_void,
    table_tag: Tag,
    feature_index: u32,
    variations_index: u32,
    start_offset: u32,
    lookup_count: *mut u32,
    mut lookup_indexes: *mut u32,
) -> u32 {
    let font = font_from_raw(font_data);

    unsafe { *lookup_count = 0; }

    if !has_table(font, table_tag) {
        return 0;
    }

    with_table(font, table_tag, |table| {
        let feature = if let Ok(Some(variation)) = table.feature_variation_at(FeatureVariationIndex(variations_index)) {
            try_ok_or!(variation.substitutions(), 0)
                .find(|s| s.index() == FeatureIndex(feature_index as u16))
                .and_then(|s| s.feature().ok())
        } else {
            try_ok_or!(table.feature_at(FeatureIndex(feature_index as u16)), 0)
        };

        let mut added = 0;
        if let Some(feature) = feature {
            for idx in feature.lookup_indices.into_iter().skip(start_offset as usize).take(lookup_count as usize) {
                unsafe {
                    *lookup_indexes = idx.0 as u32;
                    lookup_indexes = lookup_indexes.offset(1);
                    added += 1;
                }
            }
        }

        unsafe { *lookup_count = added; }

        0
    })
}

#[no_mangle]
pub extern "C" fn rb_ot_layout_has_substitution(
    font_data: *const c_void,
) -> ffi::hb_bool_t {
    font_from_raw(font_data).has_table(ttf_parser::TableName::GlyphSubstitution) as i32
}

#[no_mangle]
pub extern "C" fn rb_ot_layout_has_positioning(
    font_data: *const c_void,
) -> ffi::hb_bool_t {
    font_from_raw(font_data).has_table(ttf_parser::TableName::GlyphPositioning) as i32
}

#[no_mangle]
pub extern "C" fn hb_ot_get_var_axis_count(font_data: *const c_void) -> u16 {
    font_from_raw(font_data).variation_axes_count().unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn rb_ot_has_vorg_data(font_data: *const c_void) -> i32 {
    font_from_raw(font_data).has_table(ttf_parser::TableName::VerticalOrigin) as i32
}

#[no_mangle]
pub extern "C" fn rb_ot_get_y_origin(font_data: *const c_void, glyph: u32) -> i32 {
    font_from_raw(font_data).glyph_y_origin(GlyphId(u16::try_from(glyph).unwrap())).unwrap_or(0) as i32
}

mod metrics {
    use crate::Tag;

    pub const HORIZONTAL_ASCENDER: Tag  = Tag::from_bytes(b"hasc");
    pub const HORIZONTAL_DESCENDER: Tag = Tag::from_bytes(b"hdsc");
    pub const HORIZONTAL_LINE_GAP: Tag  = Tag::from_bytes(b"hlgp");
    pub const VERTICAL_ASCENDER: Tag    = Tag::from_bytes(b"vasc");
    pub const VERTICAL_DESCENDER: Tag   = Tag::from_bytes(b"vdsc");
    pub const VERTICAL_LINE_GAP: Tag    = Tag::from_bytes(b"vlgp");
}

#[no_mangle]
pub unsafe extern "C" fn rb_ot_metrics_get_position_common(
    font_data: *const c_void,
    coords: *const i32,
    coord_count: u32,
    scale: i32,
    tag: u32,
    position: *mut i32,
) -> i32 {
    // TODO: Never executed. Add tests.

    let font = font_from_raw(font_data);
    let coords = std::slice::from_raw_parts(coords as *const _, coord_count as usize);

    let upem = font.units_per_em().unwrap_or(0) as f32;
    let offset = match font.metrics_variation(Tag(tag), coords) {
        Ok(Some(v)) => v,
        _ => 0.0,
    };

    let rescale = |x: f32| ((x * scale as f32) / upem).round() as i32;

    match Tag(tag) {
        metrics::HORIZONTAL_ASCENDER => {
            *position = rescale((font.ascender() as f32 + offset).abs());
        }
        metrics::HORIZONTAL_DESCENDER => {
            *position = rescale(-(font.descender() as f32 + offset).abs());
        }
        metrics::HORIZONTAL_LINE_GAP => {
            *position = rescale(font.line_gap() as f32 + offset);
        }
        metrics::VERTICAL_ASCENDER => {
            let v = font.vertical_ascender().unwrap_or(0);
            *position = rescale((v as f32 + offset).abs());
        }
        metrics::VERTICAL_DESCENDER => {
            let v = font.vertical_descender().unwrap_or(0);
            *position = rescale(-(v as f32 + offset).abs());
        }
        metrics::VERTICAL_LINE_GAP => {
            let v = font.vertical_line_gap().unwrap_or(0);
            *position = rescale(v as f32 + offset);
        }
        _ => return 0,
    }

    1
}

#[no_mangle]
pub extern "C" fn rb_font_get_advance(font_data: *const c_void, glyph: u32, is_vertical: bool) -> u32 {
    let font = font_from_raw(font_data);
    let glyph = GlyphId(u16::try_from(glyph).unwrap());

    let pem = font.units_per_em().unwrap_or(1000);

    if is_vertical {
        font.glyph_ver_advance(glyph).ok().flatten().unwrap_or(pem) as u32
    } else {
        font.glyph_hor_advance(glyph).ok().flatten().unwrap_or(pem) as u32
    }
}

#[no_mangle]
pub extern "C" fn rb_font_get_advance_var(
    hb_font: *mut ffi::hb_font_t,
    font_data: *const c_void,
    hb_glyph: u32,
    is_vertical: bool,
    coords: *const i32,
    coord_count: u32,
) -> u32 {
    let advance = rb_font_get_advance(font_data, hb_glyph, is_vertical);

    let font = font_from_raw(font_data);
    let coords = unsafe { std::slice::from_raw_parts(coords as *const _, coord_count as usize) };
    let glyph = GlyphId(u16::try_from(hb_glyph).unwrap());

    if coords.is_empty() {
        return advance;
    }

    // TODO: check advance for negative values
    if !is_vertical && font.has_table(ttf_parser::TableName::HorizontalMetricsVariations) {
        let offset = font.glyph_hor_advance_variation(glyph, coords).ok().flatten().unwrap_or(0.0).round();
        return (advance as f32 + offset) as u32;
    } else if is_vertical && font.has_table(ttf_parser::TableName::VerticalMetricsVariations) {
        let offset = font.glyph_ver_advance_variation(glyph, coords).ok().flatten().unwrap_or(0.0).round();
        return (advance as f32 + offset) as u32;
    }

    unsafe { ffi::hb_ot_glyf_get_advance_var(hb_font, hb_glyph, is_vertical) }
}

#[no_mangle]
pub extern "C" fn rb_font_get_side_bearing(font_data: *const c_void, glyph: u32, is_vertical: bool) -> i32 {
    let font = font_from_raw(font_data);
    let glyph = GlyphId(u16::try_from(glyph).unwrap());

    if is_vertical {
        font.glyph_ver_side_bearing(glyph).ok().flatten().unwrap_or(0) as i32
    } else {
        font.glyph_hor_side_bearing(glyph).ok().flatten().unwrap_or(0) as i32
    }
}

#[no_mangle]
pub extern "C" fn rb_font_get_side_bearing_var(
    hb_font: *mut ffi::hb_font_t,
    font_data: *const c_void,
    hb_glyph: u32,
    is_vertical: bool,
    coords: *const i32,
    coord_count: u32,
) -> i32 {
    let side_bearing = rb_font_get_side_bearing(font_data, hb_glyph, is_vertical);

    let font = font_from_raw(font_data);
    let coords = unsafe { std::slice::from_raw_parts(coords as *const _, coord_count as usize) };
    let glyph = GlyphId(u16::try_from(hb_glyph).unwrap());

    if coords.is_empty() {
        return side_bearing;
    }

    if !is_vertical && font.has_table(ttf_parser::TableName::HorizontalMetricsVariations) {
        let offset = font.glyph_hor_side_bearing_variation(glyph, coords).ok().flatten().unwrap_or(0.0).round();
        return (side_bearing as f32 + offset) as i32;
    } else if is_vertical && font.has_table(ttf_parser::TableName::VerticalMetricsVariations) {
        let offset = font.glyph_ver_side_bearing_variation(glyph, coords).ok().flatten().unwrap_or(0.0).round();
        return (side_bearing as f32 + offset) as i32;
    }

    unsafe { ffi::hb_ot_glyf_get_side_bearing_var(hb_font, hb_glyph, is_vertical) }
}

#[no_mangle]
pub extern "C" fn rb_face_get_glyph_count(font_data: *const c_void) -> u32 {
    font_from_raw(font_data).number_of_glyphs() as u32
}

#[no_mangle]
pub extern "C" fn rb_face_get_upem(font_data: *const c_void) -> u32 {
    font_from_raw(font_data).units_per_em().unwrap_or(1000) as u32
}

#[no_mangle]
pub extern "C" fn rb_face_index_to_loc_format(font_data: *const c_void) -> u32 {
    font_from_raw(font_data).index_to_location_format().map(|f| f as u32).unwrap_or(0)
}
