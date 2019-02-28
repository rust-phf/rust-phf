extern crate phf_codegen;
extern crate unicase;

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use unicase::UniCase;

fn main() {
    let file = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&file).unwrap());

    write!(
        &mut file,
        "static MAP: ::phf::Map<u32, &'static str> = {};\n",
        phf_codegen::Map::new()
            .entry(1u32, "\"a\"")
            .entry(2u32, "\"b\"")
            .entry(3u32, "\"c\"")
            .build()
    ).unwrap();

    write!(
        &mut file,
        "static SET: ::phf::Set<u32> = {};\n",
        phf_codegen::Set::new()
            .entry(1u32)
            .entry(2u32)
            .entry(3u32)
            .build()
    ).unwrap();

    write!(
        &mut file,
        "static ORDERED_MAP: ::phf::OrderedMap<u32, &'static str> = {};\n",
        phf_codegen::OrderedMap::new()
            .entry(1u32, "\"a\"")
            .entry(2u32, "\"b\"")
            .entry(3u32, "\"c\"")
            .build()
    ).unwrap();

    write!(
        &mut file,
        "static ORDERED_SET: ::phf::OrderedSet<u32> = {};\n",
        phf_codegen::OrderedSet::new()
            .entry(1u32)
            .entry(2u32)
            .entry(3u32)
            .build()
    ).unwrap();

    write!(
        &mut file,
        "static STR_KEYS: ::phf::Map<&'static str, u32> = {};\n",
        phf_codegen::Map::new()
            .entry("a", "1")
            .entry("b", "2")
            .entry("c", "3")
            .build(),
    ).unwrap();

    write!(
        &mut file,
        "static UNICASE_MAP: ::phf::Map<::unicase::UniCase<&'static str>, \
         &'static str> = {};\n",
        phf_codegen::Map::new()
            .entry(UniCase("abc"), "\"a\"")
            .entry(UniCase("DEF"), "\"b\"")
            .build(),
    ).unwrap()
}
