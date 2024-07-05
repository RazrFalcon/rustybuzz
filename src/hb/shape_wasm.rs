#![allow(unused)]

use alloc::{borrow::ToOwned, ffi::CString, format, string::String};
use core::ffi::CStr;
use ttf_parser::{GlyphId, Tag};
use wasmtime::{self, AsContext, AsContextMut, Caller, Engine, Linker, Memory, Module, Store};

use super::{
    buffer::{hb_buffer_t, GlyphBuffer, GlyphPosition, UnicodeBuffer},
    face::hb_glyph_extents_t,
    hb_font_t, hb_glyph_info_t,
    ot_shape::{hb_ot_shape_context_t, shape_internal},
    ot_shape_plan::hb_ot_shape_plan_t,
};
use crate::shape_with_plan;

struct ShapingData<'a> {
    font: &'a hb_font_t<'a>,
    buffer: hb_buffer_t,
}

pub(crate) fn shape_with_wasm(
    // the font
    face: &hb_font_t,
    // the plan is useful maybe?
    plan: &hb_ot_shape_plan_t,
    // the text
    mut unicode_buffer: UnicodeBuffer,
) -> Option<GlyphBuffer> {
    unicode_buffer.0.guess_segment_properties();

    // If font has no Wasm blob just return None to carry on as usual.
    let wasm_blob = face
        .raw_face()
        .table(ttf_parser::Tag::from_bytes(b"Wasm"))?;

    // wasmtime stuff here

    let data = ShapingData {
        font: face,
        buffer: unicode_buffer.0,
    };

    let mut store = Store::new(&Engine::default(), data);
    let module = Module::new(&store.engine(), wasm_blob).ok()?; // returns None if couldn't parse blob.

    let mut linker = Linker::new(&store.engine());

    // Wouldn't look as ridiculous if we returned anyhow::Result from this function instead or sth.
    linker
        .func_wrap("env", "face_get_upem", face_get_upem)
        .ok()?
        .func_wrap("env", "font_get_face", font_get_face)
        .ok()?
        .func_wrap("env", "font_get_glyph", font_get_glyph)
        .ok()?
        .func_wrap("env", "font_get_scale", font_get_scale)
        .ok()?
        .func_wrap("env", "font_get_glyph_extents", font_get_glyph_extents)
        .ok()?
        .func_wrap("env", "font_glyph_to_string", font_glyph_to_string)
        .ok()?
        .func_wrap("env", "font_get_glyph_h_advance", font_get_glyph_h_advance)
        .ok()?
        .func_wrap("env", "font_get_glyph_v_advance", font_get_glyph_v_advance)
        .ok()?
        .func_wrap("env", "font_copy_glyph_outline", font_copy_glyph_outline)
        .ok()?
        .func_wrap("env", "face_copy_table", face_copy_table)
        .ok()?
        .func_wrap("env", "buffer_copy_contents", buffer_copy_contents)
        .ok()?
        .func_wrap("env", "buffer_set_contents", buffer_set_contents)
        .ok()?
        .func_wrap("env", "debugprint", debugprint)
        .ok()?
        .func_wrap("env", "shape_with", shape_with)
        .ok()?;

    // Here we are (supposedly) done creating functions.
    // draft section

    // The WASM code inside a font is expected to export a function called shape which takes five int32 arguments
    // and returns an int32 status value. (Zero for failure, any other value for success.) Three of the five
    // arguments are tokens which can be passed to the API functions exported to your WASM code by the host
    // shaping engine:
    //
    // A shape plan token, which can largely be ignored.
    // A font token.
    // A buffer token.
    // A feature array.
    // The number of features.

    let instance = linker.instantiate(&mut store, &module).ok()?;
    let shape = instance
        .get_typed_func::<(u32, u32, u32, u32, u32), i32>(&mut store, "shape")
        .ok()?;

    if let Ok(0) | Err(_) = shape.call(&mut store, (0, 0, 0, 0, 0)) {
        return None;
    };

    let ret = store.into_data().buffer;

    Some(GlyphBuffer(ret))
}

// ===========
// functions imported  into WASM
// ===========

// fn face_get_upem(face: u32) -> u32;
// Returns the units-per-em of the font face.
fn face_get_upem(caller: Caller<'_, ShapingData>, _face: u32) -> u32 {
    caller.data().font.units_per_em as u32
}

// fn font_get_face(font: u32) -> u32;
// Creates a new face token from the given font token.
fn font_get_face(caller: Caller<'_, ShapingData>, _font: u32) -> u32 {
    // From HarfBuzz docs:
    // (In the following functions, a font is a specific instantiation of a face at a
    // particular scale factor and variation position.)
    //
    // I am unsure how to represent that in rustybuzz.
    // er .. do what here?
    0
}

// fn font_get_glyph(font: u32, unicode: u32, uvs: u32) -> u32;
// Returns the nominal glyph ID for the given codepoint, using the cmap table of the font to map Unicode codepoint (and variation selector) to glyph ID.
fn font_get_glyph(
    mut caller: Caller<'_, ShapingData>,
    _font: u32,
    codepoint: u32,
    uvs: u32,
) -> u32 {
    let Some(codepoint) = char::from_u32(codepoint) else {
        return 0;
    };
    let Some(uvs) = char::from_u32(uvs) else {
        return 0;
    };
    caller
        .data()
        .font
        .glyph_variation_index(codepoint, uvs)
        .unwrap_or_default()
        .0 as u32
}

// fn font_get_scale(font: u32, x_scale: *mut i32, y_scale: *mut i32);
// Returns the scale of the current font.
// Just return the upem as rustybuzz has no scale.
fn font_get_scale(mut caller: Caller<'_, ShapingData>, _font: u32, x_scale: u32, y_scale: u32) {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let upem = caller.data().font.units_per_em();

    memory.write(
        &mut caller.as_context_mut(),
        x_scale as usize,
        &upem.to_le_bytes(),
    );
    memory.write(
        &mut caller.as_context_mut(),
        y_scale as usize,
        &upem.to_le_bytes(),
    );
}

// fn font_get_glyph_extents(font: u32, glyph: u32, extents: *mut CGlyphExtents) -> bool;
// Returns the glyph's extents for the given glyph ID at current scale and variation settings.
fn font_get_glyph_extents(
    mut caller: Caller<'_, ShapingData>,
    _font: u32,
    glyph: u32,
    extents: u32,
) -> u32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let mut glyph_extents = hb_glyph_extents_t::default();
    let ret = caller
        .data()
        .font
        .glyph_extents(GlyphId(glyph as u16), &mut glyph_extents);
    if ret {
        memory.write(
            caller.as_context_mut(),
            extents as usize,
            bytemuck::bytes_of(&glyph_extents),
        );
    }

    ret as u32
}

// fn font_glyph_to_string(font: u32, glyph: u32, str: *const u8, len: u32);
// Copies the name of the given glyph, or, if no name is available, a string of the form gXXXX into the given string.
fn font_glyph_to_string(
    mut caller: Caller<'_, ShapingData>,
    _font: u32,
    glyph: u32,
    str: u32,
    len: u32,
) {
    // len is apparently the assigned heap memory. It seems I should not allocate more than that.
    // Should not assume I am not writing over anything.
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let name = caller
        .data()
        .font
        .glyph_name(GlyphId(glyph as u16))
        .map(ToOwned::to_owned)
        .unwrap_or(format!("g{glyph:4}"));

    let name = CString::new(name).unwrap();
    let name = &name.as_bytes()[..len as usize]; // are names ever non ascii?

    memory.write(caller.as_context_mut(), str as usize, name);
}

// fn font_get_glyph_h_advance(font: u32, glyph: u32) -> i32;
// Returns the default horizontal advance for the given glyph ID the current scale and variations settings.
fn font_get_glyph_h_advance(caller: Caller<'_, ShapingData>, _font: u32, glyph: u32) -> i32 {
    caller.data().font.glyph_h_advance(GlyphId(glyph as u16))
}

// fn font_get_glyph_v_advance(font: u32, glyph: u32) -> i32;
// Returns the default vertical advance for the given glyph ID the current scale and variations settings.
fn font_get_glyph_v_advance(caller: Caller<'_, ShapingData>, _font: u32, glyph: u32) -> i32 {
    caller.data().font.glyph_v_advance(GlyphId(glyph as u16))
}

// fn font_copy_glyph_outline(font: u32, glyph: u32, outline: *mut CGlyphOutline) -> bool;
// Copies the outline of the given glyph ID, at current scale and variation settings, into the outline structure provided.
fn font_copy_glyph_outline(
    mut caller: Caller<'_, ShapingData>,
    _font: u32,
    glyph: u32,
    outline: u32,
) -> u32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let builder = CGlyphOutline {
        n_points: todo!(),
        points: todo!(),
        n_contours: todo!(),
        contours: todo!(),
    };
    // also no clue what to do here
    // let my_ol = caller.data().outline_glyph(GlyphId(glyph as u16), builder); // ??

    let Ok(()) = memory.write(caller, outline as usize, todo!("builder result goes here")) else {
        return 0;
    };

    1
}

// fn face_copy_table(font: u32, tag: u32, blob: *mut Blob) -> bool;
// Copies the binary data in the OpenType table referenced by tag into the supplied blob structure.
fn face_copy_table(mut caller: Caller<'_, ShapingData>, _font: u32, tag: u32, blob: u32) -> u32 {
    // So here to copy stuff INTO the module, I need to copy it into its heap
    // I should not assume that there is an area that's not written to,
    // so the most straightforward way to get "clean" memory is to grow it by one page,
    // and allocate there.
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let tag = tag.to_be_bytes(); // be or le?
    let Some(table) = caller.data().font.raw_face().table(Tag::from_bytes(&tag)) else {
        return 0;
    };

    let eom = memory.data_size(&caller);
    let Ok(_) = memory.grow(&mut caller.as_context_mut(), 1) else {
        return 0;
    };

    let my_blob = Blob {
        length: table.len() as u32,
        data: eom as u32,
    };

    let Ok(()) = memory.write(
        caller.as_context_mut(),
        blob as usize,
        bytemuck::bytes_of(&my_blob),
    ) else {
        return 0;
    };

    let Ok(()) = memory.write(caller.as_context_mut(), eom, table) else {
        return 0;
    };

    1
}

// fn buffer_copy_contents(buffer: u32, cbuffer: *mut CBufferContents) -> bool;
// Retrieves the contents of the host shaping engine's buffer into the buffer_contents structure. This should typically be called at the beginning of shaping.
fn buffer_copy_contents(mut caller: Caller<'_, ShapingData>, _buffer: u32, cbuffer: u32) -> u32 {
    // see face_copy_table for why we're growing memory.
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let eom = memory.data(&caller).len();
    let Ok(_) = memory.grow(&mut caller.as_context_mut(), 1) else {
        return 0;
    };

    // I need these two to be the same lifetime it seems
    let (mem_data, store_data) = memory.data_and_store_mut(&mut caller);

    let rb_buffer = &store_data.buffer;
    let length = rb_buffer.len;

    let pos_loc = eom + length * core::mem::size_of::<hb_glyph_info_t>();
    let end_loc = pos_loc + length * core::mem::size_of::<GlyphPosition>();

    // This _should_ work ..
    mem_data[eom..pos_loc].copy_from_slice(bytemuck::cast_slice(&rb_buffer.info));
    mem_data[pos_loc..end_loc].copy_from_slice(bytemuck::cast_slice(&rb_buffer.pos));

    let buffer_contents = CBufferContents {
        length: length as u32,
        info: eom as u32,
        position: pos_loc as u32,
    };
    let Ok(()) = memory.write(
        &mut caller.as_context_mut(),
        cbuffer as usize,
        bytemuck::bytes_of(&buffer_contents),
    ) else {
        return 0;
    };

    1
}

// fn buffer_set_contents(buffer: u32, cbuffer: &CBufferContents) -> bool;
// Copy the buffer_contents structure back into the host shaping engine's buffer. This should typically be called at the end of shaping.
fn buffer_set_contents(mut caller: Caller<'_, ShapingData>, _buffer: u32, cbuffer: u32) -> u32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let mut buffer = [0; core::mem::size_of::<CBufferContents>()];
    memory.read(caller.as_context_mut(), cbuffer as usize, &mut buffer);
    let Ok(buffer) = bytemuck::try_from_bytes::<CBufferContents>(&buffer) else {
        return 0;
    };

    // This functions's code from here on down is bad and I can't figure out why
    // One attempt is shown commented out below.

    caller.data_mut().buffer.clear_output();
    caller.data_mut().buffer.clear_positions();

    for i in 0..buffer.length {
        let mut info_buffer = [0; core::mem::size_of::<hb_glyph_info_t>()];
        let Ok(()) = memory.read(
            caller.as_context_mut(),
            buffer.info as usize + i as usize * core::mem::size_of::<hb_glyph_info_t>(),
            &mut info_buffer,
        ) else {
            // eprintln!("bad info being read");
            return 0;
        };
        let info = unsafe { core::mem::transmute(info_buffer) };
        caller.data_mut().buffer.info.push(info);

        let mut pos_buffer = [0; core::mem::size_of::<GlyphPosition>()];
        let Ok(()) = memory.read(
            caller.as_context_mut(),
            buffer.position as usize + i as usize * core::mem::size_of::<GlyphPosition>(),
            &mut pos_buffer,
        ) else {
            // eprintln!("bad position being read");
            return 0;
        };
        let pos = unsafe { core::mem::transmute(pos_buffer) };
        caller.data_mut().buffer.pos.push(pos);
    }

    1
}

// fn NOT_WORKING_buffer_set_contents(mut caller: Caller<'_, ShapingData>, _buffer: u32, cbuffer: u32) -> u32 {
//     let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

//     let mut buffer = [0; core::mem::size_of::<CBufferContents>()];
//     memory.read(caller.as_context_mut(), cbuffer as usize, &mut buffer);
//     let Ok(buffer_contents) = bytemuck::try_from_bytes::<CBufferContents>(&buffer) else {
//         return 0;
//     };

//     // These clears seem broken somehow

//     caller.data_mut().buffer.info.clear();
//     caller.data_mut().buffer.pos.clear();

//     // This code here was me trying to bytemuck these, but I hit a wall with the
//     // source data is not correctly aligned for target data bytemuck error
//     // The Unaligned workaround stopped the bytemuck errors but .. er .. fucked
//     // things up elsewhere.

//     #[derive(Copy, Clone)]
//     #[repr(C, packed)]
//     struct Unaligned<T>(T);
//     unsafe impl<T: bytemuck::Pod> bytemuck::Pod for Unaligned<T> {}
//     unsafe impl<T: bytemuck::Zeroable> bytemuck::Zeroable for Unaligned<T> {}

//     let mut info_buffer = Vec::new();
//     let Ok(()) = memory.read(
//         caller.as_context_mut(),
//         buffer_contents.info as usize,
//         &mut info_buffer,
//     ) else {
//         return 0;
//     };
//     let info_slice: &[Unaligned<hb_glyph_info_t>] = bytemuck::try_cast_slice(&info_buffer).unwrap();
//     caller
//         .data_mut()
//         .buffer
//         .info
//         .extend(info_slice.iter().map(|v| v.0));
//     let mut pos_buffer = Vec::new();
//     let Ok(()) = memory.read(
//         caller.as_context_mut(),
//         buffer_contents.info as usize,
//         &mut pos_buffer,
//     ) else {
//         return 0;
//     };
//     let pos_slice: &[Unaligned<GlyphPosition>] = bytemuck::try_cast_slice(&pos_buffer).unwrap();
//     caller
//         .data_mut()
//         .buffer
//         .pos
//         .extend(pos_slice.iter().map(|v| v.0));

//     1
// }

// fn debugprint(s: *const u8);
// Produces a debugging message in the host shaper's log output; the variants debugprint1 ... debugprint4 suffix the message with a comma-separated list of the integer arguments.
// rust varargs when
fn debugprint(mut caller: Caller<'_, ShapingData>, s: u32) {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    // seems reasonable?
    let bytes = &memory.data(&caller)[s as usize..];
    let msg = CStr::from_bytes_until_nul(bytes)
        .unwrap_or_default()
        .to_string_lossy();

    std::eprintln!("{msg}"); // maybe?
}

// fn shape_with(font: u32, buffer: u32, features: u32, num_features: u32, shaper: *const u8) -> i32;
// Run another shaping engine's shaping process on the given font and buffer. The only shaping engine guaranteed to be available is ot, the OpenType shaper, but others may also be available. This allows the WASM author to process a buffer "normally", before further manipulating it.
// I think we should just use the default rustybuzz shaper for now.
fn shape_with(
    mut caller: Caller<'_, ShapingData>,
    font: u32,
    buffer: u32,
    features: u32,
    num_features: u32,
    _shaper: u32,
) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    // let ret = shape_with_plan(face, &plan, unicode_buffer); // <-- uncommenting this gives a compiler error
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

/// Some data provided by ~~Harfbuzz~~. rustybuzz
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Blob {
    /// Length of the blob in bytes
    pub length: u32,
    /// A raw pointer to the contents
    pub data: u32, // *mut u8
}

// Are these correct?
unsafe impl bytemuck::Zeroable for Blob {}
unsafe impl bytemuck::Pod for Blob {}

// using rustybuzz types instead of custom types
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct CBufferContents {
    length: u32,
    info: u32,
    position: u32,
}

// Are these correct?
unsafe impl bytemuck::Zeroable for CBufferContents {}
unsafe impl bytemuck::Pod for CBufferContents {}

#[cfg(test)]
mod tests {
    use std::print;

    use super::*;

    #[test]
    fn name() -> Result<(), ()> {
        // here are the functions needed for this font:
        //
        // (import "env" "buffer_copy_contents" (func (;0;) (type 0)))
        // (import "env" "buffer_set_contents" (func (;1;) (type 0)))
        // (import "env" "font_get_glyph" (func (;2;) (type 3)))
        // (import "env" "font_get_glyph_h_advance" (func (;3;) (type 0)))
        // (import "env" "debugprint" (func (;4;) (type 2)))
        // (export "memory" (memory 0))
        // (export "shape" (func 50))

        let calculator_font =
            include_bytes!("../../tests/fonts/text-rendering-tests/Calculator-Regular.ttf");
        let face = hb_font_t::from_slice(calculator_font, 0).unwrap();

        let mut buffer = UnicodeBuffer::new();
        buffer.push_str("22/7=");

        let plan = hb_ot_shape_plan_t::new(&face, crate::Direction::LeftToRight, None, None, &[]);

        let res = shape_with_wasm(&face, &plan, buffer).unwrap();

        print!("{:?}", res.glyph_infos());
        // This is returning the glyphs for     22/7=
        // should return                        3.142857

        // Hey at least it is not None ....

        Ok(())
    }
}
