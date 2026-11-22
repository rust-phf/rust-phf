use std::borrow::Cow;
use std::fmt;
use std::hash::Hash;

use phf_shared::{FmtConst, PhfHash};

use crate::{Delegate, HashState, generate_hash_state};

/// A builder for `phf::BiMap`.
///
/// Both sides are typed keys. For example, `.entry(200u16, "OK")` emits the
/// string literal `"OK"` and indexes it for reverse lookup.
pub struct BiMap<'a, L, R> {
    path: Cow<'a, str>,
    left: Vec<L>,
    right: Vec<R>,
}

impl<'a, L, R> BiMap<'a, L, R> {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self {
            path: Cow::Borrowed("::phf"),
            left: Vec::new(),
            right: Vec::new(),
        }
    }

    /// Sets the path to the `phf` crate in the generated source.
    pub fn phf_path(&mut self, path: impl Into<Cow<'a, str>>) -> &mut Self {
        self.path = path.into();
        self
    }

    /// Adds a left/right pair in definition order.
    pub fn entry(&mut self, left: L, right: R) -> &mut Self {
        self.left.push(left);
        self.right.push(right);
        self
    }

    /// Builds both perfect hash indexes and returns the printable map.
    ///
    /// With the `quote` feature enabled, the result also implements `quote::ToTokens`.
    ///
    /// # Panics
    ///
    /// Panics if either side contains a duplicate.
    pub fn build(&self) -> DisplayBiMap<'_, L, R>
    where
        L: Hash + Eq + PhfHash + FmtConst,
        R: Hash + Eq + PhfHash + FmtConst,
    {
        DisplayBiMap {
            path: &self.path,
            left: &self.left,
            right: &self.right,
            left_index: generate_hash_state(&self.left, "left key"),
            right_index: generate_hash_state(&self.right, "right key"),
        }
    }
}

impl<L, R> FromIterator<(L, R)> for BiMap<'_, L, R> {
    fn from_iter<T: IntoIterator<Item = (L, R)>>(iter: T) -> Self {
        let (left, right) = iter.into_iter().unzip();
        Self {
            left,
            right,
            ..Self::new()
        }
    }
}

/// An adapter for printing a [`BiMap`].
pub struct DisplayBiMap<'a, L, R> {
    path: &'a str,
    left: &'a [L],
    right: &'a [R],
    left_index: HashState,
    right_index: HashState,
}

fn write_index(f: &mut fmt::Formatter<'_>, path: &str, state: &HashState) -> fmt::Result {
    fn write_slice<T: fmt::Debug>(f: &mut fmt::Formatter<'_>, items: &[T]) -> fmt::Result {
        f.write_str("&[")?;
        for item in items {
            write!(f, "\n            {item:?},")?;
        }
        f.write_str("\n        ]")
    }

    #[cfg(not(feature = "ptrhash"))]
    let (key, field, params) = (state.key, "disps", &state.disps);
    #[cfg(feature = "ptrhash")]
    let (key, field, params) = (state.seed, "pilots", &state.pilots);
    write!(
        f,
        "{path}::HashIndex {{\n        key: {key},\n        {field}: "
    )?;
    write_slice(f, params)?;
    f.write_str(",\n        idxs: ")?;
    write_slice(f, &state.map)?;
    f.write_str(",\n    }")
}

impl<L: FmtConst, R: FmtConst> fmt::Display for DisplayBiMap<'_, L, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::BiMap {{\n    left: ", self.path)?;
        write_index(f, self.path, &self.left_index)?;
        f.write_str(",\n    right: ")?;
        write_index(f, self.path, &self.right_index)?;
        f.write_str(",\n    entries: &[")?;
        for (left, right) in self.left.iter().zip(self.right) {
            write!(f, "\n        ({}, {}),", Delegate(left), Delegate(right))?;
        }
        f.write_str("\n    ],\n}")
    }
}

#[cfg(feature = "quote")]
impl<L: FmtConst, R: FmtConst> quote::ToTokens for DisplayBiMap<'_, L, R> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        crate::write_tokens(tokens, self);
    }
}

#[cfg(test)]
mod tests {
    use super::BiMap;

    #[test]
    #[should_panic(expected = "duplicate left key `\"a\"`")]
    fn duplicate_left() {
        BiMap::new().entry("a", 1).entry("a", 2).build();
    }

    #[test]
    #[should_panic(expected = "duplicate right key `1`")]
    fn duplicate_right() {
        BiMap::new().entry("a", 1).entry("b", 1).build();
    }
}
