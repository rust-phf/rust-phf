use core::hash::Hasher;

// Odd, bit-balanced multipliers: wyhash's default secret.
const K0: u64 = 0x2d358dccaa6c78a5;
const K1: u64 = 0x8bb84b93962eacc9;
const K2: u64 = 0x4b33a62ed433d4a3;
const K3: u64 = 0x4d5a2da51de1aa47;

// The state is rotated differently inside a long input and at its end, so a
// block can never stand in for the end of a shorter input. Swapping the two
// multiplicands must not line them up either: neither rotation, nor their
// sum, may be a multiple of 32.
const BLOCK_ROTATION: u32 = 23;
const END_ROTATION: u32 = 37;

/// Multiplies to 128 bits and folds the halves together.
#[inline]
const fn fold(a: u64, b: u64) -> u64 {
    let r = (a as u128) * (b as u128);
    (r as u64) ^ ((r >> 64) as u64)
}

#[inline]
fn read_u64(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

#[inline]
fn read_u32(bytes: &[u8], at: usize) -> u64 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()).into()
}

/// Folds a 16-byte block into the state.
#[inline]
fn fold_block(state: u64, block: &[u8]) -> u64 {
    fold(
        read_u64(block, 0) ^ state,
        read_u64(block, 8) ^ state.rotate_left(BLOCK_ROTATION) ^ K2,
    )
}

/// A folded-multiply hasher whose output is the same on every target, so
/// tables generated on the host stay valid on the target: bytes are read
/// little-endian, integers are hashed at a fixed width, and `u128` arithmetic
/// does not depend on the pointer width.
///
/// Key bits are XORed into the seeded state before they are multiplied, while
/// lengths rotate that state. The input cannot undo a rotation, so retrying
/// with a new seed separates keys that collided under the previous one.
///
/// Each `write` is folded in on its own, together with its length. Byte
/// strings therefore need no length prefix, and `("ab", "c")` does not hash
/// like `("a", "bc")`.
pub struct PortableHasher {
    state: u64,
    /// Integers that are not hashed yet, packed from the low bits up.
    ints: u64,
    ints_bits: u32,
}

impl PortableHasher {
    #[inline]
    pub fn new(seed: u64) -> Self {
        Self {
            // Rotating a zero state has no effect, so keep seed 0 away from it.
            state: seed ^ K1,
            ints: 0,
            ints_bits: 0,
        }
    }

    /// The state with the pending integers hashed in.
    #[inline]
    fn flushed(&self) -> u64 {
        if self.ints_bits == 0 {
            self.state
        } else {
            // Rotating by the bit count keeps a partial word apart from a full
            // one with the same value. A full word rotates by zero.
            fold(self.state.rotate_left(self.ints_bits) ^ self.ints, K0)
        }
    }

    #[inline]
    fn flush(&mut self) {
        self.state = self.flushed();
        self.ints = 0;
        self.ints_bits = 0;
    }

    /// Packs integers into 64-bit words, so a run of small ones costs one
    /// multiplication per word.
    #[inline]
    fn write_int(&mut self, i: u64, bits: u32) {
        if self.ints_bits + bits > 64 {
            self.flush();
        }
        self.ints |= i << self.ints_bits;
        self.ints_bits += bits;
    }
}

impl Hasher for PortableHasher {
    #[inline]
    fn finish(&self) -> u64 {
        fold(self.flushed() ^ K3, K1)
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.flush();
        let len = bytes.len();
        let mut state = self.state.rotate_left(len as u32);
        let (a, b) = if len > 16 {
            // Fold in 16 bytes at a time. The final 16 bytes are mixed below
            // and may overlap the last block.
            let mut rest = bytes;
            if len > 64 {
                // Long inputs go through four lanes, whose multiplications do
                // not wait for each other. Each lane starts from the state
                // plus its own constant. The input cannot undo an addition, so
                // lanes neither swap places nor cancel out when they are joined.
                let [mut s0, mut s1, mut s2, mut s3] =
                    [K0, K1, K2, K3].map(|k| state.wrapping_add(k));
                while rest.len() > 64 {
                    s0 = fold_block(s0, rest);
                    s1 = fold_block(s1, &rest[16..]);
                    s2 = fold_block(s2, &rest[32..]);
                    s3 = fold_block(s3, &rest[48..]);
                    rest = &rest[64..];
                }
                state = s0 ^ s1 ^ s2 ^ s3;
            }
            while rest.len() > 16 {
                state = fold_block(state, rest);
                rest = &rest[16..];
            }
            (read_u64(bytes, len - 16), read_u64(bytes, len - 8))
        } else if len >= 8 {
            (read_u64(bytes, 0), read_u64(bytes, len - 8))
        } else if len >= 4 {
            (read_u32(bytes, 0), read_u32(bytes, len - 4))
        } else if len > 0 {
            let packed = u64::from(bytes[0]) << 16
                | u64::from(bytes[len / 2]) << 8
                | u64::from(bytes[len - 1]);
            (packed, 0)
        } else {
            (0, 0)
        };
        self.state = fold(a ^ state, b ^ state.rotate_left(END_ROTATION) ^ K3);
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.write_int(i.into(), 8);
    }
    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.write_int(i.into(), 16);
    }
    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.write_int(i.into(), 32);
    }
    #[inline]
    fn write_u64(&mut self, i: u64) {
        // A full word cannot be packed with the integers before it.
        self.flush();
        self.ints = i;
        self.ints_bits = 64;
    }
    #[inline]
    fn write_u128(&mut self, i: u128) {
        self.write_u64(i as u64);
        self.write_u64((i >> 64) as u64);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.write_u64(i as u64);
    }
    // The default `write_isize` implementation casts to `usize` first, so we end up with
    // `i as usize as u64`, which is different on 32-bit and 64-bit architectures:
    //
    // 32-bit: `-1` -> `2^32 - 1` -> `2^32 - 1`
    // 64-bit: `-1` -> `2^64 - 1` -> `2^64 - 1`
    #[inline]
    fn write_isize(&mut self, i: isize) {
        self.write_u64(i as u64);
    }

    // Fixed-size signed integers correctly forward to unsigned implementations.
}
