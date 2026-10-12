use core::hash::Hasher;
use foldhash::fast::FoldHasher;

use crate::HashKey;

pub struct PortableFoldHasher<'a> {
    inner: FoldHasher<'a>,
}

impl<'a> PortableFoldHasher<'a> {
    #[inline]
    pub fn new(key: &'a HashKey) -> Self {
        Self {
            inner: FoldHasher::with_seed(key.seed, &key.shared_seed),
        }
    }
}

impl Hasher for PortableFoldHasher<'_> {
    #[inline]
    fn finish(&self) -> u64 {
        self.inner.finish()
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.inner.write(bytes)
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.inner.write_u8(i)
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.inner.write_u16(i);
    }
    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.inner.write_u32(i);
    }
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.inner.write_u64(i);
    }
    #[inline]
    fn write_u128(&mut self, i: u128) {
        self.inner.write_u128(i);
    }

    // Encode pointer-sized integers at a fixed width on the host and target.
    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.inner.write_u64(i as u64);
    }
    // The default `write_isize` implementation casts to `usize` first, so we end up with
    // `i as usize as u64`, which is different on 32-bit and 64-bit architectures:
    //
    // 32-bit: `-1` -> `2^32 - 1` -> `2^32 - 1`
    // 64-bit: `-1` -> `2^64 - 1` -> `2^64 - 1`
    #[inline]
    fn write_isize(&mut self, i: isize) {
        self.inner.write_u64(i as u64);
    }

    // Fixed-size signed integers correctly forward to unsigned implementations.
}
