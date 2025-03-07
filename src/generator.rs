use core::mem;

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

#[cfg(not(feature = "list"))]
struct Bucket<const LEN: usize> {
    idx: usize,
    keys: ArrayVec<usize, LEN>,
}

#[cfg(not(feature = "list"))]
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
    use mem::ManuallyDrop;
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

#[cfg(feature = "list")]
#[derive(Clone, Copy)]
struct Bucket {
    len: usize,
    id: usize,
    head_hash_id: Option<usize>,
}
#[cfg(feature = "list")]
#[derive(Clone, Copy)]
struct Buckets<const BUCKET_LEN: usize>([Bucket; BUCKET_LEN]);

#[cfg(feature = "list")]
impl<const BUCKET_LEN: usize> Buckets<BUCKET_LEN> {
    const fn new() -> Self {
        const DUMMY: Bucket = Bucket {
            len: 0,
            id: 0,
            head_hash_id: None,
        };
        let mut data = [DUMMY; BUCKET_LEN];
        let mut i = 0;
        while i < BUCKET_LEN {
            data[i].id = i;
            i += 1;
        }
        Self(data)
    }
    const fn swap_head(&mut self, bucket_id: usize, hash_id: usize) -> Option<usize> {
        debug_assert!(self.0[bucket_id].id == bucket_id);
        self.0[bucket_id].len += 1;
        self.0[bucket_id].head_hash_id.replace(hash_id)
    }
    const fn get_head(&self, idx: usize) -> Option<usize> {
        self.0[idx].head_hash_id
    }
    const fn sort(&mut self) {
        const_shellsort!(&mut self.0, |a, b| a.len > b.len);
    }
}

pub struct Generator<const LEN: usize, const BUCKET_LEN: usize> {
    rng: WyRand,
    key: u64,
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
        }
    }

    pub const fn next_key(&mut self) -> u64 {
        self.key = self.rng.rand();
        self.key
    }

    pub const fn try_generate_hash(
        &self,
        hashes: &[HashValue; LEN],
    ) -> Option<BuilderState<LEN, BUCKET_LEN>> {
        let mut try_map = [0u64; LEN];
        let mut idxs = [None; LEN];
        let mut disps = [(0, 0); BUCKET_LEN];
        #[cfg(not(feature = "list"))]
        let mut buckets = [const { Bucket::new(0) }; BUCKET_LEN];
        #[cfg(not(feature = "list"))]
        {
            let mut i = 0;
            while i < BUCKET_LEN {
                mem::forget(mem::replace(&mut buckets[i], Bucket::<LEN>::new(i)));
                i += 1;
            }
        };
        #[cfg(feature = "list")]
        let mut bucket_list = [None; LEN];
        #[cfg(feature = "list")]
        let mut buckets = Buckets::<BUCKET_LEN>::new();

        let buckets_len = BUCKET_LEN as u32;
        {
            let mut i = 0;
            while i < LEN {
                let hash = &hashes[i];
                let bucked_id = (hash.g % buckets_len) as usize;
                #[cfg(feature = "list")]
                {
                    bucket_list[i] = buckets.swap_head(bucked_id, i);
                }

                #[cfg(not(feature = "list"))]
                must_succeed(buckets[bucked_id].keys.try_push(i));
                i += 1;
            }
        }

        // Sort descending
        #[cfg(feature = "list")]
        buckets.sort();
        #[cfg(not(feature = "list"))]
        const_shellsort!(&mut buckets, |a, b| a.keys.len() > b.keys.len());

        // store whether an element from the bucket being placed is
        // located at a certain position, to allow for efficient overlap
        // checks. It works by storing the generation in each cell and
        // each new placement-attempt is a new generation, so you can tell
        // if this is legitimately full by checking that the generations
        // are equal. (A u64 is far too large to overflow in a reasonable
        // time for current hardware.)
        let mut generation = 0u64;

        #[cfg(not(feature = "list"))]
        {
            let mut i = 0;
            'buckets: while i < BUCKET_LEN {
                let bucket = &buckets.as_slice()[i];
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
                            if idxs[idx].is_some() || try_map[idx] == generation {
                                // TODO: remove.
                                // Blocked on const drop, but this is fine because `ArrayVec<T: Copy>` is just stack memory
                                mem::forget(values_to_add);
                                continue 'disps;
                            }
                            try_map[idx] = generation;
                            must_succeed(values_to_add.try_push((idx, key)));
                        }

                        // We've picked a good set of disps
                        disps[bucket.idx] = (d1, d2);
                        while let Some((idx, key)) = values_to_add.pop() {
                            idxs[idx] = Some(key);
                        }

                        // TODO: remove.
                        // Blocked on const drop, but this is fine because `ArrayVec<T: Copy>` is just stack memory
                        mem::forget(values_to_add);
                        continue 'buckets;
                    }
                }

                mem::forget(buckets);
                // Unable to find displacements for a bucket
                return None;
            }
        }
        #[cfg(feature = "list")]
        {
            let mut bucket_idx = 0;
            'buckets: while bucket_idx < BUCKET_LEN {
                let bucket_id = buckets.0[bucket_idx].id;
                let (bucket, bucket_len) = {
                    let mut bucket = [0; LEN];
                    let mut bucket_len = 0;
                    let mut bucket_head = buckets.get_head(bucket_idx);
                    while let Some(key) = bucket_head {
                        bucket[bucket_len] = key;
                        bucket_len += 1;
                        bucket_head = bucket_list[key];
                    }
                    (bucket, bucket_len)
                };
                bucket_idx += 1;

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
                        while k < bucket_len {
                            let key = bucket[k];
                            k += 1;

                            let idx = (crate::displace(hashes[key].f1, hashes[key].f2, d1, d2)
                                % (LEN as u32)) as usize;
                            if idxs[idx].is_some() || try_map[idx] == generation {
                                // TODO: remove.
                                // Blocked on const drop, but this is fine because `ArrayVec<T: Copy>` is just stack memory
                                mem::forget(values_to_add);
                                continue 'disps;
                            }
                            try_map[idx] = generation;
                            must_succeed(values_to_add.try_push((idx, key)));
                        }

                        // We've picked a good set of disps
                        disps[bucket_id] = (d1, d2);
                        while let Some((idx, key)) = values_to_add.pop() {
                            idxs[idx] = Some(key);
                        }

                        // TODO: remove.
                        // Blocked on const drop, but this is fine because `ArrayVec<T: Copy>` is just stack memory
                        mem::forget(values_to_add);
                        continue 'buckets;
                    }
                }

                // Unable to find displacements for a bucket
                return None;
            }
        }

        let mut idxs_result = [0; LEN];
        let mut i = 0;
        while i < LEN {
            idxs_result[i] = idxs[i].expect("expected generator map");
            i += 1;
        }
        let res = BuilderState {
            key: self.key,
            disps,
            idxs: idxs_result,
        };
        #[cfg(not(feature = "list"))]
        mem::forget(buckets);
        Some(res)
    }
}
