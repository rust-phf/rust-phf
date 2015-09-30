//! Compile time optimized maps and sets.
//!
//! PHF data structures can be generated via the syntax extensions in the
//! `phf_macros` crate or via code generation in the `phf_codegen` crate. See
//! the documentation of those crates for more details.
#![doc(html_root_url="https://sfackler.github.io/rust-phf/doc/v0.7.5")]
#![warn(missing_docs)]
#![cfg_attr(feature = "core", feature(no_std))]
#![cfg_attr(feature = "core", no_std)]

#[cfg(not(feature = "core"))]
extern crate std as core;

extern crate debug_builders;
extern crate phf_shared;

pub use phf_shared::PhfHash;
#[doc(inline)]
pub use map::Map;
#[doc(inline)]
pub use set::Set;
#[doc(inline)]
pub use ordered_map::OrderedMap;
#[doc(inline)]
pub use ordered_set::OrderedSet;

pub mod map;
pub mod set;
pub mod ordered_map;
pub mod ordered_set;
