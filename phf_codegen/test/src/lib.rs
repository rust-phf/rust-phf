#[cfg(test)]
mod test {
    use uncased::UncasedStr;
    use unicase::{Ascii, UniCase};

    include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

    #[test]
    fn map() {
        assert_eq!("a", MAP[&1]);
        assert_eq!("b", MAP[&2]);
        assert_eq!("c", MAP[&3]);
        assert!(!MAP.contains_key(&100));
    }

    #[test]
    fn set() {
        assert!(SET.contains(&1));
        assert!(SET.contains(&2));
        assert!(SET.contains(&3));
        assert!(!SET.contains(&4));
    }

    #[test]
    fn ordered_map() {
        assert_eq!("a", ORDERED_MAP[&1]);
        assert_eq!("b", ORDERED_MAP[&2]);
        assert_eq!("c", ORDERED_MAP[&3]);
        assert!(!ORDERED_MAP.contains_key(&100));
        assert_eq!(
            &["a", "b", "c"][..],
            &ORDERED_MAP.values().cloned().collect::<Vec<_>>()[..]
        );
    }

    #[test]
    fn ordered_set() {
        assert!(ORDERED_SET.contains(&1));
        assert!(ORDERED_SET.contains(&2));
        assert!(ORDERED_SET.contains(&3));
        assert!(!ORDERED_SET.contains(&4));
        assert_eq!(
            &[1, 2, 3][..],
            &ORDERED_SET.iter().cloned().collect::<Vec<_>>()[..]
        );
    }

    #[test]
    fn str_keys() {
        assert_eq!(1, STR_KEYS["a"]);
        assert_eq!(2, STR_KEYS["b"]);
        assert_eq!(3, STR_KEYS["c"]);
    }

    #[test]
    fn unicase_map() {
        assert_eq!("a", UNICASE_MAP[&UniCase::new("AbC")]);
        assert_eq!("a", UNICASE_MAP[&UniCase::new("abc")]);
        assert_eq!("b", UNICASE_MAP[&UniCase::new("DEf")]);
        assert!(!UNICASE_MAP.contains_key(&UniCase::new("XyZ")));

        // allow lookup with non-static slices
        let local_str_1 = "AbC".to_string();
        let local_str_2 = "abc".to_string();
        let local_str_3 = "DEf".to_string();
        assert_eq!("a", UNICASE_MAP[&UniCase::new(&*local_str_1)]);
        assert_eq!("a", UNICASE_MAP[&UniCase::new(&*local_str_2)]);
        assert_eq!("b", UNICASE_MAP[&UniCase::new(&*local_str_3)]);
    }

    #[test]
    fn unicase_ascii_map() {
        assert_eq!("a", UNICASE_ASCII_MAP[&Ascii::new("AbC")]);
        assert_eq!("a", UNICASE_ASCII_MAP[&Ascii::new("abc")]);
        assert_eq!("b", UNICASE_ASCII_MAP[&Ascii::new("DEf")]);
        assert!(!UNICASE_ASCII_MAP.contains_key(&Ascii::new("XyZ")));

        // allow lookup with non-static slices
        let local_str_1 = "AbC".to_string();
        let local_str_2 = "abc".to_string();
        let local_str_3 = "DEf".to_string();
        assert_eq!("a", UNICASE_ASCII_MAP[&Ascii::new(&*local_str_1)]);
        assert_eq!("a", UNICASE_ASCII_MAP[&Ascii::new(&*local_str_2)]);
        assert_eq!("b", UNICASE_ASCII_MAP[&Ascii::new(&*local_str_3)]);
    }

    #[test]
    fn uncased_map() {
        assert_eq!("a", UNCASED_MAP[UncasedStr::new("AbC")]);
        assert_eq!("a", UNCASED_MAP[UncasedStr::new("abc")]);
        assert_eq!("b", UNCASED_MAP[UncasedStr::new("DEf")]);
        assert!(!UNCASED_MAP.contains_key(UncasedStr::new("XyZ")));
    }

    #[test]
    fn array_keys() {
        assert_eq!(0, ARRAY_KEYS[b"foo"]);
        assert_eq!(1, ARRAY_KEYS[b"bar"]);
        assert_eq!(2, ARRAY_KEYS[b"baz"]);
    }

    #[test]
    fn byte_str_keys() {
        // slicing is required unless the key type is fixed-size
        assert_eq!(0, BYTE_STR_KEYS[&b"foo"[..]]);
        assert_eq!(1, BYTE_STR_KEYS[&b"bar"[..]]);
        assert_eq!(2, BYTE_STR_KEYS[&b"baz"[..]]);
        assert_eq!(3, BYTE_STR_KEYS[&b"quux"[..]]);
    }

    #[test]
    fn empty_map() {
        assert_eq!(None, EMPTY.get(&1));
    }

    #[test]
    fn empty_ordered_map() {
        assert_eq!(None, EMPTY_ORDERED.get(&1));
    }

    #[test]
    fn from_iter_map() {
        assert_eq!(1, FROM_ITER_MAP["one"]);
        assert_eq!(2, FROM_ITER_MAP["two"]);
        assert_eq!(3, FROM_ITER_MAP["three"]);
        assert!(!FROM_ITER_MAP.contains_key("four"));
    }

    #[test]
    fn tuple_map() {
        assert_eq!("first", TUPLE_MAP[&(1u32, "a")]);
        assert_eq!("second", TUPLE_MAP[&(2u32, "b")]);
        assert_eq!("third", TUPLE_MAP[&(3u32, "c")]);
        assert!(!TUPLE_MAP.contains_key(&(4u32, "d")));
        assert!(!TUPLE_MAP.contains_key(&(1u32, "b")));
    }

    #[test]
    fn tuple_set() {
        assert!(TUPLE_SET.contains(&(1u32, "x")));
        assert!(TUPLE_SET.contains(&(2u32, "y")));
        assert!(TUPLE_SET.contains(&(3u32, "z")));
        assert!(!TUPLE_SET.contains(&(4u32, "w")));
        assert!(!TUPLE_SET.contains(&(1u32, "y")));
    }

    #[test]
    fn nested_tuple_map() {
        assert_eq!(10, NESTED_TUPLE_MAP[&((1u32, 2u32), "nested")]);
        assert_eq!(20, NESTED_TUPLE_MAP[&((3u32, 4u32), "tuple")]);
        assert_eq!(30, NESTED_TUPLE_MAP[&((5u32, 6u32), "keys")]);
        assert!(!NESTED_TUPLE_MAP.contains_key(&((7u32, 8u32), "missing")));
        assert!(!NESTED_TUPLE_MAP.contains_key(&((1u32, 2u32), "wrong")));
    }

    #[test]
    fn mixed_tuple_map() {
        assert_eq!("value1", MIXED_TUPLE_MAP[&(true, 1u8, "test")]);
        assert_eq!("value2", MIXED_TUPLE_MAP[&(false, 2u8, "demo")]);
        assert_eq!("value3", MIXED_TUPLE_MAP[&(true, 3u8, "example")]);
        assert!(!MIXED_TUPLE_MAP.contains_key(&(false, 1u8, "test")));
        assert!(!MIXED_TUPLE_MAP.contains_key(&(true, 4u8, "missing")));
    }
}
