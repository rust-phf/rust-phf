use phf::phf_map;
use unicase::{Ascii, UniCase};

static MAP: phf::Map<UniCase<&'static str>, isize> = phf_map!(
    UniCase::ascii("FOO") => 42,
    UniCase::ascii("foo") => 42, //~ ERROR duplicate key
);

static ASCII_MAP: phf::Map<Ascii<&'static str>, isize> = phf_map!(
    Ascii::new("FOO") => 42,
    Ascii::new("foo") => 42, //~ ERROR duplicate key
);

fn main() {}
