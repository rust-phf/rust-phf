//! An order-preserving immutable set constructed at compile time.

use crate::hash::DefaultHasher;
use crate::{ordered_map, OrderedMap, PhfBorrow, PhfHash, PhfHasher};
use core::fmt;
use core::iter::FusedIterator;
use core::iter::IntoIterator;

/// An order-preserving immutable set constructed at compile time.
///
/// Unlike a `Set`, iteration order is guaranteed to match the definition
/// order.
///
/// ## Note
///
/// The fields of this struct are public so that they may be initialized by the
/// `phf_ordered_set!` macro and code generation. They are subject to change at
/// any time and should never be accessed directly.
pub struct OrderedSet<T, G = DefaultHasher>
where
    T: 'static,
{
    #[doc(hidden)]
    pub map: OrderedMap<T, (), G>,
}

impl<T, G> fmt::Debug for OrderedSet<T, G>
where
    T: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_set().entries(self).finish()
    }
}

impl<T, G> OrderedSet<T, G> {
    /// Returns the number of elements in the `OrderedSet`.
    #[inline]
    pub const fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if the `OrderedSet` contains no elements.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the values in the set.
    ///
    /// Values are returned in the same order in which they were defined.
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.map.keys(),
        }
    }
}

impl<T, G> OrderedSet<T, G>
where
    G: PhfHasher,
{
    /// Returns a reference to the set's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    #[inline]
    pub fn get_key<U: ?Sized>(&self, key: &U) -> Option<&T>
    where
        U: Eq + PhfHash,
        T: PhfBorrow<U>,
    {
        self.map.get_key(key)
    }

    /// Returns the index of the key within the list used to initialize
    /// the ordered set.
    #[inline]
    pub fn get_index<U: ?Sized>(&self, key: &U) -> Option<usize>
    where
        U: Eq + PhfHash,
        T: PhfBorrow<U>,
    {
        self.map.get_index(key)
    }

    /// Returns a reference to the key at an index
    /// within the list used to initialize the ordered set. See `.get_index(key)`.
    #[inline]
    pub fn index(&self, index: usize) -> Option<&T> {
        self.map.index(index).map(|(k, &())| k)
    }

    /// Returns true if `value` is in the `OrderedSet`.
    #[inline]
    pub fn contains<U: ?Sized>(&self, value: &U) -> bool
    where
        U: Eq + PhfHash,
        T: PhfBorrow<U>,
    {
        self.map.contains_key(value)
    }
}

impl<T, G> OrderedSet<T, G>
where
    T: Eq + PhfHash + PhfBorrow<T>,
    G: PhfHasher,
{
    // TODO: Can we compare against `OrderedSet<T, G1>`?

    /// Returns true if `other` shares no elements with `self`.
    #[inline]
    pub fn is_disjoint(&self, other: &OrderedSet<T, G>) -> bool {
        !self.iter().any(|value| other.contains(value))
    }

    /// Returns true if `other` contains all values in `self`.
    #[inline]
    pub fn is_subset(&self, other: &OrderedSet<T, G>) -> bool {
        self.iter().all(|value| other.contains(value))
    }

    /// Returns true if `self` contains all values in `other`.
    #[inline]
    pub fn is_superset(&self, other: &OrderedSet<T, G>) -> bool {
        other.is_subset(self)
    }
}

impl<'a, T, G> IntoIterator for &'a OrderedSet<T, G> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

/// An iterator over the values in a `OrderedSet`.
pub struct Iter<'a, T> {
    iter: ordered_map::Keys<'a, T, ()>,
}

impl<'a, T> Clone for Iter<'a, T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, T> fmt::Debug for Iter<'a, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a T> {
        self.iter.next_back()
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {}

impl<'a, T> FusedIterator for Iter<'a, T> {}
