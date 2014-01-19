//! Compile time optimized maps
#[crate_id="github.com/sfackler/rust-phf/phf"];
#[doc(html_root_url="http://www.rust-ci.org/sfackler/rust-phf/doc")];
#[crate_type="lib"];
#[warn(missing_doc)];

use std::iter;
use std::vec;

/// An immutable map constructed at compile time.
///
/// `PhfMap`s may be created with the `phf_map` macro:
///
/// ```rust
/// static my_map: PhfMap<int> = phf_map!(
///    "hello" => 10,
///    "world" => 11,
/// );
/// ```
///
/// # Note
///
/// The fields of this struct are public so that they may be initialized by the
/// `phf_map` macro. They are subject to change at any time and should never
/// be accessed directly.
pub struct PhfMap<T> {
    #[doc(hidden)]
    len: uint,
    #[doc(hidden)]
    keys: Keys,
    #[doc(hidden)]
    disps: &'static [(uint, uint)],
    #[doc(hidden)]
    entries: &'static [Option<(&'static str, T)>],
}

#[doc(hidden)]
pub struct Keys {
    k1: u64,
    k2_g: u64,
    k2_f1: u64,
    k2_f2: u64,
}

impl Keys {
    #[doc(hidden)]
    pub fn hash1(&self, s: &str) -> uint {
        s.hash_keyed(self.k1, self.k2_g) as uint
    }

    #[doc(hidden)]
    pub fn hash2(&self, s: &str, d1: uint, d2: uint) -> uint {
        let f1 = s.hash_keyed(self.k1, self.k2_f1) as uint;
        let f2 = s.hash_keyed(self.k1, self.k2_f2) as uint;
        d2 + f1 * d1 + f2
    }
}

impl<T> Container for PhfMap<T> {
    #[inline]
    fn len(&self) -> uint {
        self.len
    }
}

impl<'a, T> Map<&'a str, T> for PhfMap<T> {
    #[inline]
    fn find<'a>(&'a self, key: & &str) -> Option<&'a T> {
        let hash1 = self.keys.hash1(*key);
        let (d1, d2) = self.disps[hash1 % self.disps.len()];
        let hash2 = self.keys.hash2(*key, d1, d2);
        match self.entries[hash2 % self.entries.len()] {
            Some((s, ref value)) if s == *key => Some(value),
            _ => None
        }
    }
}

impl<T> PhfMap<T> {
    /// Returns an iterator over the key/value pairs in the map. Entries are
    /// retuned in an arbitrary order.
    #[inline]
    pub fn entries<'a>(&'a self) -> PhfMapEntries<'a, T> {
        PhfMapEntries {
            iter: self.entries.iter().filter_map(|e| {
                match *e {
                    Some((key, ref value)) => Some((key, value)),
                    None => None
                }
            })
        }
    }

    /// Returns an iterator over the keys in the map. Keys are returned in an
    /// arbitrary order.
    #[inline]
    pub fn keys<'a>(&'a self) -> PhfMapKeys<'a, T> {
        PhfMapKeys { iter: self.entries() }
    }

    /// Returns an iterator over the values in the map. Values are returned in
    /// an arbitrary order.
    #[inline]
    pub fn values<'a>(&'a self) -> PhfMapValues<'a, T> {
        PhfMapValues { iter: self.entries() }
    }
}

/// An iterator over the key/value pairs in a `PhfMap`.
pub struct PhfMapEntries<'a, T> {
    priv iter: iter::FilterMap<'a,
                               &'a Option<(&'static str, T)>,
                               (&'static str, &'a T),
                               vec::Items<'a, Option<(&'static str, T)>>>,
}

impl<'a, T> Iterator<(&'static str, &'a T)> for PhfMapEntries<'a, T> {
    #[inline]
    fn next(&mut self) -> Option<(&'static str, &'a T)> {
        self.iter.next()
    }
}

/// An iterator over the keys in a `PhfMap`.
pub struct PhfMapKeys<'a, T> {
    priv iter: PhfMapEntries<'a, T>,
}

impl<'a, T> Iterator<&'static str> for PhfMapKeys<'a, T> {
    #[inline]
    fn next(&mut self) -> Option<&'static str> {
        self.iter.next().map(|(key, _)| key)
    }
}

/// An iterator over the values in a `PhfMap`.
pub struct PhfMapValues<'a, T> {
    priv iter: PhfMapEntries<'a, T>,
}

impl<'a, T> Iterator<&'a T> for PhfMapValues<'a, T> {
    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next().map(|(_, value)| value)
    }
}
