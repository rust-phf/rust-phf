#![feature(plugin)]
#![plugin(phf_macros)]

extern crate phf;
extern crate unicase;

use unicase::Ascii;

static MAP: phf::Map<Ascii<&'static str>, isize> = phf_map!( //~ ERROR duplicate key Ascii::new("FOO")
    Ascii::new("FOO") => 42, //~ NOTE one occurrence here
    Ascii::new("foo") => 42, //~ NOTE one occurrence here
);
