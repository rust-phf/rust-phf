#[doc(hidden)]
pub use ::phf_macros::{
    phf_map as __phf_map,
    phf_ordered_map as __phf_ordered_map,
    phf_set as __phf_set,
    phf_ordered_set as __phf_ordered_set,
};

#[doc(hidden)]
#[macro_export]
macro_rules! item {($item:item) => ($item)}

/// Declares a `static` phf map
///
/// # Example
///
/// ```rust
/// # #[macro_use] extern crate phf;
/// type K = &'static str;
/// type V = usize;
///
/// phf_map! {
///     static MAP: phf::Map<K, V> = {
///         "KEY1" => 1,
///         "KEY2" => 2,
///     };
/// }
///
/// assert_eq!(MAP.get("KEY1"), Some(&1));
/// assert_eq!(MAP.get("KEY2"), Some(&2));
/// assert_eq!(MAP.get("MISSING_KEY"), None);
/// ```
#[macro_export]
macro_rules! phf_map {
    ( $($tt:tt)* ) => (
        $crate::item! { $crate::__phf_map! {
            $($tt)*
        }}
    );
    // this is for the readability of the macro arguments, documentation-wise
    (
        static $VARNAME:ident : $type:ty = {
            $($key_literal:tt => $value:expr),*
        };
    ) => ();
}

/// Declares a `static` phf ordered map
///
/// # Example
///
/// ```rust
/// # #[macro_use] extern crate phf;
/// type K = &'static str;
/// type V = usize;
///
/// phf_ordered_map! {
///     static MAP: phf::OrderedMap<K, V> = {
///         "KEY1" => 1,
///         "KEY2" => 2,
///     };
/// }
///
/// assert_eq!(MAP.get("KEY1"), Some(&1));
/// assert_eq!(MAP.get("KEY2"), Some(&2));
/// assert_eq!(MAP.get("MISSING_KEY"), None);
/// ```
#[macro_export]
macro_rules! phf_ordered_map {
    ( $($tt:tt)* ) => (
        $crate::item! { $crate::__phf_ordered_map! {
            $($tt)*
        }}
    );
    // this is for the readability of the macro arguments, documentation-wise
    (
        static $VARNAME:ident : $type:ty = {
            $($key_literal:tt => $value:expr),*
        };
    ) => ();
}

/// Declares a `static` phf set
///
/// # Example
///
/// ```rust
/// # #[macro_use] extern crate phf;
/// type K = &'static str;
///
/// phf_set! {
///     static SET: phf::Set<K> = {
///         "KEY1",
///         "KEY2",
///     };
/// }
///
/// assert!(SET.contains("KEY1"));
/// assert!(!SET.contains("MISSING_KEY"));
/// ```
#[macro_export]
macro_rules! phf_set {
    ( $($tt:tt)* ) => (
        $crate::item! { $crate::__phf_set! {
            $($tt)*
        }}
    );
    // this is for the readability of the macro arguments, documentation-wise
    (
        static $VARNAME:ident : $type:ty = {
            $($key_literal:tt),*
        };
    ) => ();
}

/// Declares a `static` phf ordered set
///
/// # Example
///
/// ```rust
/// # #[macro_use] extern crate phf;
/// type K = &'static str;
///
/// phf_ordered_set! {
///     static SET: phf::OrderedSet<K> = {
///         "KEY1",
///         "KEY2",
///     };
/// }
///
/// assert!(SET.contains("KEY1"));
/// assert!(!SET.contains("MISSING_KEY"));
/// ```
#[macro_export]
macro_rules! phf_ordered_set {
    ( $($tt:tt)* ) => (
        $crate::item! { $crate::__phf_ordered_set! {
            $($tt)*
        }}
    );
    // this is for the readability of the macro arguments, documentation-wise
    (
        static $VARNAME:ident : $type:ty = {
            $($key_literal:tt),*
        };
    ) => ();
}
