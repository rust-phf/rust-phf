//! Tables are generated on the host; the cross-platform CI job runs these
//! lookups on targets with a different pointer width and endianness.

use phf::phf_map;

#[test]
fn wide_integer_keys() {
    static MAP: phf::Map<u128, u8> = phf_map! {
        0u128 => 0,
        1 => 1,
        0x1_0000_0000_0000_0000 => 2,
        0xfedc_ba98_7654_3210_0123_4567_89ab_cdef => 3,
    };
    for (key, value) in [
        (0u128, 0),
        (1, 1),
        (0x1_0000_0000_0000_0000, 2),
        (0xfedc_ba98_7654_3210_0123_4567_89ab_cdef, 3),
    ] {
        assert_eq!(MAP.get(&key), Some(&value));
    }
    assert_eq!(MAP.get(&2), None);
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
