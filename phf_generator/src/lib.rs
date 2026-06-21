//! See [the `phf` crate's documentation][phf] for details.
//!
//! [phf]: https://docs.rs/phf

#![doc(html_root_url = "https://docs.rs/phf_generator/0.14.0")]
use std::iter;

use fastrand::Rng;
use phf_shared::{HashKey, Hashes, PhfHash};

const DEFAULT_LAMBDA: usize = 3;

const FIXED_SEED: u64 = 1234567890;
const EMPTY_SLOT: usize = usize::MAX;

#[cfg(feature = "ptrhash")]
pub mod ptrhash;

pub struct HashState {
    pub key: HashKey,
    pub disps: Vec<(u32, u32)>,
    pub map: Vec<usize>,
}

pub fn generate_hash<H: PhfHash>(entries: &[H]) -> HashState {
    generate_hash_with_hash_fn(entries, phf_shared::hash)
}

pub fn generate_hash_with_hash_fn<T, F>(entries: &[T], hash_fn: F) -> HashState
where
    F: Fn(&T, &HashKey) -> Hashes,
{
    let mut generator = Generator::new(entries.len());
    let mut rng = Rng::with_seed(FIXED_SEED);

    iter::repeat_with(|| rng.u64(..))
        .find(|key| {
            let hashes = entries.iter().map(|entry| hash_fn(entry, key));
            generator.reset(hashes);

            generator.try_generate_hash()
        })
        .map(|key| HashState {
            key,
            disps: generator.disps,
            map: generator.map,
        })
        .expect("failed to solve PHF")
}

struct Bucket {
    idx: usize,
    start: usize,
    len: usize,
    cursor: usize,
}

struct Generator {
    hashes: Vec<Hashes>,
    buckets: Vec<Bucket>,
    bucket_order: Vec<usize>,
    bucket_keys: Vec<usize>,
    key_buckets: Vec<usize>,
    disps: Vec<(u32, u32)>,
    map: Vec<usize>,
    try_map: Vec<u64>,
}

impl Generator {
    fn new(table_len: usize) -> Self {
        let hashes = Vec::with_capacity(table_len);

        let buckets_len = (table_len + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA;
        let buckets: Vec<_> = (0..buckets_len)
            .map(|i| Bucket {
                idx: i,
                start: 0,
                len: 0,
                cursor: 0,
            })
            .collect();
        let bucket_order = (0..buckets_len).collect();
        let bucket_keys = vec![0; table_len];
        let key_buckets = vec![0; table_len];
        let disps = vec![(0u32, 0u32); buckets_len];

        let map = vec![EMPTY_SLOT; table_len];
        let try_map = vec![0u64; table_len];

        Self {
            hashes,
            buckets,
            bucket_order,
            bucket_keys,
            key_buckets,
            disps,
            map,
            try_map,
        }
    }

    fn reset<I>(&mut self, hashes: I)
    where
        I: Iterator<Item = Hashes>,
    {
        self.buckets.iter_mut().for_each(|b| {
            b.start = 0;
            b.len = 0;
            b.cursor = 0;
        });
        self.disps.fill((0, 0));
        self.map.fill(EMPTY_SLOT);
        self.try_map.fill(0);

        self.hashes.clear();
        self.hashes.extend(hashes);
    }

    fn try_generate_hash(&mut self) -> bool {
        let buckets_len = self.buckets.len() as u32;

        // Store bucket contents in one flat buffer instead of allocating a Vec per bucket.
        for (i, hash) in self.hashes.iter().enumerate() {
            let bucket = (hash.g % buckets_len) as usize;
            self.key_buckets[i] = bucket;
            let bucket = &mut self.buckets[bucket];
            bucket.len += 1;
        }

        let mut start = 0;
        for bucket in &mut self.buckets {
            bucket.start = start;
            bucket.cursor = start;
            start += bucket.len;
        }

        for (i, &bucket) in self.key_buckets.iter().enumerate() {
            let bucket = &mut self.buckets[bucket];
            self.bucket_keys[bucket.cursor] = i;
            bucket.cursor += 1;
        }

        let buckets = &self.buckets;
        self.bucket_order
            .sort_unstable_by(|&a, &b| buckets[b].len.cmp(&buckets[a].len).then_with(|| a.cmp(&b)));

        let table_len = self.hashes.len();

        // store whether an element from the bucket being placed is
        // located at a certain position, to allow for efficient overlap
        // checks. It works by storing the generation in each cell and
        // each new placement-attempt is a new generation, so you can tell
        // if this is legitimately full by checking that the generations
        // are equal. (A u64 is far too large to overflow in a reasonable
        // time for current hardware.)
        let mut generation = 0u64;

        // the actual values corresponding to the markers above, as
        // (index, key) pairs, for adding to the main map once we've
        // chosen the right disps.
        let mut values_to_add = vec![];

        'buckets: for &bucket_idx in &self.bucket_order {
            let bucket = &self.buckets[bucket_idx];
            let keys = &self.bucket_keys[bucket.start..bucket.start + bucket.len];

            for d1 in 0..(table_len as u32) {
                'disps: for d2 in 0..(table_len as u32) {
                    values_to_add.clear();
                    generation += 1;

                    for &key in keys {
                        let idx =
                            (phf_shared::displace(self.hashes[key].f1, self.hashes[key].f2, d1, d2)
                                % (table_len as u32)) as usize;
                        if self.map[idx] != EMPTY_SLOT || self.try_map[idx] == generation {
                            continue 'disps;
                        }
                        self.try_map[idx] = generation;
                        values_to_add.push((idx, key));
                    }

                    // We've picked a good set of disps
                    self.disps[bucket.idx] = (d1, d2);
                    for &(idx, key) in &values_to_add {
                        self.map[idx] = key;
                    }
                    continue 'buckets;
                }
            }

            // Unable to find displacements for a bucket
            return false;
        }
        true
    }
}
