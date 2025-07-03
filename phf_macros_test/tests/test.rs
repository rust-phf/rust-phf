mod map {
    use phf::phf_map;
    use std::collections::{HashMap, HashSet};

    #[allow(dead_code)]
    static TRAILING_COMMA: phf::Map<&'static str, isize> = phf_map!(
        "foo" => 10,
    );

    #[allow(dead_code)]
    static NO_TRAILING_COMMA: phf::Map<&'static str, isize> = phf_map!(
        "foo" => 10
    );

    #[allow(dead_code)]
    static BYTE_STRING_KEY: phf::Map<&'static [u8], &'static str> = phf_map!(
        b"camembert" => "delicious",
    );

    #[allow(dead_code)]
    static DEREF_BYTE_STRING_KEY: phf::Map<[u8; 9], &'static str> = phf_map!(
        *b"camembert" => "delicious",
    );

    #[test]
    fn test_two() {
        static MAP: phf::Map<&'static str, isize> = phf_map!(
            "foo" => 10,
            "bar" => 11,
        );
        assert!(Some(&10) == MAP.get("foo"));
        assert!(Some(&11) == MAP.get("bar"));
        assert_eq!(None, MAP.get("asdf"));
        assert_eq!(2, MAP.len());
    }

    #[test]
    fn test_entries() {
        static MAP: phf::Map<&'static str, isize> = phf_map!(
            "foo" => 10,
            "bar" => 11,
        );
        let hash = MAP
            .entries()
            .map(|(&k, &v)| (k, v))
            .collect::<HashMap<_, isize>>();
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
        let hash = MAP.keys().copied().collect::<HashSet<_>>();
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
        let hash = MAP.values().copied().collect::<HashSet<isize>>();
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
        assert!(MAP.get("a") == Some(&0));
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
        let _ = MAP["b"];
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
        assert_eq!(Some(&[0u8, 1, 2]), MAP.get("a"));
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
        test_key_type!(i8, 0i8 => 0, 1i8 => 1, 127i8 => 2, -128i8 => 3);
    }

    #[test]
    fn test_i16_keys() {
        test_key_type!(i16, 0i16 => 0, 1i16 => 1, 32767i16 => 2, -32768i16 => 3);
    }

    #[test]
    fn test_i32_keys() {
        test_key_type!(i32, 0i32 => 0, 1i32 => 1, 2147483647i32 => 2, -2147483648i32 => 3);
    }

    #[test]
    fn test_i64_keys() {
        test_key_type!(i64, 0i64 => 0, 1i64 => 1, -9223372036854775808i64 => 2);
    }

    #[test]
    fn test_i128_keys() {
        test_key_type!(
            i128, 0i128 => 0, 1i128 => 1,
            // `syn` handles literals larger than 64-bit differently
            170141183460469231731687303715884105727i128 => 2,
            -170141183460469231731687303715884105727i128 => 3
        );
    }

    #[test]
    fn test_isize_keys() {
        if cfg!(target_pointer_width = "16") {
            test_key_type!(isize, 0isize => 0, 1isize => 1, 32767isize => 2, -32768isize => 3);
        } else if cfg!(target_pointer_width = "32") {
            test_key_type!(isize, 0isize => 0, 1isize => 1, 2147483647isize => 2, -2147483648isize => 3);
        } else if cfg!(target_pointer_width = "64") {
            test_key_type!(
                isize, 0isize => 0, 1isize => 1,
                9223372036854775807isize => 2, -9223372036854775808isize => 3
            );
        } else {
            panic!("target_pointer_width is not 16, 32, or 64")
        }
    }

    #[test]
    fn test_u8_keys() {
        test_key_type!(u8, 0u8 => 0, 1u8 => 1, 255u8 => 2);
    }

    #[test]
    fn test_u16_keys() {
        test_key_type!(u16, 0u16 => 0, 1u16 => 1, 65535u16 => 2);
    }

    #[test]
    fn test_u32_keys() {
        test_key_type!(u32, 0u32 => 0, 1u32 => 1, 4294967295u32 => 2);
    }

    #[test]
    fn test_u64_keys() {
        test_key_type!(u64, 0u64 => 0, 1u64 => 1, 18446744073709551615u64 => 2);
    }

    #[test]
    fn test_u128_keys() {
        test_key_type!(
            u128, 0u128 => 0, 1u128 => 1,
            340282366920938463463374607431768211455u128 => 2
        );
    }

    #[test]
    fn test_usize_keys() {
        if cfg!(target_pointer_width = "16") {
            test_key_type!(usize, 0usize => 0, 1usize => 1, 65535usize => 2);
        } else if cfg!(target_pointer_width = "32") {
            test_key_type!(usize, 0usize => 0, 1usize => 1, 4294967295usize => 2);
        } else if cfg!(target_pointer_width = "64") {
            test_key_type!(usize, 0usize => 0, 1usize => 1, 18446744073709551615usize => 2);
        } else {
            panic!("target_pointer_width is not 16, 32, or 64")
        }
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

    #[test]
    fn test_unicase() {
        use unicase::UniCase;
        static MAP: phf::Map<UniCase<&'static str>, isize> = phf_map!(
            UniCase::ascii("FOO") => 10,
            UniCase::unicode("Bar") => 11,
        );
        assert!(Some(&10) == MAP.get(&UniCase::new("FOo")));
        assert!(Some(&11) == MAP.get(&UniCase::new("bar")));
        assert_eq!(None, MAP.get(&UniCase::new("asdf")));
    }

    #[test]
    fn test_unicase_ascii() {
        use unicase::Ascii;
        static MAP: phf::Map<Ascii<&'static str>, isize> = phf_map!(
            Ascii::new("FOO") => 10,
            Ascii::new("Bar") => 11,
        );
        assert!(Some(&10) == MAP.get(&Ascii::new("FOo")));
        assert!(Some(&11) == MAP.get(&Ascii::new("bar")));
        assert_eq!(None, MAP.get(&Ascii::new("asdf")));
    }

    #[test]
    fn test_uncased() {
        use uncased::UncasedStr;
        static MAP: phf::Map<&'static UncasedStr, isize> = phf_map!(
            UncasedStr::new("FOO") => 10,
            UncasedStr::new("Bar") => 11,
        );
        assert!(Some(&10) == MAP.get("FOo".into()));
        assert!(Some(&11) == MAP.get("bar".into()));
        assert_eq!(None, MAP.get("asdf".into()));
    }

    #[test]
    fn test_cfgs() {
        static MY_MAP: phf::Map<&'static str, u32> = phf_map! {
            "foo" => 1, // should always be present
            #[cfg(feature = "disabled_feature")]
            "bar" => 2, // should not be present as we disable this feature
            #[cfg(feature = "enabled_feature")]
            "baz" => 3, // should be present as we enable this feature
        };
        assert_eq!(Some(&1), MY_MAP.get("foo"));
        #[cfg(feature = "disabled_feature")]
        assert_eq!(Some(&2), MY_MAP.get("bar"));
        #[cfg(not(feature = "disabled_feature"))]
        assert_eq!(None, MY_MAP.get("bar"));
        #[cfg(feature = "enabled_feature")]
        assert_eq!(Some(&3), MY_MAP.get("baz"));
        #[cfg(not(feature = "enabled_feature"))]
        assert_eq!(None, MY_MAP.get("baz"));
    }

    #[test]
    fn test_tuples() {
        static MAP: phf::Map<(u32, &str), u32> = phf_map! {
            (0, "a") => 1,
            (1, "b") => 2,
            (2, "c") => 3,
        };
        assert_eq!(Some(&1), MAP.get(&(0, "a")));
        assert_eq!(Some(&2), MAP.get(&(1, "b")));
        assert_eq!(Some(&3), MAP.get(&(2, "c")));
        assert_eq!(None, MAP.get(&(3, "d")));
    }

    #[test]
    fn test_or_pattern() {
        static MAP: phf::Map<&'static str, isize> = phf_map!(
            "foo" | "baz" => 10,
            "bar" => 20,
        );
        assert_eq!(Some(&10), MAP.get("foo"));
        assert_eq!(Some(&10), MAP.get("baz"));
        assert_eq!(Some(&20), MAP.get("bar"));
        assert_eq!(None, MAP.get("qux"));

        // Test with three or more keys
        static MAP2: phf::Map<&'static str, isize> = phf_map!(
            "foo" | "baz" | "qux" => 10,
            "bar" | "quux" => 20,
            "xyz" => 30,
        );
        assert_eq!(Some(&10), MAP2.get("foo"));
        assert_eq!(Some(&10), MAP2.get("baz"));
        assert_eq!(Some(&10), MAP2.get("qux"));
        assert_eq!(Some(&20), MAP2.get("bar"));
        assert_eq!(Some(&20), MAP2.get("quux"));
        assert_eq!(Some(&30), MAP2.get("xyz"));
        assert_eq!(None, MAP2.get("unknown"));
    }
}

mod set {
    use phf::phf_set;
    use std::collections::HashSet;

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
        assert!(SET.contains("hello"));
        assert!(SET.contains("world"));
        assert!(!SET.contains("foo"));
        assert_eq!(2, SET.len());
    }

    #[test]
    fn test_iter() {
        static SET: phf::Set<&'static str> = phf_set! {
            "hello",
            "world",
        };
        let set = SET.iter().copied().collect::<HashSet<_>>();
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

    #[test]
    fn test_cfgs() {
        static SET: phf::Set<&'static str> = phf_set! {
            "foo", // should always be present
            #[cfg(feature = "disabled_feature")]
            "bar", // should not be present as we disable this feature
            #[cfg(feature = "enabled_feature")]
            "baz", // should be present as we enable this feature by default
        };
        assert!(SET.contains("foo"));
        #[cfg(feature = "disabled_feature")]
        assert!(SET.contains("bar"));
        #[cfg(not(feature = "disabled_feature"))]
        assert!(!SET.contains("bar"));
        #[cfg(feature = "enabled_feature")]
        assert!(SET.contains("baz"));
        #[cfg(not(feature = "enabled_feature"))]
        assert!(!SET.contains("baz"));
    }

    #[test]
    fn test_tuples() {
        static SET: phf::Set<(u32, &str)> = phf_set! {
            (0, "a"),
            (1, "b"),
            (2, "c"),
        };
        assert!(SET.contains(&(0, "a")));
        assert!(SET.contains(&(1, "b")));
        assert!(SET.contains(&(2, "c")));
        assert!(!SET.contains(&(3, "d")));
    }

    #[test]
    fn test_or_pattern() {
        static SET: phf::Set<&'static str> = phf_set! {
            "foo" | "baz",
            "bar" | "qux",
        };
        assert!(SET.contains("foo"));
        assert!(SET.contains("baz"));
        assert!(SET.contains("bar"));
        assert!(SET.contains("qux"));
        assert!(!SET.contains("unknown"));
        assert_eq!(4, SET.len());

        // Test with three or more keys
        static SET2: phf::Set<&'static str> = phf_set! {
            "foo" | "baz" | "qux",
            "bar" | "quux" | "xyz",
        };
        assert!(SET2.contains("foo"));
        assert!(SET2.contains("baz"));
        assert!(SET2.contains("qux"));
        assert!(SET2.contains("bar"));
        assert!(SET2.contains("quux"));
        assert!(SET2.contains("xyz"));
        assert!(!SET2.contains("unknown"));
        assert_eq!(6, SET2.len());
    }
}

mod ordered_map {
    use phf::phf_ordered_map;

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
        assert!(Some(&10) == MAP.get("foo"));
        assert!(Some(&11) == MAP.get("bar"));
        assert_eq!(None, MAP.get("asdf"));
        assert_eq!(2, MAP.len());
    }

    #[test]
    fn test_get_index() {
        static MAP: phf::OrderedMap<&'static str, isize> = phf_ordered_map!(
            "foo" => 5,
            "bar" => 5,
            "baz" => 5,
        );
        assert_eq!(Some(0), MAP.get_index("foo"));
        assert_eq!(Some(2), MAP.get_index("baz"));
        assert_eq!(None, MAP.get_index("xyz"));

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
        let vec = MAP.keys().copied().collect::<Vec<_>>();
        assert_eq!(vec, vec!("foo", "bar", "baz"));
    }

    #[test]
    fn test_values() {
        static MAP: phf::OrderedMap<&'static str, i32> = phf_ordered_map!(
            "foo" => 10,
            "bar" => 11,
            "baz" => 12,
        );
        let vec = MAP.values().copied().collect::<Vec<_>>();
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
        let _ = MAP["b"];
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

    #[test]
    fn test_cfgs() {
        static MY_MAP: phf::OrderedMap<&'static str, u32> = phf_ordered_map! {
            "foo" => 1, // should always be present
            #[cfg(feature = "disabled_feature")]
            "bar" => 2, // should not be present as we disable this feature
            #[cfg(feature = "enabled_feature")]
            "baz" => 3, // should be present as we enable this feature
        };
        assert_eq!(Some(&1), MY_MAP.get("foo"));
        #[cfg(feature = "disabled_feature")]
        assert_eq!(Some(&2), MY_MAP.get("bar"));
        #[cfg(not(feature = "disabled_feature"))]
        assert_eq!(None, MY_MAP.get("bar"));
        #[cfg(feature = "enabled_feature")]
        assert_eq!(Some(&3), MY_MAP.get("baz"));
        #[cfg(not(feature = "enabled_feature"))]
        assert_eq!(None, MY_MAP.get("baz"));
    }

    #[test]
    fn test_tuples() {
        static MAP: phf::OrderedMap<(u32, &str), u32> = phf_ordered_map! {
            (0, "a") => 1,
            (1, "b") => 2,
            (2, "c") => 3,
        };
        assert_eq!(Some(&1), MAP.get(&(0, "a")));
        assert_eq!(Some(&2), MAP.get(&(1, "b")));
        assert_eq!(Some(&3), MAP.get(&(2, "c")));
        assert_eq!(None, MAP.get(&(3, "d")));
    }

    #[test]
    fn test_or_pattern() {
        static MAP: phf::OrderedMap<&'static str, isize> = phf_ordered_map!(
            "foo" | "baz" => 10,
            "bar" | "qux" => 20,
        );
        assert_eq!(Some(&10), MAP.get("foo"));
        assert_eq!(Some(&10), MAP.get("baz"));
        assert_eq!(Some(&20), MAP.get("bar"));
        assert_eq!(Some(&20), MAP.get("qux"));
        assert_eq!(None, MAP.get("unknown"));
        assert_eq!(4, MAP.len());

        // Test with three or more keys
        static MAP2: phf::OrderedMap<&'static str, isize> = phf_ordered_map!(
            "foo" | "baz" | "qux" => 10,
            "bar" | "quux" | "xyz" => 20,
        );
        assert_eq!(Some(&10), MAP2.get("foo"));
        assert_eq!(Some(&10), MAP2.get("baz"));
        assert_eq!(Some(&10), MAP2.get("qux"));
        assert_eq!(Some(&20), MAP2.get("bar"));
        assert_eq!(Some(&20), MAP2.get("quux"));
        assert_eq!(Some(&20), MAP2.get("xyz"));
        assert_eq!(None, MAP2.get("unknown"));
        assert_eq!(6, MAP2.len());
    }
}

mod ordered_set {
    use phf::phf_ordered_set;

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
        assert!(SET.contains("hello"));
        assert!(SET.contains("there"));
        assert!(SET.contains("world"));
        assert!(!SET.contains("foo"));
        assert_eq!(3, SET.len());
    }

    #[test]
    fn test_get_index() {
        static SET: phf::OrderedSet<&'static str> = phf_ordered_set! {
            "foo",
            "bar",
            "baz",
        };
        assert_eq!(Some(0), SET.get_index("foo"));
        assert_eq!(Some(2), SET.get_index("baz"));
        assert_eq!(None, SET.get_index("xyz"));

        assert_eq!(Some(0), SET.get_index(&*"foo".to_string()));
        assert_eq!(Some(2), SET.get_index(&*"baz".to_string()));
        assert_eq!(None, SET.get_index(&*"xyz".to_string()));
    }

    #[test]
    fn test_index() {
        static MAP: phf::OrderedSet<&'static str> = phf_ordered_set!("foo", "bar",);
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
        let vec = SET.iter().copied().collect::<Vec<_>>();
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
        static SET: phf::OrderedSet<&'static str> = phf_ordered_set!("foo",);

        for e in &SET {
            assert_eq!(&"foo", e);
        }
    }

    #[test]
    fn test_cfgs() {
        static SET: phf::OrderedSet<&'static str> = phf_ordered_set! {
            "foo", // should always be present
            #[cfg(feature = "disabled_feature")]
            "bar", // should not be present as we disable this feature
            #[cfg(feature = "enabled_feature")]
            "baz", // should be present as we enable this feature by default
        };
        assert!(SET.contains("foo"));
        #[cfg(feature = "disabled_feature")]
        assert!(SET.contains("bar"));
        #[cfg(not(feature = "disabled_feature"))]
        assert!(!SET.contains("bar"));
        #[cfg(feature = "enabled_feature")]
        assert!(SET.contains("baz"));
        #[cfg(not(feature = "enabled_feature"))]
        assert!(!SET.contains("baz"));
    }

    #[test]
    fn test_tuples() {
        static SET: phf::OrderedSet<(u32, &str)> = phf_ordered_set! {
            (0, "a"),
            (1, "b"),
            (2, "c"),
        };
        assert!(SET.contains(&(0, "a")));
        assert!(SET.contains(&(1, "b")));
        assert!(SET.contains(&(2, "c")));
        assert!(!SET.contains(&(3, "d")));
    }

    #[test]
    fn test_or_pattern() {
        static SET: phf::OrderedSet<&'static str> = phf_ordered_set! {
            "foo" | "baz",
            "bar" | "qux",
        };
        assert!(SET.contains("foo"));
        assert!(SET.contains("baz"));
        assert!(SET.contains("bar"));
        assert!(SET.contains("qux"));
        assert!(!SET.contains("unknown"));
        assert_eq!(4, SET.len());

        // Test with three or more keys
        static SET2: phf::OrderedSet<&'static str> = phf_ordered_set! {
            "foo" | "baz" | "qux",
            "bar" | "quux" | "xyz",
        };
        assert!(SET2.contains("foo"));
        assert!(SET2.contains("baz"));
        assert!(SET2.contains("qux"));
        assert!(SET2.contains("bar"));
        assert!(SET2.contains("quux"));
        assert!(SET2.contains("xyz"));
        assert!(!SET2.contains("unknown"));
        assert_eq!(6, SET2.len());
    }
}
