#![doc(html_root_url="http://sfackler.github.io/rust-phf/doc/v0.7.18")]
extern crate phf;
extern crate phf_generator;

#[cfg(test)]
mod test;

use phf::PhfHash;
use std::collections::HashSet;
use std::hash::Hash;

/// A builder for the `phf::Map` type.
pub struct Map<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
}

impl<K: Hash + PhfHash + Eq, V> Map<K, V> {
    /// Creates a new `phf::Map` builder.
    pub fn new() -> Map<K, V> {
        Map {
            keys: vec![],
            values: vec![],
        }
    }

    /// Adds an entry to the builder.
    pub fn entry(&mut self, key: K, value: V) -> &mut Map<K, V> {
        self.keys.push(key);
        self.values.push(value);
        self
    }

    /// Constructs a `phf::Map`.
    ///
    /// # Panics
    ///
    /// Panics if there are any duplicate keys.
    pub fn build(self) -> phf::Map<K, V> {
        {
            let mut set = HashSet::new();
            for key in &self.keys {
                if !set.insert(key) {
                    panic!("duplicate key");
                }
            }
        }

        let state = phf_generator::generate_hash(&self.keys);

        let mut entries = self.keys
                              .into_iter()
                              .zip(self.values)
                              .map(Some)
                              .collect::<Vec<_>>();
        let entries = state.map
                           .iter()
                           .map(|&i| entries[i].take().unwrap())
                           .collect();

        phf::Map {
            key: state.key,
            disps: phf::Slice::Dynamic(state.disps),
            entries: phf::Slice::Dynamic(entries),
        }
    }
}

/// A builder for the `phf::Set` type.
pub struct Set<T> {
    map: Map<T, ()>,
}

impl<T: Hash + PhfHash + Eq> Set<T> {
    /// Constructs a new `phf::Set` builder.
    pub fn new() -> Set<T> {
        Set { map: Map::new() }
    }

    /// Adds an entry to the builder.
    pub fn entry(&mut self, entry: T) -> &mut Set<T> {
        self.map.entry(entry, ());
        self
    }

    /// Constructs a `phf::Set`.
    ///
    /// # Panics
    ///
    /// Panics if there are any duplicate entries.
    pub fn build(self) -> phf::Set<T> {
        phf::Set {
            map: self.map.build()
        }
    }
}

/// A builder for the `phf::OrderedMap` type.
pub struct OrderedMap<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
}

impl<K: Hash + PhfHash + Eq, V> OrderedMap<K, V> {
    /// Constructs a enw `phf::OrderedMap` builder.
    pub fn new() -> OrderedMap<K, V> {
        OrderedMap {
            keys: vec![],
            values: vec![],
        }
    }

    /// Adds an entry to the builder.
    pub fn entry(&mut self, key: K, value: V) -> &mut OrderedMap<K, V> {
        self.keys.push(key);
        self.values.push(value);
        self
    }

    /// Constructs a `phf::OrderedMap`.
    ///
    /// # Panics
    ///
    /// Panics if there are any duplicate keys.
    pub fn build(self) -> phf::OrderedMap<K, V> {
        {
            let mut set = HashSet::new();
            for key in &self.keys {
                if !set.insert(key) {
                    panic!("duplicate key");
                }
            }
        }

        let state = phf_generator::generate_hash(&self.keys);

        let entries = self.keys.into_iter().zip(self.values).collect();

        phf::OrderedMap {
            key: state.key,
            disps: phf::Slice::Dynamic(state.disps),
            idxs: phf::Slice::Dynamic(state.map),
            entries: phf::Slice::Dynamic(entries),
        }
    }
}

/// A builder for the `phf::OrderedSet` type.
pub struct OrderedSet<T> {
    map: OrderedMap<T, ()>,
}

impl<T: Hash + PhfHash + Eq> OrderedSet<T> {
    /// Constructs a new `phf::OrderedSet` builder.
    pub fn new() -> OrderedSet<T> {
        OrderedSet { map: OrderedMap::new() }
    }

    /// Adds an entry to the builder.
    pub fn entry(&mut self, entry: T) -> &mut OrderedSet<T> {
        self.map.entry(entry, ());
        self
    }

    /// Constructs a `phf::OrderedSet`.
    ///
    /// # Panics
    ///
    /// Panics if there are any duplicate entries.
    pub fn build(self) -> phf::OrderedSet<T> {
        phf::OrderedSet {
            map: self.map.build(),
        }
    }
}
