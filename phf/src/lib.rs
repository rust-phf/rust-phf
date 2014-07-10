//! Compile time optimized maps
#![crate_name="phf"]
#![doc(html_root_url="http://www.rust-ci.org/sfackler")]
#![crate_type="rlib"]
#![crate_type="dylib"]
#![warn(missing_doc)]

use std::fmt;
use std::hash::Hash;
use std::slice;
use std::collections::Collection;

#[path="../../phf_shared/phf_shared.rs"]
mod phf_shared;

/// An immutable map constructed at compile time.
///
/// `PhfMap`s may be created with the `phf_map` macro:
///
/// ```rust
/// # #![feature(phase)]
/// extern crate phf;
/// #[phase(syntax)]
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
pub struct PhfMap<K, V> {
    #[doc(hidden)]
    pub k1: u64,
    #[doc(hidden)]
    pub k2: u64,
    #[doc(hidden)]
    pub disps: &'static [(uint, uint)],
    #[doc(hidden)]
    pub entries: &'static [(K, V)],
}

impl<K, V> Collection for PhfMap<K, V> {
    fn len(&self) -> uint {
        self.entries.len()
    }
}

impl<'a, K: Hash+Eq, V> Map<K, V> for PhfMap<K, V> {
    fn find<'a>(&'a self, key: &K) -> Option<&'a V> {
        self.get_entry(key, |k| key == k).map(|e| {
            let &(_, ref v) = e;
            v
        })
    }
}

impl<K: fmt::Show, V: fmt::Show> fmt::Show for PhfMap<K, V> {
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

impl<K: Hash+Eq, V> PhfMap<K, V> {
    fn get_entry<'a, T: Hash>(&'a self, key: &T, check: |&K| -> bool)
                              -> Option<&'a (K, V)> {
        let (g, f1, f2) = phf_shared::hash(key, self.k1, self.k2);
        let (d1, d2) = self.disps[g % self.disps.len()];
        let entry @ &(ref s, _) = &self.entries[phf_shared::displace(f1, f2, d1, d2) %
                                                self.entries.len()];
        if check(s) {
            Some(entry)
        } else {
            None
        }
    }

    /// Returns a reference to the map's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    pub fn find_key<'a>(&'a self, key: &K) -> Option<&'a K> {
        self.get_entry(key, |k| key == k).map(|e| {
            let &(ref k, _) = e;
            k
        })
    }

    /// Like `find`, but can operate on any type that is equivalent to a key.
    pub fn find_equiv<'a, T: Hash+Equiv<K>>(&'a self, key: &T)
                                       -> Option<&'a V> {
        self.get_entry(key, |k| key.equiv(k)).map(|e| {
            let &(_, ref v) = e;
            v
        })
    }

    /// Like `find_key`, but can operate on any type that is equivalent to a
    /// key.
    pub fn find_key_equiv<'a, T: Hash+Equiv<K>>(&'a self, key: &T)
                                           -> Option<&'a K> {
        self.get_entry(key, |k| key.equiv(k)).map(|e| {
            let &(ref k, _) = e;
            k
        })
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
        PhfMapKeys { iter: self.entries() }
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Values are returned in an arbitrary but fixed order.
    pub fn values<'a>(&'a self) -> PhfMapValues<'a, K, V> {
        PhfMapValues { iter: self.entries() }
    }
}

/// An iterator over the key/value pairs in a `PhfMap`.
pub struct PhfMapEntries<'a, K, V> {
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

/// An iterator over the keys in a `PhfMap`.
pub struct PhfMapKeys<'a, K, V> {
    iter: PhfMapEntries<'a, K, V>,
}

impl<'a, K, V> Iterator<&'a K> for PhfMapKeys<'a, K, V> {
    fn next(&mut self) -> Option<&'a K> {
        self.iter.next().map(|&(ref key, _)| key)
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

/// An iterator over the values in a `PhfMap`.
pub struct PhfMapValues<'a, K, V> {
    iter: PhfMapEntries<'a, K, V>,
}

impl<'a, K, V> Iterator<&'a V> for PhfMapValues<'a, K, V> {
    fn next(&mut self) -> Option<&'a V> {
        self.iter.next().map(|&(_, ref value)| value)
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

/// An immutable set constructed at compile time.
///
/// `PhfSet`s may be created with the `phf_set` macro:
///
/// ```rust
/// # #![feature(phase)]
/// extern crate phf;
/// #[phase(syntax)]
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
pub struct PhfSet<T> {
    #[doc(hidden)]
    pub map: PhfMap<T, ()>
}

impl<T: fmt::Show> fmt::Show for PhfSet<T> {
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

impl<'a, T: Hash+Eq> Set<T> for PhfSet<T> {
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

impl<T: Hash+Eq> PhfSet<T> {
    /// Returns a reference to the set's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    #[inline]
    pub fn find_key<'a>(&'a self, key: &T) -> Option<&'a T> {
        self.map.find_key(key)
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
pub struct PhfSetValues<'a, T> {
    iter: PhfMapKeys<'a, T, ()>,
}

impl<'a, T> Iterator<&'a T> for PhfSetValues<'a, T> {
    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

/// An order-preserving immutable map constructed at compile time.
///
/// Unlike a `PhfMap`, the order of entries in a `PhfOrderedMap` is guaranteed
/// to be the order the entries were listed in.
///
/// `PhfOrderedMap`s may be created with the `phf_ordered_map` macro:
///
/// ```rust
/// # #![feature(phase)]
/// extern crate phf;
/// #[phase(syntax)]
/// extern crate phf_mac;
///
/// use phf::PhfOrderedMap;
///
/// static MY_MAP: PhfOrderedMap<int> = phf_ordered_map! {
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
pub struct PhfOrderedMap<T> {
    #[doc(hidden)]
    pub k1: u64,
    #[doc(hidden)]
    pub k2: u64,
    #[doc(hidden)]
    pub disps: &'static [(uint, uint)],
    #[doc(hidden)]
    pub idxs: &'static [uint],
    #[doc(hidden)]
    pub entries: &'static [(&'static str, T)],
}

impl<T: fmt::Show> fmt::Show for PhfOrderedMap<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, "{{"));
        let mut first = true;
        for (k, v) in self.entries() {
            if !first {
                try!(write!(fmt, ", "));
            }
            try!(write!(fmt, "{}: {}", k, v))
            first = false;
        }
        write!(fmt, "}}")
    }
}

impl<T> Collection for PhfOrderedMap<T> {
    fn len(&self) -> uint {
        self.entries.len()
    }
}

impl<'a, T> Map<&'a str, T> for PhfOrderedMap<T> {
    fn find<'a>(&'a self, key: & &str) -> Option<&'a T> {
        self.find_entry(key).map(|&(_, ref v)| v)
    }
}

impl<T> PhfOrderedMap<T> {
    fn find_entry(&self, key: & &str) -> Option<&'static (&'static str, T)> {
        let (g, f1, f2) = phf_shared::hash(key, self.k1, self.k2);
        let (d1, d2) = self.disps[g % self.disps.len()];
        let idx = self.idxs[phf_shared::displace(f1, f2, d1, d2) % self.idxs.len()];
        let entry @ &(s, _) = &self.entries[idx];

        if s == *key {
            Some(entry)
        } else {
            None
        }
    }

    /// Returns a reference to the map's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    pub fn find_key(&self, key: & &str) -> Option<&'static str> {
        self.find_entry(key).map(|&(s, _)| s)
    }

    /// Returns an iterator over the key/value pairs in the map.
    ///
    /// Entries are retuned in the same order in which they were defined.
    pub fn entries<'a>(&'a self) -> PhfOrderedMapEntries<'a, T> {
        PhfOrderedMapEntries { iter: self.entries.iter() }
    }

    /// Returns an iterator over the keys in the map.
    ///
    /// Keys are returned in the same order in which they were defined.
    pub fn keys<'a>(&'a self) -> PhfOrderedMapKeys<'a, T> {
        PhfOrderedMapKeys { iter: self.entries() }
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Values are returned in the same order in which they were defined.
    pub fn values<'a>(&'a self) -> PhfOrderedMapValues<'a, T> {
        PhfOrderedMapValues { iter: self.entries() }
    }
}

/// An iterator over the entries in a `PhfOrderedMap`.
pub struct PhfOrderedMapEntries<'a, T> {
    iter: slice::Items<'a, (&'static str, T)>,
}

impl<'a, T> Iterator<(&'static str, &'a T)> for PhfOrderedMapEntries<'a, T> {
    fn next(&mut self) -> Option<(&'static str, &'a T)> {
        self.iter.next().map(|&(key, ref value)| (key, value))
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator<(&'static str, &'a T)>
        for PhfOrderedMapEntries<'a, T> {
    fn next_back(&mut self) -> Option<(&'static str, &'a T)> {
        self.iter.next_back().map(|&(key, ref value)| (key, value))
    }
}

impl<'a, T> RandomAccessIterator<(&'static str, &'a T)>
        for PhfOrderedMapEntries<'a, T> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<(&'static str, &'a T)> {
        // FIXME: mozilla/rust#13167
        self.iter.idx(index).map(|pair| {
            let &(key, ref value) = pair;
            (key, value)
        })
    }
}

impl<'a, T> ExactSize<(&'static str, &'a T)> for PhfOrderedMapEntries<'a, T> {}

/// An iterator over the keys in a `PhfOrderedMap`.
pub struct PhfOrderedMapKeys<'a, T> {
    iter: PhfOrderedMapEntries<'a, T>,
}

impl<'a, T> Iterator<&'static str> for PhfOrderedMapKeys<'a, T> {
    fn next(&mut self) -> Option<&'static str> {
        self.iter.next().map(|(key, _)| key)
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator<&'static str> for PhfOrderedMapKeys<'a, T> {
    fn next_back(&mut self) -> Option<&'static str> {
        self.iter.next_back().map(|(key, _)| key)
    }
}

impl<'a, T> RandomAccessIterator<&'static str> for PhfOrderedMapKeys<'a, T> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'static str> {
        self.iter.idx(index).map(|(key, _)| key)
    }
}

impl<'a, T> ExactSize<&'static str> for PhfOrderedMapKeys<'a, T> {}

/// An iterator over the values in a `PhfOrderedMap`.
pub struct PhfOrderedMapValues<'a, T> {
    iter: PhfOrderedMapEntries<'a, T>,
}

impl<'a, T> Iterator<&'a T> for PhfOrderedMapValues<'a, T> {
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next().map(|(_, value)| value)
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator<&'a T> for PhfOrderedMapValues<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        self.iter.next_back().map(|(_, value)| value)
    }
}

impl<'a, T> RandomAccessIterator<&'a T> for PhfOrderedMapValues<'a, T> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a T> {
        self.iter.idx(index).map(|(_, value)| value)
    }
}

impl<'a, T> ExactSize<&'a T> for PhfOrderedMapValues<'a, T> {}

/// An order-preserving immutable set constructed at compile time.
///
/// Unlike a `PhfSet`, the order of entries in a `PhfOrderedSet` is guaranteed
/// to be the order the entries were listed in.
///
/// `PhfOrderedSet`s may be created with the `phf_ordered_set` macro:
///
/// ```rust
/// # #![feature(phase)]
/// extern crate phf;
/// #[phase(syntax)]
/// extern crate phf_mac;
///
/// use phf::PhfOrderedSet;
///
/// static MY_SET: PhfOrderedSet = phf_ordered_set! {
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
pub struct PhfOrderedSet {
    #[doc(hidden)]
    pub map: PhfOrderedMap<()>,
}

impl fmt::Show for PhfOrderedSet {
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

impl Collection for PhfOrderedSet {
    #[inline]
    fn len(&self) -> uint {
        self.map.len()
    }
}

impl<'a> Set<&'a str> for PhfOrderedSet {
    #[inline]
    fn contains(&self, value: & &'a str) -> bool {
        self.map.contains_key(value)
    }

    #[inline]
    fn is_disjoint(&self, other: &PhfOrderedSet) -> bool {
        !self.iter().any(|value| other.contains(&value))
    }

    #[inline]
    fn is_subset(&self, other: &PhfOrderedSet) -> bool {
        self.iter().all(|value| other.contains(&value))
    }
}

impl PhfOrderedSet {
    /// Returns a reference to the set's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    #[inline]
    pub fn find_key(&self, key: & &str) -> Option<&'static str> {
        self.map.find_key(key)
    }

    /// Returns an iterator over the values in the set.
    ///
    /// Values are returned in the same order in which they were defined.
    #[inline]
    pub fn iter<'a>(&'a self) -> PhfOrderedSetValues<'a> {
        PhfOrderedSetValues { iter: self.map.keys() }
    }
}

/// An iterator over the values in a `PhfOrderedSet`.
pub struct PhfOrderedSetValues<'a> {
    iter: PhfOrderedMapKeys<'a, ()>,
}

impl<'a> Iterator<&'static str> for PhfOrderedSetValues<'a> {
    #[inline]
    fn next(&mut self) -> Option<&'static str> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a> DoubleEndedIterator<&'static str> for PhfOrderedSetValues<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'static str> {
        self.iter.next_back()
    }
}

impl<'a> RandomAccessIterator<&'static str> for PhfOrderedSetValues<'a> {
    #[inline]
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    #[inline]
    fn idx(&mut self, index: uint) -> Option<&'static str> {
        self.iter.idx(index)
    }
}

impl<'a> ExactSize<&'static str> for PhfOrderedSetValues<'a> {}
