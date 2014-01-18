//! Compile time optimized maps
#[crate_id="github.com/sfackler/rust-phf/phf"];
#[doc(html_root_url="http://www.rust-ci.org/sfackler/rust-phf/doc")];
#[crate_type="lib"];
#[warn(missing_doc)];

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
    entries: &'static [(&'static str, T)],
}

impl<T> Container for PhfMap<T> {
    #[inline]
    fn len(&self) -> uint {
        self.entries.len()
    }
}

impl<'a, T> Map<&'a str, T> for PhfMap<T> {
    #[inline]
    fn find<'a>(&'a self, key: & &str) -> Option<&'a T> {
        self.entries.bsearch(|&(val, _)| val.cmp(key)).map(|idx| {
            let (_, ref val) = self.entries[idx];
            val
        })
    }
}

impl<T> PhfMap<T> {
    /// Returns an iterator over the key/value pairs in the map. Entries are
    /// retuned in an arbitrary order.
    #[inline]
    pub fn entries<'a>(&'a self) -> PhfMapEntries<'a, T> {
        PhfMapEntries { iter: self.entries.iter() }
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
    priv iter: vec::Items<'a, (&'static str, T)>,
}

impl<'a, T> Iterator<(&'static str, &'a T)> for PhfMapEntries<'a, T> {
    #[inline]
    fn next(&mut self) -> Option<(&'static str, &'a T)> {
        self.iter.next().map(|&(key, ref value)| (key, value))
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
