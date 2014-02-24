#[feature(phase)];

extern crate collections;
#[phase(syntax)]
extern crate phf_mac;
extern crate phf;

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
