//! An immutable bidirectional map constructed at compile time.
use core::fmt;

use phf_shared::{PhfEq, PhfHash};

use crate::HashIndex;
use crate::ordered_map::{Entries, Keys, Values};

/// A one-to-one mapping with perfect hash lookups in both directions.
///
/// Each pair is stored once. Lookups use one hash and allocate no memory.
/// Iteration visits pairs in definition order.
/// Two maps are equal if they contain the same pairs in the same order.
/// The fields are public for code generation and are subject to change.
pub struct BiMap<L: 'static, R: 'static> {
    #[doc(hidden)]
    pub left: HashIndex,
    #[doc(hidden)]
    pub right: HashIndex,
    #[doc(hidden)]
    pub entries: &'static [(L, R)],
}

impl<L, R> BiMap<L, R> {
    /// Creates an empty bidirectional map.
    pub const fn new() -> Self {
        Self {
            left: HashIndex::EMPTY,
            right: HashIndex::EMPTY,
            entries: &[],
        }
    }

    /// Returns the number of pairs in the map.
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the map is empty.
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the right value paired with `left`.
    pub fn get_by_left<T>(&self, left: &T) -> Option<&R>
    where
        T: Eq + PhfHash + ?Sized,
        L: PhfEq<T>,
    {
        self.get_entry_by_left(left).map(|(_, r)| r)
    }

    /// Returns the left value paired with `right`.
    pub fn get_by_right<T>(&self, right: &T) -> Option<&L>
    where
        T: Eq + PhfHash + ?Sized,
        R: PhfEq<T>,
    {
        self.get_entry_by_right(right).map(|(l, _)| l)
    }

    /// Returns the stored pair with the given left value.
    #[inline]
    pub fn get_entry_by_left<T>(&self, left: &T) -> Option<(&L, &R)>
    where
        T: Eq + PhfHash + ?Sized,
        L: PhfEq<T>,
    {
        let (l, r) = &self.entries[self.left.candidate(left)?];
        l.phf_eq(left).then_some((l, r))
    }

    /// Returns the stored pair with the given right value.
    #[inline]
    pub fn get_entry_by_right<T>(&self, right: &T) -> Option<(&L, &R)>
    where
        T: Eq + PhfHash + ?Sized,
        R: PhfEq<T>,
    {
        let (l, r) = &self.entries[self.right.candidate(right)?];
        r.phf_eq(right).then_some((l, r))
    }

    /// Returns true if the map contains `left` on the left.
    pub fn contains_left<T>(&self, left: &T) -> bool
    where
        T: Eq + PhfHash + ?Sized,
        L: PhfEq<T>,
    {
        self.get_by_left(left).is_some()
    }

    /// Returns true if the map contains `right` on the right.
    pub fn contains_right<T>(&self, right: &T) -> bool
    where
        T: Eq + PhfHash + ?Sized,
        R: PhfEq<T>,
    {
        self.get_by_right(right).is_some()
    }

    /// Iterates over pairs in definition order.
    pub fn entries(&self) -> Entries<'_, L, R> {
        Entries {
            iter: self.entries.iter(),
        }
    }

    /// Iterates over the left values in definition order.
    pub fn left_values(&self) -> Keys<'_, L, R> {
        Keys {
            iter: self.entries(),
        }
    }

    /// Iterates over the right values in definition order.
    pub fn right_values(&self) -> Values<'_, L, R> {
        Values {
            iter: self.entries(),
        }
    }
}

impl<L, R> Default for BiMap<L, R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L: PartialEq, R: PartialEq> PartialEq for BiMap<L, R> {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl<L: Eq, R: Eq> Eq for BiMap<L, R> {}

impl<L: fmt::Debug, R: fmt::Debug> fmt::Debug for BiMap<L, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self).finish()
    }
}

impl<'a, L, R> IntoIterator for &'a BiMap<L, R> {
    type Item = (&'a L, &'a R);
    type IntoIter = Entries<'a, L, R>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries()
    }
}

#[cfg(feature = "serde")]
impl<L: serde::Serialize, R: serde::Serialize> serde::Serialize for BiMap<L, R> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(self)
    }
}
