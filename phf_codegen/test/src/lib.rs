#[cfg(test)]
mod test {
    use uncased::UncasedStr;
    use unicase::UniCase;

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
}
