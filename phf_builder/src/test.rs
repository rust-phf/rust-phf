use {Map, Set, OrderedMap, OrderedSet};

#[test]
fn map() {
    let mut builder = Map::new();
    builder.entry(1, "a").entry(2, "b").entry(3, "c");
    let map = builder.build();
    assert_eq!("a", map[&1]);
    assert_eq!("b", map[&2]);
    assert_eq!("c", map[&3]);
    assert!(!map.contains_key(&100));
}

#[test]
fn set() {
    let mut builder = Set::new();
    builder.entry(1).entry(2).entry(3);
    let set = builder.build();
    assert!(set.contains(&1));
    assert!(set.contains(&2));
    assert!(set.contains(&3));
    assert!(!set.contains(&4));
}

#[test]
fn ordered_map() {
    let mut builder = OrderedMap::new();
    builder.entry(1, "a").entry(2, "b").entry(3, "c");
    let map = builder.build();
    assert_eq!("a", map[&1]);
    assert_eq!("b", map[&2]);
    assert_eq!("c", map[&3]);
    assert!(!map.contains_key(&100));
    assert_eq!(&["a", "b", "c"][..], &map.values().cloned().collect::<Vec<_>>()[..]);
}

#[test]
fn ordered_set() {
    let mut builder = OrderedSet::new();
    builder.entry(1).entry(2).entry(3);
    let set = builder.build();
    assert!(set.contains(&1));
    assert!(set.contains(&2));
    assert!(set.contains(&3));
    assert!(!set.contains(&4));
    assert_eq!(&[1, 2, 3][..], &set.iter().cloned().collect::<Vec<_>>()[..]);
}
