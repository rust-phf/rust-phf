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

use core::mem::{transmute_copy, MaybeUninit};

use phf_generator::{HashState, DEFAULT_LAMBDA};
use phf_shared::PhfHash;

const unsafe fn const_array_assume_init<T, const N: usize>(array: &[MaybeUninit<T>; N]) -> [T; N] {
    transmute_copy(array)
}

const fn check_duplicates<Key, Value, const N: usize>(_entries: &[(Key, Value); N]) {
    // TODO: Implement once we can compare keys in const fn and produce
    //       a formatted panic message that points out the duplicate key.
}

pub const fn phf_map<Key: ~const PhfHash, Value, const N: usize>(
    entries: &[(Key, Value); N],
) -> ([(Key, Value); N], HashState<N>)
where
    (Key, Value): Copy,
    [(); (N + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA]: Sized,
{
    check_duplicates(entries);

    let mut keys = MaybeUninit::uninit_array::<N>();
    let mut i = 0;
    while i < entries.len() {
        keys[i].write(&entries[i].0);
        i += 1;
    }
    let state = phf_generator::generate_hash(unsafe { &const_array_assume_init(&keys) });

    let mut ordered_entries = MaybeUninit::uninit_array::<N>();
    i = 0;
    while i < state.map.len() {
        ordered_entries[i].write(entries[i]);
        i += 1;
    }

    (unsafe { const_array_assume_init(&ordered_entries) }, state)
}

pub const fn phf_ordered_map<Key: ~const PhfHash, Value, const N: usize>(
    entries: &[(Key, Value); N],
) -> ([(Key, Value); N], HashState<N>)
where
    (Key, Value): Copy,
    [(); (N + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA]: Sized,
{
    check_duplicates(entries);

    let mut keys = MaybeUninit::uninit_array::<N>();
    let mut i = 0;
    while i < entries.len() {
        keys[i].write(&entries[i].0);
        i += 1;
    }
    let state = phf_generator::generate_hash(unsafe { &const_array_assume_init(&keys) });

    (*entries, state)
}

pub const fn phf_set<Key: ~const PhfHash, const N: usize>(
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

pub const fn phf_ordered_set<Key: ~const PhfHash, const N: usize>(
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
