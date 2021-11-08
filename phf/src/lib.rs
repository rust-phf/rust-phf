//! Rust-PHF is a library to generate efficient lookup tables at compile time using
//! [perfect hash functions](http://en.wikipedia.org/wiki/Perfect_hash_function).
//!
//! It currently uses the
//! [CHD algorithm](http://cmph.sourceforge.net/papers/esa09.pdf) and can generate
//! a 100,000 entry map in roughly .4 seconds. By default statistics are not
//! produced, but if you set the environment variable `PHF_STATS` it will issue
//! a compiler note about how long it took.
//!
//! MSRV (minimum supported rust version) is Rust 1.46.
//!
//! ## Usage
//!
//! PHF data structures can be constructed via either the procedural
//! macros in the `phf_macros` crate or code generation supported by the
//! `phf_codegen` crate. If you prefer macros, you can easily use them by
//! enabling the `macros` feature of the `phf` crate, like:
//!
//!```toml
//! [dependencies]
//! phf = { version = "0.10", features = ["macros"] }
//! ```
//!
//! To compile the `phf` crate with a dependency on
//! libcore instead of libstd, enabling use in environments where libstd
//! will not work, set `default-features = false` for the dependency:
//!
//! ```toml
//! [dependencies]
//! # to use `phf` in `no_std` environments
//! phf = { version = "0.10", default-features = false }
//! ```
//!
//! ## Example (with the `macros` feature enabled)
//!
//! ```rust
//! use phf::phf_map;
//!
//! #[derive(Clone, Copy)]
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
//! ```
//!
//! Alternatively, you can use the [`phf_codegen`] crate to generate PHF datatypes
//! in a build script.
//!
//! [`phf_codegen`]: https://docs.rs/phf_codegen
//!
//! ## Note
//!
//! Currently, the macro syntax has some limitations and may not
//! work as you want. See [#183] or [#196] for example.
//!
//! [#183]: https://github.com/rust-phf/rust-phf/issues/183
//! [#196]: https://github.com/rust-phf/rust-phf/issues/196

// XXX: Remove on stabilization.
#![allow(incomplete_features)]
#![feature(generic_const_exprs, const_trait_impl)]
#![doc(html_root_url = "https://docs.rs/phf/0.11")]
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std as core;

// Not part of the public API. Used by the macro facade.
#[cfg(feature = "macros")]
#[doc(hidden)]
pub extern crate phf_macros as __phf_macros;

#[cfg(feature = "macros")]
#[doc(hidden)]
pub const fn build_map<Key: 'static, Value: 'static, const N: usize>(
    state: &'static ([(Key, Value); N], phf_generator::HashState<N>),
) -> Map<Key, Value>
where
    [(); (N + phf_generator::DEFAULT_LAMBDA - 1) / phf_generator::DEFAULT_LAMBDA]: Sized,
{
    Map {
        key: state.1.key,
        disps: &*state.1.disps,
        entries: &state.0,
    }
}

#[cfg(feature = "macros")]
#[doc(hidden)]
pub const fn build_ordered_map<Key: 'static, Value: 'static, const N: usize>(
    state: &'static ([(Key, Value); N], phf_generator::HashState<N>),
) -> OrderedMap<Key, Value>
where
    [(); (N + phf_generator::DEFAULT_LAMBDA - 1) / phf_generator::DEFAULT_LAMBDA]: Sized,
{
    OrderedMap {
        key: state.1.key,
        disps: &*state.1.disps,
        idxs: &*state.1.map,
        entries: &state.0,
    }
}

#[cfg(feature = "macros")]
/// Macro to create a `static` (compile-time) [`Map`].
///
/// Requires the `macros` feature.
///
/// Supported key expressions are:
/// - literals: bools, (byte) strings, bytes, chars, and integers (these must have a type suffix)
/// - arrays of `u8` integers
/// - `UniCase::unicode(string)` or `UniCase::ascii(string)` if the `unicase` feature is enabled
///
/// # Example
///
/// ```
/// use phf::{phf_map, Map};
///
/// static MY_MAP: Map<&'static str, u32> = phf_map! {
///     "hello" => 1,
///     "world" => 2,
/// };
///
/// fn main () {
///     assert_eq!(MY_MAP["hello"], 1);
/// }
/// ```
#[macro_export]
macro_rules! phf_map {
    ($($key:expr => $value:expr),* $(,)*) => {
        $crate::build_map(&$crate::__phf_macros::phf_map(&[$(($key, $value)),*]))
    };
}

#[cfg(feature = "macros")]
/// Macro to create a `static` (compile-time) [`OrderedMap`].
///
/// Requires the `macros` feature. Same usage as [`phf_map`].
#[macro_export]
macro_rules! phf_ordered_map {
    ($($key:expr => $value:expr),* $(,)*) => {
        $crate::build_ordered_map(
            &$crate::__phf_macros::phf_ordered_map(&[$(($key, $value)),*]),
        )
    };
}

#[cfg(feature = "macros")]
/// Macro to create a `static` (compile-time) [`Set`].
///
/// Requires the `macros` feature.
///
/// # Example
///
/// ```
/// use phf::{phf_set, Set};
///
/// static MY_SET: Set<&'static str> = phf_set! {
///     "hello world",
///     "hola mundo",
/// };
///
/// fn main () {
///     assert!(MY_SET.contains("hello world"));
/// }
/// ```
#[macro_export]
macro_rules! phf_set {
    ($($key:expr),* $(,)*) => {
        $crate::Set {
            map: $crate::build_map(&$crate::__phf_macros::phf_set(&[$($key),*])),
        }
    };
}

#[cfg(feature = "macros")]
/// Macro to create a `static` (compile-time) [`OrderedSet`].
///
/// Requires the `macros` feature. Same usage as [`phf_set`].
#[macro_export]
macro_rules! phf_ordered_set {
    ($($key:expr),* $(,)*) => {
        $crate::OrderedSet {
            map: $crate::build_ordered_map(
                &$crate::__phf_macros::phf_ordered_set(&[$($key),*]),
            ),
        }
    };
}

#[doc(inline)]
pub use self::map::Map;
#[doc(inline)]
pub use self::ordered_map::OrderedMap;
#[doc(inline)]
pub use self::ordered_set::OrderedSet;
#[doc(inline)]
pub use self::set::Set;
pub use phf_shared::PhfHash;

pub mod map;
pub mod ordered_map;
pub mod ordered_set;
pub mod set;
