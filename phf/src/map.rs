//! An immutable map constructed at compile time.

use crate::{
    hash::{DefaultHasher, Displacement},
    PhfBorrow, PhfHash, PhfHasher,
};
use core::fmt;
use core::iter::FusedIterator;
use core::iter::IntoIterator;
use core::ops::Index;
use core::slice;

/// An immutable map constructed at compile time.
///
/// ## Note
///
/// The fields of this struct are public so that they may be initialized by the
/// `phf_map!` macro and code generation. They are subject to change at any
/// time and should never be accessed directly.
pub struct Map<K, V, G = DefaultHasher>
where
    K: 'static,
    V: 'static,
{
    #[doc(hidden)]
    pub hasher: G,
    #[doc(hidden)]
    pub disps: &'static [Displacement],
    #[doc(hidden)]
    pub entries: &'static [(K, V)],
}

impl<K, V, G> fmt::Debug for Map<K, V, G>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_map().entries(self.entries()).finish()
    }
}

impl<'a, K, V, G, T: ?Sized> Index<&'a T> for Map<K, V, G>
where
    T: Eq + PhfHash,
    K: PhfBorrow<T>,
    G: PhfHasher,
{
    type Output = V;

    fn index(&self, k: &'a T) -> &V {
        self.get(k).expect("invalid key")
    }
}

impl<K, V, G> Map<K, V, G> {
    /// Returns the number of entries in the `Map`.
    #[inline]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the `Map` is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the key/value pairs in the map.
    ///
    /// Entries are returned in an arbitrary but fixed order.
    #[inline]
    pub fn entries(&self) -> Entries<'_, K, V> {
        Entries {
            iter: self.entries.iter(),
        }
    }

    /// Returns an iterator over the keys in the map.
    ///
    /// Keys are returned in an arbitrary but fixed order.
    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.entries(),
        }
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Values are returned in an arbitrary but fixed order.
    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            iter: self.entries(),
        }
    }
}

impl<K, V, G> Map<K, V, G>
where
    G: PhfHasher,
{
    /// Determines if `key` is in the `Map`.
    #[inline]
    pub fn contains_key<T: ?Sized>(&self, key: &T) -> bool
    where
        T: Eq + PhfHash,
        K: PhfBorrow<T>,
    {
        self.get(key).is_some()
    }

    /// Returns a reference to the value that `key` maps to.
    #[inline]
    pub fn get<T: ?Sized>(&self, key: &T) -> Option<&V>
    where
        T: Eq + PhfHash,
        K: PhfBorrow<T>,
    {
        self.get_entry(key).map(|e| e.1)
    }

    /// Returns a reference to the map's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    #[inline]
    pub fn get_key<T: ?Sized>(&self, key: &T) -> Option<&K>
    where
        T: Eq + PhfHash,
        K: PhfBorrow<T>,
    {
        self.get_entry(key).map(|e| e.0)
    }

    /// Like `get`, but returns both the key and the value.
    pub fn get_entry<T: ?Sized>(&self, key: &T) -> Option<(&K, &V)>
    where
        T: Eq + PhfHash,
        K: PhfBorrow<T>,
    {
        // Prevent panic on empty map
        if self.disps.is_empty() {
            return None;
        }
        let entry = self
            .hasher
            .hash(key)
            .displaced_get(self.entries, self.disps);
        if entry.0.borrow() == key {
            Some((&entry.0, &entry.1))
        } else {
            None
        }
    }
}

impl<'a, K, V, G> IntoIterator for &'a Map<K, V, G> {
    type Item = (&'a K, &'a V);
    type IntoIter = Entries<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Entries<'a, K, V> {
        self.entries()
    }
}

/// An iterator over the key/value pairs in a `Map`.
pub struct Entries<'a, K, V> {
    iter: slice::Iter<'a, (K, V)>,
}

impl<'a, K, V> Clone for Entries<'a, K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, K, V> fmt::Debug for Entries<'a, K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K, V> Iterator for Entries<'a, K, V> {
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        self.iter.next().map(|e| (&e.0, &e.1))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for Entries<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<(&'a K, &'a V)> {
        self.iter.next_back().map(|e| (&e.0, &e.1))
    }
}

impl<'a, K, V> ExactSizeIterator for Entries<'a, K, V> {}

impl<'a, K, V> FusedIterator for Entries<'a, K, V> {}

/// An iterator over the keys in a `Map`.
pub struct Keys<'a, K, V> {
    iter: Entries<'a, K, V>,
}

impl<'a, K, V> Clone for Keys<'a, K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, K, V> fmt::Debug for Keys<'a, K, V>
where
    K: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<&'a K> {
        self.iter.next().map(|e| e.0)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for Keys<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a K> {
        self.iter.next_back().map(|e| e.0)
    }
}

impl<'a, K, V> ExactSizeIterator for Keys<'a, K, V> {}

impl<'a, K, V> FusedIterator for Keys<'a, K, V> {}

/// An iterator over the values in a `Map`.
pub struct Values<'a, K, V> {
    iter: Entries<'a, K, V>,
}

impl<'a, K, V> Clone for Values<'a, K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, K, V> fmt::Debug for Values<'a, K, V>
where
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<&'a V> {
        self.iter.next().map(|e| e.1)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for Values<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a V> {
        self.iter.next_back().map(|e| e.1)
    }
}

impl<'a, K, V> ExactSizeIterator for Values<'a, K, V> {}

impl<'a, K, V> FusedIterator for Values<'a, K, V> {}
