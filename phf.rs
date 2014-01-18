#[crate_id="github.com/sfackler/rust-phf/phf"];
#[crate_type="lib"];

use std::vec;

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

    #[inline]
    pub fn entries<'a>(&'a self) -> PhfMapEntries<'a, T> {
        PhfMapEntries { iter: self.entries.iter() }
    }

    #[inline]
    pub fn keys<'a>(&'a self) -> PhfMapKeys<'a, T> {
        PhfMapKeys { iter: self.entries() }
    }

    #[inline]
    pub fn values<'a>(&'a self) -> PhfMapValues<'a, T> {
        PhfMapValues { iter: self.entries() }
    }
}

pub struct PhfMapEntries<'a, T> {
    priv iter: vec::Items<'a, (&'static str, T)>,
}

impl<'a, T> Iterator<(&'static str, &'a T)> for PhfMapEntries<'a, T> {
    #[inline]
    fn next(&mut self) -> Option<(&'static str, &'a T)> {
        self.iter.next().map(|&(key, ref value)| (key, value))
    }
}

pub struct PhfMapKeys<'a, T> {
    priv iter: PhfMapEntries<'a, T>,
}

impl<'a, T> Iterator<&'static str> for PhfMapKeys<'a, T> {
    #[inline]
    fn next(&mut self) -> Option<&'static str> {
        self.iter.next().map(|(key, _)| key)
    }
}

pub struct PhfMapValues<'a, T> {
    priv iter: PhfMapEntries<'a, T>,
}

impl<'a, T> Iterator<&'a T> for PhfMapValues<'a, T> {
    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next().map(|(_, value)| value)
    }
}
