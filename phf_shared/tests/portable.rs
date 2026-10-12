use phf_shared::{HashKey, PhfHash, hash};

fn check<T: PhfHash + ?Sized>(value: &T, expected: (u32, u32, u32)) {
    let key = HashKey::new(42);
    let actual = hash(value, &key);
    assert_eq!((actual.g, actual.f1, actual.f2), expected);
    #[cfg(feature = "ptrhash")]
    assert_eq!(
        phf_shared::ptrhash::hash(value, &key),
        (u64::from(expected.0) << 32) | u64::from(expected.1),
    );
}

#[test]
fn byte_hashes_match_across_targets() {
    // Fixed vectors from x86_64. Run unchanged on 32-bit and big-endian targets
    // to exercise each of foldhash's byte-loading and long-input paths.
    for (len, expected) in [
        (0, (0x110f5e99, 0x706a060b, 0x8470725e)),
        (1, (0x9880a0fb, 0xa80b010d, 0xce085921)),
        (4, (0xf9339b7d, 0xd929a65f, 0xd46ef063)),
        (8, (0xdffb1f35, 0xe6195c0b, 0xf4859928)),
        (16, (0x555a8776, 0xc6eca44a, 0x46a4546d)),
        (17, (0x296c0931, 0x0fd7da86, 0x4ec1fda7)),
        (64, (0xe09da881, 0x7529c14f, 0x385e636e)),
        (128, (0x1f64cd82, 0x086c3dc6, 0x18290ba7)),
        (129, (0x3d006032, 0xc23168d1, 0x9b526a17)),
        (256, (0x4004b677, 0x195d94a9, 0xd7c3b8f0)),
        (257, (0x702d4a0b, 0x654b2fda, 0xf1e1a84b)),
    ] {
        let bytes = (0..len).map(|i| i as u8).collect::<Vec<_>>();
        check(bytes.as_slice(), expected);
    }
}

#[test]
fn integer_hashes_match_across_targets() {
    check(&0x12345678usize, (0xe51f7946, 0xda24afc1, 0xd1edd070));
    check(&-2147483648isize, (0xe992584a, 0x033071d2, 0x8f4d2d66));
    check(
        &0xc0ac29b7c97c50dd_0000000000000001u128,
        (0x215362d0, 0x0766560e, 0x58064431),
    );
    check(
        &(
            0xabu8,
            0x1234u16,
            0x56789abcu32,
            0xfedcba9876543210u64,
            -5isize,
            "a string longer than sixteen bytes",
        ),
        (0xb9cb544c, 0x4a8f35f8, 0x429bba38),
    );
}

#[test]
fn retrying_changes_the_shared_seed() {
    let left = 0x8000000080000001_0000000000000000u128;
    let right = left | 1;
    let components = |value, seed| {
        let hash = hash(&value, &HashKey::new(seed));
        (hash.g, hash.f1, hash.f2)
    };
    assert_eq!(components(left, 0), components(right, 0));
    assert_ne!(components(left, 1), components(right, 1));
    #[cfg(feature = "ptrhash")]
    {
        let hash = |value, seed| phf_shared::ptrhash::hash(&value, &HashKey::new(seed));
        assert_eq!(hash(left, 0), hash(right, 0));
        assert_ne!(hash(left, 1), hash(right, 1));
    }
}
