extern crate phf_codegen;
extern crate unicase;

use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use unicase::UniCase;

fn main() -> io::Result<()> {
    let file = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&file)?);

    write!(&mut file, "static MAP: ::phf::Map<u32, &'static str> = ")?;
    phf_codegen::Map::new()
        .entry(1u32, "\"a\"")
        .entry(2u32, "\"b\"")
        .entry(3u32, "\"c\"")
        .build(&mut file)?;
    write!(&mut file, ";\n")?;

    write!(&mut file, "static SET: ::phf::Set<u32> = ")?;
    phf_codegen::Set::new()
        .entry(1u32)
        .entry(2u32)
        .entry(3u32)
        .build(&mut file)?;
    write!(&mut file, ";\n")?;

    write!(&mut file, "static ORDERED_MAP: ::phf::OrderedMap<u32, &'static str> = ")?;
    phf_codegen::OrderedMap::new()
        .entry(1u32, "\"a\"")
        .entry(2u32, "\"b\"")
        .entry(3u32, "\"c\"")
        .build(&mut file)?;
    write!(&mut file, ";\n")?;

    write!(&mut file, "static ORDERED_SET: ::phf::OrderedSet<u32> = ")?;
    phf_codegen::OrderedSet::new()
        .entry(1u32)
        .entry(2u32)
        .entry(3u32)
        .build(&mut file)?;
    write!(&mut file, ";\n")?;

    write!(&mut file, "static STR_KEYS: ::phf::Map<&'static str, u32> = ")?;
    phf_codegen::Map::new()
        .entry("a", "1")
        .entry("b", "2")
        .entry("c", "3")
        .build(&mut file)?;
    write!(&mut file, ";\n")?;

    write!(&mut file, "static UNICASE_MAP: ::phf::Map<::unicase::UniCase<&'static str>, \
                                                      &'static str> = ")?;
    phf_codegen::Map::new()
        .entry(UniCase::new("abc"), "\"a\"")
        .entry(UniCase::new("DEF"), "\"b\"")
        .build(&mut file)?;
    write!(&mut file, ";\n")?;

    //u32 is used here purely for a type that impls `Hash+PhfHash+Eq+fmt::Debug`, but is not required for the empty test itself
    write!(&mut file, "static EMPTY: ::phf::Map<u32, u32> = ")?;
    phf_codegen::Map::<u32>::new().build(&mut file)?;
    write!(&mut file, ";\n")?;

    write!(&mut file, "static EMPTY_ORDERED: ::phf::OrderedMap<u32, u32> = ")?;
    phf_codegen::OrderedMap::<u32>::new().build(&mut file)?;
    write!(&mut file, ";\n")?;

    write!(&mut file, "static ARRAY_KEYS: ::phf::Map<[u8; 3], u32> = ")?;
    phf_codegen::Map::<[u8; 3]>::new()
        .entry(*b"foo", "0")
        .entry(*b"bar", "1")
        .entry(*b"baz", "2")
        .build(&mut file)?;

    writeln!(&mut file, ";")?;

    write!(&mut file, "static BYTE_STR_KEYS: ::phf::Map<&[u8], u32> = ")?;
    // key type required here as it will infer `&'static [u8; 3]` instead
    phf_codegen::Map::<&[u8]>::new()
        .entry(b"foo", "0")
        .entry(b"bar", "1")
        .entry(b"baz", "2")
        .entry(b"quux", "3")
        .build(&mut file)?;
    writeln!(&mut file, ";")
}
