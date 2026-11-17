use core::hash::Hasher;

use crate::{HashKey, PhfHash, PortableHasher};

/// `key` is from `phf_generator::ptrhash::HashState`.
#[inline]
pub fn hash<T: ?Sized + PhfHash>(x: &T, key: &HashKey) -> u64 {
    let mut hasher = PortableHasher::new(*key);
    x.phf_hash(&mut hasher);
    hasher.finish()
}

/// Return the slot for `hash` with the given `pilot`.
#[inline]
pub fn slot_index(hash: u64, pilot: u8, slots_len: u32) -> u32 {
    // Mix before taking the high bits so pilots can separate hashes even when
    // the slot count is a power of two.
    let multiplier = 2 * u64::from(pilot) + 1;
    fast_reduct32(high(hash.wrapping_mul(multiplier)), slots_len)
}

/// Return an index into `phf_generator::ptrhash::HashState::map`.
///
/// * `hash` is from `hash()` in this crate.
/// * `pilots` is from `phf_generator::ptrhash::HashState::pilots`.
/// * `len` is the length of `phf_generator::ptrhash::HashState::map`.
#[inline]
pub fn get_index(hash: u64, pilots: &[u8], len: usize) -> u32 {
    let pilots_len = pilots.len() as u32;

    let bucket = fast_reduct32(low(hash), pilots_len) as usize;
    slot_index(hash, pilots[bucket], len as u32)
}

// https://lemire.me/blog/2016/06/27/a-fast-alternative-to-the-modulo-reduction/
#[inline]
pub fn fast_reduct32(x: u32, limit: u32) -> u32 {
    (((x as u64) * (limit as u64)) >> 32) as u32
}

#[inline]
fn low(v: u64) -> u32 {
    v as u32
}

#[inline]
fn high(v: u64) -> u32 {
    (v >> 32) as u32
}
