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

static KEYWORDS: PhfMap<Token> = phf_map!(
    "," => COMMA,
    "=>" => FAT_ARROW,
    "(" => LPAREN,
    ")" => RPAREN,
    "=" => EQ,
);

pub fn parse_token(tok: &str) -> Option<Token> {
    KEYWORDS.find(tok)
}
```
