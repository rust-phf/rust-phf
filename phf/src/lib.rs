//! Compile time optimized maps
//!
//! Keys can be string literals, byte string literals, byte literals, char
//! literals, or any of the fixed-size integral types.
#![doc(html_root_url="https://sfackler.github.io/doc")]
#![warn(missing_doc)]
#![feature(macro_rules, tuple_indexing)]

use std::fmt;
use std::iter;
use std::slice;
use std::collections::Collection;

pub use shared::PhfHash;

#[path="../../shared/mod.rs"]
mod shared;

/// An immutable map constructed at compile time.
///
/// `PhfMap`s may be created with the `phf_map` macro:
///
/// ```rust
/// # #![feature(phase)]
/// extern crate phf;
/// #[phase(plugin)]
/// extern crate phf_mac;
///
/// use phf::PhfMap;
///
/// static MY_MAP: PhfMap<&'static str, int> = phf_map! {
///    "hello" => 10,
///    "world" => 11,
/// };
///
/// # fn main() {}
/// ```
///
/// # Note
///
/// The fields of this struct are public so that they may be initialized by the
/// `phf_map` macro. They are subject to change at any time and should never
/// be accessed directly.
pub struct PhfMap<K:'static, V:'static> {
    #[doc(hidden)]
    pub key: u64,
    #[doc(hidden)]
    pub disps: &'static [(u32, u32)],
    #[doc(hidden)]
    pub entries: &'static [(K, V)],
}

impl<K, V> Collection for PhfMap<K, V> {
    fn len(&self) -> uint {
        self.entries.len()
    }
}

impl<'a, K, V> Map<K, V> for PhfMap<K, V> where K: PhfHash+Eq {
    fn find(&self, key: &K) -> Option<&V> {
        self.get_entry(key, |k| key == k).map(|e| &e.1)
    }
}

impl<K, V> fmt::Show for PhfMap<K, V> where K: fmt::Show, V: fmt::Show {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, "{{"));
        let mut first = true;
        for &(ref k, ref v) in self.entries() {
            if !first {
                try!(write!(fmt, ", "));
            }
            try!(write!(fmt, "{}: {}", k, v))
            first = false;
        }
        write!(fmt, "}}")
    }
}

impl<K, V> Index<K, V> for PhfMap<K, V> where K: PhfHash+Eq {
    fn index(&self, k: &K) -> &V {
        self.find(k).expect("invalid key")
    }
}

impl<K, V> PhfMap<K, V> where K: PhfHash+Eq {
    /// Returns a reference to the map's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    pub fn find_key(&self, key: &K) -> Option<&K> {
        self.get_entry(key, |k| key == k).map(|e| &e.0)
    }
}

impl<K, V> PhfMap<K, V> {
    fn get_entry<T>(&self, key: &T, check: |&K| -> bool) -> Option<&(K, V)> where T: PhfHash {
        let (g, f1, f2) = key.phf_hash(self.key);
        let (d1, d2) = self.disps[(g % (self.disps.len() as u32)) as uint];
        let entry = &self.entries[(shared::displace(f1, f2, d1, d2) % (self.entries.len() as u32))
                                  as uint];
        if check(&entry.0) {
            Some(entry)
        } else {
            None
        }
    }

    /// Like `find`, but can operate on any type that is equivalent to a key.
    pub fn find_equiv<T>(&self, key: &T) -> Option<&V> where T: PhfHash+Equiv<K> {
        self.get_entry(key, |k| key.equiv(k)).map(|e| &e.1)
    }

    /// Like `find_key`, but can operate on any type that is equivalent to a
    /// key.
    pub fn find_key_equiv<T>(&self, key: &T) -> Option<&K> where T: PhfHash+Equiv<K> {
        self.get_entry(key, |k| key.equiv(k)).map(|e| &e.0)
    }
}

impl<K, V> PhfMap<K, V> {
    /// Returns an iterator over the key/value pairs in the map.
    ///
    /// Entries are retuned in an arbitrary but fixed order.
    pub fn entries<'a>(&'a self) -> PhfMapEntries<'a, K, V> {
        PhfMapEntries { iter: self.entries.iter() }
    }

    /// Returns an iterator over the keys in the map.
    ///
    /// Keys are returned in an arbitrary but fixed order.
    pub fn keys<'a>(&'a self) -> PhfMapKeys<'a, K, V> {
        PhfMapKeys { iter: self.entries().map(|e| &e.0) }
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Values are returned in an arbitrary but fixed order.
    pub fn values<'a>(&'a self) -> PhfMapValues<'a, K, V> {
        PhfMapValues { iter: self.entries().map(|e | &e.1) }
    }
}

/// An iterator over the key/value pairs in a `PhfMap`.
pub struct PhfMapEntries<'a, K:'a, V:'a> {
    iter: slice::Items<'a, (K, V)>,
}

impl<'a, K, V> Iterator<&'a (K, V)> for PhfMapEntries<'a, K, V> {
    fn next(&mut self) -> Option<&'a (K, V)> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a (K, V)> for PhfMapEntries<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a (K, V)> {
        self.iter.next_back()
    }
}

impl<'a, K, V> ExactSize<&'a (K, V)> for PhfMapEntries<'a, K, V> {}

/// An iterator over the keys in a `PhfMap`.
pub struct PhfMapKeys<'a, K:'a, V:'a> {
    iter: iter::Map<'a, &'a (K, V), &'a K, PhfMapEntries<'a, K, V>>,
}

impl<'a, K, V> Iterator<&'a K> for PhfMapKeys<'a, K, V> {
    fn next(&mut self) -> Option<&'a K> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a K> for PhfMapKeys<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a K> {
        self.iter.next_back()
    }
}

impl<'a, K, V> ExactSize<&'a K> for PhfMapKeys<'a, K, V> {}

/// An iterator over the values in a `PhfMap`.
pub struct PhfMapValues<'a, K:'a, V:'a> {
    iter: iter::Map<'a, &'a (K, V), &'a V, PhfMapEntries<'a, K, V>>,
}

impl<'a, K, V> Iterator<&'a V> for PhfMapValues<'a, K, V> {
    fn next(&mut self) -> Option<&'a V> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a V> for PhfMapValues<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a V> {
        self.iter.next_back()
    }
}

impl<'a, K, V> ExactSize<&'a V> for PhfMapValues<'a, K, V> {}

/// An immutable set constructed at compile time.
///
/// `PhfSet`s may be created with the `phf_set` macro:
///
/// ```rust
/// # #![feature(phase)]
/// extern crate phf;
/// #[phase(plugin)]
/// extern crate phf_mac;
///
/// use phf::PhfSet;
///
/// static MY_SET: PhfSet<&'static str> = phf_set! {
///    "hello",
///    "world",
/// };
///
/// # fn main() {}
/// ```
///
/// # Note
///
/// The fields of this struct are public so that they may be initialized by the
/// `phf_set` macro. They are subject to change at any time and should never be
/// accessed directly.
pub struct PhfSet<T:'static> {
    #[doc(hidden)]
    pub map: PhfMap<T, ()>
}

impl<T> fmt::Show for PhfSet<T> where T: fmt::Show {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, "{{"));
        let mut first = true;
        for entry in self.iter() {
            if !first {
                try!(write!(fmt, ", "));
            }
            try!(write!(fmt, "{}", entry));
            first = false;
        }
        write!(fmt, "}}")
    }
}

impl<T> Collection for PhfSet<T> {
    #[inline]
    fn len(&self) -> uint {
        self.map.len()
    }
}

impl<T> Set<T> for PhfSet<T> where T: PhfHash+Eq {
    #[inline]
    fn contains(&self, value: &T) -> bool {
        self.map.contains_key(value)
    }

    #[inline]
    fn is_disjoint(&self, other: &PhfSet<T>) -> bool {
        !self.iter().any(|value| other.contains(value))
    }

    #[inline]
    fn is_subset(&self, other: &PhfSet<T>) -> bool {
        self.iter().all(|value| other.contains(value))
    }
}

impl<T> PhfSet<T> where T: PhfHash+Eq {
    /// Returns a reference to the set's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    #[inline]
    pub fn find_key(&self, key: &T) -> Option<&T> {
        self.map.find_key(key)
    }
}

impl<T> PhfSet<T> {
    /// Like `contains`, but can operate on any type that is equivalent to a
    /// value
    #[inline]
    pub fn contains_equiv<U>(&self, key: &U) -> bool where U: PhfHash+Equiv<T> {
        self.map.find_equiv(key).is_some()
    }

    /// Like `find_key`, but can operate on any type that is equivalent to a
    /// value
    #[inline]
    pub fn find_key_equiv<U>(&self, key: &U) -> Option<&T> where U: PhfHash+Equiv<T> {
        self.map.find_key_equiv(key)
    }
}

impl<T> PhfSet<T> {
    /// Returns an iterator over the values in the set.
    ///
    /// Values are returned in an arbitrary but fixed order.
    #[inline]
    pub fn iter<'a>(&'a self) -> PhfSetValues<'a, T> {
        PhfSetValues { iter: self.map.keys() }
    }
}

/// An iterator over the values in a `PhfSet`.
pub struct PhfSetValues<'a, T:'static> {
    iter: PhfMapKeys<'a, T, ()>,
}

impl<'a, T> Iterator<&'a T> for PhfSetValues<'a, T> {
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator<&'a T> for PhfSetValues<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        self.iter.next_back()
    }
}

impl<'a, T> ExactSize<&'a T> for PhfSetValues<'a, T> {}

/// An order-preserving immutable map constructed at compile time.
///
/// Unlike a `PhfMap`, iteration order is guaranteed to match the definition
/// order.
///
/// `PhfOrderedMap`s may be created with the `phf_ordered_map` macro:
///
/// ```rust
/// # #![feature(phase)]
/// extern crate phf;
/// #[phase(plugin)]
/// extern crate phf_mac;
///
/// use phf::PhfOrderedMap;
///
/// static MY_MAP: PhfOrderedMap<&'static str, int> = phf_ordered_map! {
///    "hello" => 10,
///    "world" => 11,
/// };
///
/// # fn main() {}
/// ```
///
/// # Note
///
/// The fields of this struct are public so that they may be initialized by the
/// `phf_ordered_map` macro. They are subject to change at any time and should
/// never be accessed directly.
pub struct PhfOrderedMap<K:'static, V:'static> {
    #[doc(hidden)]
    pub key: u64,
    #[doc(hidden)]
    pub disps: &'static [(u32, u32)],
    #[doc(hidden)]
    pub idxs: &'static [uint],
    #[doc(hidden)]
    pub entries: &'static [(K, V)],
}

impl<K, V> fmt::Show for PhfOrderedMap<K, V> where K: fmt::Show, V: fmt::Show {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, "{{"));
        let mut first = true;
        for &(ref k, ref v) in self.entries() {
            if !first {
                try!(write!(fmt, ", "));
            }
            try!(write!(fmt, "{}: {}", k, v))
            first = false;
        }
        write!(fmt, "}}")
    }
}

impl<K, V> Collection for PhfOrderedMap<K, V> {
    fn len(&self) -> uint {
        self.entries.len()
    }
}

impl<K, V> Map<K, V> for PhfOrderedMap<K, V> where K: PhfHash+Eq {
    fn find(&self, key: &K) -> Option<&V> {
        self.find_entry(key, |k| k == key).map(|(_, e)| &e.1)
    }
}

impl<K, V> Index<K, V> for PhfOrderedMap<K, V> where K: PhfHash+Eq {
    fn index(&self, k: &K) -> &V {
        self.find(k).expect("invalid key")
    }
}

impl<K, V> PhfOrderedMap<K, V> where K: PhfHash+Eq {
    /// Returns a reference to the map's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    pub fn find_key(&self, key: &K) -> Option<&K> {
        self.find_entry(key, |k| k == key).map(|(_, e)| &e.0)
    }

    /// Returns the index of the key within the list used to initialize
    /// the ordered map.
    pub fn find_index(&self, key: &K) -> Option<uint> {
        self.find_entry(key, |k| k == key).map(|(i, _)| i)
    }
}

impl<K, V> PhfOrderedMap<K, V> {
    fn find_entry<T>(&self, key: &T, check: |&K| -> bool) -> Option<(uint, &(K, V))>
            where T: PhfHash {
        let (g, f1, f2) = key.phf_hash(self.key);
        let (d1, d2) = self.disps[(g % (self.disps.len() as u32)) as uint];
        let idx = self.idxs[(shared::displace(f1, f2, d1, d2) % (self.idxs.len() as u32)) as uint];
        let entry = &self.entries[idx];

        if check(&entry.0) {
            Some((idx, entry))
        } else {
            None
        }
    }

    /// Like `find`, but can operate on any type that is equivalent to a key.
    pub fn find_equiv<T>(&self, key: &T) -> Option<&V> where T: PhfHash+Equiv<K> {
        self.find_entry(key, |k| key.equiv(k)).map(|(_, e)| &e.1)
    }

    /// Like `find_key`, but can operate on any type that is equivalent to a
    /// key.
    pub fn find_key_equiv<T>(&self, key: &T) -> Option<&K> where T: PhfHash+Equiv<K> {
        self.find_entry(key, |k| key.equiv(k)).map(|(_, e)| &e.0)
    }

    /// Like `find_index`, but can operate on any type that is equivalent to a
    /// key.
    pub fn find_index_equiv<T>(&self, key: &T) -> Option<uint> where T: PhfHash+Equiv<K> {
        self.find_entry(key, |k| key.equiv(k)).map(|(i, _)| i)
    }

    /// Returns an iterator over the key/value pairs in the map.
    ///
    /// Entries are returned in the same order in which they were defined.
    pub fn entries<'a>(&'a self) -> PhfOrderedMapEntries<'a, K, V> {
        PhfOrderedMapEntries { iter: self.entries.iter() }
    }

    /// Returns an iterator over the keys in the map.
    ///
    /// Keys are returned in the same order in which they were defined.
    pub fn keys<'a>(&'a self) -> PhfOrderedMapKeys<'a, K, V> {
        PhfOrderedMapKeys { iter: self.entries().map(|e| &e.0) }
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Values are returned in the same order in which they were defined.
    pub fn values<'a>(&'a self) -> PhfOrderedMapValues<'a, K, V> {
        PhfOrderedMapValues { iter: self.entries().map(|e| &e.1) }
    }
}

/// An iterator over the entries in a `PhfOrderedMap`.
pub struct PhfOrderedMapEntries<'a, K:'a, V:'a> {
    iter: slice::Items<'a, (K, V)>,
}

impl<'a, K, V> Iterator<&'a (K, V)> for PhfOrderedMapEntries<'a, K, V> {
    fn next(&mut self) -> Option<&'a (K, V)> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a (K, V)> for PhfOrderedMapEntries<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a (K, V)> {
        self.iter.next_back()
    }
}

impl<'a, K, V> RandomAccessIterator<&'a (K, V)> for PhfOrderedMapEntries<'a, K, V> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a (K, V)> {
        self.iter.idx(index)
    }
}

impl<'a, K, V> ExactSize<&'a (K, V)> for PhfOrderedMapEntries<'a, K, V> {}

/// An iterator over the keys in a `PhfOrderedMap`.
pub struct PhfOrderedMapKeys<'a, K:'a, V:'a> {
    iter: iter::Map<'a, &'a (K, V), &'a K, PhfOrderedMapEntries<'a, K, V>>,
}

impl<'a, K, V> Iterator<&'a K> for PhfOrderedMapKeys<'a, K, V> {
    fn next(&mut self) -> Option<&'a K> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a K> for PhfOrderedMapKeys<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a K> {
        self.iter.next_back()
    }
}

impl<'a, K, V> RandomAccessIterator<&'a K> for PhfOrderedMapKeys<'a, K, V> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a K> {
        self.iter.idx(index)
    }
}

impl<'a, K, V> ExactSize<&'a K> for PhfOrderedMapKeys<'a, K, V> {}

/// An iterator over the values in a `PhfOrderedMap`.
pub struct PhfOrderedMapValues<'a, K:'a, V:'a> {
    iter: iter::Map<'a, &'a (K, V), &'a V, PhfOrderedMapEntries<'a, K, V>>,
}

impl<'a, K, V> Iterator<&'a V> for PhfOrderedMapValues<'a, K, V> {
    fn next(&mut self) -> Option<&'a V> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a V> for PhfOrderedMapValues<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a V> {
        self.iter.next_back()
    }
}

impl<'a, K, V> RandomAccessIterator<&'a V> for PhfOrderedMapValues<'a, K, V> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a V> {
        self.iter.idx(index)
    }
}

impl<'a, K, V> ExactSize<&'a V> for PhfOrderedMapValues<'a, K, V> {}

/// An order-preserving immutable set constructed at compile time.
///
/// Unlike a `PhfSet`, iteration order is guaranteed to match the definition
/// order.
///
/// `PhfOrderedSet`s may be created with the `phf_ordered_set` macro:
///
/// ```rust
/// # #![feature(phase)]
/// extern crate phf;
/// #[phase(plugin)]
/// extern crate phf_mac;
///
/// use phf::PhfOrderedSet;
///
/// static MY_SET: PhfOrderedSet<&'static str> = phf_ordered_set! {
///    "hello",
///    "world",
/// };
///
/// # fn main() {}
/// ```
///
/// # Note
///
/// The fields of this struct are public so that they may be initialized by the
/// `phf_ordered_set` macro. They are subject to change at any time and should
/// never be accessed directly.
pub struct PhfOrderedSet<T:'static> {
    #[doc(hidden)]
    pub map: PhfOrderedMap<T, ()>,
}

impl<T> fmt::Show for PhfOrderedSet<T> where T: fmt::Show {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, "{{"));
        let mut first = true;
        for entry in self.iter() {
            if !first {
                try!(write!(fmt, ", "));
            }
            try!(write!(fmt, "{}", entry));
            first = false;
        }
        write!(fmt, "}}")
    }
}

impl<T> Collection for PhfOrderedSet<T> {
    #[inline]
    fn len(&self) -> uint {
        self.map.len()
    }
}

impl<T> Set<T> for PhfOrderedSet<T> where T: PhfHash+Eq {
    #[inline]
    fn contains(&self, value: &T) -> bool {
        self.map.contains_key(value)
    }

    #[inline]
    fn is_disjoint(&self, other: &PhfOrderedSet<T>) -> bool {
        !self.iter().any(|value| other.contains(value))
    }

    #[inline]
    fn is_subset(&self, other: &PhfOrderedSet<T>) -> bool {
        self.iter().all(|value| other.contains(value))
    }
}

impl<T: PhfHash+Eq> PhfOrderedSet<T> {
    /// Returns a reference to the set's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    #[inline]
    pub fn find_key(&self, key: &T) -> Option<&T> {
        self.map.find_key(key)
    }

    /// Returns the index of the key within the list used to initialize
    /// the ordered set.
    pub fn find_index(&self, key: &T) -> Option<uint> {
        self.map.find_index(key)
    }
}

impl<T> PhfOrderedSet<T> {
    /// Like `contains`, but can operate on any type that is equivalent to a
    /// value
    #[inline]
    pub fn contains_equiv<U>(&self, key: &U) -> bool where U: PhfHash+Equiv<T> {
        self.map.find_equiv(key).is_some()
    }

    /// Like `find_key`, but can operate on any type that is equivalent to a
    /// value
    #[inline]
    pub fn find_key_equiv<U>(&self, key: &U) -> Option<&T> where U: PhfHash+Equiv<T> {
        self.map.find_key_equiv(key)
    }

    /// Like `find_index`, but can operate on any type that is equivalent to a
    /// key.
    pub fn find_index_equiv<U>(&self, key: &U) -> Option<uint> where U: PhfHash+Equiv<T> {
        self.map.find_index_equiv(key)
    }

    /// Returns an iterator over the values in the set.
    ///
    /// Values are returned in the same order in which they were defined.
    #[inline]
    pub fn iter<'a>(&'a self) -> PhfOrderedSetValues<'a, T> {
        PhfOrderedSetValues { iter: self.map.keys() }
    }
}

/// An iterator over the values in a `PhfOrderedSet`.
pub struct PhfOrderedSetValues<'a, T:'a> {
    iter: PhfOrderedMapKeys<'a, T, ()>,
}

impl<'a, T> Iterator<&'a T> for PhfOrderedSetValues<'a, T> {
    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator<&'a T> for PhfOrderedSetValues<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a T> {
        self.iter.next_back()
    }
}

impl<'a, T> RandomAccessIterator<&'a T> for PhfOrderedSetValues<'a, T> {
    #[inline]
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    #[inline]
    fn idx(&mut self, index: uint) -> Option<&'a T> {
        self.iter.idx(index)
    }
}

impl<'a, T> ExactSize<&'a T> for PhfOrderedSetValues<'a, T> {}
