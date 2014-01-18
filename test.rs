#[feature(phase)];

#[phase(syntax, link)]
extern mod mphf;

use mphf::MphfMap;

#[test]
fn test_empty() {
    let map: MphfMap<int> = mphf_map!();
    assert!(map.is_empty());
}

#[test]
fn test_two() {
    static map: MphfMap<int> = mphf_map!(
        "foo" => 10,
        "bar" => 11,
    );
    assert!(Some(&10) == map.find(& &"foo"));
    assert!(Some(&11) == map.find(& &"bar"));
    assert_eq!(None, map.find(& &"asdf"));
}
