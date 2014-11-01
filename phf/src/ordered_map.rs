//! An order-preserving immutable map constructed at compile time.
use core::prelude::*;
use core::fmt;
use core::slice;
use core::iter;
use PhfHash;
use shared;
use collections::Map as MapTrait;

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
/// # Note
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

impl<K, V> Collection for OrderedMap<K, V> {
    fn len(&self) -> uint {
        self.entries.len()
    }
}

impl<K, V> MapTrait<K, V> for OrderedMap<K, V> where K: PhfHash+Eq {
    fn find(&self, key: &K) -> Option<&V> {
        self.find_entry(key, |k| k == key).map(|(_, e)| &e.1)
    }
}

impl<K, V> Index<K, V> for OrderedMap<K, V> where K: PhfHash+Eq {
    fn index(&self, k: &K) -> &V {
        self.find(k).expect("invalid key")
    }
}

impl<K, V> OrderedMap<K, V> where K: PhfHash+Eq {
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

impl<K, V> OrderedMap<K, V> {
    fn find_entry<Sized? T>(&self, key: &T, check: |&K| -> bool) -> Option<(uint, &(K, V))>
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
    pub fn find_equiv<Sized? T>(&self, key: &T) -> Option<&V> where T: PhfHash+Equiv<K> {
        self.find_entry(key, |k| key.equiv(k)).map(|(_, e)| &e.1)
    }

    /// Like `find_key`, but can operate on any type that is equivalent to a
    /// key.
    pub fn find_key_equiv<Sized? T>(&self, key: &T) -> Option<&K> where T: PhfHash+Equiv<K> {
        self.find_entry(key, |k| key.equiv(k)).map(|(_, e)| &e.0)
    }

    /// Like `find_index`, but can operate on any type that is equivalent to a
    /// key.
    pub fn find_index_equiv<Sized? T>(&self, key: &T) -> Option<uint> where T: PhfHash+Equiv<K> {
        self.find_entry(key, |k| key.equiv(k)).map(|(i, _)| i)
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
        Keys { iter: self.entries().map(|e| &e.0) }
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Values are returned in the same order in which they were defined.
    pub fn values<'a>(&'a self) -> Values<'a, K, V> {
        Values { iter: self.entries().map(|e| &e.1) }
    }
}

/// An iterator over the entries in a `OrderedMap`.
pub struct Entries<'a, K:'a, V:'a> {
    iter: slice::Items<'a, (K, V)>,
}

impl<'a, K, V> Iterator<&'a (K, V)> for Entries<'a, K, V> {
    fn next(&mut self) -> Option<&'a (K, V)> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a (K, V)> for Entries<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a (K, V)> {
        self.iter.next_back()
    }
}

impl<'a, K, V> RandomAccessIterator<&'a (K, V)> for Entries<'a, K, V> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a (K, V)> {
        self.iter.idx(index)
    }
}

impl<'a, K, V> ExactSize<&'a (K, V)> for Entries<'a, K, V> {}

/// An iterator over the keys in a `OrderedMap`.
pub struct Keys<'a, K:'a, V:'a> {
    iter: iter::Map<'a, &'a (K, V), &'a K, Entries<'a, K, V>>,
}

impl<'a, K, V> Iterator<&'a K> for Keys<'a, K, V> {
    fn next(&mut self) -> Option<&'a K> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a K> for Keys<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a K> {
        self.iter.next_back()
    }
}

impl<'a, K, V> RandomAccessIterator<&'a K> for Keys<'a, K, V> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a K> {
        self.iter.idx(index)
    }
}

impl<'a, K, V> ExactSize<&'a K> for Keys<'a, K, V> {}

/// An iterator over the values in a `OrderedMap`.
pub struct Values<'a, K:'a, V:'a> {
    iter: iter::Map<'a, &'a (K, V), &'a V, Entries<'a, K, V>>,
}

impl<'a, K, V> Iterator<&'a V> for Values<'a, K, V> {
    fn next(&mut self) -> Option<&'a V> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a V> for Values<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a V> {
        self.iter.next_back()
    }
}

impl<'a, K, V> RandomAccessIterator<&'a V> for Values<'a, K, V> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a V> {
        self.iter.idx(index)
    }
}

impl<'a, K, V> ExactSize<&'a V> for Values<'a, K, V> {}

