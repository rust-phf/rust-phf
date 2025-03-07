# cphf
[![Crates.io](https://img.shields.io/crates/v/cphf.svg)](https://crates.io/crates/cphf)
[![Workflow Status](https://github.com/Daniel-Aaron-Bloom/cphf-rs/workflows/CI/badge.svg)](https://github.com/Daniel-Aaron-Bloom/cphf-rs/actions?query=workflow%3A%22CI%22)

CPHF is a library to generate efficient lookup tables at compile time using
[perfect hash functions](http://en.wikipedia.org/wiki/Perfect_hash_function).

It currently uses the
[CHD algorithm](http://cmph.sourceforge.net/papers/esa09.pdf) and can generate
a 120 entry map in roughly 1 second.

MSRV (minimum supported rust version) is Rust 1.85.
In contrast to the excellent [`phf`](https://github.com/rust-phf/rust-phf/) crate, this crate
uses no code generation outside of normal `macro_rules`, and instead generates the map using `const` expressions.
This does mean this crate several orders of magnitude slower than `phf` (so maps with thousands of entries would probably be
better served by `phf`), but doing so allows it to avoid all the major drawbacks. Namely, [nested maps](https://github.com/rust-phf/rust-phf/issues/183) are supported [any type](https://github.com/rust-phf/rust-phf/issues/196) can be used as a key (provided it implements the correct traits and pseudo-traits).

### Usage

Simply add `cphf` as a depenency and utilize the included [`phf_ordered_map`] macro to construct an [`OrderedMap`]
or the [`phf_ordered_set`] macro to construct an [`OrderedSet`]

```rust
use cphf::{phf_ordered_map, OrderedMap};

#[derive(Clone)]
pub enum Keyword {
    Loop,
    Continue,
    Break,
    Fn,
    Extern,
}

static KEYWORDS: OrderedMap<&'static str, Keyword> = phf_ordered_map! {&'static str, Keyword;
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

Inclusion of duplicate keys will result in a compiler error.

```compile_fail
use cphf::{phf_ordered_set, OrderedSet};
static DUPLICATE_KEYS: OrderedSet<u32> = phf_ordered_set! {u32;
    0,
    1,
    0,
};
```

```compile_fail
use cphf::{phf_ordered_map, OrderedMap};
static DUPLICATE_KEYS: OrderedMap<u32, ()> = phf_ordered_map! {u32, ();
    0 => (),
    1 => (),
    0 => (),
};
```

## License

Licensed under 
* MIT license ([LICENSE](LICENSE) or https://opensource.org/licenses/MIT)

[`phf_ordered_map`]: https://docs.rs/cphf/latest/cphf/macro.phf_ordered_map.html "macro cphf::phf_ordered_map"
[`phf_ordered_set`]: https://docs.rs/cphf/latest/cphf/macro.phf_ordered_set.html "macro cphf::phf_ordered_set"
[`OrderedSet`]: https://docs.rs/cphf/latest/cphf/struct.OrderedSet.html "struct cphf::OrderedSet"
[`OrderedMap`]: https://docs.rs/cphf/latest/cphf/struct.OrderedMap.html "struct cphf::OrderedMap"
