use phf::phf_map;
use uncased::UncasedStr;

pub static MAP: phf::Map<&'static UncasedStr, isize> = phf_map!(
    UncasedStr::new("Foo") => 0,
    UncasedStr::new("Bar") => 1,
);

fn main() {}
