//! An order-preserving immutable set constructed at compile time.
use core::prelude::*;
use core::borrow::BorrowFrom;
use core::iter::RandomAccessIterator;
use core::fmt;
use ordered_map;
use {PhfHash, OrderedMap};

/// An order-preserving immutable set constructed at compile time.
///
/// Unlike a `Set`, iteration order is guaranteed to match the definition
/// order.
///
/// `OrderedSet`s may be created with the `phf_ordered_set` macro:
///
/// ```rust
/// #![feature(plugin)]
/// extern crate phf;
/// #[plugin] #[no_link]
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
/// ## Note
///
/// The fields of this struct are public so that they may be initialized by the
/// `phf_ordered_set` macro. They are subject to change at any time and should
/// never be accessed directly.
pub struct OrderedSet<T:'static> {
    #[doc(hidden)]
    pub map: OrderedMap<T, ()>,
}

impl<T> fmt::String for OrderedSet<T> where T: fmt::String {
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

impl<T> fmt::Show for OrderedSet<T> where T: fmt::Show {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, "{{"));
        let mut first = true;
        for entry in self.iter() {
            if !first {
                try!(write!(fmt, ", "));
            }
            try!(write!(fmt, "{:?}", entry));
            first = false;
        }
        write!(fmt, "}}")
    }
}

impl<T> OrderedSet<T> {
    /// Returns the number of elements in the `OrderedSet`.
    pub fn len(&self) -> uint {
        self.map.len()
    }

    /// Returns true if the `OrderedSet` contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the set's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    pub fn get_key<U: ?Sized>(&self, key: &U) -> Option<&T> where U: Eq + PhfHash + BorrowFrom<T> {
        self.map.get_key(key)
    }

    /// Returns the index of the key within the list used to initialize
    /// the ordered set.
    pub fn get_index<U: ?Sized>(&self, key: &U) -> Option<uint>
            where U: Eq + PhfHash + BorrowFrom<T> {
        self.map.get_index(key)
    }

    /// Returns true if `value` is in the `Set`.
    pub fn contains<U: ?Sized>(&self, value: &U) -> bool where U: Eq + PhfHash + BorrowFrom<T> {
        self.map.contains_key(value)
    }

    /// Returns an iterator over the values in the set.
    ///
    /// Values are returned in the same order in which they were defined.
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter { iter: self.map.keys() }
    }
}

impl<T> OrderedSet<T> where T: Eq + PhfHash {
    /// Returns true if `other` shares no elements with `self`.
    #[inline]
    pub fn is_disjoint(&self, other: &OrderedSet<T>) -> bool {
        !self.iter().any(|value| other.contains(value))
    }

    /// Returns true if `other` contains all values in `self`.
    #[inline]
    pub fn is_subset(&self, other: &OrderedSet<T>) -> bool {
        self.iter().all(|value| other.contains(value))
    }

    /// Returns true if `self` contains all values in `other`.
    #[inline]
    pub fn is_superset(&self, other: &OrderedSet<T>) -> bool {
        other.is_subset(self)
    }
}

/// An iterator over the values in a `OrderedSet`.
pub struct Iter<'a, T:'a> {
    iter: ordered_map::Keys<'a, T, ()>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a T> {
        self.iter.next_back()
    }
}

impl<'a, T> RandomAccessIterator for Iter<'a, T> {
    #[inline]
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    #[inline]
    fn idx(&mut self, index: uint) -> Option<&'a T> {
        self.iter.idx(index)
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {}
