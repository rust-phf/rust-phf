//! An order-preserving immutable set constructed at compile time.
use core::prelude::*;
use core::fmt;
use ordered_map;
use {PhfHash, OrderedMap};
use collections::Map as MapTrait;
use collections::Set as SetTrait;

/// An order-preserving immutable set constructed at compile time.
///
/// Unlike a `Set`, iteration order is guaranteed to match the definition
/// order.
///
/// `OrderedSet`s may be created with the `phf_ordered_set` macro:
///
/// ```rust
/// # #![feature(phase)]
/// extern crate phf;
/// #[phase(plugin)]
/// extern crate phf_mac;
///
/// static MY_SET: phf::OrderedSet<&'static str> = phf_ordered_set! {
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
pub struct OrderedSet<T:'static> {
    #[doc(hidden)]
    pub map: OrderedMap<T, ()>,
}

impl<T> fmt::Show for OrderedSet<T> where T: fmt::Show {
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

impl<T> Collection for OrderedSet<T> {
    #[inline]
    fn len(&self) -> uint {
        self.map.len()
    }
}

impl<T> SetTrait<T> for OrderedSet<T> where T: PhfHash+Eq {
    #[inline]
    fn contains(&self, value: &T) -> bool {
        self.map.contains_key(value)
    }

    #[inline]
    fn is_disjoint(&self, other: &OrderedSet<T>) -> bool {
        !self.iter().any(|value| other.contains(value))
    }

    #[inline]
    fn is_subset(&self, other: &OrderedSet<T>) -> bool {
        self.iter().all(|value| other.contains(value))
    }
}

impl<T: PhfHash+Eq> OrderedSet<T> {
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

impl<T> OrderedSet<T> {
    /// Like `contains`, but can operate on any type that is equivalent to a
    /// value
    #[inline]
    pub fn contains_equiv<Sized? U>(&self, key: &U) -> bool where U: PhfHash+Equiv<T> {
        self.map.find_equiv(key).is_some()
    }

    /// Like `find_key`, but can operate on any type that is equivalent to a
    /// value
    #[inline]
    pub fn find_key_equiv<Sized? U>(&self, key: &U) -> Option<&T> where U: PhfHash+Equiv<T> {
        self.map.find_key_equiv(key)
    }

    /// Like `find_index`, but can operate on any type that is equivalent to a
    /// key.
    pub fn find_index_equiv<Sized? U>(&self, key: &U) -> Option<uint> where U: PhfHash+Equiv<T> {
        self.map.find_index_equiv(key)
    }

    /// Returns an iterator over the values in the set.
    ///
    /// Values are returned in the same order in which they were defined.
    #[inline]
    pub fn iter<'a>(&'a self) -> Entries<'a, T> {
        Entries { iter: self.map.keys() }
    }
}

/// An iterator over the values in a `OrderedSet`.
pub struct Entries<'a, T:'a> {
    iter: ordered_map::Keys<'a, T, ()>,
}

impl<'a, T> Iterator<&'a T> for Entries<'a, T> {
    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator<&'a T> for Entries<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a T> {
        self.iter.next_back()
    }
}

impl<'a, T> RandomAccessIterator<&'a T> for Entries<'a, T> {
    #[inline]
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    #[inline]
    fn idx(&mut self, index: uint) -> Option<&'a T> {
        self.iter.idx(index)
    }
}

impl<'a, T> ExactSize<&'a T> for Entries<'a, T> {}

