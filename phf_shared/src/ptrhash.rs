use siphasher::sip::SipHasher13;
use crate::{ PhfHash, HashKey };


/// `key` is from `phf_generator::HashState`.
#[inline]
pub fn hash<T: ?Sized + PhfHash>(x: &T, key: &HashKey) -> u64 {
    use core::hash::Hasher;

    let mut hasher = SipHasher13::new_with_keys(0, *key);
    x.phf_hash(&mut hasher);
    hasher.finish()
}

#[inline]
pub fn hash_pilot(k: u64, pilot: u8) -> u64 {
    const C: u64 = 0x517cc1b727220a95;

    // fxhash
    C.wrapping_mul(k ^ u64::from(pilot))
}

/// Return an index into `phf_generator::HashState::map`.
///
/// * `seed` is phf seed.
/// * `hash` is from `hash()` in this crate.
/// * `pilots` is from `phf_generator::ptrhash::HashState::pilots`.
/// * `remap` is from `phf_generator::ptrhash::HashState::remap`
/// * `len` is the length of `phf_generator::ptrhash::HashState::map`.
#[inline]
pub fn get_index(seed: u64, hash: u64, pilots: &[u8], remap: &[u32], len: usize) -> u32 {
    let pilots_len = pilots.len() as u32;
    let slots_len = (len + remap.len()) as u32;

    let bucket = fast_reduct32(low(hash), pilots_len) as usize;
    let pilot = pilots[bucket];
    let pilot_hash = hash_pilot(seed, pilot);

    let index = fast_reduct32(
        high(hash) ^ high(pilot_hash) ^ low(pilot_hash),
        slots_len
    );
    let index_len = index as usize;

    if index_len < len {
        index
    } else {
        remap[index_len - len]
    }
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
