#![feature(io)]

extern crate phf_shared;
extern crate phf_generator;

use std::io;
use std::io::prelude::*;
use std::collections::HashSet;
use std::hash::Hash;
use std::fmt;

pub use phf_shared::PhfHash;

pub struct Map<K: PhfHash> {
    keys: Vec<K>,
    values: Vec<String>,
}

impl<K: Hash+PhfHash+Eq+fmt::Display> Map<K> {
    pub fn new() -> Map<K> {
        Map {
            keys: vec![],
            values: vec![],
        }
    }

    pub fn entry(&mut self, key: K, value: &str) -> &mut Map<K> {
        self.keys.push(key);
        self.values.push(value.to_string());
        self
    }

    pub fn build<W: Write>(&self, w: &mut W) -> io::Result<()> {
        let mut set = HashSet::new();
        for key in &self.keys {
            if !set.insert(key) {
                panic!("duplicate key `{}`", key);
            }
        }

        let state = phf_generator::generate_hash(&self.keys);

        try!(write!(w, "phf::Map {{ key: {}, disps: &[", state.key));
        for &(d1, d2) in &state.disps {
            try!(write!(w, "({}, {}),", d1, d2));
        }
        try!(write!(w, "], entries: &["));
        for &idx in &state.map {
            try!(write!(w, "({}, {}),", &self.keys[idx], &self.values[idx]));
        }
        write!(w, "] }}")
    }
}

pub struct Set<T: PhfHash> {
    map: Map<T>
}

impl<T: Hash+PhfHash+Eq+fmt::Display> Set<T> {
    pub fn new() -> Set<T> {
        Set {
            map: Map::new()
        }
    }

    pub fn entry(&mut self, entry: T) -> &mut Set<T> {
        self.map.entry(entry, "()");
        self
    }

    pub fn build<W: Write>(&self, w: &mut W) -> io::Result<()> {
        try!(write!(w, "phf::Set {{ map: "));
        try!(self.map.build(w));
        write!(w, " }}")
    }
}

pub struct OrderedMap<K: PhfHash> {
    keys: Vec<K>,
    values: Vec<String>,
}

impl<K: Hash+PhfHash+Eq+fmt::Display> OrderedMap<K> {
    pub fn new() -> OrderedMap<K> {
        OrderedMap {
            keys: vec![],
            values: vec![],
        }
    }

    pub fn entry(&mut self, key: K, value: &str) -> &mut OrderedMap<K> {
        self.keys.push(key);
        self.values.push(value.to_string());
        self
    }

    pub fn build<W: Write>(&self, w: &mut W) -> io::Result<()> {
        let mut set = HashSet::new();
        for key in &self.keys {
            if !set.insert(key) {
                panic!("duplicate key `{}`", key);
            }
        }

        let state = phf_generator::generate_hash(&self.keys);

        try!(write!(w, "phf::OrderedMap {{ key: {}, disps: &[", state.key));
        for &(d1, d2) in &state.disps {
            try!(write!(w, "({}, {}),", d1, d2));
        }
        try!(write!(w, "], idxs: &["));
        for &idx in &state.map {
            try!(write!(w, "{},", idx));
        }
        try!(write!(w, "], entries: &["));
        for (key, value) in self.keys.iter().zip(self.values.iter()) {
            try!(write!(w, "({}, {}),", key, value));
        }
        write!(w, "] }}")
    }
}

pub struct OrderedSet<T: PhfHash> {
    map: OrderedMap<T>
}

impl<T: Hash+PhfHash+Eq+fmt::Display> OrderedSet<T> {
    pub fn new() -> OrderedSet<T> {
        OrderedSet {
            map: OrderedMap::new()
        }
    }

    pub fn entry(&mut self, entry: T) -> &mut OrderedSet<T> {
        self.map.entry(entry, "()");
        self
    }

    pub fn build<W: Write>(&self, w: &mut W) -> io::Result<()> {
        try!(write!(w, "phf::OrderedSet {{ map: "));
        try!(self.map.build(w));
        write!(w, " }}")
    }
}
