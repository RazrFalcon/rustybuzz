use std::ffi::c_void;

use crate::{ffi, Face, Mask};
use crate::buffer::{Buffer, BufferScratchFlags};
use crate::plan::ShapePlan;
use crate::tables::gsubgpos::LookupFlags;
use super::TableIndex;
use super::apply::ApplyContext;
use super::matching::SkippyIter;

pub fn has_kerning(face: &Face) -> bool {
    unsafe { ffi::rb_ot_layout_has_kerning(face.as_ptr()) != 0 }
}

pub fn has_machine_kerning(face: &Face) -> bool {
    unsafe { ffi::rb_ot_layout_has_machine_kerning(face.as_ptr()) != 0 }
}

pub fn has_cross_kerning(face: &Face) -> bool {
    unsafe { ffi::rb_ot_layout_has_cross_kerning(face.as_ptr()) != 0 }
}

pub fn kern(plan: &ShapePlan, face: &Face, buffer: &mut Buffer) {
    unsafe { ffi::rb_ot_layout_kern(plan.as_ptr(), face.as_ptr(), buffer.as_ptr()); }
}

#[no_mangle]
pub extern "C" fn rb_kern_machine_kern(
    face: *const ffi::rb_face_t,
    buffer: *mut ffi::rb_buffer_t,
    kern_mask: ffi::rb_mask_t,
    cross_stream: ffi::rb_bool_t,
    machine: *const c_void,
    machine_get_kerning: unsafe extern "C" fn(
        *const c_void,
        ffi::rb_codepoint_t,
        ffi::rb_codepoint_t,
    ) -> ffi::rb_position_t,
) {
    let face = Face::from_ptr(face);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    let cross_stream = cross_stream != 0;
    machine_kern(face, &mut buffer, kern_mask, cross_stream, |left, right| {
        unsafe { machine_get_kerning(machine, left, right) }
    });
}

fn machine_kern(
    face: &Face,
    buffer: &mut Buffer,
    kern_mask: Mask,
    cross_stream: bool,
    get_kerning: impl Fn(u32, u32) -> i32,
) {
    let mut ctx = ApplyContext::new(TableIndex::GPOS, face, buffer);
    ctx.lookup_mask = kern_mask;
    ctx.lookup_props = u32::from(LookupFlags::IGNORE_MARKS.bits());

    let horizontal = ctx.buffer.direction.is_horizontal();

    let mut i = 0;
    while i < ctx.buffer.len {
        if (ctx.buffer.info[i].mask & kern_mask) == 0 {
            i += 1;
            continue;
        }

        let mut iter = SkippyIter::new(&ctx, i, 1, false);
        if !iter.next() {
            i += 1;
            continue;
        }

        let j = iter.index();

        let info = &ctx.buffer.info;
        let kern = get_kerning(info[i].codepoint, info[j].codepoint);

        let pos = &mut ctx.buffer.pos;
        if kern != 0 {
            if horizontal {
                if cross_stream {
                    pos[j].y_offset = kern;
                    ctx.buffer.scratch_flags |= BufferScratchFlags::HAS_GPOS_ATTACHMENT;
                } else {
                    let kern1 = kern >> 1;
                    let kern2 = kern - kern1;
                    pos[i].x_advance += kern1;
                    pos[j].x_advance += kern2;
                    pos[j].x_offset += kern2;
                }
            } else {
                if cross_stream {
                    pos[j].x_offset = kern;
                    ctx.buffer.scratch_flags |= BufferScratchFlags::HAS_GPOS_ATTACHMENT;
                } else {
                    let kern1 = kern >> 1;
                    let kern2 = kern - kern1;
                    pos[i].y_advance += kern1;
                    pos[j].y_advance += kern2;
                    pos[j].y_offset += kern2;
                }
            }

            ctx.buffer.unsafe_to_break(i, j + 1)
        }

        i = j;
    }
}
