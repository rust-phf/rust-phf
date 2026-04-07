use core::cmp;

use fastrand::Rng;
use phf_shared::ptrhash::{fast_reduct32, hash as ptrhash_hash, hash_pilot};
use phf_shared::{HashKey, PhfHash};

use crate::FIXED_SEED;

const DEFAULT_ALPHA: f64 = 0.99;
const DEFAULT_LAMBDA: f64 = 3.0;

pub struct HashState {
    pub seed: u64,
    pub pilots: Vec<u8>,
    pub remap: Vec<u32>,
    pub map: Vec<usize>,
}

pub fn generate_hash<H: PhfHash>(entries: &[H]) -> HashState {
    generate_hash_with_hash_fn(entries, ptrhash_hash)
}

pub fn generate_hash_with_hash_fn<T, F>(entries: &[T], hash_fn: F) -> HashState
where
    F: Fn(&T, &HashKey) -> u64,
{
    if entries.is_empty() {
        return HashState {
            seed: 0,
            pilots: vec![],
            remap: vec![],
            map: vec![],
        };
    }

    let mut rng = Rng::with_seed(FIXED_SEED);
    let mut hashes = vec![0; entries.len()];

    loop {
        let seed = rng.u64(..);
        for (hash, entry) in hashes.iter_mut().zip(entries) {
            *hash = hash_fn(entry, &seed);
        }

        if let Some(state) = try_generate_hash(seed, &hashes) {
            return state;
        }
    }
}

#[derive(Default)]
struct Bucket {
    keys: Vec<usize>,
}

#[derive(Clone, Copy)]
struct Slot {
    bucket: usize,
    key: usize,
}

fn try_generate_hash(seed: u64, hashes: &[u64]) -> Option<HashState> {
    let table_len = hashes.len();
    let table_len_u32 = table_len.try_into().unwrap();
    let slots_len = adjusted_slots_len(table_len_u32) as usize;
    let buckets_len = adjusted_buckets_len(table_len_u32) as usize;

    let mut buckets = (0..buckets_len)
        .map(|_| Bucket::default())
        .collect::<Vec<_>>();
    let mut pilots = vec![0; buckets_len];
    let mut order = (0..buckets_len).collect::<Vec<_>>();
    let mut slots = vec![None; slots_len];
    let mut stack = Vec::new();
    let mut recent = Vec::new();
    let mut values_to_add = Vec::with_capacity((DEFAULT_LAMBDA as usize) * 2);
    let mut already_scored = Vec::new();

    for (idx, &hash) in hashes.iter().enumerate() {
        let bucket = fast_reduct32(low(hash), buckets_len as u32) as usize;
        buckets[bucket].keys.push(idx);
    }

    order.sort_unstable_by_key(|&bucket| cmp::Reverse(buckets[bucket].keys.len()));

    for &bucket in &order {
        if buckets[bucket].keys.is_empty() {
            continue;
        }

        recent.clear();
        stack.clear();
        stack.push(bucket);

        'bucket: while let Some(bucket) = {
            stack.sort_unstable_by_key(|&bucket| buckets[bucket].keys.len());
            stack.pop()
        } {
            recent.push(bucket);

            for pilot in 0..=u8::MAX {
                if try_place_bucket(
                    &buckets,
                    &mut slots,
                    &mut pilots,
                    &mut values_to_add,
                    hashes,
                    seed,
                    bucket,
                    pilot,
                ) {
                    continue 'bucket;
                }
            }

            let mut best = None;

            'pilot: for offset in 0..=u8::MAX {
                let pilot = offset.wrapping_add(0x42);
                let pilot_hash = hash_pilot(seed, pilot);
                let mut score = 0;

                values_to_add.clear();
                already_scored.clear();

                for &key in &buckets[bucket].keys {
                    let slot = slot_index(hashes[key], pilot_hash, slots.len() as u32);
                    if values_to_add.iter().any(|&(seen, _)| seen == slot) {
                        continue 'pilot;
                    }

                    let extra = match slots[slot as usize] {
                        None => 0,
                        Some(slot) if recent.contains(&slot.bucket) => continue 'pilot,
                        Some(slot) if !already_scored.contains(&slot.bucket) => {
                            already_scored.push(slot.bucket);
                            buckets[slot.bucket].keys.len().pow(2)
                        }
                        Some(_) => 0,
                    };

                    values_to_add.push((slot, key));
                    score += extra;

                    if best
                        .map(|(best_score, _)| score >= best_score)
                        .unwrap_or(false)
                    {
                        continue 'pilot;
                    }
                }

                best = Some((score, pilot));
                if score == buckets[bucket].keys.len().pow(2) {
                    break;
                }
            }

            let (_, pilot) = best?;
            pilots[bucket] = pilot;
            let pilot_hash = hash_pilot(seed, pilot);

            for &key in &buckets[bucket].keys {
                let slot = slot_index(hashes[key], pilot_hash, slots.len() as u32) as usize;
                if let Some(previous) = slots[slot].replace(Slot { bucket, key }) {
                    stack.push(previous.bucket);

                    let previous_hash = hash_pilot(seed, pilots[previous.bucket]);
                    let slots_len = slots.len() as u32;
                    for previous_slot in buckets[previous.bucket]
                        .keys
                        .iter()
                        .map(|&key| slot_index(hashes[key], previous_hash, slots_len) as usize)
                        .filter(|&previous_slot| previous_slot != slot)
                    {
                        slots[previous_slot] = None;
                    }
                }
            }
        }
    }

    let mut map = vec![0; table_len];
    let mut remap = vec![0; slots.len() - table_len];
    let mut free_slots = Vec::new();

    for (slot, entry) in slots.iter().enumerate() {
        match (slot < table_len, entry) {
            (true, Some(entry)) => map[slot] = entry.key,
            (true, None) => free_slots.push(slot),
            (false, Some(entry)) => {
                let remapped = free_slots.pop().unwrap();
                remap[slot - table_len] = remapped.try_into().unwrap();
                map[remapped] = entry.key;
            }
            (false, None) => {}
        }
    }

    Some(HashState {
        seed,
        pilots,
        remap,
        map,
    })
}

fn adjusted_slots_len(keys_len: u32) -> u32 {
    let len = (f64::from(keys_len) / DEFAULT_ALPHA).ceil() as u32;
    len + u32::from(len.is_power_of_two())
}

fn adjusted_buckets_len(keys_len: u32) -> u32 {
    let len = (f64::from(keys_len) / DEFAULT_LAMBDA).ceil() as u32;
    len + 3
}

fn try_place_bucket(
    buckets: &[Bucket],
    slots: &mut [Option<Slot>],
    pilots: &mut [u8],
    values_to_add: &mut Vec<(u32, usize)>,
    hashes: &[u64],
    seed: u64,
    bucket: usize,
    pilot: u8,
) -> bool {
    let pilot_hash = hash_pilot(seed, pilot);
    values_to_add.clear();

    for &key in &buckets[bucket].keys {
        let slot = slot_index(hashes[key], pilot_hash, slots.len() as u32);
        if slots[slot as usize].is_some() || values_to_add.iter().any(|&(seen, _)| seen == slot) {
            return false;
        }

        values_to_add.push((slot, key));
    }

    pilots[bucket] = pilot;
    for &(slot, key) in values_to_add.iter() {
        slots[slot as usize] = Some(Slot { bucket, key });
    }

    true
}

#[inline]
fn slot_index(hash: u64, pilot_hash: u64, slots_len: u32) -> u32 {
    fast_reduct32(high(hash) ^ high(pilot_hash) ^ low(pilot_hash), slots_len)
}

#[inline]
fn low(v: u64) -> u32 {
    v as u32
}

#[inline]
fn high(v: u64) -> u32 {
    (v >> 32) as u32
}
