//! Shared code and constants between the phf and phf_mac crates
use std::hash::{Hash, Hasher};
use std::hash::sip::SipHasher;

static LOG_MAX_SIZE: uint = 21;

#[doc(hidden)]
pub static MAX_SIZE: uint = 1 << LOG_MAX_SIZE;

#[doc(hidden)]
#[inline]
pub fn hash<T: Hash>(s: &T, k1: u64, k2: u64) -> (uint, uint, uint) {
    let hash = SipHasher::new_with_keys(k1, k2).hash(s);
    let mask = (MAX_SIZE - 1) as u64;

    ((hash & mask) as uint,
     ((hash >> LOG_MAX_SIZE) & mask) as uint,
     ((hash >> (2 * LOG_MAX_SIZE)) & mask) as uint)
}

#[doc(hidden)]
#[inline]
pub fn displace(f1: uint, f2: uint, d1: uint, d2: uint) -> uint {
    d2 + f1 * d1 + f2
}
