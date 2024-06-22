use crate::hb::buffer::hb_buffer_t;
use crate::hb::face::hb_font_t;
use crate::hb::ot_shape_plan::hb_ot_shape_plan_t;

pub struct hb_aat_apply_context_t<'a> {
    pub plan: &'a hb_ot_shape_plan_t,
    pub face: &'a hb_font_t<'a>,
    pub buffer: &'a mut hb_buffer_t,
}