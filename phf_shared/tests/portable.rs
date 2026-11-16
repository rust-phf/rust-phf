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
        (0, (0x20423589, 0x77114532, 0x4b732b26)),
        (1, (0xc5c7925c, 0xd8a1c45e, 0xe49a8026)),
        (2, (0xadd87c47, 0x6bfb024d, 0xddd5d0e5)),
        (3, (0x6cd66fae, 0xa82ef676, 0x0658ca7b)),
        (4, (0x818160a3, 0x945f4598, 0xabb0b775)),
        (7, (0xfad082c2, 0x83ac469b, 0xfb67abcd)),
        (8, (0x8e9161f3, 0x73fbfa44, 0x002e4e51)),
        (16, (0xebec81b5, 0x10669275, 0x8ee2a88b)),
        (17, (0xcef4d2d6, 0x74df4cad, 0xc768366c)),
        (32, (0x0e0394c3, 0x4bf47be4, 0xf4f708c7)),
        (33, (0x5e7292ac, 0x3d2053b0, 0x4aad2304)),
        (64, (0x0a26f37d, 0x4796554a, 0x15ef8075)),
        (65, (0x05232317, 0xaa647d04, 0x4e618765)),
        (100, (0xa497e2ca, 0xd3e25380, 0x5a1db700)),
        (200, (0xb20929e9, 0x0ecb1e2b, 0xfde169df)),
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
        (0xc04a21c0, 0x8f6360bf, 0x262680e5),
    );
    check(
        &[0x1234u16, 0x5678, 0x9abc, 0xdef0, 0x0fed][..],
        (0x0c909868, 0x4af0d98d, 0x0296220a),
    );
}
