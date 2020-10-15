pub mod feature;
mod fallback;
mod map;
mod map_builder;
mod shape_normalize_context;
mod shape_plan;
mod shape_planner;

pub use map::*;
pub use map_builder::*;
pub use shape_normalize_context::*;
pub use shape_plan::*;
pub use shape_planner::*;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TableIndex {
    GSUB = 0,
    GPOS = 1,
}
