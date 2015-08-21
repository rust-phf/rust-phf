#![feature(plugin, test)]
#![plugin(phf_macros)]

extern crate test;
extern crate phf;

mod map {
    use std::collections::{BTreeMap, HashMap};
    use test::Bencher;

    use phf;

    static MAP: phf::Map<&'static str, isize> = phf_map!(
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
    );

    fn match_get(key: &str) -> Option<usize> {
        match key {
            "apple" => Some(0),
            "banana" => Some(1),
            "carrot" => Some(2),
            "doughnut" => Some(3),
            "eggplant" => Some(4),
            "frankincene" => Some(5),
            "grapes" => Some(6),
            "haggis" => Some(7),
            "ice cream" => Some(8),
            "jelly beans" => Some(9),
            "kaffir lime leaves" => Some(10),
            "lemonade" => Some(11),
            "mashmallows" => Some(12),
            "nectarines" => Some(13),
            "oranges" => Some(14),
            "pineapples" => Some(15),
            "quinoa" => Some(16),
            "rosemary" => Some(17),
            "sourdough" => Some(18),
            "tomatoes" => Some(19),
            "unleavened bread" => Some(20),
            "vanilla" => Some(21),
            "watermelon" => Some(22),
            "xinomavro grapes" => Some(23),
            "yogurt" => Some(24),
            "zucchini" => Some(25),
            _ => None
        }
    }

    #[bench]
    fn bench_match_some(b: &mut Bencher) {
        b.iter(|| {
            assert_eq!(match_get("zucchini").unwrap(), 25);
        })
    }

    #[bench]
    fn bench_match_none(b: &mut Bencher) {
        b.iter(|| {
            assert_eq!(match_get("potato"), None);
        })
    }

    #[bench]
    fn bench_btreemap_some(b: &mut Bencher) {
        let mut map = BTreeMap::new();
        for (key, value) in MAP.entries() {
            map.insert(*key, *value);
        }

        b.iter(|| {
            assert_eq!(map.get("zucchini").unwrap(), &25);
        })
    }

    #[bench]
    fn bench_hashmap_some(b: &mut Bencher) {
        let mut map = HashMap::new();
        for (key, value) in MAP.entries() {
            map.insert(*key, *value);
        }

        b.iter(|| {
            assert_eq!(map.get("zucchini").unwrap(), &25);
        })
    }

    #[bench]
    fn bench_phf_some(b: &mut Bencher) {
        b.iter(|| {
            assert_eq!(MAP.get("zucchini").unwrap(), &25);
        })
    }

    #[bench]
    fn bench_btreemap_none(b: &mut Bencher) {
        let mut map = BTreeMap::new();
        for (key, value) in MAP.entries() {
            map.insert(*key, *value);
        }

        b.iter(|| {
            assert_eq!(map.get("potato"), None);
        })
    }


    #[bench]
    fn bench_hashmap_none(b: &mut Bencher) {
        let mut map = BTreeMap::new();
        for (key, value) in MAP.entries() {
            map.insert(*key, *value);
        }

        b.iter(|| {
            assert_eq!(map.get("potato"), None);
        })
    }

    #[bench]
    fn bench_phf_none(b: &mut Bencher) {
        b.iter(|| {
            assert_eq!(MAP.get("potato"), None);
        })
    }
}
