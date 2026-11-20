use phf_shared::{PhfHash, hash};

fn check<T: PhfHash + ?Sized>(value: &T, expected: (u32, u32, u32)) {
    let key = 42;
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
    // to exercise each byte-loading path.
    for (len, expected) in [
        (0, (0xea085796, 0xffb53434, 0xcebcd66d)),
        (1, (0x05e16102, 0x00da00a2, 0xab67128e)),
        (2, (0x5c028cc2, 0x11395fe0, 0xa6c3448c)),
        (3, (0x1cd0cebd, 0x1d57532d, 0xe6e678db)),
        (4, (0x60d6e8b0, 0x8dc10445, 0xcafa28c2)),
        (7, (0x3f70c62f, 0xbbe49d54, 0x76580929)),
        (8, (0x79e34bc2, 0xd232d1f9, 0xa2e8cc97)),
        (16, (0xc22b1a41, 0x710a047d, 0x7e899d69)),
        (17, (0x9f454efa, 0xec7e4fac, 0x0c3a3942)),
        (32, (0xf0310001, 0x10e9660b, 0xcf0c0be5)),
        (33, (0x03402fc6, 0x2906586c, 0xe5e9bcad)),
        (64, (0x5de0f2c2, 0x028e8b19, 0xd80bd23e)),
        (65, (0x8f995b8f, 0xe97a6ab1, 0xa5df2f13)),
        (100, (0x8b1dc112, 0x541d7519, 0x4b92e1cd)),
        (200, (0x0707b599, 0x0a36281b, 0xd0d72141)),
    ] {
        let bytes = (0..len).map(|i| i as u8).collect::<Vec<_>>();
        check(bytes.as_slice(), expected);
    }
}

#[test]
fn integer_hashes_match_across_targets() {
    check(&0x12345678usize, (0xefdb5ad0, 0xd3ec5981, 0x8bcccbb5));
    check(&-2147483648isize, (0xc419f79b, 0xb7d90a47, 0x7776ff70));
    check(
        &0xfedcba9876543210_0123456789abcdefu128,
        (0xaafb13c7, 0xe8453bbd, 0xf547f4b5),
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
        (0x708524e5, 0x59fb9a50, 0x3bc5f817),
    );
    check(
        &[0x1234u16, 0x5678, 0x9abc, 0xdef0, 0x0fed][..],
        (0x0c909868, 0x4af0d98d, 0x0296220a),
    );
}
