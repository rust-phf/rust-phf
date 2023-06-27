use phf::phf_map;
use unicase::{Ascii, UniCase};

pub static MAP: phf::Map<UniCase<&'static str>, isize> = phf_map!(
    UniCase::ascii("Foo") => 0,
    UniCase::unicode("Bar") => 1,
);

pub static ASCII_MAP: phf::Map<Ascii<&'static str>, isize> = phf_map!(
    Ascii::new("Foo") => 0,
    Ascii::new("Bar") => 1,
);

fn main() {}
