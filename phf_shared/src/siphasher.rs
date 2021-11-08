// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A const-compatible implementation of SipHash with a 128-bit output.

use core::{hash, mem, ptr};

#[derive(Debug, Clone, Copy, Default)]
pub struct Hash128 {
    pub h1: u64,
    pub h2: u64,
}

impl const From<u128> for Hash128 {
    fn from(v: u128) -> Self {
        Self {
            h1: v as u64,
            h2: (v >> 64) as u64,
        }
    }
}

impl const From<Hash128> for u128 {
    fn from(v: Hash128) -> Self {
        (v.h1 as u128) | ((v.h2 as u128) << 64)
    }
}

/// An implementation of SipHash128 1-3.
#[derive(Debug, Clone, Copy)]
pub struct SipHasher13 {
    k0: u64,
    k1: u64,
    length: usize, // how many bytes we've processed
    state: State,  // hash State
    tail: u64,     // unprocessed bytes le
    ntail: usize,  // how many bytes in tail are valid
}

#[derive(Debug, Clone, Copy)]
struct State {
    // v0, v2 and v1, v3 show up in pairs in the algorithm,
    // and simd implementations of SipHash will use vectors
    // of v02 and v13. By placing them in this order in the struct,
    // the compiler can pick up on just a few simd optimizations by itself.
    v0: u64,
    v2: u64,
    v1: u64,
    v3: u64,
}

macro_rules! compress {
    ($state:expr) => {{
        compress!($state.v0, $state.v1, $state.v2, $state.v3)
    }};
    ($v0:expr, $v1:expr, $v2:expr, $v3:expr) => {{
        $v0 = $v0.wrapping_add($v1);
        $v1 = $v1.rotate_left(13);
        $v1 ^= $v0;
        $v0 = $v0.rotate_left(32);
        $v2 = $v2.wrapping_add($v3);
        $v3 = $v3.rotate_left(16);
        $v3 ^= $v2;
        $v0 = $v0.wrapping_add($v3);
        $v3 = $v3.rotate_left(21);
        $v3 ^= $v0;
        $v2 = $v2.wrapping_add($v1);
        $v1 = $v1.rotate_left(17);
        $v1 ^= $v2;
        $v2 = $v2.rotate_left(32);
    }};
}

impl State {
    #[inline]
    const fn c_rounds(&mut self) {
        compress!(self);
    }

    #[inline]
    const fn d_rounds(&mut self) {
        compress!(self);
        compress!(self);
        compress!(self);
    }
}

#[inline]
const fn u8to64_le(buf: &[u8], start: usize, len: usize) -> u64 {
    debug_assert!(len < 8);
    let mut i = 0; // current byte index (from LSB) in the output u64
    let mut out = 0;
    if i + 3 < len {
        out = u32::from_le_bytes([
            buf[start + i],
            buf[start + i + 1],
            buf[start + i + 2],
            buf[start + i + 3],
        ]) as u64;
        i += 4;
    }
    if i + 1 < len {
        out |= (u16::from_le_bytes([buf[start + i], buf[start + i + 1]]) as u64) << (i * 8);
        i += 2;
    }
    if i < len {
        out |= (buf[start + i] as u64) << (i * 8);
        i += 1;
    }
    debug_assert!(i == len);
    out
}

impl SipHasher13 {
    /// Creates a new `SipHasher13` that is keyed off the provided keys.
    #[inline(always)]
    pub const fn new_with_keys(key0: u64, key1: u64) -> Self {
        let mut state = SipHasher13 {
            k0: key0,
            k1: key1,
            length: 0,
            state: State {
                v0: 0,
                v1: 0xee,
                v2: 0,
                v3: 0,
            },
            tail: 0,
            ntail: 0,
        };
        state.reset();
        state
    }

    #[inline]
    const fn reset(&mut self) {
        self.length = 0;
        self.state.v0 = self.k0 ^ 0x736f6d6570736575;
        self.state.v1 = self.k1 ^ 0x646f72616e646f83;
        self.state.v2 = self.k0 ^ 0x6c7967656e657261;
        self.state.v3 = self.k1 ^ 0x7465646279746573;
        self.ntail = 0;
    }

    // A specialized write function for values with size <= 8.
    //
    // The hashing of multi-byte integers depends on endianness. E.g.:
    // - little-endian: `write_u32(0xDDCCBBAA)` == `write([0xAA, 0xBB, 0xCC, 0xDD])`
    // - big-endian:    `write_u32(0xDDCCBBAA)` == `write([0xDD, 0xCC, 0xBB, 0xAA])`
    //
    // This function does the right thing for little-endian hardware. On
    // big-endian hardware `x` must be byte-swapped first to give the right
    // behaviour. After any byte-swapping, the input must be zero-extended to
    // 64-bits. The caller is responsible for the byte-swapping and
    // zero-extension.
    #[inline]
    const fn short_write<T>(&mut self, x: u64) {
        let size = mem::size_of::<T>();
        self.length += size;

        // The original number must be zero-extended, not sign-extended.
        debug_assert!(if size < 8 { x >> (8 * size) == 0 } else { true });

        // The number of bytes needed to fill `self.tail`.
        let needed = 8 - self.ntail;

        self.tail |= x << (8 * self.ntail);
        if size < needed {
            self.ntail += size;
            return;
        }

        // `self.tail` is full, process it.
        self.state.v3 ^= self.tail;
        self.state.c_rounds();
        self.state.v0 ^= self.tail;

        self.ntail = size - needed;
        self.tail = if needed < 8 { x >> (8 * needed) } else { 0 };
    }

    /// Return a 128-bit hash
    #[inline]
    pub const fn finish128(&self) -> Hash128 {
        let mut state = self.state;

        let b: u64 = ((self.length as u64 & 0xff) << 56) | self.tail;

        state.v3 ^= b;
        state.c_rounds();
        state.v0 ^= b;

        state.v2 ^= 0xee;
        state.d_rounds();
        let h1 = state.v0 ^ state.v1 ^ state.v2 ^ state.v3;

        state.v1 ^= 0xdd;
        state.d_rounds();
        let h2 = state.v0 ^ state.v1 ^ state.v2 ^ state.v3;

        Hash128 { h1, h2 }
    }
}

impl const hash::Hasher for SipHasher13 {
    #[inline]
    fn finish(&self) -> u64 {
        self.finish128().h2
    }

    #[inline]
    fn write(&mut self, msg: &[u8]) {
        let length = msg.len();
        self.length += length;

        let mut needed = 0;

        if self.ntail != 0 {
            needed = 8 - self.ntail;
            if length < needed {
                self.tail |= u8to64_le(msg, 0, length) << (8 * self.ntail);
                self.ntail += length;
                return;
            } else {
                self.tail |= u8to64_le(msg, 0, needed) << (8 * self.ntail);
                self.state.v3 ^= self.tail;
                self.state.c_rounds();
                self.state.v0 ^= self.tail;
                self.ntail = 0;
            }
        }

        // Buffered tail is now flushed, process new input.
        let len = length - needed;
        let left = len & 0x7;

        let mut i = needed;
        while i < len - left {
            let mi = u64::from_le_bytes([
                msg[i],
                msg[i + 1],
                msg[i + 2],
                msg[i + 3],
                msg[i + 4],
                msg[i + 5],
                msg[i + 6],
                msg[i + 7],
            ]);

            self.state.v3 ^= mi;
            self.state.c_rounds();
            self.state.v0 ^= mi;

            i += 8;
        }

        self.tail = u8to64_le(msg, i, left);
        self.ntail = left;
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.short_write::<u8>(i as u64);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.short_write::<u16>(i.to_le() as u64);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.short_write::<u32>(i.to_le() as u64);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.short_write::<u64>(i.to_le());
    }

    #[inline]
    fn write_u128(&mut self, i: u128) {
        self.write(&i.to_ne_bytes())
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.short_write::<usize>(i.to_le() as u64);
    }

    #[inline]
    fn write_i8(&mut self, i: i8) {
        self.write_u8(i as u8)
    }

    #[inline]
    fn write_i16(&mut self, i: i16) {
        self.write_u16(i as u16)
    }

    #[inline]
    fn write_i32(&mut self, i: i32) {
        self.write_u32(i as u32)
    }

    #[inline]
    fn write_i64(&mut self, i: i64) {
        self.write_u64(i as u64)
    }

    #[inline]
    fn write_i128(&mut self, i: i128) {
        self.write_u128(i as u128)
    }

    #[inline]
    fn write_isize(&mut self, i: isize) {
        self.write_usize(i as usize)
    }
}

impl Hash128 {
    /// Convert into a 16-bytes vector
    pub fn as_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        let h1 = self.h1.to_le();
        let h2 = self.h2.to_le();
        unsafe {
            ptr::copy_nonoverlapping(&h1 as *const _ as *const u8, bytes.get_unchecked_mut(0), 8);
            ptr::copy_nonoverlapping(&h2 as *const _ as *const u8, bytes.get_unchecked_mut(8), 8);
        }
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::SipHasher13;
    use std::hash::{Hash, Hasher};

    // Hash just the bytes of the slice, without length prefix
    struct Bytes<'a>(&'a [u8]);

    impl<'a> Hash for Bytes<'a> {
        #[allow(unused_must_use)]
        fn hash<H: Hasher>(&self, state: &mut H) {
            let Bytes(v) = *self;
            state.write(v);
        }
    }

    fn hash_with<T: Hash>(mut st: SipHasher13, x: &T) -> [u8; 16] {
        x.hash(&mut st);
        st.finish128().as_bytes()
    }

    #[test]
    #[allow(unused_must_use)]
    fn test_siphash128_1_3() {
        let vecs: [[u8; 16]; 1] = [[
            231, 126, 188, 178, 39, 136, 165, 190, 253, 98, 219, 106, 221, 48, 48, 1,
        ]];

        let k0 = 0x_07_06_05_04_03_02_01_00;
        let k1 = 0x_0f_0e_0d_0c_0b_0a_09_08;
        let mut buf = Vec::new();
        let mut t = 0;
        let mut state_inc = SipHasher13::new_with_keys(k0, k1);

        while t < 1 {
            let vec = vecs[t];
            let out = hash_with(SipHasher13::new_with_keys(k0, k1), &Bytes(&buf));
            assert_eq!(vec, out[..]);

            let full = hash_with(SipHasher13::new_with_keys(k0, k1), &Bytes(&buf));
            let i = state_inc.finish128().as_bytes();

            assert_eq!(full, i);
            assert_eq!(full, vec);

            buf.push(t as u8);
            Hasher::write(&mut state_inc, &[t as u8]);

            t += 1;
        }
    }
}
