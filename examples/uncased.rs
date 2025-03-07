use cphf::{phf_ordered_map, OrderedMap, UncasedStr};

pub static MAP: OrderedMap<&'static UncasedStr, isize> = phf_ordered_map! {&'static UncasedStr, isize;
    UncasedStr::new("Foo") => 0,
    UncasedStr::new("Bar") => 1,
};

fn main() {
    assert_eq!(MAP.get("foo").cloned(), Some(0));
    assert_eq!(MAP.get("bAR").cloned(), Some(1));
    assert_eq!(MAP.get(&"bAR".to_owned()).cloned(), Some(1));
}
