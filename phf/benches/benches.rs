use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};
use phf::{Map, Set};
use phf_shared::{displace, hash, FastModulo, HashKey, PhfHash};
use std::borrow::Borrow;
use std::hint::black_box;
use std::time::Duration;

// --- Baseline Implementation ---
mod baseline {
    use super::*;

    pub struct Set<K: 'static> {
        pub key: HashKey,
        pub disps: &'static [(u32, u32)],
        pub entries: &'static [(K, ())],
    }

    impl<K: PhfHash + Eq> Set<K> {
        pub fn contains<T>(&self, key: &T) -> bool
        where
            K: Borrow<T>,
            T: PhfHash + Eq + ?Sized,
        {
            if self.disps.is_empty() {
                return false;
            }

            let hashes = hash(key, &self.key);
            let (d1, d2) = self.disps[(hashes.g % self.disps.len() as u32) as usize];
            let index = displace(hashes.f1, hashes.f2, d1, d2) % self.entries.len() as u32;

            let entry: &T = self.entries[index as usize].0.borrow();
            entry == key
        }
    }
}

fn gen_vec(size: usize) -> Vec<u64> {
    let mut rng = fastrand::Rng::with_seed(12345);
    (0..size).map(|_| rng.u64(..)).collect()
}

struct PhfData {
    key: HashKey,
    disps: &'static [(u32, u32)],
    entries: &'static [(u64, ())],
}

fn setup_phf_data(vec: &[u64]) -> PhfData {
    let state = phf_generator::generate_hash(vec);
    let entries_vec: Vec<_> = state.map.iter().map(|&idx| (vec[idx], ())).collect();

    PhfData {
        key: state.key,
        disps: Box::leak(state.disps.into_boxed_slice()),
        entries: Box::leak(entries_vec.into_boxed_slice()),
    }
}

fn bench_lookup_baseline(b: &mut Bencher, data: &(Vec<u64>, PhfData)) {
    let (vec, phf_data) = data;
    let set = baseline::Set {
        key: phf_data.key,
        disps: phf_data.disps,
        entries: phf_data.entries,
    };

    b.iter(|| {
        for key in vec {
            black_box(set.contains(key));
        }
    });
}

fn bench_lookup_experiment(b: &mut Bencher, data: &(Vec<u64>, PhfData)) {
    let (vec, phf_data) = data;
    let set = Set {
        map: Map {
            key: phf_data.key,
            disps: phf_data.disps,
            entries: phf_data.entries,
            disps_len: FastModulo::new(phf_data.disps.len() as u32),
            entries_len: FastModulo::new(phf_data.entries.len() as u32),
        },
    };

    b.iter(|| {
        for key in vec {
            black_box(set.contains(key));
        }
    });
}

fn compare_lookups(c: &mut Criterion) {
    let mut group = c.benchmark_group("phf_lookup_hot_path");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(3));

    let sizes = vec![
        1, 2, 5, 10, 25, 50, 75, 100, 250, 500, 1000, 2500, 5000, 7500, 10_000, 25_000, 50_000,
        75_000, 100_000, 250_000, 500_000, 750_000, 1_000_000,
    ];

    for size in &sizes {
        let vec = gen_vec(*size);
        let phf_data = setup_phf_data(&vec);
        let bench_data = (vec, phf_data);

        group.bench_with_input(
            BenchmarkId::new("baseline_lookup", size),
            &bench_data,
            bench_lookup_baseline,
        );
        group.bench_with_input(
            BenchmarkId::new("experiment_lookup", size),
            &bench_data,
            bench_lookup_experiment,
        );
    }
    group.finish();
}

criterion_group!(benches, compare_lookups);
criterion_main!(benches);
