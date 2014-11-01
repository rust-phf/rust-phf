//! An immutable map constructed at compile time.
use core::prelude::*;
use core::iter;
use core::slice;
use core::fmt;
use shared;
use shared::PhfHash;

/// An immutable map constructed at compile time.
///
/// `Map`s may be created with the `phf_map` macro:
///
/// ```rust
/// # #![feature(phase)]
/// extern crate phf;
/// #[phase(plugin)]
/// extern crate phf_mac;
///
/// static MY_MAP: phf::Map<&'static str, int> = phf_map! {
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
/// `phf_map` macro. They are subject to change at any time and should never
/// be accessed directly.
pub struct Map<K:'static, V:'static> {
    #[doc(hidden)]
    pub key: u64,
    #[doc(hidden)]
    pub disps: &'static [(u32, u32)],
    #[doc(hidden)]
    pub entries: &'static [(K, V)],
}

impl<K, V> fmt::Show for Map<K, V> where K: fmt::Show, V: fmt::Show {
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

impl<K, V> Index<K, V> for Map<K, V> where K: PhfHash+Eq {
    fn index(&self, k: &K) -> &V {
        self.find(k).expect("invalid key")
    }
}

impl<K, V> Map<K, V> where K: PhfHash+Eq {
    /// Returns a reference to the value that `key` maps to.
    pub fn find(&self, key: &K) -> Option<&V> {
        self.get_entry(key, |k| key == k).map(|e| &e.1)
    }

    /// Determines if `key` is in the `Map`.
    pub fn contains_key(&self, key: &K) -> bool {
        self.find(key).is_some()
    }

    /// Returns a reference to the map's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    pub fn find_key(&self, key: &K) -> Option<&K> {
        self.get_entry(key, |k| key == k).map(|e| &e.0)
    }
}

impl<K, V> Map<K, V> {
    /// Returns true if the `Map` is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of entries in the `Map`.
    pub fn len(&self) -> uint {
        self.entries.len()
    }

    fn get_entry<Sized? T>(&self, key: &T, check: |&K| -> bool) -> Option<&(K, V)> where T: PhfHash {
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
    pub fn find_equiv<Sized? T>(&self, key: &T) -> Option<&V> where T: PhfHash+Equiv<K> {
        self.get_entry(key, |k| key.equiv(k)).map(|e| &e.1)
    }

    /// Like `find_key`, but can operate on any type that is equivalent to a
    /// key.
    pub fn find_key_equiv<Sized? T>(&self, key: &T) -> Option<&K> where T: PhfHash+Equiv<K> {
        self.get_entry(key, |k| key.equiv(k)).map(|e| &e.0)
    }

    /// Returns an iterator over the key/value pairs in the map.
    ///
    /// Entries are retuned in an arbitrary but fixed order.
    pub fn entries<'a>(&'a self) -> Entries<'a, K, V> {
        Entries { iter: self.entries.iter() }
    }

    /// Returns an iterator over the keys in the map.
    ///
    /// Keys are returned in an arbitrary but fixed order.
    pub fn keys<'a>(&'a self) -> Keys<'a, K, V> {
        Keys { iter: self.entries().map(|e| &e.0) }
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Values are returned in an arbitrary but fixed order.
    pub fn values<'a>(&'a self) -> Values<'a, K, V> {
        Values { iter: self.entries().map(|e | &e.1) }
    }
}

/// An iterator over the key/value pairs in a `Map`.
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

impl<'a, K, V> ExactSize<&'a (K, V)> for Entries<'a, K, V> {}

/// An iterator over the keys in a `Map`.
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

impl<'a, K, V> ExactSize<&'a K> for Keys<'a, K, V> {}

/// An iterator over the values in a `Map`.
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

impl<'a, K, V> ExactSize<&'a V> for Values<'a, K, V> {}


