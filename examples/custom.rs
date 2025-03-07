use cphf::{phf_ordered_map, OrderedMap};

// Construct from a dynamically generated array of values
static BY_VAL: OrderedMap<u64, ()> = phf_ordered_map! {u64, (); = {
    let mut data = [(0u64, ()); 120];
    let mut i = 0;
    while i < data.len() {
        data[i].0 = i as u64;
        i += 1;
    }
    data
}};

// Construct from a reference to an existing slice of values;
const SOME_VALUES: &[(u64, ())] = &[(10, ()), (9, ())];
static BY_REF: OrderedMap<u64, ()> = phf_ordered_map! {u64, (); = *SOME_VALUES};

fn main() {
    BY_VAL.get(&12).unwrap();
    BY_REF.get(&10).unwrap();
}
