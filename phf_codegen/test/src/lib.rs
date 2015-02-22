#![cfg_attr(test, feature(core))]

extern crate phf;

#[cfg(test)]
mod test {
    include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

    #[test]
    fn map() {
        assert_eq!("a", MAP[1]);
        assert_eq!("b", MAP[2]);
        assert_eq!("c", MAP[3]);
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
        assert_eq!("a", ORDERED_MAP[1]);
        assert_eq!("b", ORDERED_MAP[2]);
        assert_eq!("c", ORDERED_MAP[3]);
        assert!(!ORDERED_MAP.contains_key(&100));
        assert_eq!(&["a", "b", "c"][..], ORDERED_MAP.values().cloned().collect::<Vec<_>>());
    }

    #[test]
    fn ordered_set() {
        assert!(ORDERED_SET.contains(&1));
        assert!(ORDERED_SET.contains(&2));
        assert!(ORDERED_SET.contains(&3));
        assert!(!ORDERED_SET.contains(&4));
        assert_eq!(&[1, 2, 3][..], ORDERED_SET.iter().cloned().collect::<Vec<_>>());
    }

    #[test]
    fn str_keys() {
        assert_eq!(1, STR_KEYS["a"]);
        assert_eq!(2, STR_KEYS["b"]);
        assert_eq!(3, STR_KEYS["c"]);
    }
}
