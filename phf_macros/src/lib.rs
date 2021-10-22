//! Helper functions for macros to generate PHF data structures at compile time.
//! See [the `phf` crate's documentation][phf] for details.
//!
//! [phf]: https://docs.rs/phf

// XXX: Remove on stabilization.
#![allow(incomplete_features)]
#![feature(
    const_fn_trait_bound,
    const_maybe_uninit_write,
    const_mut_refs,
    const_ptr_read,
    const_refs_to_cell,
    const_trait_impl,
    const_transmute_copy,
    generic_const_exprs,
    maybe_uninit_uninit_array
)]

use std::{
    hash::{Hash, Hasher},
    mem::{transmute_copy, MaybeUninit},
};

use phf_generator::{HashState, DEFAULT_LAMBDA};
use phf_shared::PhfHash;

const unsafe fn const_array_assume_init<T, const N: usize>(array: &[MaybeUninit<T>; N]) -> [T; N] {
    transmute_copy(array)
}

// `Key` struct previously; arbitrary hashable expression now.
// `Entry` struct previously; tuple of `Key` and an arbitrary expression as value; hashable by key
// `Map` struct previously; duplicates-checked Vec of `Entry`s.
// `Set` struct previously; duplicates-checked Vec of `Entry`s with real key and hacked `()` as value.

const fn check_duplicates<Key, Value, const N: usize>(_entries: &[(Key, Value); N]) {
    // TODO: Implement this and enable `const_panic` feature.
}

/*fn check_duplicates(entries: &[Entry]) -> parse::Result<()> {
    let mut keys = HashSet::new();
    for entry in entries {
        if !keys.insert(&entry.key.parsed) {
            return Err(Error::new_spanned(&entry.key.expr, "duplicate key"));
        }
    }
    Ok(())
}*/

pub struct Entry<'a, Key, Value>(&'a (Key, Value));

impl<'a, Key: ~const Hash, Value> const PhfHash for Entry<'a, Key, Value> {
    #[inline]
    fn phf_hash<H: ~const Hasher>(&self, state: &mut H) {
        self.0 .0.hash(state)
    }
}

pub const fn phf_map<Key: ~const Hash, Value, const N: usize>(
    entries: &[(Key, Value); N],
) -> ([(Key, Value); N], HashState<N>)
where
    (Key, Value): Copy,
    [(); (N + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA]: Sized,
{
    check_duplicates(entries);

    // Produce a hash state over all the keys in our map.
    let mut keys = MaybeUninit::uninit_array::<N>();
    let mut i = 0;
    while i < entries.len() {
        keys[i].write(Entry(&entries[i]));
        i += 1;
    }
    let state = phf_generator::generate_hash(unsafe { &const_array_assume_init(&keys) });

    // Reorder all the entries as per state's map.
    let mut ordered_entries = MaybeUninit::uninit_array::<N>();
    i = 0;
    while i < state.map.len() {
        ordered_entries[i].write(entries[i]);
        i += 1;
    }

    (unsafe { const_array_assume_init(&ordered_entries) }, state)
}

pub const fn phf_ordered_map<Key: ~const Hash, Value, const N: usize>(
    entries: &[(Key, Value); N],
) -> ([(Key, Value); N], HashState<N>)
where
    (Key, Value): Copy,
    [(); (N + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA]: Sized,
{
    check_duplicates(entries);

    // Produce a hash state over all the keys in our map.
    let mut keys = MaybeUninit::uninit_array::<N>();
    let mut i = 0;
    while i < entries.len() {
        keys[i].write(Entry(&entries[i]));
        i += 1;
    }
    let state = phf_generator::generate_hash(unsafe { &const_array_assume_init(&keys) });

    // We don't need to do any sorting here.
    (*entries, state)
}

pub const fn phf_set<Key: ~const Hash, const N: usize>(
    entries: &[Key; N],
) -> ([(Key, ()); N], HashState<N>)
where
    Key: Copy,
    [(); (N + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA]: Sized,
{
    let mut map_entries = MaybeUninit::uninit_array::<N>();
    let mut i = 0;
    while i < map_entries.len() {
        map_entries[i].write((entries[i], ()));
        i += 1;
    }

    phf_map(unsafe { &const_array_assume_init(&map_entries) })
}

pub const fn phf_ordered_set<Key: ~const Hash, const N: usize>(
    entries: &[Key; N],
) -> ([(Key, ()); N], HashState<N>)
where
    Key: Copy,
    [(); (N + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA]: Sized,
{
    let mut map_entries = MaybeUninit::uninit_array::<N>();
    let mut i = 0;
    while i < map_entries.len() {
        map_entries[i].write((entries[i], ()));
        i += 1;
    }

    phf_ordered_map(unsafe { &const_array_assume_init(&map_entries) })
}
