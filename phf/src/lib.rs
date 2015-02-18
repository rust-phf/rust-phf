//! Compile time optimized maps and sets.
//!
//! Keys can be string literals, byte string literals, byte literals, char
//! literals, or any of the fixed-size isizeegral types.
//!
//! # Examples
//!
//! ```rust
//! #![feature(plugin)]
//! #![plugin(phf_macros)]
//!
//! extern crate phf;
//!
//! #[derive(Clone)]
//! pub enum Keyword {
//!     Loop,
//!     Continue,
//!     Break,
//!     Fn,
//!     Extern,
//! }
//!
//! static KEYWORDS: phf::Map<&'static str, Keyword> = phf_map! {
//!     "loop" => Keyword::Loop,
//!     "continue" => Keyword::Continue,
//!     "break" => Keyword::Break,
//!     "fn" => Keyword::Fn,
//!     "extern" => Keyword::Extern,
//! };
//!
//! pub fn parse_keyword(keyword: &str) -> Option<Keyword> {
//!     KEYWORDS.get(keyword).cloned()
//! }
//! # fn main() {}
//! ```
#![doc(html_root_url="https://sfackler.github.io/rust-phf/doc")]
#![feature(core, no_std)]
#![warn(missing_docs)]
#![no_std]

#[macro_use]
#[cfg(feature = "core")]
extern crate core;
#[macro_use]
#[cfg(not(feature = "core"))]
extern crate std;
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

#[cfg(feature = "core")]
mod std {
    pub use core::{borrow, fmt, iter, option, ops, slice};
    pub mod prelude {
        pub use core::prelude as v1;
    }
}

#[cfg(not(feature = "core"))]
mod core {
    pub use std::{fmt, option, iter};
}
