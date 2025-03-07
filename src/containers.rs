use crate::{displace, HashValue};

pub mod ordered_map;
pub mod ordered_set;

pub use ordered_map::OrderedMap;
pub use ordered_set::OrderedSet;

/// Return an index into `phf_generator::BuilderState::map`.
///
/// * `hash` is from `hash()` in this crate.
/// * `disps` is from `phf_generator::BuilderState::disps`.
/// * `len` is the length of `phf_generator::BuilderState::map`.
#[inline]
const fn get_index(hashes: &HashValue, disps: &[(u32, u32)], len: usize) -> u32 {
    let (d1, d2) = disps[(hashes.g % (disps.len() as u32)) as usize];
    displace(hashes.f1, hashes.f2, d1, d2) % (len as u32)
}
