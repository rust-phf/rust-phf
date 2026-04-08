//! See [the `phf` crate's documentation][phf] for details.
//!
//! [phf]: https://docs.rs/phf

#![doc(html_root_url = "https://docs.rs/phf_shared/0.13.1")]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std as core;

use core::fmt;
use core::hash::{Hash, Hasher};
use core::num::Wrapping;
use siphasher::sip128::{Hash128, Hasher128, SipHasher13};

#[cfg(feature = "ptrhash")]
pub mod ptrhash;

#[non_exhaustive]
pub struct Hashes {
    pub g: u32,
    pub f1: u32,
    pub f2: u32,
}

/// A central typedef for hash keys
///
/// Makes experimentation easier by only needing to be updated here.
pub type HashKey = u64;

#[inline]
pub fn displace(f1: u32, f2: u32, d1: u32, d2: u32) -> u32 {
    (Wrapping(d2) + Wrapping(f1) * Wrapping(d1) + Wrapping(f2)).0
}

/// `key` is from `phf_generator::HashState`.
#[inline]
pub fn hash<T: ?Sized + PhfHash>(x: &T, key: &HashKey) -> Hashes {
    let mut hasher = SipHasher13::new_with_keys(0, *key);
    x.phf_hash(&mut hasher);

    let Hash128 {
        h1: lower,
        h2: upper,
    } = hasher.finish128();

    Hashes {
        g: (lower >> 32) as u32,
        f1: lower as u32,
        f2: upper as u32,
    }
}

/// Return an index into `phf_generator::HashState::map`.
///
/// * `hash` is from `hash()` in this crate.
/// * `disps` is from `phf_generator::HashState::disps`.
/// * `len` is the length of `phf_generator::HashState::map`.
#[inline]
pub fn get_index(hashes: &Hashes, disps: &[(u32, u32)], len: usize) -> u32 {
    let (d1, d2) = disps[(hashes.g % (disps.len() as u32)) as usize];
    displace(hashes.f1, hashes.f2, d1, d2) % (len as u32)
}

/// A trait implemented by types which can be used in PHF data structures.
///
/// This differs from the standard library's `Hash` trait in that `PhfHash`'s
/// results must be architecture independent so that hashes will be consistent
/// between the host and target when cross compiling.
pub trait PhfHash {
    /// Feeds the value into the state given, updating the hasher as necessary.
    fn phf_hash<H: Hasher>(&self, state: &mut H);

    /// Feeds a slice of this type into the state provided.
    fn phf_hash_slice<H: Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        for piece in data {
            piece.phf_hash(state);
        }
    }
}

/// Trait for printing types with `const` constructors, used by `phf_codegen` and `phf_macros`.
pub trait FmtConst {
    /// Print a `const` expression representing this value.
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

/// Identical to `std::borrow::Borrow` except omitting blanket impls to facilitate other
/// borrowing patterns.
///
/// The same semantic requirements apply:
///
/// > In particular `Eq`, `Ord` and `Hash` must be equivalent for borrowed and owned values:
/// > `x.borrow() == y.borrow()` should give the same result as `x == y`.
///
/// (This crate's API only requires `Eq` and `PhfHash`, however.)
///
/// ### Motivation
/// The conventional signature for lookup methods on collections looks something like this:
///
/// ```ignore
/// impl<K, V> Map<K, V> where K: PhfHash + Eq {
///     fn get<T: ?Sized>(&self, key: &T) -> Option<&V> where T: PhfHash + Eq, K: Borrow<T> {
///         ...
///     }
/// }
/// ```
///
/// This allows the key type used for lookup to be different than the key stored in the map so for
/// example you can use `&str` to look up a value in a `Map<String, _>`. However, this runs into
/// a problem in the case where `T` and `K` are both a `Foo<_>` type constructor but
/// the contained type is different (even being the same type with different lifetimes).
///
/// The main issue for this crate's API is that, with this method signature, you cannot perform a
/// lookup on a `Map<UniCase<&'static str>, _>` with a `UniCase<&'a str>` where `'a` is not
/// `'static`; there is no impl of `Borrow` that resolves to
/// `impl Borrow<UniCase<'a>> for UniCase<&'static str>` and one cannot be added either because of
/// all the blanket impls.
///
/// Instead, this trait is implemented conservatively, without blanket impls, so that impls like
/// this may be added. This is feasible since the set of types that implement `PhfHash` is
/// intentionally small.
///
/// This likely won't be fixable with specialization alone but will require full support for lattice
/// impls since we technically want to add overlapping blanket impls.
pub trait PhfBorrow<B: ?Sized> {
    /// Convert a reference to `self` to a reference to the borrowed type.
    fn borrow(&self) -> &B;
}

/// Create an impl of `FmtConst` delegating to `fmt::Debug` for types that can deal with it.
///
/// Ideally with specialization this could be just one default impl and then specialized where
/// it doesn't apply.
macro_rules! delegate_debug (
    ($ty:ty) => {
        impl FmtConst for $ty {
            fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    }
);

delegate_debug!(str);
delegate_debug!(char);
delegate_debug!(u8);
delegate_debug!(i8);
delegate_debug!(u16);
delegate_debug!(i16);
delegate_debug!(u32);
delegate_debug!(i32);
delegate_debug!(u64);
delegate_debug!(i64);
delegate_debug!(usize);
delegate_debug!(isize);
delegate_debug!(u128);
delegate_debug!(i128);
delegate_debug!(bool);

/// `impl PhfBorrow<T> for T`
macro_rules! impl_reflexive(
    ($($t:ty),*) => (
        $(impl PhfBorrow<$t> for $t {
            fn borrow(&self) -> &$t {
                self
            }
        })*
    )
);

impl_reflexive!(
    str,
    char,
    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    u64,
    i64,
    usize,
    isize,
    u128,
    i128,
    bool,
    [u8]
);

#[cfg(feature = "std")]
impl PhfBorrow<str> for String {
    fn borrow(&self) -> &str {
        self
    }
}

#[cfg(feature = "std")]
impl PhfBorrow<[u8]> for Vec<u8> {
    fn borrow(&self) -> &[u8] {
        self
    }
}

#[cfg(feature = "std")]
delegate_debug!(String);

#[cfg(feature = "std")]
impl PhfHash for String {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (**self).phf_hash(state)
    }
}

#[cfg(feature = "std")]
impl PhfHash for Vec<u8> {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (**self).phf_hash(state)
    }
}

impl<'a, T: 'a + PhfHash + ?Sized> PhfHash for &'a T {
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (*self).phf_hash(state)
    }
}

impl<'a, T: 'a + FmtConst + ?Sized> FmtConst for &'a T {
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (*self).fmt_const(f)
    }
}

impl PhfBorrow<str> for &str {
    fn borrow(&self) -> &str {
        self
    }
}

impl PhfBorrow<[u8]> for &[u8] {
    fn borrow(&self) -> &[u8] {
        self
    }
}

impl<const N: usize> PhfBorrow<[u8; N]> for &[u8; N] {
    fn borrow(&self) -> &[u8; N] {
        self
    }
}

impl PhfHash for str {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().phf_hash(state)
    }
}

impl FmtConst for [u8] {
    #[inline]
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // slices need a leading reference
        write!(f, "&{:?}", self)
    }
}

#[cfg(feature = "unicase")]
impl<S> PhfHash for unicase::UniCase<S>
where
    unicase::UniCase<S>: Hash,
{
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

#[cfg(feature = "unicase")]
impl<S> FmtConst for unicase::UniCase<S>
where
    S: AsRef<str>,
{
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_ascii() {
            f.write_str("UniCase::ascii(")?;
        } else {
            f.write_str("UniCase::unicode(")?;
        }

        self.as_ref().fmt_const(f)?;
        f.write_str(")")
    }
}

#[cfg(feature = "unicase")]
impl<'b, 'a: 'b, S: ?Sized + 'a> PhfBorrow<unicase::UniCase<&'b S>> for unicase::UniCase<&'a S> {
    fn borrow(&self) -> &unicase::UniCase<&'b S> {
        self
    }
}

#[cfg(feature = "unicase")]
impl<S> PhfHash for unicase::Ascii<S>
where
    unicase::Ascii<S>: Hash,
{
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

#[cfg(feature = "unicase")]
impl<S> FmtConst for unicase::Ascii<S>
where
    S: AsRef<str>,
{
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Ascii::new(")?;
        self.as_ref().fmt_const(f)?;
        f.write_str(")")
    }
}

#[cfg(feature = "unicase")]
impl<'b, 'a: 'b, S: ?Sized + 'a> PhfBorrow<unicase::Ascii<&'b S>> for unicase::Ascii<&'a S> {
    fn borrow(&self) -> &unicase::Ascii<&'b S> {
        self
    }
}

#[cfg(feature = "uncased")]
impl PhfHash for uncased::UncasedStr {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

#[cfg(feature = "uncased")]
impl FmtConst for uncased::UncasedStr {
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("UncasedStr::new(")?;
        self.as_str().fmt_const(f)?;
        f.write_str(")")
    }
}

#[cfg(feature = "uncased")]
impl PhfBorrow<uncased::UncasedStr> for &uncased::UncasedStr {
    fn borrow(&self) -> &uncased::UncasedStr {
        self
    }
}

macro_rules! integer_impl (
    ($t:ty) => (
        impl PhfHash for $t {
            #[inline]
            fn phf_hash<H: Hasher>(&self, state: &mut H) {
                self.to_le().hash(state);
            }

            // `phf_hash_slice` cannot use `write` due to possible differences in endianness.
        }
    )
);

integer_impl!(u16);
integer_impl!(i16);
integer_impl!(u32);
integer_impl!(i32);
integer_impl!(u64);
integer_impl!(i64);
integer_impl!(usize);
integer_impl!(isize);
integer_impl!(u128);
integer_impl!(i128);

macro_rules! single_byte_impl (
    ($t:ty) => (
        impl PhfHash for $t {
            #[inline]
            fn phf_hash<H: Hasher>(&self, state: &mut H) {
                self.hash(state);
            }

            #[inline]
            fn phf_hash_slice<H: Hasher>(slice: &[$t], state: &mut H) {
                // There is sadly no `[i8]::as_bytes` or `[bool]::as_bytes`.
                state.write(unsafe { &*(slice as *const [$t] as *const [u8]) });
            }
        }
    )
);

single_byte_impl!(u8);
single_byte_impl!(i8);
// https://doc.rust-lang.org/reference/types/boolean.html#r-type.bool.repr guarantees that `bool`
// has a fixed layout.
single_byte_impl!(bool);

impl PhfHash for char {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (*self as u32).phf_hash(state)
    }
}

impl<T: PhfHash, const N: usize> PhfHash for [T; N] {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        <[T]>::phf_hash(self, state);
    }
}

impl<T: PhfHash> PhfHash for [T] {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        T::phf_hash_slice(self, state);
    }
}

// minimize duplicated code since formatting drags in quite a bit
fn fmt_array<T: core::fmt::Debug>(array: &[T], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", array)
}

macro_rules! array_impl (
    ($t:ty) => (
        impl<const N: usize> FmtConst for [$t; N] {
            fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt_array(self, f)
            }
        }

        impl<const N: usize> PhfBorrow<[$t]> for [$t; N] {
            fn borrow(&self) -> &[$t] {
                self
            }
        }
    )
);

array_impl!(u8);
array_impl!(i8);
array_impl!(u16);
array_impl!(i16);
array_impl!(u32);
array_impl!(i32);
array_impl!(u64);
array_impl!(i64);
array_impl!(usize);
array_impl!(isize);
array_impl!(u128);
array_impl!(i128);
array_impl!(bool);
array_impl!(char);

macro_rules! tuple_impl {
    ($($t:ident),+) => {
        impl<$($t: PhfHash),+> PhfHash for ($($t,)+) {
            fn phf_hash<HS: Hasher>(&self, state: &mut HS) {
                #[allow(non_snake_case)]
                let ($($t,)+) = self;
                $(
                    $t.phf_hash(state);
                )+
            }
        }

        impl<$($t: PhfHash),+> PhfBorrow<($($t,)+)> for ($($t,)+) {
            fn borrow(&self) -> &($($t,)+) {
                self
            }
        }

        impl<$($t: FmtConst),+> FmtConst for ($($t,)+) {
            fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                #[allow(non_snake_case)]
                let ($($t,)+) = self;
                write!(f, "(")?;
                let mut first = true;
                $(
                    if !core::mem::replace(&mut first, false) {
                        write!(f, ", ")?;
                    }
                    $t.fmt_const(f)?;
                )+
                write!(f, ")")
            }
        }
    };
}

tuple_impl!(A);
tuple_impl!(A, B);
tuple_impl!(A, B, C);
tuple_impl!(A, B, C, D);
tuple_impl!(A, B, C, D, E);
tuple_impl!(A, B, C, D, E, F);
tuple_impl!(A, B, C, D, E, F, G);
tuple_impl!(A, B, C, D, E, F, G, HT);
tuple_impl!(A, B, C, D, E, F, G, HT, I);
tuple_impl!(A, B, C, D, E, F, G, HT, I, J);
tuple_impl!(A, B, C, D, E, F, G, HT, I, J, K);
tuple_impl!(A, B, C, D, E, F, G, HT, I, J, K, L);

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Debug)]
    enum HashCall {
        Bytes(Vec<u8>),
        U8(u8),
        U16(u16),
        U32(u32),
        U64(u64),
        U128(u128),
        Usize(usize),
        I8(i8),
        I16(i16),
        I32(i32),
        I64(i64),
        I128(i128),
        Isize(isize),
        // Ideally we'd handle `write_length_prefix` and `write_str` as well, but they are unstable.
    }

    #[derive(PartialEq, Debug)]
    struct TestHasher {
        calls: Vec<HashCall>,
    }

    impl Hasher for TestHasher {
        fn finish(&self) -> u64 {
            panic!("only used for tests");
        }
        fn write(&mut self, bytes: &[u8]) {
            self.calls.push(HashCall::Bytes(bytes.to_vec()));
        }
        fn write_u8(&mut self, i: u8) {
            self.calls.push(HashCall::U8(i));
        }
        fn write_u16(&mut self, i: u16) {
            self.calls.push(HashCall::U16(i));
        }
        fn write_u32(&mut self, i: u32) {
            self.calls.push(HashCall::U32(i));
        }
        fn write_u64(&mut self, i: u64) {
            self.calls.push(HashCall::U64(i));
        }
        fn write_u128(&mut self, i: u128) {
            self.calls.push(HashCall::U128(i));
        }
        fn write_usize(&mut self, i: usize) {
            self.calls.push(HashCall::Usize(i));
        }
        fn write_i8(&mut self, i: i8) {
            self.calls.push(HashCall::I8(i));
        }
        fn write_i16(&mut self, i: i16) {
            self.calls.push(HashCall::I16(i));
        }
        fn write_i32(&mut self, i: i32) {
            self.calls.push(HashCall::I32(i));
        }
        fn write_i64(&mut self, i: i64) {
            self.calls.push(HashCall::I64(i));
        }
        fn write_i128(&mut self, i: i128) {
            self.calls.push(HashCall::I128(i));
        }
        fn write_isize(&mut self, i: isize) {
            self.calls.push(HashCall::Isize(i));
        }
    }

    fn test_hash<T: PhfHash>(x: T) -> Vec<HashCall> {
        let mut state = TestHasher { calls: Vec::new() };
        x.phf_hash(&mut state);
        state.calls
    }

    #[test]
    fn byte_slices_are_hashed_efficiently() {
        assert_eq!(
            test_hash(&[1u8, 2, 3]),
            [HashCall::Bytes([1, 2, 3].to_vec())]
        );
        assert_eq!(
            test_hash(&[1i8, 2, 3]),
            [HashCall::Bytes([1, 2, 3].to_vec())]
        );
        assert_eq!(
            test_hash(&[false, true]),
            [HashCall::Bytes([0, 1].to_vec())]
        );
    }

    #[test]
    fn slices_and_arrays_are_hashed_consistently() {
        assert_eq!(test_hash(&[1u8, 2, 3]), test_hash(&[1u8, 2, 3][..]));
        assert_eq!(test_hash(&[1u16, 2, 3]), test_hash(&[1u16, 2, 3][..]));
    }
}
