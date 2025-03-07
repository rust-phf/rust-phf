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
uses no code generation outside of normal `macro_rules`.  Instead it generates maps using `const` expressions.
As such, this crate is several orders of magnitude slower than `phf` (so maps with thousands of entries would probably be
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

### Advanced Usage

If we wanted to make `Keyword` above into a key are as follows, we would implement traits (and pseudo-traits) like this:

```rust
use core::borrow::Borrow;
use cphf::{ConstKey, PhfKey, PhfKeyProxy, Hasher};

// We now require `Eq` to be a key
#[derive(Clone, PartialEq, Eq)]
pub enum Keyword {
    Loop,
    Continue,
    Break,
    Fn,
    Extern,
}

// Define a new type for our pseudo-trait
pub struct KeywordMarker;

// Glue our marker type to the real type.
impl PhfKey for Keyword {
    type ConstKey = KeywordMarker;
}
impl ConstKey for KeywordMarker {
    type PhfKey = Keyword;
}

// Implement our pseudo-trait. These are `const` inherent methods since there
// is current no method of adding `const` methods to a trait
impl KeywordMarker {
    pub const fn pfh_hash(value: &Keyword, state: &mut Hasher) {
        // We can only use `const` functions here
        // If Keyword were `Copy` would could use `*value as u32`
        // But for the sake of this example, we will do this the hard way
        use Keyword::*;
        let value = match value {
            Loop => 0,
            Continue => 1,
            Break => 2,
            Fn => 3,
            Extern => 4,
        };
        <u32 as PhfKey>::ConstKey::pfh_hash(&value, state)
    }
    pub const fn pfh_eq(lhs: &Keyword, rhs: &Keyword) -> bool {
        // We can only use `const` equality functions here
        // If Keyword were `Copy` would could use `*lhs as u32 == *rhs as u32`
        // But for the sake of this example, we will do this the hard way
        use Keyword::*;
        match (lhs, rhs) {
            (Loop, Loop) | (Continue, Continue) | (Break, Break) | (Fn, Fn) | (Extern, Extern) => true,
            _ => false,
        }
    }
}

// Finally we add a trait implementation to allow usage of indexing methods like `get`.
// You can make this broad or narrow as you would like, doing so just modifies what types callers can pass to `OrderedMap::get`,
// but `?Sized + Borrow<Self>` is a pretty good baseline.
impl<PK: ?Sized + Borrow<Keyword>> PhfKeyProxy<PK> for Keyword {
    fn pfh_hash(pk: &PK, state: &mut Hasher) {
        // Call the above hash function (or else make an equivalent hash somehow)
        KeywordMarker::pfh_hash(pk.borrow(), state)
    }
    fn pfh_eq(&self, other: &PK) -> bool {
        // Implement equality
        self == other.borrow()
    }
}

// Now we can make use `Keyword` as a key
static KEYWORDS: cphf::OrderedMap<Keyword, &'static str> = cphf::phf_ordered_map! {Keyword, &'static str;
    Keyword::Loop => "loop",
    Keyword::Continue => "continue",
    Keyword::Break => "break",
    Keyword::Fn => "fn",
    Keyword::Extern => "extern",
};
```


## License

Licensed under 
* MIT license ([LICENSE](LICENSE) or https://opensource.org/licenses/MIT)

[`phf_ordered_map`]: https://docs.rs/cphf/latest/cphf/macro.phf_ordered_map.html "macro cphf::phf_ordered_map"
[`phf_ordered_set`]: https://docs.rs/cphf/latest/cphf/macro.phf_ordered_set.html "macro cphf::phf_ordered_set"
[`OrderedSet`]: https://docs.rs/cphf/latest/cphf/struct.OrderedSet.html "struct cphf::OrderedSet"
[`OrderedMap`]: https://docs.rs/cphf/latest/cphf/struct.OrderedMap.html "struct cphf::OrderedMap"
