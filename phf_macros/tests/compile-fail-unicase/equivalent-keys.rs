#![feature(plugin)]
#![plugin(phf_macros)]

extern crate phf;
extern crate unicase;

use unicase::UniCase;

static MAP: phf::Map<UniCase<&'static str>, isize> = phf_map!( //~ ERROR duplicate key UniCase::unicode("Ma\u{df}e")
    UniCase::unicode("Maße") => 42, //~ NOTE one occurrence here
    UniCase::unicode("MASSE") => 42, //~ NOTE one occurrence here
);
