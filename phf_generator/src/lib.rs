//! See [the `phf` crate's documentation][phf] for details.
//!
//! [phf]: https://docs.rs/phf

// XXX: Temporary until stabilization.
#![allow(incomplete_features)]
#![feature(
    const_fn_trait_bound,
    const_option,
    const_panic,
    const_trait_impl,
    const_mut_refs,
    generic_const_exprs
)]
#![doc(html_root_url = "https://docs.rs/phf_generator/0.11")]

pub mod rng;
#[cfg(feature = "const-api")]
mod utils;

use phf_shared::{HashKey, PhfHash};
use rng::Rng;

#[doc(hidden)]
pub const DEFAULT_LAMBDA: usize = 5;

const FIXED_SEED: u64 = 1234567890;

#[cfg(feature = "const-api")]
pub struct HashState<const N: usize>
where
    [(); (N + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA]: Sized,
{
    pub key: HashKey,
    pub disps: utils::ArrayVec<(u32, u32), { (N + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA }>,
    pub map: utils::ArrayVec<usize, N>,
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
    use utils::ArrayVec;

    #[derive(Clone, Copy)]
    struct Bucket<const N: usize> {
        idx: usize,
        keys: ArrayVec<usize, N>,
    }

    impl<const N: usize> const Default for Bucket<N> {
        fn default() -> Self {
            Bucket {
                idx: 0,
                keys: ArrayVec::new(0),
            }
        }
    }

    let hashes = {
        let mut hashes = [phf_shared::Hashes::default(); N];
        let mut i = 0;
        while i < N {
            hashes[i] = phf_shared::hash(&entries[i], &key);
            i += 1;
        }
        hashes
    };

    let mut buckets = {
        let mut buckets = [Bucket::<N>::default(); { (N + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA }];
        let mut i = 0;
        while i < buckets.len() {
            buckets[i].idx = i;
            i += 1;
        }
        buckets
    };

    let mut i = 0;
    while i < hashes.len() {
        buckets[(hashes[i].g % (buckets.len() as u32)) as usize]
            .keys
            .push(i);
        i += 1;
    }

    // Sort descending
    {
        const fn partition<const N: usize>(
            buckets: &mut [Bucket<N>],
            mut start: usize,
            mut end: usize,
        ) -> usize {
            let pivot_idx = start;
            let pivot = buckets[start];

            while start < end {
                // Increment start until an element smaller than pivot is found.
                while start < buckets.len() && pivot.keys.len() <= buckets[start].keys.len() {
                    start += 1;
                }

                // Decrement end until an element greater than pivot is found.
                while pivot.keys.len() > buckets[end].keys.len() {
                    end -= 1;
                }

                // If start and end have not crossed each other, swap them.
                if start < end {
                    let temp = buckets[start];
                    buckets[start] = buckets[end];
                    buckets[end] = temp;
                }
            }

            // Swap pivot element and end to put pivot in its correct place.
            let temp = buckets[end];
            buckets[end] = buckets[pivot_idx];
            buckets[pivot_idx] = temp;

            end
        }

        const fn quick_sort<const N: usize>(start: usize, end: usize, buckets: &mut [Bucket<N>]) {
            if start < end {
                let part = partition(buckets, start, end);

                // Sort elements before and after partition.
                quick_sort(start, part - 1, buckets);
                quick_sort(part + 1, end, buckets);
            }
        }

        quick_sort(0, buckets.len(), &mut buckets)
    }

    let mut map: ArrayVec<Option<usize>, N> = ArrayVec::new(None);
    let mut disps: ArrayVec<(u32, u32), { (N + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA }> =
        ArrayVec::new((0, 0));

    // store whether an element from the bucket being placed is
    // located at a certain position, to allow for efficient overlap
    // checks. It works by storing the generation in each cell and
    // each new placement-attempt is a new generation, so you can tell
    // if this is legitimately full by checking that the generations
    // are equal. (A u64 is far too large to overflow in a reasonable
    // time for current hardware.)
    let mut try_map = [0u64; N];
    let mut generation = 0u64;

    // the actual values corresponding to the markers above, as
    // (index, key) pairs, for adding to the main map, once we've
    // chosen the right disps.
    let mut values_to_add: ArrayVec<(usize, usize), N> = ArrayVec::new((0, 0));

    let mut i1 = 0;
    'buckets: while i1 < buckets.len() {
        let bucket = &buckets[i1];
        i1 += 1;

        let mut d1 = 0;
        while d1 < N {
            let mut d2 = 0;
            'disps: while d2 < N {
                values_to_add.clear();
                generation += 1;

                let mut i2 = 0;
                while i2 < bucket.keys.len() {
                    let key = bucket.keys.get(i2);
                    let idx =
                        (phf_shared::displace(hashes[key].f1, hashes[key].f2, d1 as u32, d2 as u32)
                            % (N as u32)) as usize;
                    if map.get_ref(idx).is_some() || try_map[idx] == generation {
                        d2 += 1;
                        continue 'disps;
                    }
                    try_map[idx] = generation;
                    values_to_add.push((idx, key));
                    i2 += 1;
                }

                // We've picked a good set of disps
                disps.set(bucket.idx, (d1 as u32, d2 as u32));
                i2 = 0;
                while i2 < values_to_add.len() {
                    let &(idx, key) = values_to_add.get_ref(i2);
                    map.set(idx, Some(key));
                    i2 += 1;
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
            let mut result: ArrayVec<usize, N> = ArrayVec::new(0);
            let mut i = 0;
            while i < map.len() {
                result.set(i, map.get(i).unwrap());
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
