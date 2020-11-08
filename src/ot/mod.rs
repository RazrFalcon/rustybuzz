pub mod feature;
mod apply;
mod complex_shaper;
mod contextual;
mod fallback;
mod kern;
mod layout;
mod map;
mod map_builder;
mod matching;
mod normalize;
mod position;
mod shape_plan;
mod shape_planner;
mod substitute;

pub use complex_shaper::*;
pub use layout::TableIndex;
pub use map::*;
pub use map_builder::*;
pub use normalize::*;
pub use shape_plan::*;
pub use shape_planner::*;

use ttf_parser::parser::NumFrom;

use crate::{ffi, Tag};

fn table_data(face: *const ffi::rb_face_t, table_tag: Tag) -> &'static [u8] {
    unsafe {
        let mut len = 0;
        let data = ffi::rb_face_get_table_data(face, table_tag, &mut len);
        std::slice::from_raw_parts(data, usize::num_from(len))
    }
}
