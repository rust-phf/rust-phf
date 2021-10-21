//! See [the `phf` crate's documentation][phf] for details.
//!
//! [phf]: https://docs.rs/phf

// XXX: Temporary until stabilization.
#![allow(incomplete_features)]
#![feature(
    const_fn_trait_bound,
    const_option,
    const_trait_impl,
    const_mut_refs,
    generic_const_exprs
)]
#![doc(html_root_url = "https://docs.rs/phf_generator/0.10")]

pub mod rng;

use phf_shared::{HashKey, PhfHash};
use rng::Rng;

const DEFAULT_LAMBDA: usize = 5;

const FIXED_SEED: u64 = 1234567890;

#[cfg(feature = "const-api")]
pub struct HashState<const N: usize>
where
    [(); (N + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA]: Sized,
{
    pub key: HashKey,
    pub disps: [(u32, u32); (N + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA],
    pub map: [usize; N],
}

#[cfg(not(feature = "const-api"))]
pub struct HashState {
    pub key: HashKey,
    pub disps: Vec<(u32, u32)>,
    pub map: Vec<usize>,
}

#[cfg(feature = "const-api")]
pub const fn generate_hash<H: ~const PhfHash, const N: usize>(entries: &[H; N]) -> HashState<N>
where
    [(); (N + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA]: Sized,
{
    let mut rng = Rng::new(FIXED_SEED);
    loop {
        match try_generate_hash(entries, rng.generate()) {
            Some(state) => break state,
            None => continue,
        }
    }
}

#[cfg(not(feature = "const-api"))]
pub fn generate_hash<H: PhfHash>(entries: &[H]) -> HashState {
    let mut rng = Rng::new(FIXED_SEED);
    loop {
        match try_generate_hash(entries, rng.generate()) {
            Some(state) => break state,
            None => continue,
        }
    }
}

#[cfg(feature = "const-api")]
const fn try_generate_hash<H: ~const PhfHash, const N: usize>(
    entries: &[H; N],
    key: HashKey,
) -> Option<HashState<N>>
where
    [(); (N + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA]: Sized,
{
    assert_ne!(N, usize::MAX);

    struct Bucket<const N: usize> {
        idx: usize,
        keys: [usize; N],
    }

    impl<const N: usize> const Default for Bucket<N> {
        #[inline(always)]
        fn default() -> Self {
            Self {
                idx: 0,
                // We use usize::MAX as a marker to distinguish what is an actual
                // key and what is not due to fixed allocation sizes. We previously
                // assert that `N` is not `usize::MAX` to avoid ambiguity.
                keys: [usize::MAX; N],
            }
        }
    }

    let mut hashes: [_; N] = [phf_shared::Hashes::default(); N];
    let mut i = 0;
    while i < N {
        hashes[i] = phf_shared::hash(&entries[i], &key);
        i += 1;
    }

    const BUCKETS_LEN: usize = (N + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA;
    let mut buckets: [Bucket<N>; BUCKETS_LEN] = [Bucket::default(); BUCKETS_LEN];
    i = 0;
    while i < BUCKETS_LEN {
        buckets[i].idx = i;
        i += 1;
    }

    i = 0;
    let mut key_lens: [usize; N] = [0; N];
    while i < N {
        let bucket = (hashes[i].g % (BUCKETS_LEN as u32)) as usize;
        buckets[bucket].keys[key_lens[bucket]] = i;
        key_lens[bucket] += 1;
    }

    // Sort descending
    // buckets.sort_by(|a, b| a.keys.len().cmp(&b.keys.len()).reverse());
    // TODO

    let mut map: [Option<usize>; N] = [None; N];
    let mut disps: [(u32, u32); BUCKETS_LEN] = [(0, 0); BUCKETS_LEN];

    // store whether an element from the bucket being placed is
    // located at a certain position, to allow for efficient overlap
    // checks. It works by storing the generation in each cell and
    // each new placement-attempt is a new generation, so you can tell
    // if this is legitimately full by checking that the generations
    // are equal. (A u64 is far too large to overflow in a reasonable
    // time for current hardware.)
    let mut try_map: [u64; N] = [0; N];
    let mut generation = 0u64;

    // the actual values corresponding to the markers above, as
    // (index, key) pairs, for adding to the main map once we've
    // chosen the right disps.
    let mut values_to_add_len: usize = 0;
    let mut values_to_add: [(usize, usize); N] = [(0, 0); N];

    i = 0;
    'buckets: while i < buckets.len() {
        let bucket = &buckets[i];
        let mut d1 = 0;
        while d1 < N {
            let mut d2 = 0;
            'disps: while d2 < N {
                let mut j = 0;
                while j < N {
                    values_to_add[j] = (0, 0);
                    j += 1;
                }
                generation += 1;

                j = 0;
                while j < N {
                    let key = bucket.keys[j];
                    let idx =
                        (phf_shared::displace(hashes[key].f1, hashes[key].f2, d1 as u32, d2 as u32)
                            % (N as u32)) as usize;
                    if map[idx].is_some() || try_map[idx] == generation {
                        d2 += 1;
                        continue 'disps;
                    }
                    try_map[idx] = generation;
                    values_to_add[values_to_add_len] = (idx, key);
                    values_to_add_len += 1;
                    j += 1;
                }

                // We've picked a good set of disps.
                disps[bucket.idx] = (d1 as u32, d2 as u32);
                j = 0;
                while j < N {
                    let &(idx, key) = &values_to_add[j];
                    map[idx] = Some(key);
                    j += 1;
                }
                continue 'buckets;
            }
            d1 += 1;
        }

        // Unable to find displacements for a bucket
        return None;
    }

    Some(HashState {
        key,
        disps,
        map: {
            let mut result = [0; N];
            i = 0;
            while i < N {
                result[i] = map[i].unwrap();
                i += 1;
            }
            result
        },
    })
}

#[cfg(not(feature = "const-api"))]
fn try_generate_hash<H: PhfHash>(entries: &[H], key: HashKey) -> Option<HashState> {
    struct Bucket {
        idx: usize,
        keys: Vec<usize>,
    }

    let hashes: Vec<_> = entries
        .iter()
        .map(|entry| phf_shared::hash(entry, &key))
        .collect();

    let buckets_len = (hashes.len() + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA;
    let mut buckets = (0..buckets_len)
        .map(|i| Bucket {
            idx: i,
            keys: vec![],
        })
        .collect::<Vec<_>>();

    for (i, hash) in hashes.iter().enumerate() {
        buckets[(hash.g % (buckets_len as u32)) as usize]
            .keys
            .push(i);
    }

    // Sort descending
    buckets.sort_by(|a, b| a.keys.len().cmp(&b.keys.len()).reverse());

    let table_len = hashes.len();
    let mut map = vec![None; table_len];
    let mut disps = vec![(0u32, 0u32); buckets_len];

    // store whether an element from the bucket being placed is
    // located at a certain position, to allow for efficient overlap
    // checks. It works by storing the generation in each cell and
    // each new placement-attempt is a new generation, so you can tell
    // if this is legitimately full by checking that the generations
    // are equal. (A u64 is far too large to overflow in a reasonable
    // time for current hardware.)
    let mut try_map = vec![0u64; table_len];
    let mut generation = 0u64;

    // the actual values corresponding to the markers above, as
    // (index, key) pairs, for adding to the main map once we've
    // chosen the right disps.
    let mut values_to_add = vec![];

    'buckets: for bucket in &buckets {
        for d1 in 0..(table_len as u32) {
            'disps: for d2 in 0..(table_len as u32) {
                values_to_add.clear();
                generation += 1;

                for &key in &bucket.keys {
                    let idx = (phf_shared::displace(hashes[key].f1, hashes[key].f2, d1, d2)
                        % (table_len as u32)) as usize;
                    if map[idx].is_some() || try_map[idx] == generation {
                        continue 'disps;
                    }
                    try_map[idx] = generation;
                    values_to_add.push((idx, key));
                }

                // We've picked a good set of disps
                disps[bucket.idx] = (d1, d2);
                for &(idx, key) in &values_to_add {
                    map[idx] = Some(key);
                }
                continue 'buckets;
            }
        }

        // Unable to find displacements for a bucket
        return None;
    }

    Some(HashState {
        key,
        disps,
        map: map.into_iter().map(|i| i.unwrap()).collect(),
    })
}
