#[feature(phase)];

#[phase(syntax, link)]
extern mod phf;

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
fn test_empty() {
    let map: PhfMap<int> = phf_map!();
    assert!(map.is_empty());
}

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
