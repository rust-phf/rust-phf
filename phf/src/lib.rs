//! Compile time optimized maps and sets.
//!
//! Keys can be string literals, byte string literals, byte literals, char
//! literals, or any of the fixed-size integral types.
#![doc(html_root_url="https://sfackler.github.io/doc")]
#![warn(missing_docs)]
#![feature(macro_rules, tuple_indexing, phase, globs)]
#![no_std]

#[phase(plugin, link)]
extern crate core;
extern crate collections;

pub use shared::PhfHash;
pub use map::Map;
pub use set::Set;
pub use ordered_map::OrderedMap;
pub use ordered_set::OrderedSet;

#[path="../../shared/mod.rs"]
mod shared;
pub mod map;
pub mod set;
pub mod ordered_map;
pub mod ordered_set;

mod std {
    pub use core::fmt;
}

