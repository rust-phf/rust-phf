Rust-PHF
=========

Rust-PHF is a library to generate efficient lookup tables at compile time using
[perfect hash functions](http://en.wikipedia.org/wiki/Perfect_hash_function).

It is still in the very early stages, but the plan is to use the
[CHD algorithm](http://cmph.sourceforge.net/papers/esa09.pdf).

Example
=======

```rust
#[feature(phase)];

#[phase(syntax)]
extern mod phf_mac;
extern mod phf;

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
