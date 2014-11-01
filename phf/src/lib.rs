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
#[doc(inline)]
pub use map::Map;
#[doc(inline)]
pub use set::Set;
#[doc(inline)]
pub use ordered_map::OrderedMap;
#[doc(inline)]
pub use ordered_set::OrderedSet;

#[deprecated = "renamed to Map"]
pub use Map as PhfMap;
#[deprecated = "renamed to Set"]
pub use Set as PhfSet;
#[deprecated = "renamed to OrderedMap"]
pub use OrderedMap as PhfOrderedMap;
#[deprecated = "renamed to OrderedSet"]
pub use OrderedSet as PhfOrderedSet;

#[path="../../shared/mod.rs"]
mod shared;
pub mod map;
pub mod set;
pub mod ordered_map;
pub mod ordered_set;

mod std {
    pub use core::fmt;
}

