use phf::phf_map;

#[test]
fn wide_integer_keys() {
    static MAP: phf::Map<u128, u8> = phf_map! {
        0xc0ac29b7c97c50dd_0000000000000000u128 => 0,
        0xc0ac29b7c97c50dd_0000000000000001 => 1,
        0x8000000080000001_0000000000000000 => 2,
        0x8000000080000001_0000000000000001 => 3,
    };
    for (key, value) in [
        (0xc0ac29b7c97c50dd_0000000000000000u128, 0),
        (0xc0ac29b7c97c50dd_0000000000000001, 1),
        (0x8000000080000001_0000000000000000, 2),
        (0x8000000080000001_0000000000000001, 3),
    ] {
        assert_eq!(MAP.get(&key), Some(&value));
    }
    assert_eq!(MAP.get(&0), None);
}

#[test]
fn pointer_sized_tuple_keys() {
    static MAP: phf::Map<(isize, usize, &str), u8> = phf_map! {
        (-1isize, 0usize, "short") => 1,
        (-2147483648, 0x12345678, "a string longer than sixteen bytes") => 2,
        (2147483647, 0xffffffff, "\0\u{1f980}") => 3,
    };
    assert_eq!(MAP.get(&(-1, 0, "short")), Some(&1));
    assert_eq!(
        MAP.get(&(
            -2147483648,
            0x12345678,
            "a string longer than sixteen bytes"
        )),
        Some(&2)
    );
    assert_eq!(MAP.get(&(2147483647, 0xffffffff, "\0\u{1f980}")), Some(&3));
    assert_eq!(MAP.get(&(-1, 0, "missing")), None);
}
