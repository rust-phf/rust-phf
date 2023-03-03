#![doc = include_str!("../README.md")]
#![doc(html_root_url = "https://docs.rs/phf/0.11")]
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std as core;

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
pub use phf_macros::phf_map;

#[cfg(feature = "macros")]
/// Macro to create a `static` (compile-time) [`OrderedMap`].
///
/// Requires the `macros` feature. Same usage as [`phf_map`].
pub use phf_macros::phf_ordered_map;

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
pub use phf_macros::phf_set;

#[cfg(feature = "macros")]
/// Macro to create a `static` (compile-time) [`OrderedSet`].
///
/// Requires the `macros` feature. Same usage as [`phf_set`].
pub use phf_macros::phf_ordered_set;

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
