use phf::{phf_map, phf_ordered_map, phf_ordered_set, phf_set};
use unicase::UniCase;

static DUPLICATE_MAP: phf::Map<(char, &str), u8> = phf_map! {
    ('p', "a" | "a") => 0,
};

static DUPLICATE_SET: phf::Set<[u8; 2]> = phf_set! {
    [0u8 | 1, 2],
    [1, 2],
};

static DUPLICATE_UNICASE: phf::Set<UniCase<&str>> = phf_set! {
    UniCase::ascii("FOO" | "foo"),
};

static MIXED_TYPES: phf::OrderedMap<(u8, &str), u8> = phf_ordered_map! {
    (0u8 | 1u16, "a") => 0,
};

static UNSUFFIXED: phf::OrderedSet<(u8, &str)> = phf_ordered_set! {
    (0 | 1u8, "a"),
};

fn main() {}
