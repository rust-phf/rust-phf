#![feature(plugin, test)]
#![plugin(phf_macros)]

extern crate test;
extern crate phf;
extern crate phf_shared;

mod map {
    use std::collections::{BTreeMap, HashMap};
    use test::Bencher;

    use phf;

    macro_rules! map_and_match {
        ($map:ident, $match_get:ident, $phf_match_get:ident, $($key:expr => $value:expr,)+) => {
            static $map: phf::Map<&'static str, usize> = phf_map! {
                $($key => $value),+
            };

            fn $match_get(key: &str) -> Option<usize> {
                match key {
                    $($key => Some($value),)+
                    _ => None
                }
            }

            fn $phf_match_get(key: &str) -> Option<usize> {
                phf_match!(key {
                    $($key => Some($value),)+
                    _ => None
                })
            }
        }
    }

    map_and_match! { MAP, match_get, phf_match_get,
        "apple" => 0,
        "banana" => 1,
        "carrot" => 2,
        "doughnut" => 3,
        "eggplant" => 4,
        "frankincene" => 5,
        "grapes" => 6,
        "haggis" => 7,
        "ice cream" => 8,
        "jelly beans" => 9,
        "kaffir lime leaves" => 10,
        "lemonade" => 11,
        "mashmallows" => 12,
        "nectarines" => 13,
        "oranges" => 14,
        "pineapples" => 15,
        "quinoa" => 16,
        "rosemary" => 17,
        "sourdough" => 18,
        "tomatoes" => 19,
        "unleavened bread" => 20,
        "vanilla" => 21,
        "watermelon" => 22,
        "xinomavro grapes" => 23,
        "yogurt" => 24,
        "zucchini" => 25,
    }

    fn get_matching_items() -> Vec<(&'static str, usize)> {
        MAP.entries()
            .map(|(key, value)| (*key, *value))
            .collect()
    }

    fn get_non_matching_items() -> Vec<String> {
        MAP.entries()
            .map(|(key, _)| (format!("{}0", key)))
            .collect()
    }

    fn get_btreemap() -> BTreeMap<&'static str, usize> {
        let mut map = BTreeMap::new();
        for (key, value) in MAP.entries() {
            map.insert(*key, *value);
        }

        map
    }

    fn get_hashmap() -> HashMap<&'static str, usize> {
        let mut map = HashMap::new();
        for (key, value) in MAP.entries() {
            map.insert(*key, *value);
        }

        map
    }

    fn check_match<F: Fn(&str) -> Option<usize>>(items: &[(&str, usize)], f: F) {
        for &(key, value) in items.iter() {
            assert_eq!((f)(key), Some(value));
        }
    }

    fn check_not_match<F: Fn(&str) -> Option<usize>>(items: &[String], f: F) {
        for key in items.iter() {
            assert_eq!((f)(&key), None);
        }
    }

    #[bench]
    fn bench_match_some(b: &mut Bencher) {
        let items = get_matching_items();

        b.iter(|| {
            check_match(&items, match_get)
        })
    }

    #[bench]
    fn bench_match_none(b: &mut Bencher) {
        let items = get_non_matching_items();

        b.iter(|| {
            check_not_match(&items, match_get);
        })
    }

    #[bench]
    fn bench_phf_match_some(b: &mut Bencher) {
        let items = get_matching_items();

        b.iter(|| {
            check_match(&items, phf_match_get)
        })
    }

    #[bench]
    fn bench_phf_match_none(b: &mut Bencher) {
        let items = get_non_matching_items();

        b.iter(|| {
            check_not_match(&items, phf_match_get);
        })
    }

    #[bench]
    fn bench_btreemap_some(b: &mut Bencher) {
        let items = get_matching_items();
        let map = get_btreemap();

        b.iter(|| {
            check_match(&items, |key| map.get(key).map(|value| *value));
        })
    }

    #[bench]
    fn bench_btreemap_none(b: &mut Bencher) {
        let items = get_non_matching_items();
        let map = get_btreemap();

        b.iter(|| {
            check_not_match(&items, |key| map.get(key).map(|value| *value));
        })
    }

    #[bench]
    fn bench_hashmap_some(b: &mut Bencher) {
        let items = get_matching_items();
        let map = get_hashmap();

        b.iter(|| {
            check_match(&items, |key| map.get(key).map(|value| *value));
        })
    }

    #[bench]
    fn bench_hashmap_none(b: &mut Bencher) {
        let items = get_non_matching_items();
        let map = get_hashmap();

        b.iter(|| {
            check_not_match(&items, |key| map.get(key).map(|value| *value));
        })
    }

    #[bench]
    fn bench_phf_some(b: &mut Bencher) {
        let items = get_matching_items();

        b.iter(|| {
            check_match(&items, |key| MAP.get(key).map(|value| *value));
        })
    }

    #[bench]
    fn bench_phf_none(b: &mut Bencher) {
        let items = get_non_matching_items();

        b.iter(|| {
            check_not_match(&items, |key| MAP.get(key).map(|value| *value));
        })
    }
}
