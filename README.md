Rust-PHF
=========

[![Build Status](https://travis-ci.org/sfackler/rust-phf.png?branch=master)](https://travis-ci.org/sfackler/rust-phf) [![Latest Version](https://img.shields.io/crates/v/phf.svg)](https://crates.io/crates/phf)

[Documentation](https://docs.rs/phf/0.7.23/phf)

Rust-PHF is a library to generate efficient lookup tables at compile time using
[perfect hash functions](http://en.wikipedia.org/wiki/Perfect_hash_function).

It currently uses the
[CHD algorithm](http://cmph.sourceforge.net/papers/esa09.pdf) and can generate
a 100,000 entry map in roughly .4 seconds. By default statistics are not
produced, but if you set the environment variable `PHF_STATS` it will issue
a compiler note about how long it took.

Usage
=====

##### Release 0.8.0 requires Rust 1.32.0

PHF data structures can be constucted via either the procedural 
macros in the `phf_macros` crate or code generation supported by the 
`phf_codegen` crate.

To compile the `phf` crate with a dependency on
libcore instead of libstd, enabling use in environments where libstd 
will not work, set `default-features = false` for the dependency:

```toml
[dependencies]
# to use `phf` in `no_std` environments
phf = { version = "0.8", default-features = false }
```

phf_macros
===========

```rust
use phf::phf_map;

#[derive(Clone)]
pub enum Keyword {
    Loop,
    Continue,
    Break,
    Fn,
    Extern,
}

static KEYWORDS: phf::Map<&'static str, Keyword> = phf_map! {
    "loop" => Keyword::Loop,
    "continue" => Keyword::Continue,
    "break" => Keyword::Break,
    "fn" => Keyword::Fn,
    "extern" => Keyword::Extern,
};

pub fn parse_keyword(keyword: &str) -> Option<Keyword> {
    KEYWORDS.get(keyword).cloned()
}
```

```toml
[dependencies]
phf = { version = "0.7", features = ["macros"] }
```

phf_codegen
===========

build.rs

```rust
extern crate phf_codegen;

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    write!(&mut file, "static KEYWORDS: phf::Map<&'static str, Keyword> = ").unwrap();
    phf_codegen::Map::new()
        .entry("loop", "Keyword::Loop")
        .entry("continue", "Keyword::Continue")
        .entry("break", "Keyword::Break")
        .entry("fn", "Keyword::Fn")
        .entry("extern", "Keyword::Extern")
        .build(&mut file)
        .unwrap();
    write!(&mut file, ";\n").unwrap();
}
```

lib.rs

```rust
extern crate phf;

#[derive(Clone)]
enum Keyword {
    Loop,
    Continue,
    Break,
    Fn,
    Extern,
}

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

pub fn parse_keyword(keyword: &str) -> Option<Keyword> {
    KEYWORDS.get(keyword).cloned()
}
```
