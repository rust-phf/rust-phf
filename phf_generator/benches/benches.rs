use std::iter;

use criterion::measurement::Measurement;
use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};

use fastrand::Rng;

use phf_generator::generate_hash;

fn gen_vec(seed: u64, len: usize) -> Vec<u64> {
    let mut rng = Rng::with_seed(seed);
    iter::repeat_with(|| rng.u64(..)).take(len).collect()
}

fn bench_hash<M: Measurement>(b: &mut Bencher<M>, len: &usize) {
    let mut seed = 0xAAAAAAAAAAAAAAAA;
    b.iter_batched_ref(
        || {
            seed += 1;
            gen_vec(seed, *len)
        },
        |v| generate_hash(v),
        criterion::BatchSize::PerIteration,
    );
}

fn gen_hash_small(c: &mut Criterion) {
    let sizes = vec![0, 1, 2, 5, 10, 25, 50, 75];
    for size in &sizes {
        c.bench_with_input(BenchmarkId::new("gen_hash_small", *size), size, bench_hash);
    }
}

fn gen_hash_med(c: &mut Criterion) {
    let sizes = vec![100, 250, 500, 1000, 2500, 5000, 7500];
    for size in &sizes {
        c.bench_with_input(BenchmarkId::new("gen_hash_medium", *size), size, bench_hash);
    }
}

fn gen_hash_large(c: &mut Criterion) {
    let sizes = vec![10_000, 25_000, 50_000, 75_000];
    for size in &sizes {
        c.bench_with_input(BenchmarkId::new("gen_hash_large", *size), size, bench_hash);
    }
}

fn gen_hash_xlarge(c: &mut Criterion) {
    let sizes = vec![100_000, 250_000, 500_000, 750_000, 1_000_000];
    for size in &sizes {
        c.bench_with_input(BenchmarkId::new("gen_hash_xlarge", *size), size, bench_hash);
    }
}

criterion_group!(
    benches,
    gen_hash_small,
    gen_hash_med,
    gen_hash_large,
    gen_hash_xlarge
);

criterion_main!(benches);
