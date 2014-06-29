Rust-PHF
=========

[![Build Status](https://travis-ci.org/sfackler/rust-phf.png?branch=master)](https://travis-ci.org/sfackler/rust-phf)

Rust-PHF is a library to generate efficient lookup tables at compile time using
[perfect hash functions](http://en.wikipedia.org/wiki/Perfect_hash_function).

It currently uses the
[CHD algorithm](http://cmph.sourceforge.net/papers/esa09.pdf) and can generate
a 10,000 entry map in roughly .25 seconds.

Documentation is available at http://www.rust-ci.org/sfackler/rust-phf/doc/phf/.

Example
=======

```rust
#![feature(phase)]

#[phase(syntax)]
extern crate phf_mac;
extern crate phf;

use phf::PhfMap;

static KEYWORDS: PhfMap<Keyword> = phf_map!(
    "loop" => LOOP,
    "continue" => CONTINUE,
    "break" => BREAK,
    "fn" => FN,
    "extern" => EXTERN,
);

pub fn parse_keyword(keyword: &str) -> Option<Keyword> {
    KEYWORDS.find(keyword).map(|t| t.clone())
}
```
