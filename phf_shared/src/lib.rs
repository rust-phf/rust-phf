#![doc(html_root_url = "https://docs.rs/phf_shared/0.9")]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std as core;

use core::fmt;
use core::hash::{Hash, Hasher};
use core::iter;
use rand::distributions;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use siphasher::sip128::{Hash128, Hasher128, SipHasher13};

#[cfg(feature = "macros")]
use {
    proc_macro2::TokenStream,
    quote::{quote, ToTokens},
};

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

/// Displacement Type
// NOTE: this type alias is not displayed correctly on the docs page
//       there are some github issues but its not clear how this is going to be resolved
// NOTE: if we simplify the crate structure (move everything to `phf`) then this is resolved
pub type Displacement = (u32, u32);

/// [`PhfHasher`] Result Type
pub struct Hashes {
    /// Base Index Parameter
    g: u32,

    /// Displacement Parameter 1
    f1: u32,

    /// Displacement Parameter 2
    f2: u32,
}

impl Hashes {
    /// Computes the base index in a range of length `len`.
    #[inline]
    pub const fn index(&self, len: usize) -> usize {
        (self.g % (len as u32)) as usize
    }

    /// Returns a shared reference to an element from `entries` using the base index.
    #[inline]
    pub const fn get<'t, T>(&self, entries: &'t [T]) -> &'t T {
        &entries[self.index(entries.len())]
    }

    /// Returns a mutable reference to an element from `entries` using the base index.
    #[inline]
    pub fn get_mut<'t, T>(&self, entries: &'t mut [T]) -> &'t mut T {
        &mut entries[self.index(entries.len())]
    }

    #[inline]
    const fn displace(f1: u32, f2: u32, d1: u32, d2: u32) -> u32 {
        f2.wrapping_add(d2.wrapping_add(f1.wrapping_mul(d1)))
    }

    /// Computes the displaced index in a range of length `len`.
    #[inline]
    pub const fn displaced_index(&self, len: usize, d: Displacement) -> usize {
        (Self::displace(self.f1, self.f2, d.0, d.1) % (len as u32)) as usize
    }

    /// Returns a shared reference to an element from `entries` using the displacements `disps`.
    #[inline]
    pub const fn displaced_get<'t, T>(&self, entries: &'t [T], disps: &[Displacement]) -> &'t T {
        &entries[self.displaced_index(entries.len(), *self.get(disps))]
    }

    /// Returns a mutable reference to an element from `entries` using the displacements `disps`.
    #[inline]
    pub fn displaced_get_mut<'t, T>(
        &self,
        entries: &'t mut [T],
        disps: &[Displacement],
    ) -> &'t mut T {
        &mut entries[self.displaced_index(entries.len(), *self.get(disps))]
    }
}

/// Hasher Trait
///
/// To implement a custom hasher for compile-time map/set generation the following other traits
/// also need to be implemented:
/// - `phf_macros` macro generation: `quote::ToTokens`
/// - `phf_codegen` code generation: [`FmtConstPath`]
///
/// The [`Self::Generator`] is used to create a distribution over hasher parameters which we can
/// then try to build a perfect hash function from. See [`generate_hash`].
///
/// By default the [`SipHasher`] is used for both kinds of compile-time generation.
pub trait PhfHasher {
    /// Hasher Generation Iterator
    type Generator: Iterator<Item = Self>;

    /// Builds an instance of the Hasher Generation Iterator.
    fn generator() -> Self::Generator;

    /// Hashes the data at `x` returning the computed displacement parameters.
    fn hash<T: ?Sized + PhfHash>(&self, x: &T) -> Hashes;
}

/// Hash State for a PHF
pub struct HashState<G> {
    /// [`PhfHasher`] State
    pub hasher: G,

    /// Computed Displacements
    pub disps: Vec<Displacement>,

    /// Computed Map
    pub map: Vec<usize>,
}

/// Runs [`try_generate_hash`] for each instance of [`PhfHasher`] from
/// [`PhfHasher::generator`] until generation converges on a PHF.
pub fn generate_hash<H, G>(entries: &[H]) -> HashState<G>
where
    H: PhfHash,
    G: PhfHasher,
{
    G::generator()
        .find_map(|hasher| try_generate_hash(entries, hasher))
        .expect("failed to solve PHF")
}

/// Tries to find a PHF for the elements in `entries` using `hasher`.
pub fn try_generate_hash<H, G>(entries: &[H], hasher: G) -> Option<HashState<G>>
where
    H: PhfHash,
    G: PhfHasher,
{
    const DEFAULT_LAMBDA: usize = 5;

    struct Bucket {
        idx: usize,
        keys: Vec<usize>,
    }

    let hashes: Vec<_> = entries.iter().map(|entry| hasher.hash(entry)).collect();

    let buckets_len = (hashes.len() + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA;
    let mut buckets = (0..buckets_len)
        .map(|i| Bucket {
            idx: i,
            keys: vec![],
        })
        .collect::<Vec<_>>();

    for (i, hash) in hashes.iter().enumerate() {
        hash.get_mut(&mut buckets).keys.push(i);
    }

    // Sort descending.
    buckets.sort_by(|a, b| a.keys.len().cmp(&b.keys.len()).reverse());

    let table_len = hashes.len();
    let mut map = vec![None; table_len];
    let mut disps = vec![Displacement::default(); buckets_len];

    // Store whether an element from the bucket being placed is
    // located at a certain position, to allow for efficient overlap
    // checks. It works by storing the generation in each cell and
    // each new placement-attempt is a new generation, so you can tell
    // if this is legitimately full by checking that the generations
    // are equal. (A u64 is far too large to overflow in a reasonable
    // time for current hardware.)
    let mut try_map = vec![0u64; table_len];
    let mut generation = 0u64;

    // The actual values corresponding to the markers above, as
    // (index, key) pairs, for adding to the main map once we've
    // chosen the right disps.
    let mut values_to_add = vec![];

    'buckets: for bucket in &buckets {
        // TODO: displacement iterator (so we can remove the u32 dependency here)
        for d1 in 0..(table_len as u32) {
            'disps: for d2 in 0..(table_len as u32) {
                values_to_add.clear();
                generation += 1;

                for &key in &bucket.keys {
                    let idx = hashes[key].displaced_index(table_len, (d1, d2));
                    if map[idx].is_some() || try_map[idx] == generation {
                        continue 'disps;
                    }
                    try_map[idx] = generation;
                    values_to_add.push((idx, key));
                }

                // We've picked a good set of disps
                disps[bucket.idx] = (d1, d2);
                for &(idx, key) in &values_to_add {
                    map[idx] = Some(key);
                }
                continue 'buckets;
            }
        }

        // Unable to find displacements for a bucket.
        return None;
    }

    Some(HashState {
        hasher,
        disps,
        map: map.into_iter().map(Option::unwrap).collect(),
    })
}

/// Trait for printing types with `const` constructors, used by `phf_codegen` and `phf_macros`.
pub trait FmtConst {
    /// Print a `const` expression representing this value.
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl<'a, T: 'a + FmtConst + ?Sized> FmtConst for &'a T {
    #[inline]
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (*self).fmt_const(f)
    }
}

/// Trait for printing types with `const` constructors which have relative paths, used by
/// `phf_codegen` and `phf_macros`.
pub trait FmtConstPath {
    /// Default Path
    const DEFAULT_PATH: &'static str;

    /// Print a `const` expression representing this value at the given `path`.
    fn fmt_const_with_path(&self, path: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl<'a, T: 'a + FmtConstPath + ?Sized> FmtConstPath for &'a T {
    const DEFAULT_PATH: &'static str = <T as FmtConstPath>::DEFAULT_PATH;

    #[inline]
    fn fmt_const_with_path(&self, path: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (*self).fmt_const_with_path(path, f)
    }
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

impl<'a> PhfBorrow<str> for &'a str {
    fn borrow(&self) -> &str {
        self
    }
}

impl<'a> PhfBorrow<[u8]> for &'a [u8] {
    fn borrow(&self) -> &[u8] {
        self
    }
}

impl PhfHash for str {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().phf_hash(state)
    }
}

impl PhfHash for [u8] {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
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
        // transmute is not stable in const fns (rust-lang/rust#53605), so
        // `UncasedStr::new` can't be a const fn itself, but we can inline the
        // call to transmute here in the meantime.
        f.write_str("unsafe { ::std::mem::transmute::<&'static str, &'static UncasedStr>(")?;
        self.as_str().fmt_const(f)?;
        f.write_str(") }")
    }
}

#[cfg(feature = "uncased")]
impl PhfBorrow<uncased::UncasedStr> for &uncased::UncasedStr {
    fn borrow(&self) -> &uncased::UncasedStr {
        self
    }
}

macro_rules! sip_impl (
    (le $t:ty) => (
        impl PhfHash for $t {
            #[inline]
            fn phf_hash<H: Hasher>(&self, state: &mut H) {
                self.to_le().hash(state);
            }
        }
    );
    ($t:ty) => (
        impl PhfHash for $t {
            #[inline]
            fn phf_hash<H: Hasher>(&self, state: &mut H) {
                self.hash(state);
            }
        }
    )
);

sip_impl!(u8);
sip_impl!(i8);
sip_impl!(le u16);
sip_impl!(le i16);
sip_impl!(le u32);
sip_impl!(le i32);
sip_impl!(le u64);
sip_impl!(le i64);
sip_impl!(le u128);
sip_impl!(le i128);
sip_impl!(bool);

impl PhfHash for char {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (*self as u32).phf_hash(state)
    }
}

// minimize duplicated code since formatting drags in quite a bit
fn fmt_array(array: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", array)
}

macro_rules! array_impl (
    ($t:ty, $n:expr) => (
        impl PhfHash for [$t; $n] {
            #[inline]
            fn phf_hash<H: Hasher>(&self, state: &mut H) {
                state.write(self);
            }
        }

        impl FmtConst for [$t; $n] {
            fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt_array(self, f)
            }
        }

        impl PhfBorrow<[$t]> for [$t; $n] {
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

/// The Default Hasher Implementation
pub type DefaultHasher = SipHasher;

/// [`SipHasher13`] Implementation of [`PhfHasher`]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SipHasher {
    /// Hasher Key
    pub key: SipHasherKey,
}

impl SipHasher {
    #[inline]
    const fn new(key: SipHasherKey) -> Self {
        Self { key }
    }
}

/// SipHasher Key Type
pub type SipHasherKey = u64;

/// SipHasher Generator Type
type SipHasherGeneratorType = iter::Map<
    distributions::DistIter<distributions::Standard, SmallRng, SipHasherKey>,
    fn(SipHasherKey) -> SipHasher,
>;

/// [`SipHasher`] Generator
pub struct SipHasherGenerator(SipHasherGeneratorType);

impl SipHasherGenerator {
    const FIXED_SEED: u64 = 1234567890;

    #[inline]
    fn new() -> Self {
        Self(
            SmallRng::seed_from_u64(Self::FIXED_SEED)
                .sample_iter(distributions::Standard)
                .map(SipHasher::new),
        )
    }
}

impl Iterator for SipHasherGenerator {
    type Item = SipHasher;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl PhfHasher for SipHasher {
    type Generator = SipHasherGenerator;

    #[inline]
    fn generator() -> Self::Generator {
        SipHasherGenerator::new()
    }

    fn hash<T: ?Sized + PhfHash>(&self, x: &T) -> Hashes {
        let mut hasher = SipHasher13::new_with_keys(0, self.key);
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
}

impl FmtConstPath for SipHasher {
    // FIXME: what is the right path to put here? would be easier if we had only one crate
    const DEFAULT_PATH: &'static str = "::phf::hash";

    #[inline]
    fn fmt_const_with_path(&self, path: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{:?}", path, self)
    }
}

#[cfg(feature = "macros")]
impl ToTokens for SipHasher {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let key = self.key;
        // FIXME: what is the right path to put here? would be easier if we had only one crate
        tokens.extend(quote! {
            ::phf_shared::SipHasher {
                key: #key,
            }
        });
    }
}
