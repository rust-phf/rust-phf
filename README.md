Rust-PHF
=========

[![Build Status](https://travis-ci.org/sfackler/rust-phf.png?branch=master)](https://travis-ci.org/sfackler/rust-phf)

Rust-PHF is a library to generate efficient lookup tables at compile time using
[perfect hash functions](http://en.wikipedia.org/wiki/Perfect_hash_function).

It currently uses the
[CHD algorithm](http://cmph.sourceforge.net/papers/esa09.pdf) and can generate
a 100,000 entry map in roughly .4 seconds. By default statistics are not
produced, but if you use the `phf_mac` crate with the `stats` feature enabled
(writing `phf_macros/stats` in the `[dependencies]` section of your
`Cargo.toml` instead of `phf_macros`) and set the environment variable
`PHF_STATS` it will issue a compiler note about how long it took.

Documentation is available at https://sfackler.github.io/rust-phf/doc/phf

Example
=======

```rust
#![feature(plugin)]
#![plugin(phf_macros)]

extern crate phf;

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
