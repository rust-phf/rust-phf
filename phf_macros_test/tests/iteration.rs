use std::fmt::Debug;
use std::iter::FusedIterator;

use phf::{Map, OrderedMap, OrderedSet, Set, phf_map, phf_ordered_map, phf_ordered_set, phf_set};

fn assert_iteration<I>(iter: I, expected: &[I::Item])
where
    I: Clone + DoubleEndedIterator + ExactSizeIterator + FusedIterator,
    I::Item: Debug + PartialEq,
{
    assert_eq!(iter.len(), expected.len());
    assert_eq!(iter.clone().collect::<Vec<_>>(), expected);
    let reversed = iter.clone().rev().collect::<Vec<_>>();
    assert!(reversed.iter().eq(expected.iter().rev()));

    // Exercise every split between forward and backward consumption, including
    // clones and exact size hints after partially consuming the iterator.
    for split in 0..=expected.len() {
        let mut iter = iter.clone();
        for item in &expected[..split] {
            assert_eq!(iter.next().as_ref(), Some(item));
        }
        for end in (split..=expected.len()).rev() {
            let remaining = end - split;
            assert_eq!(iter.len(), remaining);
            assert_eq!(iter.size_hint(), (remaining, Some(remaining)));
            assert_eq!(iter.clone().collect::<Vec<_>>(), expected[split..end]);
            if end > split {
                assert_eq!(iter.next_back().as_ref(), Some(&expected[end - 1]));
            }
        }
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }
}

#[test]
fn empty() {
    static MAP: Map<&str, u32> = phf_map! {};
    static SET: Set<&str> = phf_set! {};
    for map in [&MAP, &Map::new(), &Map::default()] {
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        assert_eq!(map.get("missing"), None);
        assert_iteration(map.entries(), &[]);
        assert_iteration(map.keys(), &[]);
        assert_iteration(map.values(), &[]);
    }
    assert!(SET.is_empty());
    assert_iteration(SET.iter(), &[]);
}

#[test]
fn singleton() {
    static MAP: Map<&str, u32> = phf_map! { "a" => 1 };
    static SET: Set<&str> = phf_set! { "a" };
    const LEN: usize = MAP.len();

    assert_eq!(LEN, 1);
    assert!(!MAP.is_empty());
    assert_eq!(MAP.get("a"), Some(&1));
    assert_eq!(MAP.get("missing"), None);
    assert_iteration(MAP.entries(), &[(&"a", &1)]);
    assert_iteration(MAP.keys(), &[&"a"]);
    assert_iteration(MAP.values(), &[&1]);
    assert_iteration((&MAP).into_iter(), &[(&"a", &1)]);
    assert_eq!(format!("{MAP:?}"), r#"{"a": 1}"#);

    assert_eq!(SET.len(), 1);
    assert!(SET.contains("a"));
    assert!(!SET.contains("missing"));
    assert_iteration(SET.iter(), &[&"a"]);
    assert_iteration((&SET).into_iter(), &[&"a"]);
    assert_eq!(format!("{SET:?}"), r#"{"a"}"#);
}

#[test]
fn multiple_entries() {
    static MAP: Map<&str, u32> = phf_map! { "a" => 1, "b" => 2, "c" => 3 };
    static SET: Set<&str> = phf_set! { "a", "b", "c" };

    let entries = MAP.entries().collect::<Vec<_>>();
    let mut sorted = entries.clone();
    sorted.sort();
    assert_eq!(sorted, [(&"a", &1), (&"b", &2), (&"c", &3)]);
    assert_eq!(MAP.len(), entries.len());
    assert_iteration(MAP.entries(), &entries);
    assert_iteration(
        MAP.keys(),
        &entries.iter().map(|entry| entry.0).collect::<Vec<_>>(),
    );
    assert_iteration(
        MAP.values(),
        &entries.iter().map(|entry| entry.1).collect::<Vec<_>>(),
    );

    let keys = SET.iter().collect::<Vec<_>>();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(sorted, [&"a", &"b", &"c"]);
    assert_eq!(SET.len(), keys.len());
    assert_iteration(SET.iter(), &keys);
}

#[test]
fn ordered_entries() {
    static MAP: OrderedMap<&str, u32> = phf_ordered_map! { "b" => 2, "a" => 1, "c" => 3 };
    static SET: OrderedSet<&str> = phf_ordered_set! { "b", "a", "c" };

    assert_eq!(MAP.len(), 3);
    assert_iteration(MAP.entries(), &[(&"b", &2), (&"a", &1), (&"c", &3)]);
    assert_eq!(SET.len(), 3);
    assert_iteration(SET.iter(), &[&"b", &"a", &"c"]);
    for (index, key) in ["b", "a", "c"].iter().enumerate() {
        assert_eq!(MAP.get_index(key), Some(index));
        assert_eq!(SET.get_index(key), Some(index));
    }
}

#[test]
fn serialization() {
    static EMPTY: Map<&str, u32> = phf_map! {};
    static MAP: Map<&str, u32> = phf_map! { "a" => 1 };

    // Compare JSON text directly: parsing it into a map would hide duplicate keys.
    assert_eq!(serde_json::to_string(&EMPTY).unwrap(), "{}");
    assert_eq!(serde_json::to_string(&MAP).unwrap(), r#"{"a":1}"#);
}
