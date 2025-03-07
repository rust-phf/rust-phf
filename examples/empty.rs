use cphf::{phf_ordered_map, phf_ordered_set, OrderedMap, OrderedSet};

// Three different ways to make empty maps
static EMPTY_MAP_1: OrderedMap<u32, &str> = phf_ordered_map! {};
static EMPTY_MAP_2: OrderedMap<u32, &str> = phf_ordered_map! {u32, &'static str; };
static EMPTY_MAP_3: OrderedMap<u32, &str> = phf_ordered_map! {u32, &'static str;= []};

// Three different ways to make empty sets
static EMPTY_SET_1: OrderedSet<u32> = phf_ordered_set! {};
static EMPTY_SET_2: OrderedSet<u32> = phf_ordered_set! {u32; };
static EMPTY_SET_3: OrderedSet<u32> = phf_ordered_set! {u32;= []};

fn main() {
    let _ = (&EMPTY_MAP_1, &EMPTY_MAP_2, &EMPTY_MAP_3);
    let _ = (&EMPTY_SET_1, &EMPTY_SET_2, &EMPTY_SET_3);
}
