// Copyright (c) 2024 Orson Peters
// Distributed with permission under the zlib license.
// Minor modifications made to remove any platform dependant code.

use core::hash::{BuildHasher, Hasher};

// Arbitrary constants with high entropy. Hexadecimal digits of pi were used.
pub(crate) const ARBITRARY0: u64 = 0x243f6a8885a308d3;
const ARBITRARY1: u64 = 0x13198a2e03707344;
const ARBITRARY2: u64 = 0xa4093822299f31d0;
const ARBITRARY3: u64 = 0x082efa98ec4e6c89;
const ARBITRARY4: u64 = 0x452821e638d01377;

/// Used for FixedState, and RandomState if atomics for dynamic init are unavailable.
const FIXED_GLOBAL_SEED: [u64; 4] = [ARBITRARY1, ARBITRARY2, ARBITRARY3, ARBITRARY4];

#[inline(always)]
pub(super) const fn folded_multiply(x: u64, y: u64) -> u64 {
    // We compute the full u64 x u64 -> u128 product, this is a single mul
    // instruction on x86-64, one mul plus one mulhi on ARM64.
    let full = (x as u128) * (y as u128);
    let lo = full as u64;
    let hi = (full >> 64) as u64;

    // The middle bits of the full product fluctuate the most with small
    // changes in the input. This is the top bits of lo and the bottom bits
    // of hi. We can thus make the entire output fluctuate with small
    // changes to the input by XOR'ing these two halves.
    lo ^ hi
}

/// The foldhash implementation optimized for speed.
pub mod fast {
    use super::*;

    /// A [`BuildHasher`] for [`fast::FoldHasher`]s that all have the same fixed seed.
    pub struct FixedState {
        per_hasher_seed: u64,
    }

    impl FixedState {
        /// Creates a [`FixedState`] with the given seed.
        #[inline(always)]
        pub const fn with_seed(seed: u64) -> Self {
            Self {
                per_hasher_seed: seed,
            }
        }
    }

    impl BuildHasher for FixedState {
        type Hasher = FoldHasher;

        #[inline(always)]
        fn build_hasher(&self) -> FoldHasher {
            FoldHasher::with_seed(self.per_hasher_seed, &FIXED_GLOBAL_SEED)
        }
    }

    /// A [`Hasher`] instance implementing foldhash, optimized for speed.
    ///
    /// It can't be created directly, see [`FixedState`].
    #[derive(Clone)]
    pub struct FoldHasher {
        accumulator: u64,
        sponge: u128,
        sponge_len: u8,
        fold_seed: u64,
        expand_seed: u64,
        expand_seed2: u64,
        expand_seed3: u64,
    }

    impl FoldHasher {
        #[inline]
        pub(crate) fn with_seed(per_hasher_seed: u64, global_seed: &[u64; 4]) -> FoldHasher {
            FoldHasher {
                accumulator: per_hasher_seed,
                sponge: 0,
                sponge_len: 0,
                fold_seed: global_seed[0],
                expand_seed: global_seed[1],
                expand_seed2: global_seed[2],
                expand_seed3: global_seed[3],
            }
        }

        #[inline(always)]
        fn write_num<T: Into<u128>>(&mut self, x: T) {
            let bits: usize = 8 * core::mem::size_of::<T>();
            if self.sponge_len as usize + bits > 128 {
                let lo = self.sponge as u64;
                let hi = (self.sponge >> 64) as u64;
                self.accumulator = folded_multiply(lo ^ self.accumulator, hi ^ self.fold_seed);
                self.sponge = x.into();
                self.sponge_len = bits as u8;
            } else {
                self.sponge |= x.into() << self.sponge_len;
                self.sponge_len += bits as u8;
            }
        }
    }

    impl Hasher for FoldHasher {
        #[inline(always)]
        fn write(&mut self, bytes: &[u8]) {
            let len = bytes.len();
            if len <= 16 {
                let mut s0 = self.accumulator;
                let mut s1 = self.expand_seed;
                // XOR the input into s0, s1, then multiply and fold.
                if len >= 8 {
                    s0 ^= u64::from_le_bytes(bytes[0..8].try_into().unwrap());
                    s1 ^= u64::from_le_bytes(bytes[len - 8..].try_into().unwrap());
                } else if len >= 4 {
                    s0 ^= u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as u64;
                    s1 ^= u32::from_le_bytes(bytes[len - 4..].try_into().unwrap()) as u64;
                } else if len > 0 {
                    let lo = bytes[0];
                    let mid = bytes[len / 2];
                    let hi = bytes[len - 1];
                    s0 ^= lo as u64;
                    s1 ^= ((hi as u64) << 8) | mid as u64;
                }
                self.accumulator = folded_multiply(s0, s1);
            } else if len < 256 {
                self.accumulator = hash_bytes_medium(
                    bytes,
                    self.accumulator,
                    self.accumulator.wrapping_add(self.expand_seed),
                    self.fold_seed,
                );
            } else {
                self.accumulator = hash_bytes_long(
                    bytes,
                    self.accumulator,
                    self.accumulator.wrapping_add(self.expand_seed),
                    self.accumulator.wrapping_add(self.expand_seed2),
                    self.accumulator.wrapping_add(self.expand_seed3),
                    self.fold_seed,
                );
            }
        }

        #[inline(always)]
        fn write_u8(&mut self, i: u8) {
            self.write_num(i);
        }

        #[inline(always)]
        fn write_u16(&mut self, i: u16) {
            self.write_num(i);
        }

        #[inline(always)]
        fn write_u32(&mut self, i: u32) {
            self.write_num(i);
        }

        #[inline(always)]
        fn write_u64(&mut self, i: u64) {
            self.write_num(i);
        }

        #[inline(always)]
        fn write_u128(&mut self, i: u128) {
            let lo = i as u64;
            let hi = (i >> 64) as u64;
            self.accumulator = folded_multiply(lo ^ self.accumulator, hi ^ self.fold_seed);
        }

        #[inline(always)]
        fn write_usize(&mut self, i: usize) {
            // u128 doesn't implement From<usize>.
            self.write_num(i as u64);
        }

        #[inline(always)]
        fn finish(&self) -> u64 {
            if self.sponge_len > 0 {
                let lo = self.sponge as u64;
                let hi = (self.sponge >> 64) as u64;
                folded_multiply(lo ^ self.accumulator, hi ^ self.fold_seed)
            } else {
                self.accumulator
            }
        }
    }
}

/// Hashes strings >= 16 bytes, has unspecified behavior when bytes.len() < 16.
fn hash_bytes_medium(bytes: &[u8], mut s0: u64, mut s1: u64, fold_seed: u64) -> u64 {
    // Process 32 bytes per iteration, 16 bytes from the start, 16 bytes from
    // the end. On the last iteration these two chunks can overlap, but that is
    // perfectly fine.
    let left_to_right = bytes.chunks_exact(16);
    let mut right_to_left = bytes.rchunks_exact(16);
    for lo in left_to_right {
        let hi = right_to_left.next().unwrap();
        let unconsumed_start = lo.as_ptr();
        let unconsumed_end = hi.as_ptr_range().end;
        if unconsumed_start >= unconsumed_end {
            break;
        }

        let a = u64::from_le_bytes(lo[0..8].try_into().unwrap());
        let b = u64::from_le_bytes(lo[8..16].try_into().unwrap());
        let c = u64::from_le_bytes(hi[0..8].try_into().unwrap());
        let d = u64::from_le_bytes(hi[8..16].try_into().unwrap());
        s0 = folded_multiply(a ^ s0, c ^ fold_seed);
        s1 = folded_multiply(b ^ s1, d ^ fold_seed);
    }

    s0 ^ s1
}

/// Hashes strings >= 16 bytes, has unspecified behavior when bytes.len() < 16.
#[cold]
#[inline(never)]
fn hash_bytes_long(
    bytes: &[u8],
    mut s0: u64,
    mut s1: u64,
    mut s2: u64,
    mut s3: u64,
    fold_seed: u64,
) -> u64 {
    let chunks = bytes.chunks_exact(64);
    let remainder = chunks.remainder().len();
    for chunk in chunks {
        let a = u64::from_le_bytes(chunk[0..8].try_into().unwrap());
        let b = u64::from_le_bytes(chunk[8..16].try_into().unwrap());
        let c = u64::from_le_bytes(chunk[16..24].try_into().unwrap());
        let d = u64::from_le_bytes(chunk[24..32].try_into().unwrap());
        let e = u64::from_le_bytes(chunk[32..40].try_into().unwrap());
        let f = u64::from_le_bytes(chunk[40..48].try_into().unwrap());
        let g = u64::from_le_bytes(chunk[48..56].try_into().unwrap());
        let h = u64::from_le_bytes(chunk[56..64].try_into().unwrap());
        s0 = folded_multiply(a ^ s0, e ^ fold_seed);
        s1 = folded_multiply(b ^ s1, f ^ fold_seed);
        s2 = folded_multiply(c ^ s2, g ^ fold_seed);
        s3 = folded_multiply(d ^ s3, h ^ fold_seed);
    }
    s0 ^= s2;
    s1 ^= s3;

    if remainder > 0 {
        hash_bytes_medium(&bytes[bytes.len() - remainder.max(16)..], s0, s1, fold_seed)
    } else {
        s0 ^ s1
    }
}
