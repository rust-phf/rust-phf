use core::{fmt, slice::Iter};

use crate::{BuilderState, HashValue, Hasher, PhfKeyProxy};

use super::get_index;

/// An order-preserving set which can be constructed at compile time.
///
/// "Order-preserving" means iteration order is guaranteed to match the definition
/// order.
pub struct OrderedSet<K: 'static> {
    key: u64,
    disps: &'static [(u32, u32)],
    idxs: &'static [usize],
    entries: &'static [K],
}

impl<T> fmt::Debug for OrderedSet<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_set().entries(self).finish()
    }
}

impl<T> PartialEq for OrderedSet<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl<T> Eq for OrderedSet<T> where T: Eq {}

impl<K> Default for OrderedSet<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> OrderedSet<K> {
    /// Create an set map.
    #[inline]
    pub const fn new() -> Self {
        Self {
            key: 0,
            disps: &[],
            idxs: &[],
            entries: &[],
        }
    }

    /// Create a new map from [`crate::SetBuilder`] entries.
    #[doc(hidden)]
    #[inline(always)]
    pub const fn from<const LEN: usize, const BUCKET_LEN: usize>(
        entries: &'static [K],
        state: &'static BuilderState<LEN, BUCKET_LEN>,
    ) -> Self {
        Self {
            key: state.key,
            disps: state.disps.as_slice(),
            idxs: &state.idxs,
            entries,
        }
    }

    /// Returns the number of entries in the map.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the map is empty.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the value that `key` maps to.
    #[inline(always)]
    pub fn get<T: ?Sized>(&self, key: &T) -> Option<&K>
    where
        K: PhfKeyProxy<T>,
    {
        self.get_internal(key).map(|(_, e)| e)
    }

    /// Determines if `key` is in the `OrderedMap`.
    #[inline(always)]
    pub fn contains<T: ?Sized>(&self, key: &T) -> bool
    where
        K: PhfKeyProxy<T>,
    {
        self.get(key).is_some()
    }

    /// Returns the index of the key within the list used to initialize
    /// the ordered map.
    #[inline(always)]
    pub fn get_index<T: ?Sized>(&self, key: &T) -> Option<usize>
    where
        K: PhfKeyProxy<T>,
    {
        self.get_internal(key).map(|(i, _)| i)
    }

    /// Returns references to both the key and values at an index
    /// within the list used to initialize the ordered map. See `.get_index(key)`.
    #[inline(always)]
    pub fn index(&self, index: usize) -> Option<&K> {
        self.entries.get(index)
    }

    /// Returns an iterator over the values in the set.
    ///
    /// Values are returned in the same order in which they were defined.
    pub fn iter(&self) -> Iter<'_, K> {
        self.entries.iter()
    }

    fn get_internal<T: ?Sized>(&self, key: &T) -> Option<(usize, &K)>
    where
        K: PhfKeyProxy<T>,
    {
        if self.disps.is_empty() {
            return None;
        } //Prevent panic on empty map
        let mut state = Hasher::new_with_keys(0, self.key);
        K::pfh_hash(key, &mut state);
        let hashes = HashValue::finalize(state);
        let idx_index = get_index(&hashes, self.disps, self.idxs.len());
        let idx = self.idxs[idx_index as usize];
        let entry = &self.entries[idx];

        if entry.pfh_eq(key) {
            Some((idx, entry))
        } else {
            None
        }
    }

    /// Returns true if `other` shares no elements with `self`.
    #[inline]
    pub fn is_disjoint(&self, other: &OrderedSet<K>) -> bool
    where
        K: PhfKeyProxy<K>,
    {
        !self.iter().any(|value| other.contains(value))
    }

    /// Returns true if `other` contains all values in `self`.
    #[inline]
    pub fn is_subset(&self, other: &OrderedSet<K>) -> bool
    where
        K: PhfKeyProxy<K>,
    {
        self.iter().all(|value| other.contains(value))
    }

    /// Returns true if `self` contains all values in `other`.
    #[inline]
    pub fn is_superset(&self, other: &OrderedSet<K>) -> bool
    where
        K: PhfKeyProxy<K>,
    {
        other.is_subset(self)
    }
}

impl<'a, T> IntoIterator for &'a OrderedSet<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}
