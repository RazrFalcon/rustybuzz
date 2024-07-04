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
    let module_name = module.name().unwrap_or_default();

    let mut linker = Linker::new(&store.engine());

    // ====
    // below are the functions that are imported by harfbuzz-wasm crate.
    // I copied the signatures here and I am trying to satisfy the compiler as much as I can
    // pointer land is beyond me.

    // fn face_get_upem(face: u32) -> u32;
    // Returns the units-per-em of the font face.
    let face_get_upem = |caller: Caller<'_, ShapingData>, _face: u32| -> u32 {
        caller.data().font.units_per_em as u32
    };
    linker
        .func_wrap(module_name, "face_get_upem", face_get_upem)
        .ok()?;

    // fn font_get_face(font: u32) -> u32;
    // Creates a new face token from the given font token.
    let font_get_face = |caller: Caller<'_, ShapingData>, _: u32| {
        // From HarfBuzz docs:
        // (In the following functions, a font is a specific instantiation of a face at a
        // particular scale factor and variation position.)
        //
        // I am unsure how to represent that in rustybuzz.
        // er .. do what here?
        0
    };
    linker
        .func_wrap(module_name, "font_get_face", font_get_face)
        .ok()?;

    // fn font_get_glyph(font: u32, unicode: u32, uvs: u32) -> u32;
    // Returns the nominal glyph ID for the given codepoint, using the cmap table of the font to map Unicode codepoint (and variation selector) to glyph ID.
    let font_get_glyph =
        |mut caller: Caller<'_, ShapingData>, _: u32, codepoint: u32, uvs: u32| -> u32 {
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
        };
    linker
        .func_wrap(module_name, "font_get_glyph", font_get_glyph)
        .ok()?;

    // fn font_get_scale(font: u32, x_scale: *mut i32, y_scale: *mut i32);
    // Returns the scale of the current font.
    // Just return the upem as rustybuzz has no scale.
    let font_get_scale =
        |mut caller: Caller<'_, ShapingData>, _: u32, x_scale: u32, y_scale: u32| {
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
        };
    linker
        .func_wrap(module_name, "font_get_scale", font_get_scale)
        .ok()?;

    // fn font_get_glyph_extents(font: u32, glyph: u32, extents: *mut CGlyphExtents) -> bool;
    // Returns the glyph's extents for the given glyph ID at current scale and variation settings.
    let font_get_glyph_extents =
        |mut caller: Caller<'_, ShapingData>, _: u32, glyph: u32, extents: u32| -> u32 {
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

            let mut glyph_extents = hb_glyph_extents_t::default();
            let ret = caller
                .data()
                .font
                .glyph_extents(GlyphId(glyph as u16), &mut glyph_extents);
            if ret {
                // WASM is little endian. MacOS is little endian, I think.
                // Maybe should use bytemuck as a dependency instead.
                let glyph_extents = unsafe { std::mem::transmute(glyph_extents) };
                memory.write(caller.as_context_mut(), extents as usize, glyph_extents);
            }

            ret as u32
        };
    linker
        .func_wrap(
            module_name,
            "font_get_glyph_extents",
            font_get_glyph_extents,
        )
        .ok()?;

    // fn font_glyph_to_string(font: u32, glyph: u32, str: *const u8, len: u32);
    // Copies the name of the given glyph, or, if no name is available, a string of the form gXXXX into the given string.
    let font_glyph_to_string =
        |mut caller: Caller<'_, ShapingData>, _: u32, glyph: u32, str: u32, _len: u32| {
            // what am I supposed to do with `len` here? Should I trim the name to `len` length somehow?
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

            let name = caller
                .data()
                .font
                .glyph_name(GlyphId(glyph as u16))
                .map(ToOwned::to_owned)
                .unwrap_or(format!("g{glyph:4}"));

            let name = CString::new(name).unwrap();

            memory.write(caller.as_context_mut(), str as usize, name.as_bytes());
        };
    linker
        .func_wrap(module_name, "font_glyph_to_string", font_glyph_to_string)
        .ok()?;

    // fn font_get_glyph_h_advance(font: u32, glyph: u32) -> i32;
    // Returns the default horizontal advance for the given glyph ID the current scale and variations settings.
    let font_get_glyph_h_advance = |caller: Caller<'_, ShapingData>, _: u32, glyph: u32| -> i32 {
        caller.data().font.glyph_h_advance(GlyphId(glyph as u16))
    };
    linker
        .func_wrap(
            module_name,
            "font_get_glyph_h_advance",
            font_get_glyph_h_advance,
        )
        .ok()?;

    // fn font_get_glyph_v_advance(font: u32, glyph: u32) -> i32;
    // Returns the default vertical advance for the given glyph ID the current scale and variations settings.
    let font_get_glyph_v_advance = |caller: Caller<'_, ShapingData>, _: u32, glyph: u32| -> i32 {
        caller.data().font.glyph_v_advance(GlyphId(glyph as u16))
    };
    linker
        .func_wrap(
            module_name,
            "font_get_glyph_v_advance",
            font_get_glyph_v_advance,
        )
        .ok()?;

    // fn font_copy_glyph_outline(font: u32, glyph: u32, outline: *mut CGlyphOutline) -> bool;
    // Copies the outline of the given glyph ID, at current scale and variation settings, into the outline structure provided.
    let font_copy_glyph_outline =
        |mut caller: Caller<'_, ShapingData>, _: u32, glyph: u32, outline: u32| -> u32 {
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

            let builder = CGlyphOutline {
                n_points: todo!(),
                points: todo!(),
                n_contours: todo!(),
                contours: todo!(),
            };
            // also no clue what to do here
            // let my_ol = caller.data().outline_glyph(GlyphId(glyph as u16), builder); // ??

            let Ok(()) = memory.write(caller, outline as usize, todo!("builder result goes here"))
            else {
                return 0;
            };

            1
        };
    linker
        .func_wrap(
            module_name,
            "font_copy_glyph_outline",
            font_copy_glyph_outline,
        )
        .ok()?;

    // fn face_copy_table(font: u32, tag: u32, blob: *mut Blob) -> bool;
    // Copies the binary data in the OpenType table referenced by tag into the supplied blob structure.
    let face_copy_table =
        |mut caller: Caller<'_, ShapingData>, _: u32, tag: u32, blob: u32| -> u32 {
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

            let tag = tag.to_be_bytes(); // be or le?
            let Some(table) = caller.data().font.raw_face().table(Tag::from_bytes(&tag)) else {
                return 0;
            };

            let arbitrary = blob + std::mem::size_of::<Blob>() as u32;

            let my_blob = Blob {
                length: table.len() as u32,
                data: arbitrary, // is this correct?
            };
            let my_blob: [u8; std::mem::size_of::<Blob>()] =
                unsafe { std::mem::transmute(my_blob) };

            let Ok(()) = memory.write(caller.as_context_mut(), blob as usize, &my_blob) else {
                return 0;
            };

            let Ok(()) = memory.write(caller.as_context_mut(), arbitrary as usize, table) else {
                return 0;
            };

            1
        };
    linker
        .func_wrap(module_name, "face_copy_table", face_copy_table)
        .ok()?;

    // fn buffer_copy_contents(buffer: u32, cbuffer: *mut CBufferContents) -> bool;
    // Retrieves the contents of the host shaping engine's buffer into the buffer_contents structure. This should typically be called at the beginning of shaping.
    let buffer_copy_contents =
        |mut caller: Caller<'_, ShapingData>, _buffer: u32, cbuffer: u32| -> u32 {
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

            let rb_buffer = &caller.as_context().data().buffer;
            let length = rb_buffer.len as u32;

            let arbitrary_info = cbuffer + std::mem::size_of::<CBufferContents>() as u32;
            let arbitrary_pos =
                arbitrary_info + length * std::mem::size_of::<hb_glyph_info_t>() as u32;

            // // I can't figure out this section for now.
            //
            // for (i, (info, pos)) in rb_buffer
            //     .info
            //     .iter()
            //     .copied()
            //     .zip(rb_buffer.pos.iter().copied())
            //     .enumerate()
            // {
            //     let info: [u8; std::mem::size_of::<hb_glyph_info_t>()] =
            //         unsafe { std::mem::transmute(info) };
            //     let Ok(()) = memory.write(
            //         &mut caller.as_context_mut(),
            //         arbitrary_info as usize + i * std::mem::size_of::<hb_glyph_info_t>(),
            //         &info,
            //     ) else {
            //         return 0;
            //     };

            //     let pos: [u8; std::mem::size_of::<GlyphPosition>()] =
            //         unsafe { std::mem::transmute(pos) };
            //     let Ok(()) = memory.write(
            //         &mut caller.as_context_mut(),
            //         arbitrary_pos as usize + i * std::mem::size_of::<GlyphPosition>(),
            //         &pos,
            //     ) else {
            //         return 0;
            //     };
            // }

            let buffer_contents = CBufferContents {
                length,
                info: arbitrary_info,
                position: arbitrary_pos,
            };
            let buffer_contents: [u8; std::mem::size_of::<CBufferContents>()] =
                unsafe { std::mem::transmute(buffer_contents) };

            let Ok(()) = memory.write(
                &mut caller.as_context_mut(),
                cbuffer as usize,
                &buffer_contents,
            ) else {
                return 0;
            };

            1
        };
    linker.func_wrap(module_name, "buffer_copy_contents", buffer_copy_contents);

    // fn buffer_set_contents(buffer: u32, cbuffer: &CBufferContents) -> bool;
    // Copy the buffer_contents structure back into the host shaping engine's buffer. This should typically be called at the end of shaping.
    let buffer_set_contents =
        |mut caller: Caller<'_, ShapingData>, _buffer: u32, cbuffer: u32| -> u32 {
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

            let mut buffer = [0; 12];

            memory.read(caller.as_context_mut(), cbuffer as usize, &mut buffer);
            let buffer: CBufferContents = unsafe { std::mem::transmute(buffer) };

            caller.data_mut().buffer.clear_output();
            caller.data_mut().buffer.clear_positions();

            for i in 0..buffer.length {
                let mut info_buffer = [0; 20];
                memory.read(
                    caller.as_context_mut(),
                    (buffer.info + i) as usize,
                    &mut info_buffer,
                );
                let info = unsafe { std::mem::transmute(info_buffer) };
                caller.data_mut().buffer.info.push(info);

                let mut pos_buffer = [0; 20];
                memory.read(
                    caller.as_context_mut(),
                    (buffer.position + i) as usize,
                    &mut pos_buffer,
                );
                let pos = unsafe { std::mem::transmute(pos_buffer) };
                caller.data_mut().buffer.pos.push(pos);
            }

            1
        };
    linker
        .func_wrap(module_name, "buffer_set_contents", buffer_set_contents)
        .ok()?;

    // fn debugprint(s: *const u8);
    // Produces a debugging message in the host shaper's log output; the variants debugprint1 ... debugprint4 suffix the message with a comma-separated list of the integer arguments.
    // rust varargs when
    let debugprint = |mut caller: Caller<'_, ShapingData>, s: u32| {
        let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

        // seems reasonable?
        let bytes = &memory.data(&caller)[s as usize..];
        let msg = CStr::from_bytes_until_nul(bytes)
            .unwrap_or_default()
            .to_string_lossy();

        std::eprintln!("{msg}"); // maybe?
    };
    linker
        .func_wrap(module_name, "debugprint", debugprint)
        .ok()?;

    // fn shape_with(font: u32, buffer: u32, features: u32, num_features: u32, shaper: *const u8) -> i32;
    // Run another shaping engine's shaping process on the given font and buffer. The only shaping engine guaranteed to be available is ot, the OpenType shaper, but others may also be available. This allows the WASM author to process a buffer "normally", before further manipulating it.
    // I think we should just use the default rustybuzz shaper for now.
    let shape_with = |mut caller: Caller<'_, ShapingData>,
                      font: u32,
                      buffer: u32,
                      features: u32,
                      num_features: u32,
                      _shaper: u32|
     -> i32 {
        let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

        // let ret = shape_with_plan(face, &plan, unicode_buffer); // <-- uncommenting this gives a compiler error
        todo!()
    };
    linker
        .func_wrap(module_name, "shape_with", shape_with)
        .ok()?;

    // Here we are (supposedly) done creating functions.
    // draft section

    let instance = linker.instantiate(&mut store, &module).ok()?;
    let shape = instance
        .get_typed_func::<(u32, u32, u32, u32, u32), i32>(&mut store, "shape") // not sure if this is correct yet
        .ok()?;

    let ret = shape.call(&mut store, (0, 0, 0, 0, 0));

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
#[derive(Debug)]
#[repr(C)]
pub struct Blob {
    /// Length of the blob in bytes
    pub length: u32,
    /// A raw pointer to the contents
    pub data: u32, // *mut u8
}

// using rustybuzz types instead of custom types
#[derive(Debug)]
#[repr(C)]
struct CBufferContents {
    length: u32,
    info: u32,
    position: u32,
}
