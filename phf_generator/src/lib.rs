//! See [the `phf` crate's documentation][phf] for details.
//!
//! [phf]: https://docs.rs/phf

#![doc(html_root_url = "https://docs.rs/phf_generator/0.14.0")]
use std::iter;

use fastrand::Rng;
use phf_shared::{HashKey, Hashes, PhfHash, displace, fast_reduct32};

const DEFAULT_LAMBDA: usize = 3;
/// Neighbouring slots tried by `d2` before `d1` moves the bucket elsewhere.
const WINDOW: u32 = 64;

const FIXED_SEED: u64 = 1234567890;

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
    // Slots taken by the buckets placed so far. Kept apart from `map` so that
    // the search touches one byte per slot.
    occupied: Vec<bool>,
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

        let map = vec![0; table_len];
        let occupied = vec![false; table_len];

        Self {
            hashes,
            buckets,
            bucket_order,
            bucket_keys,
            key_buckets,
            disps,
            map,
            occupied,
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
        // `map` needs no reset: a successful attempt writes every slot.
        self.occupied.fill(false);

        self.hashes.clear();
        self.hashes.extend(hashes);
    }

    fn try_generate_hash(&mut self) -> bool {
        let buckets_len = self.buckets.len() as u32;

        // Store bucket contents in one flat buffer instead of allocating a Vec per bucket.
        for (i, hash) in self.hashes.iter().enumerate() {
            let bucket = fast_reduct32(hash.g, buckets_len) as usize;
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

        let table_len = self.hashes.len() as u32;
        // Adding `step` to `d2` moves a key to the next slot.
        let step = (u32::MAX / table_len.max(1)).wrapping_add(1);

        // the slots tried for the bucket being placed, as (index, key)
        // pairs, for adding to the main map once we've chosen the right
        // disps.
        let mut values_to_add = vec![];

        'buckets: for &bucket_idx in &self.bucket_order {
            let bucket = &self.buckets[bucket_idx];
            let keys = &self.bucket_keys[bucket.start..bucket.start + bucket.len];

            // `d1` sends each key of the bucket to an unrelated slot, and `d2` then
            // walks the slots next to it. Walking is cheap, but walking alone piles
            // keys into runs that later buckets have to step through, so most of the
            // search is left to `d1`. Its values are drawn at random because evenly
            // spaced ones can keep missing the last free slots.
            let mut jumps = Rng::with_seed(FIXED_SEED);
            for d1 in iter::repeat_with(|| jumps.u32(..)).take(table_len as usize) {
                'disps: for d2 in (0..WINDOW.min(table_len)).map(|i| i.wrapping_mul(step)) {
                    values_to_add.clear();

                    for &key in keys {
                        let hashes = &self.hashes[key];
                        let idx = fast_reduct32(displace(hashes.f1, hashes.f2, d1, d2), table_len)
                            as usize;
                        if self.occupied[idx] {
                            continue 'disps;
                        }
                        // Walking shifts the whole bucket, so keys that share a slot keep
                        // sharing it unless rounding happens to split them. Jump instead.
                        if values_to_add.iter().any(|&(seen, _)| seen == idx) {
                            break 'disps;
                        }
                        values_to_add.push((idx, key));
                    }

                    // We've picked a good set of disps
                    self.disps[bucket.idx] = (d1, d2);
                    for &(idx, key) in &values_to_add {
                        self.occupied[idx] = true;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookups_find_every_key() {
        for len in [0, 1, 2, 3, 4, 7, 8, 16, 100, 1_000, 100_000] {
            let keys = (0..len as u32).collect::<Vec<_>>();
            let state = generate_hash(&keys);

            for (index, key) in keys.iter().enumerate() {
                let hashes = phf_shared::hash(key, &state.key);
                let slot = phf_shared::get_index(&hashes, &state.disps, state.map.len());
                assert_eq!(state.map[slot as usize], index);
            }
        }
    }
}
