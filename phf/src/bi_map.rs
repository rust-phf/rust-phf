//! An immutable map constructed at compile time.
use core::fmt;
use core::iter::FusedIterator;
use core::iter::IntoIterator;
use core::slice;
use phf_shared::{self, HashKey, PhfBorrow, PhfHash};
#[cfg(feature = "serde")]
use serde::ser::{Serialize, SerializeMap, Serializer};

/// An immutable bidirectional-map constructed at compile time.
///
/// ## Note
///
/// The fields of this struct are public so that they may be initialized by the
/// `phf_map!` macro and code generation. They are subject to change at any
/// time and should never be accessed directly.
pub struct BiMap<L: 'static, R: 'static> {
    #[doc(hidden)]
    pub key0: HashKey,
    #[doc(hidden)]
    pub key1: HashKey,
    #[doc(hidden)]
    pub disps0: &'static [(u32, u32)],
    #[doc(hidden)]
    pub disps1: &'static [(u32, u32)],
    #[doc(hidden)]
    pub idxs0: &'static [usize],
    #[doc(hidden)]
    pub idxs1: &'static [usize],
    #[doc(hidden)]
    pub entries: &'static [(L, R)],
}

impl<L, R> fmt::Debug for BiMap<L, R>
where
    L: fmt::Debug,
    R: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_map().entries(self.entries()).finish()
    }
}

impl<L, R> Default for BiMap<L, R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L, R> PartialEq for BiMap<L, R>
where
    L: PartialEq,
    R: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key0 == other.key0
            && self.key1 == other.key1
            && self.disps0 == other.disps0
            && self.disps1 == other.disps1
            && self.idxs0 == other.idxs0
            && self.idxs1 == other.idxs1
            && self.entries == other.entries
    }
}

impl<L, R> Eq for BiMap<L, R>
where
    L: Eq,
    R: Eq,
{
}

impl<L, R> BiMap<L, R> {
    /// Create a new, empty, immutable map.
    #[inline]
    pub const fn new() -> Self {
        Self {
            key0: 0,
            key1: 0,
            disps0: &[],
            disps1: &[],
            idxs0: &[],
            idxs1: &[],
            entries: &[],
        }
    }

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

    /// Determines if `key` is in the `Map`.
    pub fn contains_left<T: ?Sized>(&self, key: &T) -> bool
    where
        T: Eq + PhfHash,
        L: PhfBorrow<T>,
    {
        self.get_entry_by_left(key).is_some()
    }

    /// Determines if `key` is in the `Map`.
    pub fn contains_right<T: ?Sized>(&self, key: &T) -> bool
    where
        T: Eq + PhfHash,
        R: PhfBorrow<T>,
    {
        self.get_entry_by_right(key).is_some()
    }

    /// Like `get`, but returns both the key and the value.
    pub fn get_entry_by_left<T: ?Sized>(&self, key: &T) -> Option<(&L, &R)>
    where
        T: Eq + PhfHash,
        L: PhfBorrow<T>,
    {
        if self.disps0.is_empty() {
            return None;
        } //Prevent panic on empty map
        let hashes = phf_shared::hash(key, &self.key0);
        let idx_index = phf_shared::get_index(&hashes, self.disps0, self.entries.len());
        let idx = self.idxs0[idx_index as usize];
        let entry = &self.entries[idx];

        let b: &T = entry.0.borrow();
        if b == key {
            Some((&entry.0, &entry.1))
        } else {
            None
        }
    }

    /// Like `get`, but returns both the key and the value.
    pub fn get_entry_by_right<T: ?Sized>(&self, key: &T) -> Option<(&L, &R)>
    where
        T: Eq + PhfHash,
        R: PhfBorrow<T>,
    {
        if self.disps0.is_empty() {
            return None;
        } //Prevent panic on empty map
        let hashes = phf_shared::hash(key, &self.key1);
        let idx_index = phf_shared::get_index(&hashes, self.disps1, self.entries.len());
        let idx = self.idxs1[idx_index as usize];
        let entry = &self.entries[idx];

        let b: &T = entry.1.borrow();
        if b == key {
            Some((&entry.0, &entry.1))
        } else {
            None
        }
    }

    /// Returns an iterator over the key/value pairs in the map.
    ///
    /// Entries are returned in an arbitrary but fixed order.
    pub fn entries(&self) -> Entries<'_, L, R> {
        Entries {
            iter: self.entries.iter(),
        }
    }

    /// Returns an iterator over the keys in the map.
    ///
    /// Keys are returned in an arbitrary but fixed order.
    pub fn left_entries(&self) -> LeftEntries<'_, L, R> {
        LeftEntries {
            iter: self.entries(),
        }
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Values are returned in an arbitrary but fixed order.
    pub fn right_entries(&self) -> RightEntries<'_, L, R> {
        RightEntries {
            iter: self.entries(),
        }
    }
}

impl<'a, L, R> IntoIterator for &'a BiMap<L, R> {
    type Item = (&'a L, &'a R);
    type IntoIter = Entries<'a, L, R>;

    fn into_iter(self) -> Entries<'a, L, R> {
        self.entries()
    }
}

/// An iterator over the key/value pairs in a `Map`.
pub struct Entries<'a, L, R> {
    iter: slice::Iter<'a, (L, R)>,
}

impl<'a, L, R> Clone for Entries<'a, L, R> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, L, R> fmt::Debug for Entries<'a, L, R>
where
    L: fmt::Debug,
    R: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, L, R> Iterator for Entries<'a, L, R> {
    type Item = (&'a L, &'a R);

    fn next(&mut self) -> Option<(&'a L, &'a R)> {
        self.iter.next().map(|&(ref k, ref v)| (k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, L, R> DoubleEndedIterator for Entries<'a, L, R> {
    fn next_back(&mut self) -> Option<(&'a L, &'a R)> {
        self.iter.next_back().map(|e| (&e.0, &e.1))
    }
}

impl<'a, L, R> ExactSizeIterator for Entries<'a, L, R> {}

impl<'a, L, R> FusedIterator for Entries<'a, L, R> {}

/// An iterator over the keys in a `Map`.
pub struct LeftEntries<'a, L, R> {
    iter: Entries<'a, L, R>,
}

impl<'a, L, R> Clone for LeftEntries<'a, L, R> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, L, R> fmt::Debug for LeftEntries<'a, L, R>
where
    L: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, L, R> Iterator for LeftEntries<'a, L, R> {
    type Item = &'a L;

    fn next(&mut self) -> Option<&'a L> {
        self.iter.next().map(|e| e.0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, L, R> DoubleEndedIterator for LeftEntries<'a, L, R> {
    fn next_back(&mut self) -> Option<&'a L> {
        self.iter.next_back().map(|e| e.0)
    }
}

impl<'a, L, R> ExactSizeIterator for LeftEntries<'a, L, R> {}

impl<'a, L, R> FusedIterator for LeftEntries<'a, L, R> {}

/// An iterator over the values in a `Map`.
pub struct RightEntries<'a, L, R> {
    iter: Entries<'a, L, R>,
}

impl<'a, L, R> Clone for RightEntries<'a, L, R> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, L, R> fmt::Debug for RightEntries<'a, L, R>
where
    R: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, L, R> Iterator for RightEntries<'a, L, R> {
    type Item = &'a R;

    fn next(&mut self) -> Option<&'a R> {
        self.iter.next().map(|e| e.1)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, L, R> DoubleEndedIterator for RightEntries<'a, L, R> {
    fn next_back(&mut self) -> Option<&'a R> {
        self.iter.next_back().map(|e| e.1)
    }
}

impl<'a, L, R> ExactSizeIterator for RightEntries<'a, L, R> {}

impl<'a, L, R> FusedIterator for RightEntries<'a, L, R> {}

#[cfg(feature = "serde")]
impl<L, R> Serialize for BiMap<L, R>
where
    L: Serialize,
    R: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (l, r) in self.entries() {
            map.serialize_entry(l, r)?;
        }
        map.end()
    }
}
