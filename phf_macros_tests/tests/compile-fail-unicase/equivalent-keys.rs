use unicase::UniCase;
use phf::phf_map;

static MAP: phf::Map<UniCase<&'static str>, isize> = phf_map!(
    UniCase::ascii("FOO") => 42,
    UniCase::ascii("foo") => 42, //~ ERROR duplicate key UniCase("FOO")
);

fn main() {}
