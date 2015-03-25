#![doc(html_root_url="http://sfackler.github.io/rust-phf/doc")]

use std::hash::{Hasher, Hash, SipHasher};

#[inline]
pub fn displace(f1: u32, f2: u32, d1: u32, d2: u32) -> u32 {
    d2 + f1 * d1 + f2
}

#[inline]
fn split(hash: u64) -> (u32, u32, u32) {
    const BITS: u32 = 21;
    const MASK: u64 = (1 << BITS) - 1;

    ((hash & MASK) as u32,
     ((hash >> BITS) & MASK) as u32,
     ((hash >> (2 * BITS)) & MASK) as u32)
}

/// A trait implemented by types which can be used in PHF data structures
pub trait PhfHash {
    /// Hashes the value of `self`, factoring in a seed
    fn phf_hash(&self, seed: u64) -> (u32, u32, u32);
}

impl<'a> PhfHash for &'a str {
    #[inline]
    fn phf_hash(&self, seed: u64) -> (u32, u32, u32) {
        self.as_bytes().phf_hash(seed)
    }
}

impl<'a> PhfHash for &'a [u8] {
    #[inline]
    fn phf_hash(&self, seed: u64) -> (u32, u32, u32) {
        (*self).phf_hash(seed)
    }
}

impl PhfHash for str {
    #[inline]
    fn phf_hash(&self, seed: u64) -> (u32, u32, u32) {
        self.as_bytes().phf_hash(seed)
    }
}

impl PhfHash for [u8] {
    #[inline]
    fn phf_hash(&self, seed: u64) -> (u32, u32, u32) {
        let mut state = SipHasher::new_with_keys(seed, 0);
        Hasher::write(&mut state, self);
        split(state.finish())
    }
}


macro_rules! sip_impl(
    ($t:ty) => (
        impl PhfHash for $t {
            #[inline]
            fn phf_hash(&self, seed: u64) -> (u32, u32, u32) {
                let mut hasher = SipHasher::new_with_keys(seed, 0);
                self.hash(&mut hasher);
                split(hasher.finish())
            }
        }
    )
);

sip_impl!(u8);
sip_impl!(i8);
sip_impl!(u16);
sip_impl!(i16);
sip_impl!(u32);
sip_impl!(i32);
sip_impl!(u64);
sip_impl!(i64);
sip_impl!(char);
sip_impl!(bool);

macro_rules! array_impl(
    ($t:ty, $n:expr) => (
        impl PhfHash for [$t; $n] {
            #[inline]
            fn phf_hash(&self, seed: u64) -> (u32, u32, u32) {
                let mut hasher = SipHasher::new_with_keys(seed, 0);
                Hasher::write(&mut hasher, self);
                split(hasher.finish())
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
