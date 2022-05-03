use phf::phf_map;
use unicase::UniCase;

pub static MAP: phf::Map<UniCase<&'static str>, isize> = phf_map!(
    UniCase::ascii("Foo") => 0,
    UniCase::unicode("Bar") => 1,
);

fn main() {}
