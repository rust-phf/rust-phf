//! An immutable set constructed at compile time.
use core::prelude::*;
use Map;
use core::fmt;
use shared::PhfHash;
use collections::Set as SetTrait;
use collections::Map as MapTrait;
use map;

/// An immutable set constructed at compile time.
///
/// `Set`s may be created with the `phf_set` macro:
///
/// ```rust
/// # #![feature(phase)]
/// extern crate phf;
/// #[phase(plugin)]
/// extern crate phf_mac;
///
/// use phf::Set;
///
/// static MY_SET: Set<&'static str> = phf_set! {
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
pub struct Set<T:'static> {
    #[doc(hidden)]
    pub map: Map<T, ()>
}

impl<T> fmt::Show for Set<T> where T: fmt::Show {
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

impl<T> Collection for Set<T> {
    #[inline]
    fn len(&self) -> uint {
        self.map.len()
    }
}

impl<T> SetTrait<T> for Set<T> where T: PhfHash+Eq {
    #[inline]
    fn contains(&self, value: &T) -> bool {
        self.map.contains_key(value)
    }

    #[inline]
    fn is_disjoint(&self, other: &Set<T>) -> bool {
        !self.iter().any(|value| other.contains(value))
    }

    #[inline]
    fn is_subset(&self, other: &Set<T>) -> bool {
        self.iter().all(|value| other.contains(value))
    }
}

impl<T> Set<T> where T: PhfHash+Eq {
    /// Returns a reference to the set's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    #[inline]
    pub fn find_key(&self, key: &T) -> Option<&T> {
        self.map.find_key(key)
    }
}

impl<T> Set<T> {
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
}

impl<T> Set<T> {
    /// Returns an iterator over the values in the set.
    ///
    /// Values are returned in an arbitrary but fixed order.
    #[inline]
    pub fn iter<'a>(&'a self) -> Entries<'a, T> {
        Entries { iter: self.map.keys() }
    }
}

/// An iterator over the values in a `Set`.
pub struct Entries<'a, T:'static> {
    iter: map::Keys<'a, T, ()>,
}

impl<'a, T> Iterator<&'a T> for Entries<'a, T> {
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator<&'a T> for Entries<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        self.iter.next_back()
    }
}

impl<'a, T> ExactSize<&'a T> for Entries<'a, T> {}


