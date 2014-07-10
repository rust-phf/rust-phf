//! Compile time optimized maps
#![crate_name="phf"]
#![doc(html_root_url="http://www.rust-ci.org/sfackler")]
#![crate_type="rlib"]
#![crate_type="dylib"]
#![warn(missing_doc)]

use std::fmt;
use std::hash::{Hash, Hasher};
use std::hash::sip::SipHasher;
use std::slice;
use std::collections::Collection;

static LOG_MAX_SIZE: uint = 21;

#[doc(hidden)]
pub static MAX_SIZE: uint = 1 << LOG_MAX_SIZE;

#[doc(hidden)]
#[inline]
pub fn hash<T: Hash>(s: &T, k1: u64, k2: u64) -> (uint, uint, uint) {
    let hash = SipHasher::new_with_keys(k1, k2).hash(s);
    let mask = (MAX_SIZE - 1) as u64;

    ((hash & mask) as uint,
     ((hash >> LOG_MAX_SIZE) & mask) as uint,
     ((hash >> (2 * LOG_MAX_SIZE)) & mask) as uint)
}

#[doc(hidden)]
#[inline]
pub fn displace(f1: uint, f2: uint, d1: uint, d2: uint) -> uint {
    d2 + f1 * d1 + f2
}

/// An immutable map constructed at compile time.
///
/// Keys may be either string literals or binary string literals.
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
        let (g, f1, f2) = hash(key, self.k1, self.k2);
        let (d1, d2) = self.disps[g % self.disps.len()];
        let entry @ &(ref s, _) = &self.entries[displace(f1, f2, d1, d2) %
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
/// Values may be either string literals or binary string literals.
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
/// Keys may be either string literals or binary string literals.
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
pub struct PhfOrderedMap<K, V> {
    #[doc(hidden)]
    pub k1: u64,
    #[doc(hidden)]
    pub k2: u64,
    #[doc(hidden)]
    pub disps: &'static [(uint, uint)],
    #[doc(hidden)]
    pub idxs: &'static [uint],
    #[doc(hidden)]
    pub entries: &'static [(K, V)],
}

impl<K: fmt::Show, V: fmt::Show> fmt::Show for PhfOrderedMap<K, V> {
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

impl<'a, K: Hash+Eq, V> Map<K, V> for PhfOrderedMap<K, V> {
    fn find<'a>(&'a self, key: &K) -> Option<&'a V> {
        self.find_entry(key).map(|e| {
            let &(_, ref v) = e;
            v
        })
    }
}

impl<K: Hash+Eq, V> PhfOrderedMap<K, V> {
    fn find_entry<'a>(&'a self, key: &K) -> Option<&'a (K, V)> {
        let (g, f1, f2) = hash(key, self.k1, self.k2);
        let (d1, d2) = self.disps[g % self.disps.len()];
        let idx = self.idxs[displace(f1, f2, d1, d2) % self.idxs.len()];
        let entry @ &(ref s, _) = &self.entries[idx];

        if s == key {
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
        self.find_entry(key).map(|e| {
            let &(ref k, _) = e;
            k
        })
    }
}

impl<K, V> PhfOrderedMap<K, V> {
    /// Returns an iterator over the key/value pairs in the map.
    ///
    /// Entries are retuned in the same order in which they were defined.
    pub fn entries<'a>(&'a self) -> PhfOrderedMapEntries<'a, K, V> {
        PhfOrderedMapEntries { iter: self.entries.iter() }
    }

    /// Returns an iterator over the keys in the map.
    ///
    /// Keys are returned in the same order in which they were defined.
    pub fn keys<'a>(&'a self) -> PhfOrderedMapKeys<'a, K, V> {
        PhfOrderedMapKeys { iter: self.entries() }
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Values are returned in the same order in which they were defined.
    pub fn values<'a>(&'a self) -> PhfOrderedMapValues<'a, K, V> {
        PhfOrderedMapValues { iter: self.entries() }
    }
}

/// An iterator over the entries in a `PhfOrderedMap`.
pub struct PhfOrderedMapEntries<'a, K, V> {
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

impl<'a, K, V> DoubleEndedIterator<&'a (K, V)>
        for PhfOrderedMapEntries<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a (K, V)> {
        self.iter.next_back()
    }
}

impl<'a, K, V> RandomAccessIterator<&'a (K, V)>
        for PhfOrderedMapEntries<'a, K, V> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a (K, V)> {
        self.iter.idx(index)
    }
}

impl<'a, K, V> ExactSize<&'a (K, V)> for PhfOrderedMapEntries<'a, K, V> {}

/// An iterator over the keys in a `PhfOrderedMap`.
pub struct PhfOrderedMapKeys<'a, K, V> {
    iter: PhfOrderedMapEntries<'a, K, V>,
}

impl<'a, K, V> Iterator<&'a K> for PhfOrderedMapKeys<'a, K, V> {
    fn next(&mut self) -> Option<&'a K> {
        self.iter.next().map(|&(ref key, _)| key)
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a K> for PhfOrderedMapKeys<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a K> {
        self.iter.next_back().map(|&(ref key, _)| key)
    }
}

impl<'a, K, V> RandomAccessIterator<&'a K> for PhfOrderedMapKeys<'a, K, V> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a K> {
        self.iter.idx(index).map(|&(ref key, _)| key)
    }
}

impl<'a, K, V> ExactSize<&'a K> for PhfOrderedMapKeys<'a, K, V> {}

/// An iterator over the values in a `PhfOrderedMap`.
pub struct PhfOrderedMapValues<'a, K, V> {
    iter: PhfOrderedMapEntries<'a, K, V>,
}

impl<'a, K, V> Iterator<&'a V> for PhfOrderedMapValues<'a, K, V> {
    fn next(&mut self) -> Option<&'a V> {
        self.iter.next().map(|&(_, ref value)| value)
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a V> for PhfOrderedMapValues<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a V> {
        self.iter.next_back().map(|&(_, ref value)| value)
    }
}

impl<'a, K, V> RandomAccessIterator<&'a V> for PhfOrderedMapValues<'a, K, V> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a V> {
        self.iter.idx(index).map(|&(_, ref value)| value)
    }
}

impl<'a, K, V> ExactSize<&'a V> for PhfOrderedMapValues<'a, K, V> {}

/// An order-preserving immutable set constructed at compile time.
///
/// Values may be either string literals or binary string literals.
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
pub struct PhfOrderedSet<T> {
    #[doc(hidden)]
    pub map: PhfOrderedMap<T, ()>,
}

impl<T: fmt::Show> fmt::Show for PhfOrderedSet<T> {
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

impl<T: Hash+Eq> Set<T> for PhfOrderedSet<T> {
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

impl<T: Hash+Eq> PhfOrderedSet<T> {
    /// Returns a reference to the set's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    #[inline]
    pub fn find_key<'a>(&'a self, key: &T) -> Option<&'a T> {
        self.map.find_key(key)
    }
}

impl<T> PhfOrderedSet<T> {
    /// Returns an iterator over the values in the set.
    ///
    /// Values are returned in the same order in which they were defined.
    #[inline]
    pub fn iter<'a>(&'a self) -> PhfOrderedSetValues<'a, T> {
        PhfOrderedSetValues { iter: self.map.keys() }
    }
}

/// An iterator over the values in a `PhfOrderedSet`.
pub struct PhfOrderedSetValues<'a, T> {
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
