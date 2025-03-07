//! CPHF is a library to generate efficient lookup tables at compile time using
//! [perfect hash functions](http://en.wikipedia.org/wiki/Perfect_hash_function).
//!
//! It currently uses the
//! [CHD algorithm](http://cmph.sourceforge.net/papers/esa09.pdf) and can generate
//! a 120 entry map in roughly 1 second.
//!
//! MSRV (minimum supported rust version) is Rust 1.85.
//! In contrast to the excellent [`phf`](https://github.com/rust-phf/rust-phf/) crate, this crate
//! uses no code generation outside of normal `macro_rules`, and instead generates the map using `const` expressions.
//! This does mean this crate several orders of magnitude slower than `phf` (so maps with thousands of entries would probably be
//! better served by `phf`), but doing so allows it to avoid all the major drawbacks. Namely, [nested maps](https://github.com/rust-phf/rust-phf/issues/183) are supported [any type](https://github.com/rust-phf/rust-phf/issues/196) can be used as a key (provided it implements the correct traits and pseudo-traits).
//!
//! ## Usage
//!
//! Simply add `cphf` as a depenency and utilize the included [`phf_ordered_map`] macro to construct an [`OrderedMap`]
//! or the [`phf_ordered_set`] macro to construct an [`OrderedSet`]
//!
//! ```
//! use cphf::{phf_ordered_map, OrderedMap};
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
//! static KEYWORDS: OrderedMap<&'static str, Keyword> = phf_ordered_map! {&'static str, Keyword;
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
//! ```
//!
//! Inclusion of duplicate keys will result in a compiler error.
//!
//! ```compile_fail
//! use cphf::{phf_ordered_set, OrderedSet};
//! static DUPLICATE_KEYS: OrderedSet<u32> = phf_ordered_set! {u32;
//!     0,
//!     1,
//!     0,
//! };
//! ```
//!
//! ```compile_fail
//! use cphf::{phf_ordered_map, OrderedMap};
//! static DUPLICATE_KEYS: OrderedMap<u32, ()> = phf_ordered_map! {u32, ();
//!     0 => (),
//!     1 => (),
//!     0 => (),
//! };
//! ```

#![no_std]

use const_siphasher::sip128::Hash128;

pub use const_siphasher::sip128::SipHasher13 as Hasher;
#[doc(hidden)]
pub use sort_const::const_shellsort;

mod containers;
mod generator;
mod keys;
mod macros;
mod rand;

pub use containers::*;
pub use generator::Generator;
pub use keys::*;
pub use macros::*;

/// The final computed state during map generation
///
/// Exporting this way hides the actual type so it can only be used by the macros.
#[doc(hidden)]
pub type BuilderState<const LEN: usize, const BUCKET_LEN: usize> =
    generator::BuilderState<LEN, BUCKET_LEN>;

/// A hash result broken down into parts for ease of use in displacement
#[doc(hidden)]
#[derive(Default)]
pub struct HashValue {
    g: u32,
    f1: u32,
    f2: u32,
}

impl HashValue {
    pub const fn new() -> Self {
        HashValue { f1: 0, f2: 0, g: 0 }
    }

    /// Construct a hash from the state
    pub const fn finalize(hasher: Hasher) -> Self {
        let Hash128 {
            h1: lower,
            h2: upper,
        } = hasher.finish128();

        HashValue {
            g: (lower >> 32) as u32,
            f1: lower as u32,
            f2: upper as u32,
        }
    }

    pub const fn eq(&self, other: &Self) -> bool {
        self.g == other.g && self.f1 == other.f1 && self.f2 == other.f2
    }

    pub const fn gt(&self, other: &Self) -> bool {
        let lhs = ((self.g as u128) << 64) | ((self.f1 as u128) << 32) | self.f2 as u128;
        let rhs = ((other.g as u128) << 64) | ((other.f1 as u128) << 32) | other.f2 as u128;
        lhs > rhs
    }
}

/// Displace as `d2 + f1*d1 + f2`
#[inline(always)]
const fn displace(f1: u32, f2: u32, d1: u32, d2: u32) -> u32 {
    d2.wrapping_add(f1.wrapping_mul(d1)).wrapping_add(f2)
}
