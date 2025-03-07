use const_panic::concat_panic;

use crate::PhfKey;

const DEFAULT_LAMBDA: usize = 5;
const MAX_GENERATIONS: usize = 1000;

/// Generate an [`crate::OrderedSet`].
///
/// Supports three syntaxes.
///
/// First is just empty `phf_ordered_set!()`, which just calls [`crate::OrderedSet::new`].
///
/// The second syntax is `phf_ordered_set!(KeyType; v1, v2, v3)` which creates a `OrderedSet<KeyType>` with the values `v1`, `v2`, and `v3` in that order.
///
/// The third syntax is `phf_ordered_set!(KeyType;= expr)` where `expr` evaluates to `[KeyType]` or `[KeyType; LEN]` (note not `&[KeyType]`, so you will have to dereference a slice should you wish to use one).
#[macro_export]
macro_rules! phf_ordered_set {
    () => { $crate::OrderedSet::new() };
    ($k:ty; $($key:expr),* $(,)?) => {{
        struct Builder;
        impl $crate::SetBuilder for Builder {
            type Key = $k;
            const ENTRIES: &[Self::Key] = &[$($key),*];
        }
        $crate::phf_ordered_set_from_builder!(Builder)
    }};
    ($k:ty; = $e:expr) => {{
        struct Builder;
        impl $crate::SetBuilder for Builder {
            type Key = $k;
            const ENTRIES: &[Self::Key] = &$e;
        }
        $crate::phf_ordered_set_from_builder!(Builder)
    }};
}

/// Generate an [`crate::OrderedMap`].
///
/// Supports three syntaxes.
///
/// First is just empty `phf_ordered_set!()`, which just calls [`crate::OrderedMap::new`].
///
/// The second syntax is `phf_ordered_map!(KeyType, ValType; k1 => v1, k2 => v2, k3 => v3)` which creates a `OrderedMap<KeyType, ValType>` with the key-value pairs `(k1, v1)`, `(k2, v2)`, and `(k3, v3)` in that order.
///
/// The third syntax is `phf_ordered_map!(KeyType, ValType;= expr)` where `expr` evaluates to `[(KeyType, ValType)]` or `[(KeyType, ValType); LEN]` (note not `&[(KeyType, ValType)]`, so you will have to dereference a slice should you wish to use one).
#[macro_export]
macro_rules! phf_ordered_map {
    () => { $crate::OrderedMap::new() };
    ($k:ty, $v:ty; $($key:expr => $val:expr),* $(,)?) => {{
        struct Builder;
        impl $crate::MapBuilder for Builder {
            type Key = $k;
            type Value = $v;
            const ENTRIES: &[(Self::Key, Self::Value)] = &[$(($key, $val)),*];
        }
        $crate::phf_ordered_map_from_builder!(Builder)
    }};
    ($k:ty, $v:ty; = $e:expr) => {{
        struct Builder;
        impl $crate::MapBuilder for Builder {
            type Key = $k;
            type Value = $v;
            const ENTRIES: &[(Self::Key, Self::Value)] = &$e;
        }
        $crate::phf_ordered_map_from_builder!(Builder)
    }};
}

/// Takes the name of a type which implements [`crate::SetBuilder`] and returns an [`crate::OrderedSet`] filled with
/// the entries from [`crate::SetBuilder::ENTRIES`].
#[macro_export]
macro_rules! phf_ordered_set_from_builder {
    ($builder:ty) => {
        $crate::__phf_container_from_builder!(
            $builder,
            $crate::OrderedSet::from,
            $crate::SetBuilder,
            $crate::__phf_ordered_set_from_builder_get_entry
        )
    };
}

/// Takes the name of a type which implements [`crate::MapBuilder`] and returns an [`crate::OrderedMap`] filled with
/// the entries from [`crate::MapBuilder::ENTRIES`].
#[macro_export]
macro_rules! phf_ordered_map_from_builder {
    ($builder:ty) => {
        $crate::__phf_container_from_builder!(
            $builder,
            $crate::OrderedMap::from,
            $crate::MapBuilder,
            $crate::__phf_ordered_map_from_builder_get_entry
        )
    };
}

// A helper to re-use macros between sets and maps
#[doc(hidden)]
#[macro_export]
macro_rules! __phf_ordered_set_from_builder_get_entry {
    ($entry:expr) => {
        &$entry
    };
}

// A helper to re-use macros between sets and maps
#[doc(hidden)]
#[macro_export]
macro_rules! __phf_ordered_map_from_builder_get_entry {
    ($entry:expr) => {
        &$entry.0
    };
}

/// Takes the name of a type which implements `$btrait` and returns a `$container` filled with
/// the entries from `$builder::ENTRIES`.
#[macro_export]
#[doc(hidden)]
macro_rules! __phf_container_from_builder {
    ($builder:ty, $container:path, $btrait:ty, $get_key:path) => {{
        const LEN: usize = <$builder as $btrait>::ENTRIES.len();
        const LAMBDA: usize = <$builder as $btrait>::LAMBDA;
        const BUCKET_LEN: usize = (LEN + LAMBDA - 1) / LAMBDA;
        type PhfKey = <$builder as $btrait>::Key;
        type ConstKey = <PhfKey as $crate::PhfKey>::ConstKey;
        const STATE: &$crate::BuilderState<LEN, BUCKET_LEN> = &{
            let entries: &[_] = <$builder as $btrait>::ENTRIES;

            // Check for duplicates by doing a single pass hash
            {
                let mut hash_idx = [(0_u128, 0); LEN];
                let mut i = 0;
                while i < hash_idx.len() {
                    let mut hasher = $crate::Hasher::new_with_keys(0, 0);
                    let entry = { $get_key!(entries[i]) };
                    ConstKey::pfh_hash(entry, &mut hasher);
                    hash_idx[i].0 = hasher.finish128().as_u128();
                    hash_idx[i].1 = i;
                    i += 1;
                }

                $crate::const_shellsort!(&mut hash_idx, |a, b| a.0 > b.0);
                let mut i = 1;
                while i < hash_idx.len() {
                    let mut j = i;
                    while j > 0 {
                        j -= 1;
                        if hash_idx[j].0 != hash_idx[i].0 {
                            break;
                        }
                        let (i, j) = (hash_idx[i].1, hash_idx[j].1);
                        if ConstKey::pfh_eq({ $get_key!(entries[i]) }, { $get_key!(entries[j]) }) {
                            $crate::duplicate_keys(i, j);
                        }
                    }
                    i += 1;
                }
            }

            let mut generator = $crate::Generator::<LEN, BUCKET_LEN>::new();
            let mut generations = 0;
            loop {
                let key = generator.next_key();
                $crate::check_generations(generations, key);
                generations += 1;

                let hashes = {
                    let mut hashes = [const { $crate::HashValue::new() }; LEN];
                    let mut i = 0;
                    while i < hashes.len() {
                        let mut hasher = $crate::Hasher::new_with_keys(0, key);
                        let entry = { $get_key!(entries[i]) };
                        ConstKey::pfh_hash(entry, &mut hasher);
                        hashes[i] = $crate::HashValue::finalize(hasher);
                        i += 1;
                    }
                    hashes
                };
                match generator.try_generate_hash(&hashes) {
                    Ok(state) => break state,
                    Err(e) => generator = ::core::mem::ManuallyDrop::into_inner(e),
                }
            }
        };
        $container(<$builder as $btrait>::ENTRIES, STATE)
    }};
}

/// A trait which stores the entries necessary to create an [`crate::OrderedSet`]
pub trait SetBuilder {
    /// The key type of the set
    type Key: 'static + PhfKey;

    /// The list of set entries
    const ENTRIES: &[Self::Key];

    /// The bucketizing factor used during perfect-hash construction
    const LAMBDA: usize = DEFAULT_LAMBDA;
}

/// A trait which stores the entries necessary to create an [`crate::OrderedMap`]
pub trait MapBuilder {
    /// The key type of the map
    type Key: 'static + PhfKey;

    /// The value type of the map
    type Value: 'static;

    /// The list of map entries
    const ENTRIES: &[(Self::Key, Self::Value)];

    /// The bucketizing factor used during perfect-hash construction
    const LAMBDA: usize = DEFAULT_LAMBDA;
}

/// Panic because duplicate keys were found
#[doc(hidden)]
#[track_caller]
pub const fn duplicate_keys(i: usize, j: usize) -> ! {
    concat_panic!("duplicate keys at index `", i, "` and index `", j, "`")
}

/// Panic if too many generations have occured
#[doc(hidden)]
#[track_caller]
pub const fn check_generations(generations: usize, key: u64) {
    if generations > MAX_GENERATIONS {
        concat_panic!("generations=", generations, " key=", key)
    }
}
