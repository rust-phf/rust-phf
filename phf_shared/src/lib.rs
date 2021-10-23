//! See [the `phf` crate's documentation][phf] for details.
//!
//! [phf]: https://docs.rs/phf

// XXX: Temporary until stabilization.
#![feature(const_fn_trait_bound, const_mut_refs, const_panic, const_trait_impl)]
#![doc(html_root_url = "https://docs.rs/phf_shared/0.11")]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std as core;

mod siphasher;

use core::fmt;
use core::hash::Hasher;
use siphasher::{Hash128, Hasher128, SipHasher13};

#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct Hashes {
    pub g: u32,
    pub f1: u32,
    pub f2: u32,
}

impl const Default for Hashes {
    #[inline(always)]
    fn default() -> Self {
        Self { g: 0, f1: 0, f2: 0 }
    }
}

/// A central typedef for hash keys
///
/// Makes experimentation easier by only needing to be updated here.
pub type HashKey = u64;

#[inline]
pub const fn displace(f1: u32, f2: u32, d1: u32, d2: u32) -> u32 {
    d2.wrapping_add(f1.wrapping_mul(d1)).wrapping_add(f2)
}

/// `key` is from `phf_generator::HashState`.
#[cfg(not(feature = "const-api"))]
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

/// `key` is from `phf_generator::HashState`.
#[cfg(feature = "const-api")]
#[inline]
pub const fn hash<T: ?Sized + ~const PhfHash>(x: &T, key: &HashKey) -> Hashes {
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
pub const fn get_index(hashes: &Hashes, disps: &[(u32, u32)], len: usize) -> u32 {
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
    #[cfg(not(feature = "const-api"))]
    fn phf_hash<H: Hasher>(&self, state: &mut H);

    /// Feeds the value into the state given, updating the hasher as necessary.
    #[cfg(feature = "const-api")]
    fn phf_hash<H: ~const Hasher>(&self, state: &mut H);

    /// Feeds a slice of this type into the state provided.
    #[cfg(not(feature = "const-api"))]
    fn phf_hash_slice<H: Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        for piece in data {
            piece.phf_hash(state);
        }
    }

    /// Feeds a slice of this type into the state provided.
    #[cfg(feature = "const-api")]
    #[default_method_body_is_const]
    fn phf_hash_slice<H: ~const Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        let mut i = 0;
        while i < data.len() {
            data[i].phf_hash(state);
            i += 1;
        }
    }
}

/// Trait for printing types with `const` constructors, used by `phf_codegen` and `phf_macros`.
// TODO: Is a const variant of this trait needed?
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
/// `x.borrow() == y.borrow()` should give the same result as `x == y`.
///
/// (This crate's API only requires `Eq` and `PhfHash`, however.)
///
/// ### Motivation
/// The conventional signature for lookup methods on collections looks something like this:
///
/// ```rust,ignore
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
delegate_debug!(u128);
delegate_debug!(i128);
delegate_debug!(bool);

/// `impl PhfBorrow<T> for T`
macro_rules! impl_reflexive(
    ($($t:ty),*) => (
        $(
            #[cfg(not(feature = "const-api"))]
            impl PhfBorrow<$t> for $t {
                fn borrow(&self) -> &$t {
                    self
                }
            }

            #[cfg(feature = "const-api")]
            impl const PhfBorrow<$t> for $t {
                fn borrow(&self) -> &$t {
                    self
                }
            }
        )*
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

#[cfg(not(feature = "const-api"))]
impl<'a, T: 'a + PhfHash + ?Sized> PhfHash for &'a T {
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (*self).phf_hash(state)
    }
}

#[cfg(feature = "const-api")]
impl<'a, T: 'a + ~const PhfHash + ?Sized> const PhfHash for &'a T {
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (*self).phf_hash(state)
    }
}

impl<'a, T: 'a + FmtConst + ?Sized> FmtConst for &'a T {
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (*self).fmt_const(f)
    }
}

#[cfg(not(feature = "const-api"))]
impl<'a> PhfBorrow<str> for &'a str {
    fn borrow(&self) -> &str {
        self
    }
}

#[cfg(feature = "const-api")]
impl<'a> const PhfBorrow<str> for &'a str {
    fn borrow(&self) -> &str {
        self
    }
}

#[cfg(not(feature = "const-api"))]
impl<'a> PhfBorrow<[u8]> for &'a [u8] {
    fn borrow(&self) -> &[u8] {
        self
    }
}

#[cfg(feature = "const-api")]
impl<'a> PhfBorrow<[u8]> for &'a [u8] {
    fn borrow(&self) -> &[u8] {
        self
    }
}

#[cfg(not(feature = "const-api"))]
impl PhfHash for str {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().phf_hash(state)
    }
}

#[cfg(feature = "const-api")]
impl const PhfHash for str {
    #[inline]
    fn phf_hash<H: ~const Hasher>(&self, state: &mut H) {
        self.as_bytes().phf_hash(state)
    }
}

#[cfg(not(feature = "const-api"))]
impl PhfHash for [u8] {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        state.write(self);
    }
}

#[cfg(feature = "const-api")]
impl const PhfHash for [u8] {
    #[inline]
    fn phf_hash<H: ~const Hasher>(&self, state: &mut H) {
        state.write(self);
    }
}

impl FmtConst for [u8] {
    #[inline]
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // slices need a leading reference
        write!(f, "&{:?}", self)
    }
}

#[cfg(all(feature = "unicase", not(feature = "const-api")))]
impl<S> PhfHash for unicase::UniCase<S>
where
    unicase::UniCase<S>: core::hash::Hash,
{
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        <Self as core::hash::Hash>::hash(self, state)
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

#[cfg(all(feature = "uncased", not(feature = "const-api")))]
impl PhfHash for uncased::UncasedStr {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

#[cfg(feature = "uncased")]
impl FmtConst for uncased::UncasedStr {
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // transmute is not stable in const fns (rust-lang/rust#53605), so
        // `UncasedStr::new` can't be a const fn itself, but we can inline the
        // call to transmute here in the meantime.
        f.write_str("unsafe { ::core::mem::transmute::<&'static str, &'static UncasedStr>(")?;
        self.as_str().fmt_const(f)?;
        f.write_str(") }")
    }
}

#[cfg(all(feature = "uncased", not(feature = "const-api")))]
impl PhfBorrow<uncased::UncasedStr> for &uncased::UncasedStr {
    fn borrow(&self) -> &uncased::UncasedStr {
        self
    }
}

#[cfg(all(feature = "uncased", feature = "const-api"))]
impl const PhfBorrow<uncased::UncasedStr> for &uncased::UncasedStr {
    fn borrow(&self) -> &uncased::UncasedStr {
        self
    }
}

// XXX: Macro can be simplified once const Hash trait impls
//      landed in upstream Rust.
macro_rules! sip_impl {
    (le $t:ty, $meth:ident) => {
        #[cfg(not(feature = "const-api"))]
        impl PhfHash for $t {
            #[inline]
            fn phf_hash<H: Hasher>(&self, state: &mut H) {
                state.$meth(self.to_le());
            }
        }

        #[cfg(feature = "const-api")]
        impl const PhfHash for $t {
            #[inline]
            fn phf_hash<H: ~const Hasher>(&self, state: &mut H) {
                state.$meth(self.to_le());
            }
        }
    };
    ($t:ty, $meth:ident) => {
        #[cfg(not(feature = "const-api"))]
        impl PhfHash for $t {
            #[inline]
            fn phf_hash<H: Hasher>(&self, state: &mut H) {
                state.$meth(*self);
            }
        }

        #[cfg(feature = "const-api")]
        impl const PhfHash for $t {
            #[inline]
            fn phf_hash<H: ~const Hasher>(&self, state: &mut H) {
                state.$meth(*self);
            }
        }
    };
}

sip_impl!(u8, write_u8);
sip_impl!(i8, write_i8);
sip_impl!(le u16, write_u16);
sip_impl!(le i16, write_i16);
sip_impl!(le u32, write_u32);
sip_impl!(le i32, write_i32);
sip_impl!(le u64, write_u64);
sip_impl!(le i64, write_i64);
sip_impl!(le u128, write_u128);
sip_impl!(le i128, write_i128);

#[cfg(not(feature = "const-api"))]
impl PhfHash for bool {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(*self as u8);
    }
}

#[cfg(feature = "const-api")]
impl const PhfHash for bool {
    #[inline]
    fn phf_hash<H: ~const Hasher>(&self, state: &mut H) {
        state.write_u8(*self as u8);
    }
}

#[cfg(not(feature = "const-api"))]
impl PhfHash for char {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(*self as u32);
    }
}

#[cfg(feature = "const-api")]
impl const PhfHash for char {
    #[inline]
    fn phf_hash<H: ~const Hasher>(&self, state: &mut H) {
        state.write_u32(*self as u32);
    }
}

// minimize duplicated code since formatting drags in quite a bit
fn fmt_array(array: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", array)
}

macro_rules! array_impl (
    ($t:ty, $n:expr) => (
        #[cfg(not(feature = "const-api"))]
        impl PhfHash for [$t; $n] {
            #[inline]
            fn phf_hash<H: Hasher>(&self, state: &mut H) {
                state.write(self);
            }
        }

        #[cfg(feature = "const-api")]
        impl const PhfHash for [$t; $n] {
            #[inline]
            fn phf_hash<H: ~const Hasher>(&self, state: &mut H) {
                state.write(self);
            }
        }

        impl FmtConst for [$t; $n] {
            fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt_array(self, f)
            }
        }

        #[cfg(not(feature = "const-api"))]
        impl PhfBorrow<[$t]> for [$t; $n] {
            fn borrow(&self) -> &[$t] {
                self
            }
        }

        #[cfg(feature = "const-api")]
        impl const PhfBorrow<[$t]> for [$t; $n] {
            fn borrow(&self) -> &[$t] {
                self
            }
        }
    )
);

array_impl!(u8, 1);
array_impl!(u8, 2);
array_impl!(u8, 3);
array_impl!(u8, 4);
array_impl!(u8, 5);
array_impl!(u8, 6);
array_impl!(u8, 7);
array_impl!(u8, 8);
array_impl!(u8, 9);
array_impl!(u8, 10);
array_impl!(u8, 11);
array_impl!(u8, 12);
array_impl!(u8, 13);
array_impl!(u8, 14);
array_impl!(u8, 15);
array_impl!(u8, 16);
array_impl!(u8, 17);
array_impl!(u8, 18);
array_impl!(u8, 19);
array_impl!(u8, 20);
array_impl!(u8, 21);
array_impl!(u8, 22);
array_impl!(u8, 23);
array_impl!(u8, 24);
array_impl!(u8, 25);
array_impl!(u8, 26);
array_impl!(u8, 27);
array_impl!(u8, 28);
array_impl!(u8, 29);
array_impl!(u8, 30);
array_impl!(u8, 31);
array_impl!(u8, 32);
