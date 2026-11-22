use phf::{BiMap, phf_bimap};

#[test]
fn both_directions_share_the_stored_pair() {
    static MAP: BiMap<(u16, &str), (char, i8)> = phf_bimap! {
        #[cfg(any())]
        "disabled" => UNKNOWN,
        (42u16, "green") => ('g', -2i8),
        (10, "red") => ('r', 3),
        #[cfg(all())]
        #[cfg(all())]
        (42, "blue") => ('b', 3),
        #[cfg(any())]
        (42, "green") => ('r', 3),
    };
    const LEN: usize = MAP.len();

    assert_eq!(LEN, 3);
    assert!(!MAP.is_empty());
    let color = String::from("red");
    let key = (10, color.as_str());
    assert_eq!(MAP.get_by_left(&key), Some(&('r', 3)));
    assert_eq!(MAP.get_by_right(&('r', 3)), Some(&(10, "red")));
    assert!(MAP.contains_left(&key));
    assert!(MAP.contains_right(&('r', 3)));
    assert_eq!(MAP.get_by_left(&(42, "red")), None);
    assert_eq!(MAP.get_by_right(&('r', -2)), None);

    let expected = [
        (&(42, "green"), &('g', -2)),
        (&(10, "red"), &('r', 3)),
        (&(42, "blue"), &('b', 3)),
    ];
    assert!((&MAP).into_iter().eq(expected));
    assert!(MAP.left_values().eq(expected.map(|(l, _)| l)));
    assert!(MAP.right_values().eq(expected.map(|(_, r)| r)));
    for (l, r) in MAP.entries() {
        let by_left = MAP.get_entry_by_left(l).unwrap();
        let by_right = MAP.get_entry_by_right(r).unwrap();
        assert!(std::ptr::eq(l, by_right.0));
        assert!(std::ptr::eq(r, by_left.1));
    }
}

#[test]
fn equality_includes_definition_order() {
    static MAP: BiMap<char, u8> = phf_bimap! { 'a' => 1u8, 'b' => 2 };
    assert_eq!(MAP, phf_bimap! { 'a' => 1u8, 'b' => 2 });
    assert_ne!(MAP, phf_bimap! { 'a' => 2u8, 'b' => 1 });
    assert_ne!(MAP, phf_bimap! { 'b' => 2u8, 'a' => 1 });
}

#[test]
fn same_value_can_appear_on_opposite_sides() {
    static MAP: BiMap<&str, &str> = phf_bimap! { "a" => "b", "b" => "c" };
    assert_eq!(MAP.get_by_left("b"), Some(&"c"));
    assert_eq!(MAP.get_by_right("b"), Some(&"a"));
    assert!(!MAP.contains_left("c"));
    assert!(!MAP.contains_right("a"));
    assert_eq!(serde_json::to_string(&MAP).unwrap(), r#"{"a":"b","b":"c"}"#);
}

#[test]
fn empty() {
    static MAP: BiMap<&str, u8> = phf_bimap! {};
    for map in [&MAP, &BiMap::new(), &BiMap::default()] {
        assert_eq!(map, &MAP);
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        assert_eq!(map.get_by_left("missing"), None);
        assert_eq!(map.get_by_right(&0), None);
        assert_eq!(map.entries().len(), 0);
    }
}
