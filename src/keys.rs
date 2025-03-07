use core::{borrow::Borrow, marker::PhantomData};

pub use crate::Hasher;

/// This type allows the
pub trait PhfKey: Eq {
    /// The associated type with the appropriate constant functions
    type ConstKey: ConstKey<PhfKey = Self> + ?Sized;
}

/// Runtime support for indexing into a map.
///
/// Keys support any number of proxy types by providing the ability to hash and compare equality.
pub trait PhfKeyProxy<PK: ?Sized> {
    /// Hash the proxy key in the same way that `Self` would be hashed
    fn pfh_hash(pk: &PK, state: &mut Hasher);

    /// Compare the proxy key to `self`
    fn pfh_eq(&self, other: &PK) -> bool;
}

/// A type which can be hashed and equality checked at compile time.
///
/// This type must implement:
/// * `const fn pfh_hash(k: &Self::PhfKey, state: &mut Hasher)`
/// * `const fn pfh_eq(lhs: &Self::PhfKey, rhs: &Self::PhfKey) -> bool`
pub trait ConstKey {
    type PhfKey: PhfKey<ConstKey = Self> + ?Sized;
}

/// The glue between between the [`ConstKey`] and [`PhfKey`] for primitives.
#[doc(hidden)]
pub struct PrimitiveKey<T: ?Sized>(PhantomData<T>);

macro_rules! prim_impl {
    ($({$($gen:tt)*})? $t:ty, |$v:ident, $s:ident| $h:expr, |$l:ident, $r:ident| $e:expr) => {
        impl$($($gen)*)? PhfKey for $t {
            type ConstKey = PrimitiveKey<$t>;
        }
        impl$($($gen)*)? ConstKey for PrimitiveKey<$t> {
            type PhfKey = $t;
        }
        impl$($($gen)*)? PrimitiveKey<$t> {
            pub const fn pfh_hash($v: &<Self as ConstKey>::PhfKey, $s: &mut Hasher) {
                $h
            }
            pub const fn pfh_eq($l: &<Self as ConstKey>::PhfKey, $r: &<Self as ConstKey>::PhfKey) -> bool {
                $e
            }
        }

    };
    ($t:ty, |$v:ident, $s:ident| $h:expr) => {
        prim_impl!{$t, |$v, $s| $h, |lhs, rhs| *lhs == *rhs}
    };
}
prim_impl! {u8, |v, s| s.write_u8(*v)}
prim_impl! {i8, |v, s| s.write_u8(*v as u8)}
prim_impl! {u16, |v, s| s.write_u16(*v)}
prim_impl! {i16, |v, s| s.write_u16(*v as u16)}
prim_impl! {u32, |v, s| s.write_u32(*v)}
prim_impl! {i32, |v, s| s.write_u32(*v as u32)}
prim_impl! {u64, |v, s| s.write_u64(*v)}
prim_impl! {i64, |v, s| s.write_u64(*v as u64)}
prim_impl! {usize, |v, s| s.write_usize(*v)}
prim_impl! {isize, |v, s| s.write_usize(*v as usize)}
prim_impl! {u128, |v, s| s.write(&v.to_le_bytes())}
prim_impl! {i128, |v, s| s.write(&v.to_le_bytes())}
prim_impl! {[u8], |v, s| s.write(v), |lhs, rhs| {
    if lhs.len() != rhs.len() { return false }
    let mut i = 0;
    while i < lhs.len() {
        if lhs[i] != rhs[i] { return false }
        i += 1;
    }
    true
}}
prim_impl! {str, |v, s| s.write(v.as_bytes()), |lhs, rhs| {
    let lhs = lhs.as_bytes();
    let rhs = rhs.as_bytes();
    if lhs.len() != rhs.len() { return false }
    let mut i = 0;
    while i < lhs.len() {
        if lhs[i] != rhs[i] { return false }
        i += 1;
    }
    true
}}

macro_rules! prim_impl_hash {
    ($($t:ty),* $(,)?) => {$(
    impl<PK: ?Sized + Borrow<$t>> PhfKeyProxy<PK> for $t {
        #[inline(always)]
        fn pfh_hash(pk: &PK, state: &mut Hasher) {
            <Self as PhfKey>::ConstKey::pfh_hash(pk.borrow(), state)
        }
        #[inline(always)]
        fn pfh_eq(&self, other: &PK) -> bool {
            self == other.borrow()
        }
    }
    )*};
}
prim_impl_hash! {u8, i8, u16, i16, u32, i32, u64, i64, usize, isize, u128, i128, [u8], str}

macro_rules! prim_impl_static {
    ($($t:ty),* $(,)?) => {$(
        prim_impl!{&'static $t, |v, s| PrimitiveKey::<$t>::pfh_hash(*v, s), |lhs, rhs| PrimitiveKey::<$t>::pfh_eq(*lhs, *rhs)}
    )*};
}
prim_impl_static! {u8, i8, u16, i16, u32, i32, u64, i64, usize, isize, u128, i128, [u8], str, UncasedStr}

impl<PK: ?Sized, T: ?Sized + PhfKeyProxy<PK>> PhfKeyProxy<PK> for &'_ T {
    #[inline(always)]
    fn pfh_hash(pk: &PK, state: &mut Hasher) {
        T::pfh_hash(pk, state);
    }
    #[inline(always)]
    fn pfh_eq(&self, other: &PK) -> bool {
        (*self).pfh_eq(other)
    }
}
impl<PK: ?Sized, T: ?Sized + PhfKeyProxy<PK>> PhfKeyProxy<PK> for &'_ mut T {
    #[inline(always)]
    fn pfh_hash(pk: &PK, state: &mut Hasher) {
        T::pfh_hash(pk, state);
    }
    #[inline(always)]
    fn pfh_eq(&self, other: &PK) -> bool {
        (**self).pfh_eq(other)
    }
}

#[repr(transparent)]
pub struct UncasedStr(pub str);

impl UncasedStr {
    /// Cost-free conversion from an `&str` reference to an `UncasedStr`.
    #[inline(always)]
    pub const fn new(string: &str) -> &UncasedStr {
        // This is a `newtype`-like transformation. `repr(transparent)` ensures
        // that this is safe and correct.
        unsafe { core::mem::transmute(string) }
    }

    /// Returns `self` as an `&str`.
    #[inline(always)]
    pub const fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the length, in bytes, of `self`.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.as_str().len()
    }

    /// Returns `true`` if `self`` has a length of zero bytes.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }

    /// Returns true if `self == other`
    #[inline]
    pub const fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        let mut a = self.as_str().as_bytes();
        let mut b = other.as_str().as_bytes();

        while let ([first_a, rest_a @ ..], [first_b, rest_b @ ..]) = (a, b) {
            if first_a.eq_ignore_ascii_case(first_b) {
                a = rest_a;
                b = rest_b;
            } else {
                return false;
            }
        }

        true
    }
}

impl PartialEq for UncasedStr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

impl Eq for UncasedStr {}

prim_impl! {UncasedStr,
    |v, s| {
        let mut a = v.as_str().as_bytes();
        while let [first_a, rest_a @ ..] = a {
            s.write_u8(first_a.to_ascii_lowercase());
            a = rest_a;
        }
    }, |lhs, rhs| lhs.eq(rhs)
}

impl<PK: ?Sized + Borrow<str>> PhfKeyProxy<PK> for UncasedStr {
    #[inline(always)]
    fn pfh_hash(pk: &PK, state: &mut Hasher) {
        <Self as PhfKey>::ConstKey::pfh_hash(UncasedStr::new(pk.borrow()), state)
    }
    #[inline(always)]
    fn pfh_eq(&self, other: &PK) -> bool {
        self == UncasedStr::new(other.borrow())
    }
}
