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
    fn ordered_map() {
        assert_eq!(1, ORDERED_MAP["a"]);
        assert_eq!(2, ORDERED_MAP["b"]);
        assert_eq!(3, ORDERED_MAP["c❤️"]);
        assert!(!ORDERED_MAP.contains_key("d"));
        assert_eq!(&[1, 2, 3][..], &ORDERED_MAP.values().cloned().collect::<Vec<_>>()[..]);
    }

    #[test]
    fn ordered_set() {
        assert!(ORDERED_SET.contains("a"));
        assert!(ORDERED_SET.contains("b"));
        assert!(ORDERED_SET.contains("c❤️"));
        assert!(!ORDERED_SET.contains("d"));
        assert_eq!(&["a", "b", "c❤️"][..], &ORDERED_SET.iter().cloned().collect::<Vec<_>>()[..]);
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
