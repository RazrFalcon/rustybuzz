mod extended_kerning;
mod feature_mappings;
mod feature_selector;
mod map;
mod metamorphosis;
mod tracking;

pub use map::*;

use crate::buffer::hb_buffer_t;
use crate::hb_font_t;
use crate::shape_plan::hb_ot_shape_plan_t;

pub fn substitute(plan: &hb_ot_shape_plan_t, face: &hb_font_t, buffer: &mut hb_buffer_t) {
    metamorphosis::apply(plan, face, buffer);
}

pub fn position(plan: &hb_ot_shape_plan_t, face: &hb_font_t, buffer: &mut hb_buffer_t) {
    extended_kerning::apply(plan, face, buffer);
}

pub fn track(plan: &hb_ot_shape_plan_t, face: &hb_font_t, buffer: &mut hb_buffer_t) {
    tracking::apply(plan, face, buffer);
}

pub fn zero_width_deleted_glyphs(buffer: &mut hb_buffer_t) {
    for i in 0..buffer.len {
        if buffer.info[i].glyph_id == 0xFFFF {
            buffer.pos[i].x_advance = 0;
            buffer.pos[i].y_advance = 0;
            buffer.pos[i].x_offset = 0;
            buffer.pos[i].y_offset = 0;
        }
    }
}

pub fn remove_deleted_glyphs(buffer: &mut hb_buffer_t) {
    buffer.delete_glyphs_inplace(|info| info.glyph_id == 0xFFFF)
}
