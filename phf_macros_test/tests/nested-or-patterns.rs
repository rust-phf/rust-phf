use phf::{phf_map, phf_ordered_map, phf_ordered_set, phf_set};

#[test]
fn tuple_map() {
    static MAP: phf::Map<(char, &str), u8> = phf_map! {
        ('p', "Point2d" | "Point3d") => 1 | 2,
        ('v' | 's', "Vector2d" | "Vector3d") | ('x', "Other") => 4,
    };

    for name in ["Point2d", "Point3d"] {
        assert_eq!(MAP.get(&('p', name)), Some(&3));
    }
    for prefix in ['v', 's'] {
        for name in ["Vector2d", "Vector3d"] {
            assert_eq!(MAP.get(&(prefix, name)), Some(&4));
        }
    }
    assert_eq!(MAP.get(&('x', "Other")), Some(&4));
    assert_eq!(MAP.get(&('p', "Vector2d")), None);
    assert_eq!(MAP.len(), 7);
}

#[test]
fn nested_tuple_ordered_map() {
    static MAP: phf::OrderedMap<(u8, (&str, bool)), u8> = phf_ordered_map! {
        (1u8 | 2, ("a" | "b", true | false)) => 3,
        (3, ("c", true)) | (4, ("d", false)) => 4,
    };

    let expected = [
        ((1, ("a", true)), 3),
        ((1, ("a", false)), 3),
        ((1, ("b", true)), 3),
        ((1, ("b", false)), 3),
        ((2, ("a", true)), 3),
        ((2, ("a", false)), 3),
        ((2, ("b", true)), 3),
        ((2, ("b", false)), 3),
        ((3, ("c", true)), 4),
        ((4, ("d", false)), 4),
    ];
    assert_eq!(MAP.len(), expected.len());
    for (index, (key, value)) in expected.iter().enumerate() {
        assert_eq!(MAP.index(index), Some((key, value)));
        assert_eq!(MAP.get(key), Some(value));
    }
    assert_eq!(MAP.get(&(3, ("a", true))), None);
}

#[test]
fn array_set() {
    static SET: phf::Set<[u8; 2]> = phf_set! {
        [0u8 | 1, 2 | 3],
        [4, 5],
    };

    for first in [0, 1] {
        for second in [2, 3] {
            assert!(SET.contains(&[first, second]));
        }
    }
    assert!(SET.contains(&[4, 5]));
    assert!(!SET.contains(&[0, 5]));
    assert_eq!(SET.len(), 5);
}

#[test]
fn grouped_ordered_set() {
    macro_rules! make_set {
        ($key:expr) => {
            phf_ordered_set! { $key }
        };
    }

    static SET: phf::OrderedSet<(&str,)> = make_set!(("a" | ("b" | "c"),));

    assert_eq!(
        SET.iter().copied().collect::<Vec<_>>(),
        [("a",), ("b",), ("c",)]
    );
    for key in [("a",), ("b",), ("c",)] {
        assert!(SET.contains(&key));
    }
    assert!(!SET.contains(&("d",)));
}

#[test]
fn wrapped_keys() {
    use uncased::UncasedStr;
    use unicase::{Ascii, UniCase};

    static UNICASE: phf::Set<UniCase<&str>> = phf_set! {
        UniCase::ascii("foo" | "bar"),
        UniCase::unicode("baz" | "qux"),
    };
    for key in ["FOO", "BAR", "BAZ", "QUX"] {
        assert!(UNICASE.contains(&UniCase::new(key)));
    }
    assert_eq!(UNICASE.len(), 4);

    static ASCII: phf::Set<Ascii<&str>> = phf_set! { Ascii::new("foo" | "bar") };
    assert!(ASCII.contains(&Ascii::new("FOO")));
    assert!(ASCII.contains(&Ascii::new("BAR")));
    assert_eq!(ASCII.len(), 2);

    static UNCASED: phf::Set<&UncasedStr> = phf_set! { UncasedStr::new("foo" | "bar") };
    assert!(UNCASED.contains(UncasedStr::new("FOO")));
    assert!(UNCASED.contains(UncasedStr::new("BAR")));
    assert_eq!(UNCASED.len(), 2);

    static SIGNED: phf::Set<i8> = phf_set! { -(1i8 | 2) };
    assert!(SIGNED.contains(&-1));
    assert!(SIGNED.contains(&-2));
    assert_eq!(SIGNED.len(), 2);

    static BYTES: phf::Set<[u8; 2]> = phf_set! { *(b"ab" | b"cd") };
    assert!(BYTES.contains(b"ab"));
    assert!(BYTES.contains(b"cd"));
    assert_eq!(BYTES.len(), 2);
}

#[test]
fn cfg_tuple_keys() {
    static MAP: phf::OrderedMap<(u8, &str), u8> = phf_ordered_map! {
        #[cfg(any())]
        (0, "off" | "disabled") => 0,
        #[cfg(all())]
        (1u8, "on" | "enabled") => 1,
        (2, "last") => 2,
    };

    assert_eq!(MAP.len(), 3);
    assert_eq!(MAP.index(0), Some((&(1, "on"), &1)));
    assert_eq!(MAP.index(1), Some((&(1, "enabled"), &1)));
    assert_eq!(MAP.index(2), Some((&(2, "last"), &2)));
    assert_eq!(MAP.get(&(0, "off")), None);
    assert_eq!(MAP.get(&(0, "disabled")), None);
}
