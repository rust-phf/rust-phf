#[cfg(test)]
mod test {
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
}
