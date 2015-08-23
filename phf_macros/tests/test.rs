#![feature(plugin)]
#![plugin(phf_macros)]

extern crate phf;
extern crate phf_shared;

mod map {
    use std::collections::{HashMap, HashSet};
    use phf;

    #[allow(dead_code)]
    static TRAILING_COMMA: phf::Map<&'static str, isize> = phf_map!(
        "foo" => 10,
    );

    #[allow(dead_code)]
    static NO_TRAILING_COMMA: phf::Map<&'static str, isize> = phf_map!(
        "foo" => 10
    );

    #[test]
    fn test_two() {
        static MAP: phf::Map<&'static str, isize> = phf_map!(
            "foo" => 10,
            "bar" => 11,
        );
        assert!(Some(&10) == MAP.get(&("foo")));
        assert!(Some(&11) == MAP.get(&("bar")));
        assert_eq!(None, MAP.get(&("asdf")));
        assert_eq!(2, MAP.len());
    }

    #[test]
    fn test_entries() {
        static MAP: phf::Map<&'static str, isize> = phf_map!(
            "foo" => 10,
            "bar" => 11,
        );
        let hash = MAP.entries().map(|(&k, &v)| (k, v)).collect::<HashMap<_, isize>>();
        assert!(Some(&10) == hash.get(&("foo")));
        assert!(Some(&11) == hash.get(&("bar")));
        assert_eq!(2, hash.len());
    }

    #[test]
    fn test_keys() {
        static MAP: phf::Map<&'static str, isize> = phf_map!(
            "foo" => 10,
            "bar" => 11,
        );
        let hash = MAP.keys().map(|&e| e).collect::<HashSet<_>>();
        assert!(hash.contains(&("foo")));
        assert!(hash.contains(&("bar")));
        assert_eq!(2, hash.len());
    }

    #[test]
    fn test_values() {
        static MAP: phf::Map<&'static str, isize> = phf_map!(
            "foo" => 10,
            "bar" => 11,
        );
        let hash = MAP.values().map(|&e| e).collect::<HashSet<isize>>();
        assert!(hash.contains(&10));
        assert!(hash.contains(&11));
        assert_eq!(2, hash.len());
    }

    #[test]
    fn test_large() {
        static MAP: phf::Map<&'static str, isize> = phf_map!(
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
        assert!(MAP.get(&("a")) == Some(&0));
    }

    #[test]
    fn test_macro_key() {
        static MAP: phf::Map<&'static str, isize> = phf_map!(
            concat!("foo", "bar") => 1
        );
        assert!(Some(&1) == MAP.get(&("foobar")));
    }

    #[test]
    fn test_non_static_str_key() {
        static MAP: phf::Map<&'static str, isize> = phf_map!(
            "a" => 0,
        );
        assert_eq!(Some(&0), MAP.get(&*"a".to_string()));
    }

    #[test]
    fn test_index_ok() {
        static MAP: phf::Map<&'static str, isize> = phf_map!(
            "a" => 0,
        );
        assert_eq!(0, MAP["a"]);
    }

    #[test]
    #[should_panic]
    fn test_index_fail() {
        static MAP: phf::Map<&'static str, isize> = phf_map!(
            "a" => 0,
        );
        MAP["b"];
    }

    macro_rules! test_key_type(
        ($t:ty, $($k:expr => $v:expr),+) => ({
            static MAP: phf::Map<$t, isize> = phf_map! {
                $($k => $v),+
            };
            $(
                assert_eq!(Some(&$v), MAP.get(&$k));
            )+
        })
    );

    #[test]
    fn test_array_vals() {
        static MAP: phf::Map<&'static str, [u8; 3]> = phf_map!(
            "a" => [0u8, 1, 2],
        );
        assert_eq!(Some(&[0u8, 1, 2]), MAP.get(&("a")));
    }

    #[test]
    fn test_array_keys() {
        static MAP: phf::Map<[u8; 2], isize> = phf_map!(
            [0u8, 1] => 0,
            [2, 3u8] => 1,
            [4, 5] => 2,
        );
        assert_eq!(Some(&0), MAP.get(&[0u8, 1u8]));
    }

    #[test]
    fn test_byte_keys() {
        test_key_type!(u8, b'a' => 0, b'b' => 1);
    }

    #[test]
    fn test_char_keys() {
        test_key_type!(char, 'a' => 0, 'b' => 1);
    }

    #[test]
    fn test_i8_keys() {
        test_key_type!(i8, 0i8 => 0, 1i8 => 1);
    }

    #[test]
    fn test_i16_keys() {
        test_key_type!(i16, 0i16 => 0, 1i16 => 1);
    }

    #[test]
    fn test_i32_keys() {
        test_key_type!(i32, 0i32 => 0, 1i32 => 1);
    }

    #[test]
    fn test_i64_keys() {
        test_key_type!(i64, 0i64 => 0, 1i64 => 1);
    }

    #[test]
    fn test_u8_keys() {
        test_key_type!(u8, 0u8 => 0, 1u8 => 1);
    }

    #[test]
    fn test_u16_keys() {
        test_key_type!(u16, 0u16 => 0, 1u16 => 1);
    }

    #[test]
    fn test_u32_keys() {
        test_key_type!(u32, 0u32 => 0, 1u32 => 1);
    }

    #[test]
    fn test_u64_keys() {
        test_key_type!(u64, 0u64 => 0, 1u64 => 1);
    }

    #[test]
    fn test_bool_keys() {
        test_key_type!(bool, false => 0, true => 1);
    }

    #[test]
    fn test_into_iterator() {
        static MAP: phf::Map<&'static str, isize> = phf_map!(
            "foo" => 10,
        );

        for (k, v) in &MAP {
            assert_eq!(&"foo", k);
            assert_eq!(&10, v)
        }
    }
}

mod set {
    use std::collections::HashSet;
    use phf;

    #[allow(dead_code)]
    static TRAILING_COMMA: phf::Set<&'static str> = phf_set! {
        "foo",
    };

    #[allow(dead_code)]
    static NO_TRAILING_COMMA: phf::Set<&'static str> = phf_set! {
        "foo"
    };

    #[test]
    fn test_two() {
        static SET: phf::Set<&'static str> = phf_set! {
            "hello",
            "world",
        };
        assert!(SET.contains(&"hello"));
        assert!(SET.contains(&"world"));
        assert!(!SET.contains(&"foo"));
        assert_eq!(2, SET.len());
    }

    #[test]
    fn test_iter() {
        static SET: phf::Set<&'static str> = phf_set! {
            "hello",
            "world",
        };
        let set = SET.iter().map(|e| *e).collect::<HashSet<_>>();
        assert!(set.contains(&"hello"));
        assert!(set.contains(&"world"));
        assert_eq!(2, set.len());
    }

    #[test]
    fn test_non_static_str_contains() {
        static SET: phf::Set<&'static str> = phf_set! {
            "hello",
            "world",
        };
        assert!(SET.contains(&*"hello".to_string()));
    }

    #[test]
    fn test_into_iterator() {
        static SET: phf::Set<&'static str> = phf_set! {
            "hello",
        };

        for e in &SET {
            assert_eq!(&"hello", e);
        }
    }
}

mod ordered_map {
    use phf;

    #[allow(dead_code)]
    static TRAILING_COMMA: phf::OrderedMap<&'static str, isize> = phf_ordered_map!(
        "foo" => 10,
    );

    #[allow(dead_code)]
    static NO_TRAILING_COMMA: phf::OrderedMap<&'static str, isize> = phf_ordered_map!(
        "foo" => 10
    );

    #[test]
    fn test_two() {
        static MAP: phf::OrderedMap<&'static str, isize> = phf_ordered_map!(
            "foo" => 10,
            "bar" => 11,
        );
        assert!(Some(&10) == MAP.get(&"foo"));
        assert!(Some(&11) == MAP.get(&"bar"));
        assert_eq!(None, MAP.get(&"asdf"));
        assert_eq!(2, MAP.len());
    }

    #[test]
    fn test_get_index() {
        static MAP: phf::OrderedMap<&'static str, isize> = phf_ordered_map!(
            "foo" => 5,
            "bar" => 5,
            "baz" => 5,
        );
        assert_eq!(Some(0), MAP.get_index(&"foo"));
        assert_eq!(Some(2), MAP.get_index(&"baz"));
        assert_eq!(None, MAP.get_index(&"xyz"));

        assert_eq!(Some(0), MAP.get_index(&*"foo".to_string()));
        assert_eq!(Some(2), MAP.get_index(&*"baz".to_string()));
        assert_eq!(None, MAP.get_index(&*"xyz".to_string()));
    }

    #[test]
    fn test_index() {
        static MAP: phf::OrderedMap<&'static str, isize> = phf_ordered_map!(
            "foo" => 5,
            "bar" => 6,
        );
        assert_eq!(Some((&"foo", &5)), MAP.index(0));
        assert_eq!(Some((&"bar", &6)), MAP.index(1));
        assert_eq!(None, MAP.index(2));
    }

    #[test]
    fn test_entries() {
        static MAP: phf::OrderedMap<&'static str, i32> = phf_ordered_map!(
            "foo" => 10,
            "bar" => 11,
            "baz" => 12,
        );
        let vec = MAP.entries().map(|(&k, &v)| (k, v)).collect::<Vec<_>>();
        assert_eq!(vec, vec!(("foo", 10), ("bar", 11), ("baz", 12)));
    }

    #[test]
    fn test_keys() {
        static MAP: phf::OrderedMap<&'static str, isize> = phf_ordered_map!(
            "foo" => 10,
            "bar" => 11,
            "baz" => 12,
        );
        let vec = MAP.keys().map(|&e| e).collect::<Vec<_>>();
        assert_eq!(vec, vec!("foo", "bar", "baz"));
    }

    #[test]
    fn test_values() {
        static MAP: phf::OrderedMap<&'static str, i32> = phf_ordered_map!(
            "foo" => 10,
            "bar" => 11,
            "baz" => 12,
        );
        let vec = MAP.values().map(|&v| v).collect::<Vec<_>>();
        assert_eq!(vec, vec!(10, 11, 12));
    }

    #[test]
    fn test_index_ok() {
        static MAP: phf::OrderedMap<&'static str, isize> = phf_ordered_map!(
            "a" => 0,
        );
        assert_eq!(0, MAP["a"]);
    }

    #[test]
    #[should_panic]
    fn test_index_fail() {
        static MAP: phf::OrderedMap<&'static str, isize> = phf_ordered_map!(
            "a" => 0,
        );
        MAP["b"];
    }

    #[test]
    fn test_non_static_str_key() {
        static MAP: phf::OrderedMap<&'static str, isize> = phf_ordered_map!(
            "a" => 0,
        );
        assert_eq!(Some(&0), MAP.get(&*"a".to_string()));
    }

    #[test]
    fn test_into_iterator() {
        static MAP: phf::OrderedMap<&'static str, isize> = phf_ordered_map!(
            "foo" => 10,
        );

        for (k, v) in &MAP {
            assert_eq!(&"foo", k);
            assert_eq!(&10, v)
        }
    }
}

mod ordered_set {
    use phf;

    #[allow(dead_code)]
    static TRAILING_COMMA: phf::OrderedSet<&'static str> = phf_ordered_set! {
        "foo",
    };

    #[allow(dead_code)]
    static NO_TRAILING_COMMA: phf::OrderedSet<&'static str> = phf_ordered_set! {
        "foo"
    };

    #[test]
    fn test_two() {
        static SET: phf::OrderedSet<&'static str> = phf_ordered_set! {
            "hello",
            "there",
            "world",
        };
        assert!(SET.contains(&"hello"));
        assert!(SET.contains(&"there"));
        assert!(SET.contains(&"world"));
        assert!(!SET.contains(&"foo"));
        assert_eq!(3, SET.len());
    }

    #[test]
    fn test_get_index() {
        static SET: phf::OrderedSet<&'static str> = phf_ordered_set! {
            "foo",
            "bar",
            "baz",
        };
        assert_eq!(Some(0), SET.get_index(&"foo"));
        assert_eq!(Some(2), SET.get_index(&"baz"));
        assert_eq!(None, SET.get_index(&"xyz"));

        assert_eq!(Some(0), SET.get_index(&*"foo".to_string()));
        assert_eq!(Some(2), SET.get_index(&*"baz".to_string()));
        assert_eq!(None, SET.get_index(&*"xyz".to_string()));
    }

    #[test]
    fn test_index() {
        static MAP: phf::OrderedSet<&'static str> = phf_ordered_set!(
            "foo",
            "bar",
        );
        assert_eq!(Some(&"foo"), MAP.index(0));
        assert_eq!(Some(&"bar"), MAP.index(1));
        assert_eq!(None, MAP.index(2));
    }

    #[test]
    fn test_iter() {
        static SET: phf::OrderedSet<&'static str> = phf_ordered_set! {
            "hello",
            "there",
            "world",
        };
        let vec = SET.iter().map(|&e| e).collect::<Vec<_>>();
        assert_eq!(vec, vec!("hello", "there", "world"));
    }

    #[test]
    fn test_non_static_str_contains() {
        static SET: phf::OrderedSet<&'static str> = phf_ordered_set! {
            "hello",
            "world",
        };
        assert!(SET.contains(&*"hello".to_string()));
    }

    #[test]
    fn test_into_iterator() {
        static SET: phf::OrderedSet<&'static str> = phf_ordered_set!(
            "foo",
        );

        for e in &SET {
            assert_eq!(&"foo", e);
        }
    }
}

mod match_ {
    #[test]
    fn test_zero() {
        assert_eq!(phf_match!("foo" { _ => 0 }), 0);
        assert_eq!(phf_match!("bar" { _ => 0 }), 0);

        assert_eq!(phf_match!("foo" { _ => 0, }), 0);
        assert_eq!(phf_match!("foo" { _ => { 0 } }), 0);
        assert_eq!(phf_match!("foo" { _ => { 0 }, }), 0);
    }

    #[test]
    fn test_one() {
        assert_eq!(phf_match!("foo" { "foo" => 0, _ => 1 }), 0);
        assert_eq!(phf_match!("bar" { "foo" => 0, _ => 1 }), 1);
    }

    #[test]
    fn test_two() {
        assert_eq!(phf_match!("foo" { "foo" => 0, "bar" => 1, _ => 2 }), 0);
        assert_eq!(phf_match!("bar" { "foo" => 0, "bar" => 1, _ => 2 }), 1);
        assert_eq!(phf_match!("baz" { "foo" => 0, "bar" => 1, _ => 2 }), 2);
    }

    #[test]
    fn test_three() {
        assert_eq!(phf_match!("foo" { "foo" => 0, "bar" => 1, "baz" => 2, _ => 3 }), 0);
        assert_eq!(phf_match!("bar" { "foo" => 0, "bar" => 1, "baz" => 2, _ => 3 }), 1);
        assert_eq!(phf_match!("baz" { "foo" => 0, "bar" => 1, "baz" => 2, _ => 3 }), 2);
        assert_eq!(phf_match!("boo" { "foo" => 0, "bar" => 1, "baz" => 2, _ => 3 }), 3);
    }
}
