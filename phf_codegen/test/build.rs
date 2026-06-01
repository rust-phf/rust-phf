use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use quote::{ToTokens, quote};
use uncased::UncasedStr;
use unicase::{Ascii, UniCase};

fn main() -> io::Result<()> {
    let file = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&file)?);

    writeln!(
        &mut file,
        "static MAP: ::phf::Map<u32, &'static str> = \n{};",
        phf_codegen::Map::new()
            .entry(1u32, "\"a\"")
            .entry(2u32, "\"b\"")
            .entry(3u32, "\"c\"")
            .build()
    )?;

    writeln!(
        &mut file,
        "static SET: ::phf::Set<u32> = \n{};",
        phf_codegen::Set::new()
            .entry(1u32)
            .entry(2u32)
            .entry(3u32)
            .build()
    )?;

    writeln!(
        &mut file,
        "static ORDERED_MAP: ::phf::OrderedMap<u32, &'static str> = \n{};",
        phf_codegen::OrderedMap::new()
            .entry(1u32, "\"a\"")
            .entry(2u32, "\"b\"")
            .entry(3u32, "\"c\"")
            .build()
    )?;

    writeln!(
        &mut file,
        "static ORDERED_SET: ::phf::OrderedSet<u32> = \n{};",
        phf_codegen::OrderedSet::new()
            .entry(1u32)
            .entry(2u32)
            .entry(3u32)
            .build()
    )?;

    writeln!(
        &mut file,
        "static STR_KEYS: ::phf::Map<&'static str, u32> = \n{};",
        phf_codegen::Map::new()
            .entry("a", "1")
            .entry("b", "2")
            .entry("c", "3")
            .build()
    )?;

    write!(
        &mut file,
        "static UNICASE_MAP: ::phf::Map<::unicase::UniCase<&'static str>, &'static str> = \n{};",
        phf_codegen::Map::new()
            .entry(UniCase::new("abc"), "\"a\"")
            .entry(UniCase::new("DEF"), "\"b\"")
            .build()
    )?;

    write!(
        &mut file,
        "static UNICASE_ASCII_MAP: ::phf::Map<::unicase::Ascii<&'static str>, &'static str> = \n{};",
        phf_codegen::Map::new()
            .entry(Ascii::new("abc"), "\"a\"")
            .entry(Ascii::new("DEF"), "\"b\"")
            .build()
    )?;

    write!(
        &mut file,
        "static UNCASED_MAP: ::phf::Map<&'static ::uncased::UncasedStr, &'static str> = \n{};",
        phf_codegen::Map::new()
            .entry(UncasedStr::new("abc"), "\"a\"")
            .entry(UncasedStr::new("DEF"), "\"b\"")
            .build()
    )?;

    //u32 is used here purely for a type that impls `Hash+PhfHash+Eq+fmt::Debug`, but is not required for the empty test itself
    writeln!(
        &mut file,
        "static EMPTY: ::phf::Map<u32, u32> = \n{};",
        phf_codegen::Map::<u32>::new().build()
    )?;

    writeln!(
        &mut file,
        "static EMPTY_ORDERED: ::phf::OrderedMap<u32, u32> = \n{};",
        phf_codegen::OrderedMap::<u32>::new().build()
    )?;

    writeln!(
        &mut file,
        "static ARRAY_KEYS: ::phf::Map<[u8; 3], u32> = \n{};",
        phf_codegen::Map::<[u8; 3]>::new()
            .entry(*b"foo", "0")
            .entry(*b"bar", "1")
            .entry(*b"baz", "2")
            .build()
    )?;

    // key type required here as it will infer `&'static [u8; 3]` instead
    writeln!(
        &mut file,
        "static BYTE_STR_KEYS: ::phf::Map<&[u8], u32> = \n{};",
        phf_codegen::Map::<&[u8]>::new()
            .entry(b"foo", "0")
            .entry(b"bar", "1")
            .entry(b"baz", "2")
            .entry(b"quux", "3")
            .build()
    )?;

    writeln!(
        &mut file,
        "static BYTE_VEC_KEYS: ::phf::Map<&'static [u8], u32> = \n{};",
        phf_codegen::Map::<Vec<u8>>::new()
            .entry(b"foo".to_vec(), "0")
            .entry(b"bar".to_vec(), "1")
            .entry(b"baz".to_vec(), "2")
            .build()
    )?;

    writeln!(
        &mut file,
        "static U32_VEC_KEYS: ::phf::Map<&'static [u32], &'static str> = \n{};",
        phf_codegen::Map::<Vec<u32>>::new()
            .entry(vec![1u32, 2, 3], "\"a\"")
            .entry(vec![4u32, 5, 6], "\"b\"")
            .entry(vec![7u32, 8, 9], "\"c\"")
            .build()
    )?;

    writeln!(
        &mut file,
        "static U32_VEC_SET: ::phf::Set<&'static [u32]> = \n{};",
        phf_codegen::Set::<Vec<u32>>::new()
            .entry(vec![1u32, 2, 3])
            .entry(vec![4u32, 5, 6])
            .entry(vec![7u32, 8, 9])
            .build()
    )?;

    writeln!(
        &mut file,
        "static U32_VEC_ORDERED_MAP: ::phf::OrderedMap<&'static [u32], &'static str> = \n{};",
        phf_codegen::OrderedMap::<Vec<u32>>::new()
            .entry(vec![1u32, 2, 3], "\"a\"")
            .entry(vec![4u32, 5, 6], "\"b\"")
            .entry(vec![7u32, 8, 9], "\"c\"")
            .build()
    )?;

    writeln!(
        &mut file,
        "static U32_VEC_ORDERED_SET: ::phf::OrderedSet<&'static [u32]> = \n{};",
        phf_codegen::OrderedSet::<Vec<u32>>::new()
            .entry(vec![1u32, 2, 3])
            .entry(vec![4u32, 5, 6])
            .entry(vec![7u32, 8, 9])
            .build()
    )?;

    // Test FromIterator implementation
    writeln!(
        &mut file,
        "static FROM_ITER_MAP: ::phf::Map<&'static str, u32> = \n{};",
        vec![("one", "1"), ("two", "2"), ("three", "3")]
            .into_iter()
            .collect::<phf_codegen::Map<_>>()
            .build()
    )?;

    // Test tuple keys for Map
    writeln!(
        &mut file,
        "static TUPLE_MAP: ::phf::Map<(u32, &'static str), &'static str> = \n{};",
        phf_codegen::Map::new()
            .entry((1u32, "a"), "\"first\"")
            .entry((2u32, "b"), "\"second\"")
            .entry((3u32, "c"), "\"third\"")
            .build()
    )?;

    // Test tuple keys for Set
    writeln!(
        &mut file,
        "static TUPLE_SET: ::phf::Set<(u32, &'static str)> = \n{};",
        phf_codegen::Set::new()
            .entry((1u32, "x"))
            .entry((2u32, "y"))
            .entry((3u32, "z"))
            .build()
    )?;

    writeln!(
        &mut file,
        "static ORDERED_TUPLE_MAP: ::phf::OrderedMap<(u32, &'static str), &'static str> = \n{};",
        phf_codegen::OrderedMap::new()
            .entry((1u32, "a"), "\"first\"")
            .entry((2u32, "b"), "\"second\"")
            .entry((3u32, "c"), "\"third\"")
            .build()
    )?;

    writeln!(
        &mut file,
        "static ORDERED_TUPLE_SET: ::phf::OrderedSet<(u32, &'static str)> = \n{};",
        phf_codegen::OrderedSet::new()
            .entry((1u32, "x"))
            .entry((2u32, "y"))
            .entry((3u32, "z"))
            .build()
    )?;

    // Test nested tuple keys for Map
    writeln!(
        &mut file,
        "static NESTED_TUPLE_MAP: ::phf::Map<((u32, u32), &'static str), u32> = \n{};",
        phf_codegen::Map::new()
            .entry(((1u32, 2u32), "nested"), "10")
            .entry(((3u32, 4u32), "tuple"), "20")
            .entry(((5u32, 6u32), "keys"), "30")
            .build()
    )?;

    // Test mixed type tuple keys
    writeln!(
        &mut file,
        "static MIXED_TUPLE_MAP: ::phf::Map<(bool, u8, &'static str), &'static str> = \n{};",
        phf_codegen::Map::new()
            .entry((true, 1u8, "test"), "\"value1\"")
            .entry((false, 2u8, "demo"), "\"value2\"")
            .entry((true, 3u8, "example"), "\"value3\"")
            .build()
    )?;

    let mut quoted_map_tokens = phf_codegen::Map::new();
    quoted_map_tokens
        .entry(1u32, "\"a\"")
        .entry(2u32, "\"b\"")
        .entry(3u32, "\"c\"");
    let quoted_map_tokens: proc_macro2::TokenStream = quoted_map_tokens.build().to_token_stream();

    let mut quoted_set = phf_codegen::Set::new();
    quoted_set.entry(1u32).entry(2u32).entry(3u32);
    let quoted_set = quoted_set.build();

    let mut quoted_ordered_map = phf_codegen::OrderedMap::new();
    quoted_ordered_map
        .entry(1u32, "\"a\"")
        .entry(2u32, "\"b\"")
        .entry(3u32, "\"c\"");
    let quoted_ordered_map = quoted_ordered_map.build();

    let mut quoted_ordered_set = phf_codegen::OrderedSet::new();
    quoted_ordered_set.entry(1u32).entry(2u32).entry(3u32);
    let quoted_ordered_set = quoted_ordered_set.build();

    writeln!(
        &mut file,
        "{}",
        quote! {
            static TO_TOKEN_STREAM_MAP: ::phf::Map<u32, &'static str> = #quoted_map_tokens;
            static QUOTED_SET: ::phf::Set<u32> = #quoted_set;
            static QUOTED_ORDERED_MAP: ::phf::OrderedMap<u32, &'static str> = #quoted_ordered_map;
            static QUOTED_ORDERED_SET: ::phf::OrderedSet<u32> = #quoted_ordered_set;
        }
    )
}
