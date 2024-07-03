#![allow(unused)]

use wasmtime::{self, Caller, Engine, Linker, Module};

use super::hb_font_t;
use super::ot_shape::{hb_ot_shape_context_t, shape_internal};
use super::ot_shape_plan::hb_ot_shape_plan_t;
use crate::{script, Feature, GlyphBuffer, UnicodeBuffer};

pub(crate) fn shape_with_wasm(
    // the font
    face: &hb_font_t,
    // the plan is useful maybe?
    plan: &hb_ot_shape_plan_t,
    // the text
    mut buffer: UnicodeBuffer,
) -> Option<GlyphBuffer> {
    buffer.0.guess_segment_properties();

    // If font has no Wasm blob just return None to carry on as usual.
    let wasm_blob = face
        .raw_face()
        .table(ttf_parser::Tag::from_bytes(b"Wasm"))?;

    // wasmtime stuff here

    let engine = Engine::default();
    let module = Module::new(&engine, wasm_blob).ok()?; // returns None if couldn't parse blob.
    let name = module.name().unwrap_or_default();

    let mut linker = Linker::new(&engine);

    // fn face_get_upem(face: u32) -> u32;
    // Returns the units-per-em of the font face.
    linker.func_wrap(
        name,
        "face_get_upem",
        |mut caller: Caller<'_, &hb_font_t>, _face: u32| -> u32 {
            caller.data().units_per_em as u32
        },
    );

    // From HarfBuzz docs: (In the following functions, a font is a specific instantiation of a face at a particular scale factor and variation position.)
    // I am unsure how to represent that in rustybuzz.

    // fn font_get_face(font: u32) -> u32;
    // Creates a new face token from the given font token.
    linker.func_wrap(
        name,
        "font_get_face",
        |mut caller: Caller<'_, &hb_font_t>, _: u32| {
            //
            todo!()
        },
    );

    // fn font_get_glyph(font: u32, unicode: u32, uvs: u32) -> u32;
    // Returns the nominal glyph ID for the given codepoint, using the cmap table of the font to map Unicode codepoint (and variation selector) to glyph ID.
    linker.func_wrap(
        name,
        "font_get_glyph",
        |mut caller: Caller<'_, &hb_font_t>, _: u32, codepoint: u32, uvs: u32| -> u32 {
            char::from_u32(codepoint)
                .and_then(|codepoint| {
                    caller
                        .data()
                        .glyph_variation_index(codepoint, char::from_u32(uvs)?)
                })
                .unwrap_or_default()
                .0 as u32
        },
    );

    // fn font_get_scale(font: u32, x_scale: *mut i32, y_scale: *mut i32);
    // Returns the scale of the current font.
    linker.func_wrap(
        name,
        "font_get_scale",
        |mut caller: Caller<'_, &hb_font_t>, font: u32, x_scale: *mut i32, y_scale: *mut i32| {
            // This signature gives a compiler error.
            todo!()
        },
    );

    // fn font_get_glyph_extents(font: u32, glyph: u32, extents: *mut CGlyphExtents) -> bool;
    // fn font_glyph_to_string(font: u32, glyph: u32, str: *const u8, len: u32);
    // fn font_get_glyph_h_advance(font: u32, glyph: u32) -> i32;
    // fn font_get_glyph_v_advance(font: u32, glyph: u32) -> i32;
    // fn font_copy_glyph_outline(font: u32, glyph: u32, outline: *mut CGlyphOutline) -> bool;
    // fn face_copy_table(font: u32, tag: u32, blob: *mut Blob) -> bool;
    // fn buffer_copy_contents(buffer: u32, cbuffer: *mut CBufferContents) -> bool;
    // fn buffer_set_contents(buffer: u32, cbuffer: &CBufferContents) -> bool;
    // fn debugprint(s: *const u8);
    // fn shape_with(font: u32, buffer: u32, features: u32, num_features: u32, shaper: *const u8) -> i32;




    // Some(shape_with_plan(face, &plan, buffer))

    todo!()
}

// ===========
// structs used into WASM
// ===========

#[repr(C)]
#[derive(Clone, Debug)]
enum PointType {
    MoveTo,
    LineTo,
    QuadraticTo,
    CubicTo,
}

#[repr(C)]
#[derive(Clone, Debug)]
struct CGlyphOutlinePoint {
    x: f32,
    y: f32,
    pointtype: PointType,
}

#[repr(C)]
struct CGlyphOutline {
    n_points: usize,
    points: *mut CGlyphOutlinePoint,
    n_contours: usize,
    contours: *mut usize,
}

// Glyph extents
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct CGlyphExtents {
    /// The scaled left side bearing of the glyph
    pub x_bearing: i32,
    /// The scaled coordinate of the top of the glyph
    pub y_bearing: i32,
    /// The width of the glyph
    pub width: i32,
    /// The height of the glyph
    pub height: i32,
}

/// Some data provided by ~~Harfbuzz~~. rustybuzz
#[derive(Debug)]
#[repr(C)]
pub struct Blob {
    /// Length of the blob in bytes
    pub length: u32,
    /// A raw pointer to the contents
    pub data: *mut u8,
}

/// Glyph information in a buffer item provided by ~~Harfbuzz~~ rustybuzz
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CGlyphInfo {
    pub codepoint: u32,
    pub mask: u32,
    pub cluster: u32,
    pub var1: u32,
    pub var2: u32,
}

/// Glyph positioning information in a buffer item provided by ~~Harfbuzz~~ rustybuzz
#[derive(Debug, Clone)]
#[repr(C)]
pub struct CGlyphPosition {
    pub x_advance: i32,
    pub y_advance: i32,
    pub x_offset: i32,
    pub y_offset: i32,
    pub var: u32,
}

#[derive(Debug)]
#[repr(C)]
struct CBufferContents {
    length: u32,
    info: *mut CGlyphInfo,
    position: *mut CGlyphPosition,
}
