//! An immutable map constructed at compile time.
use core::prelude::*;
use core::borrow::BorrowFrom;
use core::iter;
use core::slice;
use core::fmt;
use PhfHash;
use phf_shared;

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

impl<K, V, Sized? T> Index<T, V> for Map<K, V> where T: Eq + PhfHash + BorrowFrom<K> {
    fn index(&self, k: &T) -> &V {
        self.get(k).expect("invalid key")
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

    /// Determines if `key` is in the `Map`.
    pub fn contains_key<Sized? T>(&self, key: &T) -> bool where T: Eq + PhfHash + BorrowFrom<K> {
        self.get(key).is_some()
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

    /// Like `get`, but returns both the key and the value.
    pub fn get_entry<Sized? T>(&self, key: &T) -> Option<(&K, &V)>
            where T: Eq + PhfHash + BorrowFrom<K> {
        let (g, f1, f2) = key.phf_hash(self.key);
        let (d1, d2) = self.disps[(g % (self.disps.len() as u32)) as uint];
        let entry = &self.entries[(phf_shared::displace(f1, f2, d1, d2) % (self.entries.len() as u32))
                                  as uint];
        if BorrowFrom::borrow_from(&entry.0) == key {
            Some((&entry.0, &entry.1))
        } else {
            None
        }
    }

    /// Returns an iterator over the key/value pairs in the map.
    ///
    /// Entries are retuned in an arbitrary but fixed order.
    pub fn entries<'a>(&'a self) -> Entries<'a, K, V> {
        Entries { iter: self.entries.iter().map(|&(ref k, ref v)| (k, v)) }
    }

    /// Returns an iterator over the keys in the map.
    ///
    /// Keys are returned in an arbitrary but fixed order.
    pub fn keys<'a>(&'a self) -> Keys<'a, K, V> {
        Keys { iter: self.entries().map(|e| e.0) }
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Values are returned in an arbitrary but fixed order.
    pub fn values<'a>(&'a self) -> Values<'a, K, V> {
        Values { iter: self.entries().map(|e | e.1) }
    }
}

/// An iterator over the key/value pairs in a `Map`.
pub struct Entries<'a, K:'a, V:'a> {
    iter: iter::Map<'a, &'a (K, V), (&'a K, &'a V), slice::Items<'a, (K, V)>>,
}

impl<'a, K, V> Iterator<(&'a K, &'a V)> for Entries<'a, K, V> {
    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<(&'a K, &'a V)> for Entries<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a V)> {
        self.iter.next_back()
    }
}

impl<'a, K, V> ExactSizeIterator<(&'a K, &'a V)> for Entries<'a, K, V> {}

/// An iterator over the keys in a `Map`.
pub struct Keys<'a, K:'a, V:'a> {
    iter: iter::Map<'a, (&'a K, &'a V), &'a K, Entries<'a, K, V>>,
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

impl<'a, K, V> ExactSizeIterator<&'a K> for Keys<'a, K, V> {}

/// An iterator over the values in a `Map`.
pub struct Values<'a, K:'a, V:'a> {
    iter: iter::Map<'a, (&'a K, &'a V), &'a V, Entries<'a, K, V>>,
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

impl<'a, K, V> ExactSizeIterator<&'a V> for Values<'a, K, V> {}
