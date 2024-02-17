use phf::phf_map;
use uncased::UncasedStr;

static MAP: phf::Map<&'static UncasedStr, isize> = phf_map!(
    UncasedStr::new("FOO") => 42,
    UncasedStr::new("foo") => 42, //~ ERROR duplicate key UncasedStr("FOO")
);

fn main() {}
