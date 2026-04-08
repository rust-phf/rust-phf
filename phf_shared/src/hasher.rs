use core::hash::Hasher;
use siphasher::sip128::{Hash128, Hasher128};

pub struct PortableSipHasher<H> {
    inner: H,
}

impl<H> PortableSipHasher<H> {
    pub fn new(inner: H) -> Self {
        Self { inner }
    }
}

impl<H: Hasher> Hasher for PortableSipHasher<H> {
    fn finish(&self) -> u64 {
        self.inner.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.inner.write(bytes)
    }

    fn write_u8(&mut self, i: u8) {
        self.inner.write_u8(i)
    }

    // `SipHasher` invokes `to_le` on integers before encoding them, treating them as byte arrays
    // rather than values, which is not the right interpretation for us; fix that by flipping
    // endianness twice.
    fn write_u16(&mut self, i: u16) {
        self.inner.write_u16(i.to_le());
    }
    fn write_u32(&mut self, i: u32) {
        self.inner.write_u32(i.to_le());
    }
    fn write_u64(&mut self, i: u64) {
        self.inner.write_u64(i.to_le());
    }
    fn write_u128(&mut self, i: u128) {
        self.inner.write_u128(i.to_le());
    }
    fn write_usize(&mut self, i: usize) {
        self.inner.write_usize(i.to_le());
    }

    // Signed integers correctly forward to unsigned implementations.
}

impl<H: Hasher128> Hasher128 for PortableSipHasher<H> {
    fn finish128(&self) -> Hash128 {
        self.inner.finish128()
    }
}
