extern crate phf_codegen;

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    let file = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&file).unwrap());

    write!(&mut file, "static MAP: ::phf::Map<&'static str, u32> = ").unwrap();
    phf_codegen::Map::new()
        .entry("a", "1")
        .entry("b", "2")
        .entry("c❤️", "3")
        .build(&mut file)
        .unwrap();
    write!(&mut file, ";\n").unwrap();

    write!(&mut file, "static SET: ::phf::Set<&'static str> = ").unwrap();
    phf_codegen::Set::new()
        .entry("a")
        .entry("b")
        .entry("c❤️")
        .build(&mut file)
        .unwrap();
    write!(&mut file, ";\n").unwrap();

    write!(
        &mut file,
        "static ORDERED_MAP: ::phf::OrderedMap<&'static str, u32> = "
    ).unwrap();
    phf_codegen::OrderedMap::new()
        .entry("a", "1")
        .entry("b", "2")
        .entry("c❤️", "3")
        .build(&mut file)
        .unwrap();
    write!(&mut file, ";\n").unwrap();

    write!(
        &mut file,
        "static ORDERED_SET: ::phf::OrderedSet<&'static str> = "
    ).unwrap();
    phf_codegen::OrderedSet::new()
        .entry("a")
        .entry("b")
        .entry("c❤️")
        .build(&mut file)
        .unwrap();
    write!(&mut file, ";\n").unwrap();

    write!(
        &mut file,
        "static SLICE_KEYS: ::phf::Map<&'static [u8], u32> = "
    ).unwrap();
    phf_codegen::Map::new()
        .entry(&b"a"[..], "1")
        .entry(&b"b"[..], "2")
        .entry(&b"c\xff"[..], "3")
        .build(&mut file)
        .unwrap();
    write!(&mut file, ";\n").unwrap();

    write!(&mut file, "static EMPTY: ::phf::Map<&'static str, u32> = ").unwrap();
    phf_codegen::Map::<&'static str>::new()
        .build(&mut file)
        .unwrap();
    write!(&mut file, ";\n").unwrap();
}
