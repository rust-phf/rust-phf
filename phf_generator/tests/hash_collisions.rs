use phf_generator::generate_hash;
use phf_shared::{get_index, hash};

#[test]
fn fixed_shared_seed_collisions() {
    // These upper words cancel foldhash's fixed shared seed and the shared
    // seed derived from zero, respectively. Varying only the per-hasher seed
    // cannot separate the keys in each family.
    for upper in [0xc0ac29b7c97c50ddu128, 0x8000000080000001] {
        let keys = (0..64)
            .map(|lower| (upper << 64) | lower)
            .collect::<Vec<_>>();
        let state = generate_hash(&keys);
        for (index, key) in keys.iter().enumerate() {
            let hashes = hash(key, &state.key);
            let slot = get_index(&hashes, &state.disps, state.map.len()) as usize;
            assert_eq!(state.map[slot], index);
        }

        #[cfg(feature = "ptrhash")]
        {
            let state = phf_generator::ptrhash::generate_hash(&keys);
            for (index, key) in keys.iter().enumerate() {
                let hash = phf_shared::ptrhash::hash(key, &state.key);
                let slot = phf_shared::ptrhash::get_index(
                    state.key.seed(),
                    hash,
                    &state.pilots,
                    state.map.len(),
                ) as usize;
                assert_eq!(state.map[slot], index);
            }
        }
    }
}

#[test]
#[should_panic(expected = "failed to solve PHF after 1000 seeds")]
fn inseparable_keys_terminate() {
    // The low-level generator also accepts custom PhfHash implementations that
    // cannot distinguish unequal keys. Do not loop indefinitely in that case.
    phf_generator::generate_hash_with_hash_fn(&[0, 1], |_, key| hash(&0u64, key));
}

#[cfg(feature = "ptrhash")]
#[test]
#[should_panic(expected = "failed to solve PHF after 1000 seeds")]
fn inseparable_ptrhash_keys_terminate() {
    phf_generator::ptrhash::generate_hash_with_hash_fn(&[0, 1], |_, _| 0);
}
