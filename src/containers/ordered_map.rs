use core::{fmt, ops::Index};

use crate::{BuilderState, HashValue, Hasher, PhfKeyProxy};

use super::get_index;

/// An order-preserving map which can be constructed at compile time.
///
/// "Order-preserving" means iteration order is guaranteed to match the definition
/// order.
pub struct OrderedMap<K: 'static, V: 'static> {
    key: u64,
    disps: &'static [(u32, u32)],
    idxs: &'static [usize],
    entries: &'static [(K, V)],
}

impl<K, V> fmt::Debug for OrderedMap<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_map().entries(self.entries()).finish()
    }
}

impl<'a, K, V, T: ?Sized> Index<&'a T> for OrderedMap<K, V>
where
    K: PhfKeyProxy<T>,
{
    type Output = V;

    fn index(&self, k: &'a T) -> &V {
        self.get(k).expect("invalid key")
    }
}

impl<K: PartialEq, V: PartialEq> PartialEq for OrderedMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}
impl<K: Eq, V: Eq> Eq for OrderedMap<K, V> {}

impl<K, V> Default for OrderedMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
impl<K, V> OrderedMap<K, V> {
    /// Create an empty map.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            key: 0,
            disps: &[],
            idxs: &[],
            entries: &[],
        }
    }

    /// Create a new map from [`crate::MapBuilder`] entries.
    #[doc(hidden)]
    #[inline(always)]
    pub const fn from<const LEN: usize, const BUCKET_LEN: usize>(
        entries: &'static [(K, V)],
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
    pub fn get<T: ?Sized>(&self, key: &T) -> Option<&V>
    where
        K: PhfKeyProxy<T>,
    {
        self.get_entry(key).map(|e| e.1)
    }

    /// Returns a reference to the map's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    #[inline(always)]
    pub fn get_key<T: ?Sized>(&self, key: &T) -> Option<&K>
    where
        K: PhfKeyProxy<T>,
    {
        self.get_entry(key).map(|e| e.0)
    }

    /// Determines if `key` is in the `OrderedMap`.
    #[inline(always)]
    pub fn contains_key<T: ?Sized>(&self, key: &T) -> bool
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
    pub fn index(&self, index: usize) -> Option<(&K, &V)> {
        self.entries.get(index).map(|(k, v)| (k, v))
    }

    /// Like `get`, but returns both the key and the value.
    #[inline(always)]
    pub fn get_entry<T: ?Sized>(&self, key: &T) -> Option<(&K, &V)>
    where
        K: PhfKeyProxy<T>,
    {
        self.get_internal(key).map(|(_, e)| e)
    }

    fn get_internal<T: ?Sized>(&self, key: &T) -> Option<(usize, (&K, &V))>
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

        if entry.0.pfh_eq(key) {
            Some((idx, (&entry.0, &entry.1)))
        } else {
            None
        }
    }

    /// Returns an iterator over the key/value pairs in the map.
    ///
    /// Entries are returned in the same order in which they were defined.
    pub fn entries(&self) -> Entries<'_, K, V> {
        const fn map<K, V>(kv: &(K, V)) -> (&K, &V) {
            (&kv.0, &kv.1)
        }
        self.entries.iter().map(map)
    }

    /// Returns an iterator over the keys in the map.
    ///
    /// Keys are returned in the same order in which they were defined.
    pub fn keys(&self) -> Keys<'_, K, V> {
        const fn map<K, V>(kv: &(K, V)) -> &K {
            &kv.0
        }
        self.entries.iter().map(map)
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Values are returned in the same order in which they were defined.
    pub fn values(&self) -> Values<'_, K, V> {
        const fn map<K, V>(kv: &(K, V)) -> &V {
            &kv.1
        }
        self.entries.iter().map(map)
    }
}

type Iter<'a, K, V> = core::slice::Iter<'a, (K, V)>;
pub type Entries<'a, K, V> = core::iter::Map<Iter<'a, K, V>, fn(&(K, V)) -> (&K, &V)>;
pub type Keys<'a, K, V> = core::iter::Map<Iter<'a, K, V>, fn(&(K, V)) -> &K>;
pub type Values<'a, K, V> = core::iter::Map<Iter<'a, K, V>, fn(&(K, V)) -> &V>;

impl<'a, K, V> IntoIterator for &'a OrderedMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Entries<'a, K, V>;

    fn into_iter(self) -> Entries<'a, K, V> {
        self.entries()
    }
}
