//! An order-preserving immutable map constructed at compile time.
use std::prelude::v1::*;
use std::borrow::Borrow;
use std::iter::{IntoIterator, RandomAccessIterator};
use std::ops::Index;
use std::fmt;
use std::slice;

use PhfHash;
use phf_shared;

/// An order-preserving immutable map constructed at compile time.
///
/// Unlike a `Map`, iteration order is guaranteed to match the definition
/// order.
///
/// ## Note
///
/// The fields of this struct are public so that they may be initialized by the
/// `phf_ordered_map!` macro and code generation. They are subject to change at
/// any time and should never be accessed directly.
pub struct OrderedMap<K:'static, V:'static> {
    #[doc(hidden)]
    pub key: u64,
    #[doc(hidden)]
    pub disps: &'static [(u32, u32)],
    #[doc(hidden)]
    pub idxs: &'static [usize],
    #[doc(hidden)]
    pub entries: &'static [(K, V)],
}

impl<K, V> fmt::Debug for OrderedMap<K, V> where K: fmt::Debug, V: fmt::Debug {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.entries().fold(fmt.debug_map(), |b, (k, v)| b.entry(k, v)).finish()
    }
}

impl<'a, K, V, T: ?Sized> Index<&'a T> for OrderedMap<K, V> where T: Eq + PhfHash, K: Borrow<T> {
    type Output = V;

    fn index(&self, k: &'a T) -> &V {
        self.get(k).expect("invalid key")
    }
}

impl<K, V> OrderedMap<K, V> {
    /// Returns the number of entries in the `Map`.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the `Map` is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the value that `key` maps to.
    pub fn get<T: ?Sized>(&self, key: &T) -> Option<&V> where T: Eq + PhfHash, K: Borrow<T> {
        self.get_entry(key).map(|e| e.1)
    }

    /// Returns a reference to the map's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    pub fn get_key<T: ?Sized>(&self, key: &T) -> Option<&K> where T: Eq + PhfHash, K: Borrow<T> {
        self.get_entry(key).map(|e| e.0)
    }

    /// Determines if `key` is in the `Map`.
    pub fn contains_key<T: ?Sized>(&self, key: &T) -> bool where T: Eq + PhfHash, K: Borrow<T> {
        self.get(key).is_some()
    }

    /// Returns the index of the key within the list used to initialize
    /// the ordered map.
    pub fn get_index<T: ?Sized>(&self, key: &T) -> Option<usize>
            where T: Eq + PhfHash, K: Borrow<T> {
        self.get_internal(key).map(|(i, _)| i)
    }

    /// Like `get`, but returns both the key and the value.
    pub fn get_entry<T: ?Sized>(&self, key: &T) -> Option<(&K, &V)>
            where T: Eq + PhfHash, K: Borrow<T> {
        self.get_internal(key).map(|(_, e)| e)
    }

    fn get_internal<T: ?Sized>(&self, key: &T) -> Option<(usize, (&K, &V))>
            where T: Eq + PhfHash, K: Borrow<T> {
        let (g, f1, f2) = key.phf_hash(self.key);
        let (d1, d2) = self.disps[(g % (self.disps.len() as u32)) as usize];
        let idx = self.idxs[(phf_shared::displace(f1, f2, d1, d2) % (self.idxs.len() as u32)) as usize];
        let entry = &self.entries[idx];

        let b: &T = entry.0.borrow();
        if b == key {
            Some((idx, (&entry.0, &entry.1)))
        } else {
            None
        }
    }

    /// Returns an iterator over the key/value pairs in the map.
    ///
    /// Entries are returned in the same order in which they were defined.
    pub fn entries<'a>(&'a self) -> Entries<'a, K, V> {
        Entries { iter: self.entries.iter() }
    }

    /// Returns an iterator over the keys in the map.
    ///
    /// Keys are returned in the same order in which they were defined.
    pub fn keys<'a>(&'a self) -> Keys<'a, K, V> {
        Keys { iter: self.entries() }
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Values are returned in the same order in which they were defined.
    pub fn values<'a>(&'a self) -> Values<'a, K, V> {
        Values { iter: self.entries() }
    }
}

impl<'a, K, V> IntoIterator for &'a OrderedMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Entries<'a, K, V>;

    fn into_iter(self) -> Entries<'a, K, V> {
        self.entries()
    }
}

/// An iterator over the entries in a `OrderedMap`.
pub struct Entries<'a, K:'a, V:'a> {
    iter: slice::Iter<'a, (K, V)>,
}

impl<'a, K, V> Iterator for Entries<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        self.iter.next().map(|e| (&e.0, &e.1))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for Entries<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a V)> {
        self.iter.next_back().map(|e| (&e.0, &e.1))
    }
}

impl<'a, K, V> RandomAccessIterator for Entries<'a, K, V> {
    fn indexable(&self) -> usize {
        self.iter.indexable()
    }

    fn idx(&mut self, index: usize) -> Option<(&'a K, &'a V)> {
        self.iter.idx(index).map(|e| (&e.0, &e.1))
    }
}

impl<'a, K, V> ExactSizeIterator for Entries<'a, K, V> {}

/// An iterator over the keys in a `OrderedMap`.
pub struct Keys<'a, K:'a, V:'a> {
    iter: Entries<'a, K, V>,
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<&'a K> {
        self.iter.next().map(|e| e.0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for Keys<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a K> {
        self.iter.next_back().map(|e| e.0)
    }
}

impl<'a, K, V> RandomAccessIterator for Keys<'a, K, V> {
    fn indexable(&self) -> usize {
        self.iter.indexable()
    }

    fn idx(&mut self, index: usize) -> Option<&'a K> {
        self.iter.idx(index).map(|e| e.0)
    }
}

impl<'a, K, V> ExactSizeIterator for Keys<'a, K, V> {}

/// An iterator over the values in a `OrderedMap`.
pub struct Values<'a, K:'a, V:'a> {
    iter: Entries<'a, K, V>,
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<&'a V> {
        self.iter.next().map(|e| e.1)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for Values<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a V> {
        self.iter.next_back().map(|e| e.1)
    }
}

impl<'a, K, V> RandomAccessIterator for Values<'a, K, V> {
    fn indexable(&self) -> usize {
        self.iter.indexable()
    }

    fn idx(&mut self, index: usize) -> Option<&'a V> {
        self.iter.idx(index).map(|e| e.1)
    }
}

impl<'a, K, V> ExactSizeIterator for Values<'a, K, V> {}
