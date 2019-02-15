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

    write!(&mut file, "static MAP: ::phf::Map<u32, &'static str> = ").unwrap();
    phf_codegen::Map::new()
        .entry(1u32, "\"a\"")
        .entry(2u32, "\"b\"")
        .entry(3u32, "\"c\"")
        .build(&mut file)
        .unwrap();
    write!(&mut file, ";\n").unwrap();

    write!(&mut file, "static SET: ::phf::Set<u32> = ").unwrap();
    phf_codegen::Set::new()
        .entry(1u32)
        .entry(2u32)
        .entry(3u32)
        .build(&mut file)
        .unwrap();
    write!(&mut file, ";\n").unwrap();

    write!(&mut file, "static ORDERED_MAP: ::phf::OrderedMap<u32, &'static str> = ").unwrap();
    phf_codegen::OrderedMap::new()
        .entry(1u32, "\"a\"")
        .entry(2u32, "\"b\"")
        .entry(3u32, "\"c\"")
        .build(&mut file)
        .unwrap();
    write!(&mut file, ";\n").unwrap();

    write!(&mut file, "static ORDERED_SET: ::phf::OrderedSet<u32> = ").unwrap();
    phf_codegen::OrderedSet::new()
        .entry(1u32)
        .entry(2u32)
        .entry(3u32)
        .build(&mut file)
        .unwrap();
    write!(&mut file, ";\n").unwrap();

    write!(&mut file, "static STR_KEYS: ::phf::Map<&'static str, u32> = ").unwrap();
    phf_codegen::Map::new()
        .entry("a", "1")
        .entry("b", "2")
        .entry("c", "3")
        .build(&mut file)
        .unwrap();
    write!(&mut file, ";\n").unwrap();

    write!(&mut file, "static UNICASE_MAP: ::phf::Map<::unicase::UniCase<&'static str>, \
                                                      &'static str> = ").unwrap();
    phf_codegen::Map::new()
        .entry(UniCase("abc"), "\"a\"")
        .entry(UniCase("DEF"), "\"b\"")
        .build(&mut file)
        .unwrap();
    write!(&mut file, ";\n").unwrap();

    let mut formatted_map = phf_codegen::Map::new();

    formatted_map
        .entry("a", "1")
        .entry("b", "2")
        .entry("c", "3");

    write!(&mut file, "static FORMATTED_MAP: phf::Map<&'static str, u32> = {};\n", formatted_map).unwrap();

    let mut formatted_ordered_map = phf_codegen::OrderedMap::new();

    formatted_ordered_map
        .entry("a", "1")
        .entry("b", "2")
        .entry("c", "3");

    write!(&mut file, "static FORMATTED_ORDERED_MAP: phf::OrderedMap<&'static str, u32> = {};\n", formatted_ordered_map).unwrap();

    let mut formatted_set = phf_codegen::Set::new();

    formatted_set
        .entry("a")
        .entry("b")
        .entry("c");

    write!(&mut file, "static FORMATTED_SET: phf::Set<&'static str> = {};\n", formatted_set).unwrap();

    let mut formatted_ordered_set = phf_codegen::OrderedSet::new();

    formatted_ordered_set
        .entry("a")
        .entry("b")
        .entry("c");

    write!(&mut file, "static FORMATTED_ORDERED_SET: phf::OrderedSet<&'static str> = {};\n", formatted_ordered_set).unwrap();
}
