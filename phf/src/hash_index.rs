use phf_shared::{HashKey, PhfHash};

/// A perfect hash index into a separate slice of entries.
#[doc(hidden)]
pub struct HashIndex {
    pub key: HashKey,
    #[cfg(not(feature = "ptrhash"))]
    pub disps: &'static [(u32, u32)],
    #[cfg(feature = "ptrhash")]
    pub pilots: &'static [u8],
    // Both hash layouts address slots with u32, including on 64-bit targets.
    pub idxs: &'static [u32],
}

impl HashIndex {
    pub(crate) const EMPTY: Self = Self {
        key: 0,
        #[cfg(not(feature = "ptrhash"))]
        disps: &[],
        #[cfg(feature = "ptrhash")]
        pilots: &[],
        idxs: &[],
    };

    // A miss can hash to any entry, so the caller must still compare the key.
    #[inline]
    pub(crate) fn candidate<T: PhfHash + ?Sized>(&self, key: &T) -> Option<usize> {
        if self.idxs.is_empty() {
            return None;
        }
        #[cfg(not(feature = "ptrhash"))]
        let slot = phf_shared::get_index(
            &phf_shared::hash(key, &self.key),
            self.disps,
            self.idxs.len(),
        );
        #[cfg(feature = "ptrhash")]
        let slot = phf_shared::ptrhash::get_index(
            phf_shared::ptrhash::hash(key, &self.key),
            self.pilots,
            self.idxs.len(),
        );
        Some(self.idxs[slot as usize] as usize)
    }
}
