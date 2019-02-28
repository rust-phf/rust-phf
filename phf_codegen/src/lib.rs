//! A set of builders to generate Rust source for PHF data structures at
//! compile time.
//!
//! The provided builders are intended to be used in a Cargo build script to
//! generate a Rust source file that will be included in a library at build
//! time.
//!
//! # Examples
//!
//! build.rs
//!
//! ```rust,no_run
//! extern crate phf_codegen;
//!
//! use std::env;
//! use std::fs::File;
//! use std::io::{BufWriter, Write};
//! use std::path::Path;
//!
//! fn main() {
//!     let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
//!     let mut file = BufWriter::new(File::create(&path).unwrap());
//!
//!     write!(
//!         &mut file,
//!         "static KEYWORDS: phf::Map<&'static str, Keyword> = {};\n",
//!         phf_codegen::Map::new()
//!             .entry("loop", "Keyword::Loop")
//!             .entry("continue", "Keyword::Continue")
//!             .entry("break", "Keyword::Break")
//!             .entry("fn", "Keyword::Fn")
//!             .entry("extern", "Keyword::Extern")
//!             .build(),
//!     ).unwrap();
//! }
//! ```
//!
//! lib.rs
//!
//! ```ignore
//! extern crate phf;
//!
//! #[derive(Clone)]
//! enum Keyword {
//!     Loop,
//!     Continue,
//!     Break,
//!     Fn,
//!     Extern,
//! }
//!
//! include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
//!
//! pub fn parse_keyword(keyword: &str) -> Option<Keyword> {
//!     KEYWORDS.get(keyword).cloned()
//! }
//! ```
//!
//! # Note
//!
//! The compiler's stack will overflow when processing extremely long method
//! chains (500+ calls). When generating large PHF data structures, consider
//! looping over the entries or making each call a separate statement:
//!
//! ```rust
//! let entries = [("hello", "1"), ("world", "2")];
//!
//! let mut builder = phf_codegen::Map::new();
//! for &(key, value) in &entries {
//!     builder.entry(key, value);
//! }
//! // ...
//! ```
//!
//! ```rust
//! let mut builder = phf_codegen::Map::new();
//! builder.entry("hello", "1");
//! builder.entry("world", "2");
//! // ...
//! ```
#![doc(html_root_url="https://docs.rs/phf_codegen/0.7")]
extern crate phf_shared;
extern crate phf_generator;

use phf_shared::PhfHash;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;

/// A builder for the `phf::Map` type.
pub struct Map<K> {
    keys: Vec<K>,
    values: Vec<String>,
    path: String,
}

impl<K: Hash+PhfHash+Eq+Debug> Map<K> {
    /// Creates a new `phf::Map` builder.
    pub fn new() -> Map<K> {
        // FIXME rust#27438
        //
        // On Windows/MSVC there are major problems with the handling of dllimport.
        // Here, because downstream build scripts only invoke generics from phf_codegen,
        // the linker ends up throwing a way a bunch of static symbols we actually need.
        // This works around the problem, assuming that all clients call `Map::new` by
        // calling a non-generic function.
        fn noop_fix_for_27438() {
        }
        noop_fix_for_27438();

        Map {
            keys: vec![],
            values: vec![],
            path: String::from("::phf"),
        }
    }

    /// Set the path to the `phf` crate from the global namespace
    pub fn phf_path(&mut self, path: &str) -> &mut Map<K> {
        self.path = path.to_owned();
        self
    }

    /// Adds an entry to the builder.
    ///
    /// `value` will be written exactly as provided in the constructed source.
    pub fn entry(&mut self, key: K, value: &str) -> &mut Map<K> {
        self.keys.push(key);
        self.values.push(value.to_owned());
        self
    }

    /// Constructs a `phf::Map`, returning a `Display` implementor.
    ///
    /// # Panics
    ///
    /// Panics if there are any duplicate keys.
    pub fn build<'a>(&'a self) -> MapView<'a, K> {
        let mut set = HashSet::new();
        for key in &self.keys {
            if !set.insert(key) {
                panic!("duplicate key `{:?}`", key);
            }
        }

        let state = phf_generator::generate_hash(&self.keys);
        MapView { map: self, state }
    }
}

/// A valid [`Map`](struct.Map.html) that can be displayed.
pub struct MapView<'a, K> {
    map: &'a Map<K>,
    state: phf_generator::HashState,
}

impl<'a, K: Hash+PhfHash+Eq+Debug> Display for MapView<'a, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f,
                    "{}::Map {{
    key: {},
    disps: {}::Slice::Static(&[",
                    self.map.path, self.state.key, self.map.path));
        for &(d1, d2) in &self.state.disps {
            try!(write!(f,
                        "
        ({}, {}),",
                        d1,
                        d2));
        }
        try!(write!(f,
                    "
    ]),
    entries: {}::Slice::Static(&[", self.map.path));
        for &idx in &self.state.map {
            try!(write!(f,
                        "
        ({:?}, {}),",
                        &self.map.keys[idx],
                        &self.map.values[idx]));
        }
        write!(f,
               "
    ]),
}}")
    }
}

/// A builder for the `phf::Set` type.
pub struct Set<T> {
    map: Map<T>,
}

impl<T: Hash+PhfHash+Eq+Debug> Set<T> {
    /// Constructs a new `phf::Set` builder.
    pub fn new() -> Set<T> {
        Set {
            map: Map::new(),
        }
    }

    /// Set the path to the `phf` crate from the global namespace
    pub fn phf_path(&mut self, path: &str) -> &mut Set<T> {
        self.map.phf_path(path);
        self
    }

    /// Adds an entry to the builder.
    pub fn entry(&mut self, entry: T) -> &mut Set<T> {
        self.map.entry(entry, "()");
        self
    }

    /// Constructs a `phf::Set`, returning a `Display` implementor.
    ///
    /// # Panics
    ///
    /// Panics if there are any duplicate entries.
    pub fn build<'a>(&'a self) -> SetView<'a, T> {
        SetView { path: &self.map.path, map: self.map.build() }
    }
}

/// A valid [`Set`](struct.Set.html) that can be displayed.
pub struct SetView<'a, T> {
    path: &'a str,
    map: MapView<'a, T>,
}

impl<'a, K: Hash+PhfHash+Eq+Debug> Display for SetView<'a, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}::Set {{ map: {} }}", self.path, self.map)
    }
}

/// A builder for the `phf::OrderedMap` type.
pub struct OrderedMap<K> {
    keys: Vec<K>,
    values: Vec<String>,
    path: String,
}

impl<K: Hash+PhfHash+Eq+Debug> OrderedMap<K> {
    /// Constructs a enw `phf::OrderedMap` builder.
    pub fn new() -> OrderedMap<K> {
        OrderedMap {
            keys: vec![],
            values: vec![],
            path: String::from("::phf"),
        }
    }

    /// Set the path to the `phf` crate from the global namespace
    pub fn phf_path(&mut self, path: &str) -> &mut OrderedMap<K> {
        self.path = path.to_owned();
        self
    }

    /// Adds an entry to the builder.
    ///
    /// `value` will be written exactly as provided in the constructed source.
    pub fn entry(&mut self, key: K, value: &str) -> &mut OrderedMap<K> {
        self.keys.push(key);
        self.values.push(value.to_owned());
        self
    }

    /// Constructs a `phf::OrderedMap`, returning a `Display` implementor.
    ///
    /// # Panics
    ///
    /// Panics if there are any duplicate keys.
    pub fn build<'a>(&'a self) -> OrderedMapView<'a, K> {
        let mut set = HashSet::new();
        for key in &self.keys {
            if !set.insert(key) {
                panic!("duplicate key `{:?}`", key);
            }
        }

        let state = phf_generator::generate_hash(&self.keys);
        OrderedMapView { state, map: self }
    }
}

/// A valid [`OrderedMap`](struct.OrderedMap.html) that can be displayed.
pub struct OrderedMapView<'a, K> {
    map: &'a OrderedMap<K>,
    state: ::phf_generator::HashState,
}

impl<'a, K: Hash+PhfHash+Eq+Debug> Display for OrderedMapView<'a, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f,
                    "{}::OrderedMap {{
    key: {},
    disps: {}::Slice::Static(&[",
                    self.map.path, self.state.key, self.map.path));
        for &(d1, d2) in &self.state.disps {
            try!(write!(f,
                        "
        ({}, {}),",
                        d1,
                        d2));
        }
        try!(write!(f,
                    "
    ]),
    idxs: {}::Slice::Static(&[", self.map.path));
        for &idx in &self.state.map {
            try!(write!(f,
                        "
        {},",
                        idx));
        }
        try!(write!(f,
                    "
    ]),
    entries: {}::Slice::Static(&[", self.map.path));
        for (key, value) in self.map.keys.iter().zip(self.map.values.iter()) {
            try!(write!(f,
                        "
        ({:?}, {}),",
                        key,
                        value));
        }
        write!(f,
               "
    ]),
}}")
    }
}

/// A builder for the `phf::OrderedSet` type.
pub struct OrderedSet<T> {
    map: OrderedMap<T>,
}

impl<T: Hash+PhfHash+Eq+Debug> OrderedSet<T> {
    /// Constructs a new `phf::OrderedSet` builder.
    pub fn new() -> OrderedSet<T> {
        OrderedSet {
            map: OrderedMap::new(),
        }
    }

    /// Set the path to the `phf` crate from the global namespace
    pub fn phf_path(&mut self, path: &str) -> &mut OrderedSet<T> {
        self.map.phf_path(path);
        self
    }

    /// Adds an entry to the builder.
    pub fn entry(&mut self, entry: T) -> &mut OrderedSet<T> {
        self.map.entry(entry, "()");
        self
    }

    /// Constructs a `phf::OrderedSet`, returning a `Display` implementor.
    ///
    /// # Panics
    ///
    /// Panics if there are any duplicate entries.
    pub fn build<'a>(&'a self) -> OrderedSetView<'a, T> {
        OrderedSetView { path: &self.map.path, map: self.map.build() }
    }
}

/// A valid [`OrderedSet`](struct.OrderedMap.html) that can be displayed.
pub struct OrderedSetView<'a, T> {
    path: &'a str,
    map: OrderedMapView<'a, T>
}

impl<'a, T: Hash+PhfHash+Eq+Debug> Display for OrderedSetView<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}::OrderedSet {{ map: {} }}", self.path, self.map)
    }
}
