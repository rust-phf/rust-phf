use core::mem::{forget, replace, ManuallyDrop};

use arrayvec_const::ArrayVec;
use sort_const::const_shellsort;

use crate::rand::WyRand;

use super::HashValue;

const FIXED_SEED: u64 = 1234567890;

/// The final computed state during map generation
#[doc(hidden)]
pub struct BuilderState<const LEN: usize, const BUCKET_LEN: usize> {
    pub(crate) key: u64,
    pub(crate) disps: [(u32, u32); BUCKET_LEN],
    pub(crate) idxs: [usize; LEN],
}

struct Bucket<const LEN: usize> {
    idx: usize,
    keys: ArrayVec<usize, LEN>,
}

impl<const LEN: usize> Bucket<LEN> {
    const fn new(idx: usize) -> Self {
        Bucket {
            idx,
            keys: ArrayVec::new(),
        }
    }
}

// TODO: improve error message
const fn must_succeed<T, E>(r: Result<T, E>) -> T {
    union Transmute<T, E> {
        mr: ManuallyDrop<Result<T, E>>,
        mrm: ManuallyDrop<Result<ManuallyDrop<T>, ManuallyDrop<E>>>,
    }
    // Wouldn't need this with const_precise_live_drops :(
    let mr = ManuallyDrop::new(r);
    let rm = ManuallyDrop::into_inner(unsafe { Transmute { mr }.mrm });
    match rm {
        Ok(t) => ManuallyDrop::into_inner(t),
        Err(_e) => {
            panic!("failed")
        }
    }
}

const fn inc_u32(v: &mut u32) -> u32 {
    let r = *v;
    *v += 1;
    r
}

pub struct Generator<const LEN: usize, const BUCKET_LEN: usize> {
    rng: WyRand,
    key: u64,
    buckets: [Bucket<LEN>; BUCKET_LEN],
    disps: [(u32, u32); BUCKET_LEN],
    idxs: [Option<usize>; LEN],
    try_map: [u64; LEN],
}

impl<const LEN: usize, const BUCKET_LEN: usize> Default for Generator<LEN, BUCKET_LEN> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const LEN: usize, const BUCKET_LEN: usize> Generator<LEN, BUCKET_LEN> {
    pub const fn new() -> Self {
        Self {
            rng: WyRand::new(FIXED_SEED),
            key: 0,
            buckets: [const { Bucket::new(0) }; BUCKET_LEN],
            disps: [(0, 0); BUCKET_LEN],
            idxs: [None; LEN],
            try_map: [0u64; LEN],
        }
    }

    const fn clear(&mut self) {
        {
            let mut i = 0;
            while i < BUCKET_LEN {
                forget(replace(&mut self.buckets[i], Bucket::new(i)));
                i += 1;
            }
        };

        self.disps = [(0, 0); BUCKET_LEN];
        self.idxs = [None; LEN];
        self.try_map = [0u64; LEN];
    }

    pub const fn next_key(&mut self) -> u64 {
        self.key = self.rng.rand();
        self.key
    }

    pub const fn try_generate_hash(
        mut self,
        hashes: &[HashValue; LEN],
    ) -> Result<BuilderState<LEN, BUCKET_LEN>, ManuallyDrop<Self>> {
        self.clear();

        let buckets_len = BUCKET_LEN as u32;
        {
            let mut i = 0;
            while i < LEN {
                let hash = &hashes[i];

                let bucket = &mut self.buckets[(hash.g % buckets_len) as usize];
                must_succeed(bucket.keys.try_push(i));
                i += 1;
            }
        }

        // Sort descending
        const_shellsort!(&mut self.buckets, |a, b| a.keys.len() > b.keys.len());

        // store whether an element from the bucket being placed is
        // located at a certain position, to allow for efficient overlap
        // checks. It works by storing the generation in each cell and
        // each new placement-attempt is a new generation, so you can tell
        // if this is legitimately full by checking that the generations
        // are equal. (A u64 is far too large to overflow in a reasonable
        // time for current hardware.)
        let mut generation = 0u64;

        let mut i = 0;
        'buckets: while i < BUCKET_LEN {
            let bucket = &self.buckets.as_slice()[i];
            i += 1;

            let mut d1 = 0;
            while d1 < LEN as u32 {
                let d1 = inc_u32(&mut d1);

                let mut d2 = 0;
                'disps: while d2 < LEN as u32 {
                    let d2 = inc_u32(&mut d2);

                    // the actual values corresponding to the markers above, as
                    // (index, key) pairs, for adding to the main map once we've
                    // chosen the right disps.
                    let mut values_to_add = ArrayVec::<_, LEN>::new();
                    generation += 1;

                    let mut k = 0;
                    while k < bucket.keys.len() {
                        let key = bucket.keys.as_slice()[k];
                        k += 1;

                        let idx = (crate::displace(hashes[key].f1, hashes[key].f2, d1, d2)
                            % (LEN as u32)) as usize;
                        if self.idxs[idx].is_some() || self.try_map[idx] == generation {
                            // TODO: remove.
                            // Blocked on const drop, but this is fine because `ArrayVec<T: Copy>` is just stack memory
                            forget(values_to_add);
                            continue 'disps;
                        }
                        self.try_map[idx] = generation;
                        must_succeed(values_to_add.try_push((idx, key)));
                    }

                    // We've picked a good set of disps
                    self.disps[bucket.idx] = (d1, d2);
                    while let Some((idx, key)) = values_to_add.pop() {
                        self.idxs[idx] = Some(key);
                    }

                    // TODO: remove.
                    // Blocked on const drop, but this is fine because `ArrayVec<T: Copy>` is just stack memory
                    forget(values_to_add);
                    continue 'buckets;
                }
            }

            // Unable to find displacements for a bucket
            return Err(ManuallyDrop::new(self));
        }

        let mut idxs = [0; LEN];
        let mut i = 0;
        while i < LEN {
            idxs[i] = self.idxs[i].expect("expected generator map");
            i += 1;
        }
        let res = BuilderState {
            key: self.key,
            disps: self.disps,
            idxs,
        };
        forget(self);
        Ok(res)
    }
}
