#![doc(html_root_url = "https://docs.rs/phf_shared/0.7.20")]
#![no_std]

extern crate siphasher;

use core::hash::Hasher;
use siphasher::sip::SipHasher13;

#[inline]
pub fn displace(f1: u32, f2: u32, d1: u32, d2: u32) -> u32 {
    d2 + f1 * d1 + f2
}

#[inline]
pub fn split(hash: u64) -> (u32, u32, u32) {
    const BITS: u32 = 21;
    const MASK: u64 = (1 << BITS) - 1;

    (
        (hash & MASK) as u32,
        ((hash >> BITS) & MASK) as u32,
        ((hash >> (2 * BITS)) & MASK) as u32,
    )
}

/// `key` is from `phf_generator::HashState::key`.
#[inline]
pub fn hash<T: ?Sized + AsRef<[u8]>>(x: &T, key: u64) -> u64 {
    let mut hasher = SipHasher13::new_with_keys(0, key);
    hasher.write(x.as_ref());
    hasher.finish()
}

/// Return an index into `phf_generator::HashState::map`.
///
/// * `hash` is from `hash()` in this crate.
/// * `disps` is from `phf_generator::HashState::disps`.
/// * `len` is the length of `phf_generator::HashState::map`.
#[inline]
pub fn get_index(hash: u64, disps: &[(u32, u32)], len: usize) -> u32 {
    let (g, f1, f2) = split(hash);
    let (d1, d2) = disps[(g % (disps.len() as u32)) as usize];
    displace(f1, f2, d1, d2) % (len as u32)
}
