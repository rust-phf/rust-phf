//! An order-preserving immutable map constructed at compile time.
use core::prelude::*;
use core::borrow::BorrowFrom;
use core::fmt;
use core::slice;

use PhfHash;
use phf_shared;

/// An order-preserving immutable map constructed at compile time.
///
/// Unlike a `Map`, iteration order is guaranteed to match the definition
/// order.
///
/// `OrderedMap`s may be created with the `phf_ordered_map` macro:
///
/// ```rust
/// # #![feature(phase)]
/// extern crate phf;
/// #[phase(plugin)]
/// extern crate phf_mac;
///
/// static MY_MAP: phf::OrderedMap<&'static str, int> = phf_ordered_map! {
///    "hello" => 10,
///    "world" => 11,
/// };
///
/// # fn main() {}
/// ```
///
/// ## Note
///
/// The fields of this struct are public so that they may be initialized by the
/// `phf_ordered_map` macro. They are subject to change at any time and should
/// never be accessed directly.
pub struct OrderedMap<K:'static, V:'static> {
    #[doc(hidden)]
    pub key: u64,
    #[doc(hidden)]
    pub disps: &'static [(u32, u32)],
    #[doc(hidden)]
    pub idxs: &'static [uint],
    #[doc(hidden)]
    pub entries: &'static [(K, V)],
}

impl<K, V> fmt::Show for OrderedMap<K, V> where K: fmt::Show, V: fmt::Show {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, "{{"));
        let mut first = true;
        for (k, v) in self.entries() {
            if !first {
                try!(write!(fmt, ", "));
            }
            try!(write!(fmt, "{}: {}", k, v));
            first = false;
        }
        write!(fmt, "}}")
    }
}

impl<K, V, Sized? T> Index<T, V> for OrderedMap<K, V> where T: Eq + PhfHash + BorrowFrom<K> {
    fn index(&self, k: &T) -> &V {
        self.get(k).expect("invalid key")
    }
}

impl<K, V> OrderedMap<K, V> {
    /// Returns the number of entries in the `Map`.
    pub fn len(&self) -> uint {
        self.entries.len()
    }

    /// Returns true if the `Map` is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the value that `key` maps to.
    pub fn get<Sized? T>(&self, key: &T) -> Option<&V> where T: Eq + PhfHash + BorrowFrom<K> {
        self.get_entry(key).map(|e| e.1)
    }

    /// Returns a reference to the map's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    pub fn get_key<Sized? T>(&self, key: &T) -> Option<&K> where T: Eq + PhfHash + BorrowFrom<K> {
        self.get_entry(key).map(|e| e.0)
    }

    /// Determines if `key` is in the `Map`.
    pub fn contains_key<Sized? T>(&self, key: &T) -> bool where T: Eq + PhfHash + BorrowFrom<K> {
        self.get(key).is_some()
    }

    /// Returns the index of the key within the list used to initialize
    /// the ordered map.
    pub fn get_index<Sized? T>(&self, key: &T) -> Option<uint>
            where T: Eq + PhfHash + BorrowFrom<K> {
        self.get_internal(key).map(|(i, _)| i)
    }

    /// Like `get`, but returns both the key and the value.
    pub fn get_entry<Sized? T>(&self, key: &T) -> Option<(&K, &V)>
            where T: Eq + PhfHash + BorrowFrom<K> {
        self.get_internal(key).map(|(_, e)| e)
    }

    fn get_internal<Sized? T>(&self, key: &T) -> Option<(uint, (&K, &V))>
            where T: Eq + PhfHash + BorrowFrom<K> {
        let (g, f1, f2) = key.phf_hash(self.key);
        let (d1, d2) = self.disps[(g % (self.disps.len() as u32)) as uint];
        let idx = self.idxs[(phf_shared::displace(f1, f2, d1, d2) % (self.idxs.len() as u32)) as uint];
        let entry = &self.entries[idx];

        if BorrowFrom::borrow_from(&entry.0) == key {
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

/// An iterator over the entries in a `OrderedMap`.
pub struct Entries<'a, K:'a, V:'a> {
    iter: slice::Iter<'a, (K, V)>,
}

impl<'a, K, V> Iterator<(&'a K, &'a V)> for Entries<'a, K, V> {
    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        self.iter.next().map(|e| (&e.0, &e.1))
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<(&'a K, &'a V)> for Entries<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a V)> {
        self.iter.next_back().map(|e| (&e.0, &e.1))
    }
}

impl<'a, K, V> RandomAccessIterator<(&'a K, &'a V)> for Entries<'a, K, V> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<(&'a K, &'a V)> {
        self.iter.idx(index).map(|e| (&e.0, &e.1))
    }
}

impl<'a, K, V> ExactSizeIterator<(&'a K, &'a V)> for Entries<'a, K, V> {}

/// An iterator over the keys in a `OrderedMap`.
pub struct Keys<'a, K:'a, V:'a> {
    iter: Entries<'a, K, V>,
}

impl<'a, K, V> Iterator<&'a K> for Keys<'a, K, V> {
    fn next(&mut self) -> Option<&'a K> {
        self.iter.next().map(|e| e.0)
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a K> for Keys<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a K> {
        self.iter.next_back().map(|e| e.0)
    }
}

impl<'a, K, V> RandomAccessIterator<&'a K> for Keys<'a, K, V> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a K> {
        self.iter.idx(index).map(|e| e.0)
    }
}

impl<'a, K, V> ExactSizeIterator<&'a K> for Keys<'a, K, V> {}

/// An iterator over the values in a `OrderedMap`.
pub struct Values<'a, K:'a, V:'a> {
    iter: Entries<'a, K, V>,
}

impl<'a, K, V> Iterator<&'a V> for Values<'a, K, V> {
    fn next(&mut self) -> Option<&'a V> {
        self.iter.next().map(|e| e.1)
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a V> for Values<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a V> {
        self.iter.next_back().map(|e| e.1)
    }
}

impl<'a, K, V> RandomAccessIterator<&'a V> for Values<'a, K, V> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a V> {
        self.iter.idx(index).map(|e| e.1)
    }
}

impl<'a, K, V> ExactSizeIterator<&'a V> for Values<'a, K, V> {}
