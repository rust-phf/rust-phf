#![feature(phase)]

extern crate collections;
#[phase(syntax)]
extern crate phf_mac;
extern crate phf;

mod map {
    use collections::{HashMap, HashSet};
    use phf::PhfMap;

    #[allow(dead_code)]
    static TRAILING_COMMA: PhfMap<int> = phf_map!(
        "foo" => 10,
    );

    #[allow(dead_code)]
    static NO_TRAILING_COMMA: PhfMap<int> = phf_map!(
        "foo" => 10
    );

    #[test]
    fn test_two() {
        static map: PhfMap<int> = phf_map!(
            "foo" => 10,
            "bar" => 11,
        );
        assert!(Some(&10) == map.find(& &"foo"));
        assert!(Some(&11) == map.find(& &"bar"));
        assert_eq!(None, map.find(& &"asdf"));
        assert_eq!(2, map.len());
    }

    #[test]
    fn test_entries() {
        static map: PhfMap<int> = phf_map!(
            "foo" => 10,
            "bar" => 11,
        );
        let mut hash = HashMap::new();
        for (key, &value) in map.entries() {
            hash.insert(key, value);
        }
        assert!(Some(&10) == hash.find(& &"foo"));
        assert!(Some(&11) == hash.find(& &"bar"));
        assert_eq!(2, hash.len());
    }

    #[test]
    fn test_keys() {
        static map: PhfMap<int> = phf_map!(
            "foo" => 10,
            "bar" => 11,
        );
        let mut hash = HashSet::new();
        for key in map.keys() {
            hash.insert(key);
        }
        assert!(hash.contains(& &"foo"));
        assert!(hash.contains(& &"bar"));
        assert_eq!(2, hash.len());
    }

    #[test]
    fn test_values() {
        static map: PhfMap<int> = phf_map!(
            "foo" => 10,
            "bar" => 11,
        );
        let mut hash = HashSet::new();
        for &value in map.values() {
            hash.insert(value);
        }
        assert!(hash.contains(&10));
        assert!(hash.contains(&11));
        assert_eq!(2, hash.len());
    }

    #[test]
    fn test_large() {
        static map: PhfMap<int> = phf_map!(
            "a" => 0,
            "b" => 1,
            "c" => 2,
            "d" => 3,
            "e" => 4,
            "f" => 5,
            "g" => 6,
            "h" => 7,
            "i" => 8,
            "j" => 9,
            "k" => 10,
            "l" => 11,
            "m" => 12,
            "n" => 13,
            "o" => 14,
            "p" => 15,
            "q" => 16,
            "r" => 17,
            "s" => 18,
            "t" => 19,
            "u" => 20,
            "v" => 21,
            "w" => 22,
            "x" => 23,
            "y" => 24,
            "z" => 25,
        );
        assert!(map.find(& &"a") == Some(&0));
    }

    #[test]
    fn test_macro_key() {
        static map: PhfMap<int> = phf_map!(
            concat!("foo", "bar") => 1
        );
        assert!(Some(&1) == map.find(& &"foobar"));
    }
}

mod set {
    use collections::HashSet;
    use phf::{PhfMap, PhfSet};

    #[allow(dead_code)]
    static TRAILING_COMMA: PhfSet = phf_set! {
        "foo",
    };

    #[allow(dead_code)]
    static NO_TRAILING_COMMA: PhfSet = phf_set! {
        "foo"
    };

    #[test]
    fn test_two() {
        static SET: PhfSet = phf_set! {
            "hello",
            "world",
        };
        assert!(SET.contains(& &"hello"));
        assert!(SET.contains(& &"world"));
        assert!(!SET.contains(& &"foo"));
        assert_eq!(2, SET.len());
    }

    #[test]
    fn test_iter() {
        static SET: PhfSet = phf_set! {
            "hello",
            "world",
        };
        let set = SET.iter().collect::<HashSet<_>>();
        assert!(set.contains(& &"hello"));
        assert!(set.contains(& &"world"));
        assert_eq!(2, set.len());
    }
}

mod ordered_map {
    use phf::PhfOrderedMap;

    #[allow(dead_code)]
    static TRAILING_COMMA: PhfOrderedMap<int> = phf_ordered_map!(
        "foo" => 10,
    );

    #[allow(dead_code)]
    static NO_TRAILING_COMMA: PhfOrderedMap<int> = phf_ordered_map!(
        "foo" => 10
    );

    #[test]
    fn test_two() {
        static map: PhfOrderedMap<int> = phf_ordered_map!(
            "foo" => 10,
            "bar" => 11,
        );
        assert!(Some(&10) == map.find(& &"foo"));
        assert!(Some(&11) == map.find(& &"bar"));
        assert_eq!(None, map.find(& &"asdf"));
        assert_eq!(2, map.len());
    }

    #[test]
    fn test_entries() {
        static MAP: PhfOrderedMap<int> = phf_ordered_map!(
            "foo" => 10,
            "bar" => 11,
            "baz" => 12,
        );
        let vec = MAP.entries().map(|(k, &v)| (k, v)).collect::<Vec<_>>();
        assert_eq!(vec, vec!(("foo", 10), ("bar", 11), ("baz", 12)));
    }

    #[test]
    fn test_keys() {
        static MAP: PhfOrderedMap<int> = phf_ordered_map!(
            "foo" => 10,
            "bar" => 11,
            "baz" => 12,
        );
        let vec = MAP.keys().collect::<Vec<_>>();
        assert_eq!(vec, vec!("foo", "bar", "baz"));
    }

    #[test]
    fn test_values() {
        static MAP: PhfOrderedMap<int> = phf_ordered_map!(
            "foo" => 10,
            "bar" => 11,
            "baz" => 12,
        );
        let vec = MAP.values().map(|&v| v).collect::<Vec<_>>();
        assert_eq!(vec, vec!(10, 11, 12));
    }
}

mod ordered_set {
    use phf::{PhfOrderedMap, PhfOrderedSet};

    #[allow(dead_code)]
    static TRAILING_COMMA: PhfOrderedSet = phf_ordered_set! {
        "foo",
    };

    #[allow(dead_code)]
    static NO_TRAILING_COMMA: PhfOrderedSet = phf_ordered_set! {
        "foo"
    };

    #[test]
    fn test_two() {
        static SET: PhfOrderedSet = phf_ordered_set! {
            "hello",
            "there",
            "world",
        };
        assert!(SET.contains(& &"hello"));
        assert!(SET.contains(& &"there"));
        assert!(SET.contains(& &"world"));
        assert!(!SET.contains(& &"foo"));
        assert_eq!(3, SET.len());
    }

    #[test]
    fn test_iter() {
        static SET: PhfOrderedSet = phf_ordered_set! {
            "hello",
            "there",
            "world",
        };
        let vec = SET.iter().collect::<Vec<_>>();
        assert_eq!(vec, vec!("hello", "there", "world"));
    }
}

