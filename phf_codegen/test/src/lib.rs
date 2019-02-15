extern crate phf;
extern crate unicase;

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
    fn ordered_map() {
        assert_eq!("a", ORDERED_MAP[&1]);
        assert_eq!("b", ORDERED_MAP[&2]);
        assert_eq!("c", ORDERED_MAP[&3]);
        assert!(!ORDERED_MAP.contains_key(&100));
        assert_eq!(&["a", "b", "c"][..], &ORDERED_MAP.values().cloned().collect::<Vec<_>>()[..]);
    }

    #[test]
    fn ordered_set() {
        assert!(ORDERED_SET.contains(&1));
        assert!(ORDERED_SET.contains(&2));
        assert!(ORDERED_SET.contains(&3));
        assert!(!ORDERED_SET.contains(&4));
        assert_eq!(&[1, 2, 3][..], &ORDERED_SET.iter().cloned().collect::<Vec<_>>()[..]);
    }

    #[test]
    fn str_keys() {
        assert_eq!(1, STR_KEYS["a"]);
        assert_eq!(2, STR_KEYS["b"]);
        assert_eq!(3, STR_KEYS["c"]);
    }

    #[test]
    fn unicase_map() {
        assert_eq!("a", UNICASE_MAP[&UniCase("AbC")]);
        assert_eq!("a", UNICASE_MAP[&UniCase("abc")]);
        assert_eq!("b", UNICASE_MAP[&UniCase("DEf")]);
        assert!(!UNICASE_MAP.contains_key(&UniCase("XyZ")));
    }

    #[test]
    fn formatted_map() {
        assert_eq!(FORMATTED_MAP["a"], 1);
        assert_eq!(FORMATTED_MAP["b"], 2);
        assert_eq!(FORMATTED_MAP["c"], 3);
        assert!(!FORMATTED_MAP.contains_key("d"));
        assert!(!FORMATTED_MAP.contains_key("A"));
    }

    #[test]
    fn formatted_set() {
        assert!(FORMATTED_SET.contains("a"));
        assert!(FORMATTED_SET.contains("b"));
        assert!(FORMATTED_SET.contains("c"));
        assert!(!FORMATTED_SET.contains("d"));
        assert!(!FORMATTED_SET.contains("A"));
    }

    #[test]
    fn formatted_ordered_map() {
        assert_eq!(FORMATTED_ORDERED_MAP["a"], 1);
        assert_eq!(FORMATTED_ORDERED_MAP["b"], 2);
        assert_eq!(FORMATTED_ORDERED_MAP["c"], 3);
        assert!(!FORMATTED_ORDERED_MAP.contains_key("d"));
        assert!(!FORMATTED_ORDERED_MAP.contains_key("A"));
        assert_eq!(&FORMATTED_ORDERED_MAP.keys().cloned().collect::<Vec<_>>(), &["a", "b", "c"]);
        assert_eq!(&FORMATTED_ORDERED_MAP.values().cloned().collect::<Vec<_>>(), &[1, 2, 3]);
    }

    #[test]
    fn formatted_ordered_set() {
        assert!(FORMATTED_ORDERED_SET.contains("a"));
        assert!(FORMATTED_ORDERED_SET.contains("b"));
        assert!(FORMATTED_ORDERED_SET.contains("c"));
        assert!(!FORMATTED_ORDERED_SET.contains("d"));
        assert!(!FORMATTED_ORDERED_SET.contains("A"));
        assert_eq!(&FORMATTED_ORDERED_SET.iter().cloned().collect::<Vec<_>>(), &["a", "b", "c"]);
    }
}
