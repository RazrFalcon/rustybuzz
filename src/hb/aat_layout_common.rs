use crate::hb::aat_map::range_flags_t;
use crate::hb::buffer::hb_buffer_t;
use crate::hb::face::hb_font_t;
use crate::hb::hb_mask_t;
use crate::hb::ot_shape_plan::hb_ot_shape_plan_t;

// We need this ugly struct to circumvent issues with the borrow
// checker
pub struct range_flag_t_wrapper<'a> {
    range_flags: Option<&'a mut [range_flags_t]>,
    range_flags_index: usize,
}

impl<'a> range_flag_t_wrapper<'a> {
    pub fn new() -> Self {
        Self {
            range_flags: None,
            range_flags_index: 0,
        }
    }

    pub fn set_range_flags(&mut self, range_flags: &'a mut [range_flags_t]) {
        self.range_flags = Some(range_flags);
    }

    pub fn set_range_flags_index(&mut self, index: usize) {
        self.range_flags_index = index;
    }

    pub fn get(&self) -> Option<&[range_flags_t]> {
        let index = self.range_flags_index;

        if let Some(range_flags) = &self.range_flags {
            return range_flags.get(index..);
        }

        None
    }
}

pub struct hb_aat_apply_context_t<'a> {
    pub plan: &'a hb_ot_shape_plan_t,
    pub face: &'a hb_font_t<'a>,
    pub buffer: &'a mut hb_buffer_t,
    pub range_flags: range_flag_t_wrapper<'a>,
    pub subtable_flags: hb_mask_t,
}

impl<'a> hb_aat_apply_context_t<'a> {
    pub fn new(
        plan: &'a hb_ot_shape_plan_t,
        face: &'a hb_font_t<'a>,
        buffer: &'a mut hb_buffer_t,
    ) -> Self {
        Self {
            plan,
            face,
            buffer,
            range_flags: range_flag_t_wrapper::new(),
            subtable_flags: 0,
        }
    }
}
