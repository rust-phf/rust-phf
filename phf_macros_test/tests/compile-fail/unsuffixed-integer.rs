use phf::{phf_map, phf_ordered_map, phf_ordered_set, phf_set};

static MAP: phf::Map<u64, u64> = phf_map! {
    0 => 0u64,
    1u64 => 1u64,
};

static SET: phf::Set<u64> = phf_set! {
    0,
    1u64,
};

static TUPLE_MAP: phf::Map<(u32, &'static str), u8> = phf_map! {
    (0, "a") => 1u8,
    (1u32, "b") => 2u8,
};

static ARRAY_MAP: phf::Map<[u8; 2], u8> = phf_map! {
    [0, 1u8] => 1u8,
    [2u8, 3u8] => 2u8,
};

static ORDERED_MAP: phf::OrderedMap<u64, u64> = phf_ordered_map! {
    0 => 0u64,
    1u64 => 1u64,
};

static ORDERED_SET: phf::OrderedSet<u64> = phf_ordered_set! {
    0,
    1u64,
};

static MIXED_INTEGER_TYPES: phf::Set<u64> = phf_set! {
    0u64,
    1u32,
};

static MIXED_KEY_TYPES: phf::Set<char> = phf_set! {
    'a',
    97u32,
};

fn main() {}
