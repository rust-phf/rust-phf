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

#[inline]
pub fn hash<T: ?Sized + Hash>(x: &T, seed: u64) -> (u32, u32, u32) {
    let mut hasher = SipHasher::new_with_keys(seed, 0);
    x.hash(&mut hasher);
    split(hasher.finish())
}
