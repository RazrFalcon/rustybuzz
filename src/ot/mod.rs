pub mod feature;
mod fallback;
mod map;
mod map_builder;
mod normalize;
mod shape_plan;
mod shape_planner;
mod complex_shaper;

pub use map::*;
pub use map_builder::*;
pub use normalize::*;
pub use shape_plan::*;
pub use shape_planner::*;
pub use complex_shaper::*;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TableIndex {
    GSUB = 0,
    GPOS = 1,
}
