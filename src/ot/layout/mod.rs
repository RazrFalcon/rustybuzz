//! OpenType layout.

#![allow(dead_code)]

mod apply;
mod common;
mod context_lookups;
mod dyn_array;
mod gpos;
mod gsub;
mod matching;

pub const MAX_NESTING_LEVEL: usize = 6;
pub const MAX_CONTEXT_LENGTH: usize = 64;
