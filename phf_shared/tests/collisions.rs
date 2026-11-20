//! Keys must not collide under every seed, or table generation never ends.

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use phf_shared::{PhfHash, hash};

fn assert_separable<T: PhfHash>(left: &T, right: &T) {
    for seed in 0..64 {
        let (left, right) = (hash(left, &seed), hash(right, &seed));
        assert_ne!(
            (left.g, left.f1, left.f2),
            (right.g, right.f1, right.f2),
            "seed {seed}"
        );
    }
}

/// Delegates to `Hash`, which writes a string without a length prefix.
#[derive(Hash)]
struct Unprefixed(&'static str);

impl PhfHash for Unprefixed {
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

#[test]
fn input_cannot_cancel_its_length() {
    // These differ exactly by what XORing a multiple of their lengths into
    // the state would add, so a hasher that mixes the length in that way
    // gives both the same hash under every seed.
    assert_separable(
        &Unprefixed("\u{7ff}\x7f=\u{7ff}\u{d7ff}"),
        &Unprefixed("^`1\x18\u{5ae}\x13\u{376}n\u{cd}>2\u{45a}"),
    );
}

#[test]
fn byte_strings_need_no_length_prefix() {
    // Byte strings are written without a length prefix, so the hasher has to
    // keep apart keys that differ only in length. Lengths 64 apart rotate the
    // state by the same amount and differ only in how many blocks are folded.
    for byte in [0, b'a'] {
        let key = [byte; 130];
        for seed in 0..64 {
            let mut seen = HashSet::new();
            for len in 0..=key.len() {
                let hashes = hash(&key[..len], &seed);
                // PtrHash uses only these two.
                assert!(
                    seen.insert((hashes.g, hashes.f1)),
                    "length {len}, seed {seed}"
                );
            }
        }
    }
    // Nor do neighbouring strings run together.
    assert_separable(&("ab", "c"), &("a", "bc"));
    assert_separable(&("", "a"), &("a", ""));
}

/// Writes each field one byte at a time and ends it with a word.
struct Terminated(&'static [&'static str]);

impl PhfHash for Terminated {
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        for field in self.0 {
            for byte in field.bytes() {
                state.write(&[byte]);
            }
            state.write_u64(1);
        }
    }
}

#[test]
fn word_ahead_of_bytes_is_hashed() {
    // The first terminator equals the length of the write that follows it.
    // Taking it for a length prefix and skipping it would lose where the
    // field ends.
    assert_separable(&Terminated(&["ab", "c"]), &Terminated(&["a", "bc"]));
}

fn blocks(first: u8, second: u8) -> [u8; 80] {
    let mut bytes = [0; 80];
    bytes[..16].fill(first);
    bytes[16..32].fill(second);
    bytes
}

#[test]
fn lanes_do_not_swap_or_cancel() {
    // Inputs over 64 bytes are folded in four lanes of 16-byte blocks. Lanes
    // sharing a starting state could swap blocks, or cancel out on equal
    // blocks, under every seed.
    assert_separable(&blocks(1, 2), &blocks(2, 1));
    assert_separable(&blocks(1, 1), &blocks(2, 2));
}

/// Writes one small integer per byte, like `UniCase` and `UncasedStr`.
struct Bytewise(&'static [u8]);

impl PhfHash for Bytewise {
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        for &byte in self.0 {
            state.write_u8(byte);
        }
    }
}

#[test]
fn trailing_zero_integers_are_hashed() {
    // Small integers are packed into words. A partial word must differ from a
    // longer run or a full word that packs to the same value.
    let runs: [&[u8]; 5] = [
        &[],
        &[1],
        &[1, 0],
        &[1, 0, 0, 0, 0, 0, 0, 0],
        &[1, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    for (i, left) in runs.iter().enumerate() {
        for right in &runs[i + 1..] {
            assert_separable(&Bytewise(left), &Bytewise(right));
        }
    }
}
