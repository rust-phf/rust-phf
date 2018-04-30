extern crate phf;

#[cfg(test)]
mod test {
    include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

    #[test]
    fn map() {
        assert_eq!(1, MAP["a"]);
        assert_eq!(2, MAP["b"]);
        assert_eq!(3, MAP["c❤️"]);
        assert!(!MAP.contains_key("d"));
    }

    #[test]
    fn set() {
        assert!(SET.contains("a"));
        assert!(SET.contains("b"));
        assert!(SET.contains("c❤️"));
        assert!(!SET.contains("d"));
    }

    #[test]
    fn slice_keys() {
        assert_eq!(1, SLICE_KEYS[&b"a"[..]]);
        assert_eq!(2, SLICE_KEYS[&b"b"[..]]);
        assert_eq!(3, SLICE_KEYS[&b"c\xff"[..]]);
    }

    #[test]
    fn empty_map() {
        // Check we not crash on empty map.
        assert_eq!(None, EMPTY.get(""));
    }
}
