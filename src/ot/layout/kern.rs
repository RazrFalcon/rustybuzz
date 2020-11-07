use std::ffi::c_void;

use super::apply::ApplyContext;
use super::matching::SkippyIter;
use crate::buffer::{Buffer, BufferScratchFlags};
use crate::{ffi, Font, Mask};

#[no_mangle]
pub extern "C" fn rb_kern_machine_kern(
    ctx: *const ffi::rb_ot_apply_context_t,
    font: *const ffi::rb_font_t,
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
    let ctx = ApplyContext::from_ptr(ctx);
    let font = Font::from_ptr(font);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    let cross_stream = cross_stream != 0;
    kern(&ctx, font, &mut buffer, kern_mask, cross_stream, |left, right| {
        unsafe { machine_get_kerning(machine, left, right) }
    });
}

fn kern(
    ctx: &ApplyContext,
    _: &Font,
    buffer: &mut Buffer,
    kern_mask: Mask,
    cross_stream: bool,
    get_kerning: impl Fn(u32, u32) -> i32,
) {
    let len = buffer.len;
    let horizontal = buffer.direction.is_horizontal();

    let mut i = 0;
    while i < len {
        if (buffer.info[i].mask & kern_mask) == 0 {
            i += 1;
            continue;
        }

        let mut iter = SkippyIter::new(&ctx, i, 1, false);
        if !iter.next() {
            i += 1;
            continue;
        }

        let j = iter.index();

        let info = &buffer.info;
        let kern = get_kerning(info[i].codepoint, info[j].codepoint);

        let pos = &mut buffer.pos;
        if kern != 0 {
            if horizontal {
                if cross_stream {
                    pos[j].y_offset = kern;
                    buffer.scratch_flags |= BufferScratchFlags::HAS_GPOS_ATTACHMENT;
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
                    buffer.scratch_flags |= BufferScratchFlags::HAS_GPOS_ATTACHMENT;
                } else {
                    let kern1 = kern >> 1;
                    let kern2 = kern - kern1;
                    pos[i].y_advance += kern1;
                    pos[j].y_advance += kern2;
                    pos[j].y_offset += kern2;
                }
            }

            buffer.unsafe_to_break(i, j + 1)
        }

        i = j;
    }
}
