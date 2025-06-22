// Regression test for https://github.com/rust-phf/rust-phf/issues/299

use phf::phf_map;
use unicase::UniCase;

pub(crate) static KEYWORDS: phf::Map<UniCase<&'static str>, usize> = phf_map! {
    "foo" => 0,
    UniCase::ascii("FOO") => 1,
};

fn main(){
    KEYWORDS.get("foo").unwrap();
}
