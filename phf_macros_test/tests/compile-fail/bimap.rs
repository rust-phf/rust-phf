use phf::{BiMap, phf_bimap};

static LEFT: BiMap<u8, &str> = phf_bimap! {
    b'A' => "first",
    65 => "second",
};

static RIGHT: BiMap<&str, u8> = phf_bimap! {
    "first" => 1u8,
    "second" => 1,
};

static MIXED: BiMap<&str, u8> = phf_bimap! {
    "first" => 1u8,
    "second" => 2u16,
};

static OR: BiMap<&str, char> = phf_bimap! {
    "color" => 'r' | 'g',
};

fn main() {}
