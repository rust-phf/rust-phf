use std::hint::black_box;
use std::iter;

use criterion::measurement::Measurement;
#[cfg(feature = "ptrhash")]
use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};

use fastrand::Rng;

use phf_generator::generate_hash;
#[cfg(feature = "ptrhash")]
use phf_generator::{ptrhash, HashState};

fn gen_numbers(len: usize) -> Vec<u64> {
    let mut rng = Rng::with_seed(0xAAAAAAAAAAAAAAAA);
    iter::repeat_with(|| rng.u64(..)).take(len).collect()
}

#[cfg(feature = "ptrhash")]
fn gen_strings(len: usize) -> Vec<String> {
    let mut rng = Rng::with_seed(0x5555555555555555);
    let mut chars = iter::repeat_with(|| rng.alphanumeric());

    (0..len)
        .map(move |_| chars.by_ref().take(32).collect::<String>())
        .collect()
}

fn bench_default_hash<M: Measurement>(b: &mut Bencher<M>, len: &usize) {
    let numbers = gen_numbers(*len);
    b.iter(|| black_box(generate_hash(black_box(&numbers))))
}

#[cfg(feature = "ptrhash")]
fn bench_ptrhash<M: Measurement>(b: &mut Bencher<M>, keys: &&[&str]) {
    b.iter(|| black_box(ptrhash::generate_hash(black_box(keys))))
}

#[cfg(feature = "ptrhash")]
fn bench_default<M: Measurement>(b: &mut Bencher<M>, keys: &&[&str]) {
    b.iter(|| black_box(generate_hash(black_box(keys))))
}

#[cfg(feature = "ptrhash")]
struct LookupBench {
    keys: Vec<String>,
    default: HashState,
    ptrhash: ptrhash::HashState,
    hits: Vec<String>,
    misses: Vec<String>,
}

#[cfg(feature = "ptrhash")]
fn build_lookup_bench(len: usize) -> LookupBench {
    let keys = gen_strings(len);
    let key_refs: Vec<_> = keys.iter().map(String::as_str).collect();
    let default = generate_hash(&key_refs);
    let ptrhash = ptrhash::generate_hash(&key_refs);
    let hits = (0..64)
        .map(|idx| keys[idx % len].clone())
        .collect::<Vec<_>>();
    let misses = (0..64)
        .map(|idx| format!("missing:{len}:{idx}"))
        .collect::<Vec<_>>();

    LookupBench {
        keys,
        default,
        ptrhash,
        hits,
        misses,
    }
}

#[cfg(feature = "ptrhash")]
fn default_lookup(data: &LookupBench, key: &str) -> bool {
    let hashes = phf_shared::hash(key, &data.default.key);
    let slot = phf_shared::get_index(&hashes, &data.default.disps, data.default.map.len()) as usize;
    data.keys[data.default.map[slot]] == key
}

#[cfg(feature = "ptrhash")]
fn ptrhash_lookup(data: &LookupBench, key: &str) -> bool {
    let hash = phf_shared::ptrhash::hash(key, &data.ptrhash.seed);
    let slot = phf_shared::ptrhash::get_index(
        data.ptrhash.seed,
        hash,
        &data.ptrhash.pilots,
        &data.ptrhash.remap,
        data.ptrhash.map.len(),
    ) as usize;
    data.keys[data.ptrhash.map[slot]] == key
}

#[cfg(feature = "ptrhash")]
fn bench_lookup_queries<M: Measurement>(
    b: &mut Bencher<M>,
    data: &LookupBench,
    queries: &[String],
    lookup: fn(&LookupBench, &str) -> bool,
) {
    b.iter(|| {
        let mut found = 0usize;
        for query in queries {
            found += usize::from(lookup(data, black_box(query.as_str())));
        }
        black_box(found)
    })
}

fn gen_hash_small(c: &mut Criterion) {
    let sizes = vec![0, 1, 2, 5, 10, 25, 50, 75];
    for size in &sizes {
        c.bench_with_input(
            BenchmarkId::new("gen_hash_small", *size),
            size,
            bench_default_hash,
        );
    }
}

fn gen_hash_med(c: &mut Criterion) {
    let sizes = vec![100, 250, 500, 1000, 2500, 5000, 7500];
    for size in &sizes {
        c.bench_with_input(
            BenchmarkId::new("gen_hash_medium", *size),
            size,
            bench_default_hash,
        );
    }
}

fn gen_hash_large(c: &mut Criterion) {
    let sizes = vec![10_000, 25_000, 50_000, 75_000];
    for size in &sizes {
        c.bench_with_input(
            BenchmarkId::new("gen_hash_large", *size),
            size,
            bench_default_hash,
        );
    }
}

fn gen_hash_xlarge(c: &mut Criterion) {
    let sizes = vec![100_000, 250_000, 500_000, 750_000, 1_000_000];
    for size in &sizes {
        c.bench_with_input(
            BenchmarkId::new("gen_hash_xlarge", *size),
            size,
            bench_default_hash,
        );
    }
}

#[cfg(feature = "ptrhash")]
fn gen_hash_compare(c: &mut Criterion) {
    for size in [0, 1, 10, 100, 1_000, 10_000] {
        let keys = gen_strings(size);
        let key_refs = keys.iter().map(String::as_str).collect::<Vec<_>>();
        let key_slice = key_refs.as_slice();
        let mut group = c.benchmark_group(format!("gen_hash_compare/{size}"));

        group.bench_with_input(BenchmarkId::new("default", size), &key_slice, bench_default);
        group.bench_with_input(BenchmarkId::new("ptrhash", size), &key_slice, bench_ptrhash);
        group.finish();
    }
}

#[cfg(feature = "ptrhash")]
fn lookup_hits(c: &mut Criterion) {
    for size in [1, 10, 100, 1_000, 10_000] {
        let data = build_lookup_bench(size);
        let mut group = c.benchmark_group(format!("lookup_hits/{size}"));
        group.throughput(Throughput::Elements(data.hits.len() as u64));
        group.bench_function("default", |b| {
            bench_lookup_queries(b, &data, &data.hits, default_lookup)
        });
        group.bench_function("ptrhash", |b| {
            bench_lookup_queries(b, &data, &data.hits, ptrhash_lookup)
        });
        group.finish();
    }
}

#[cfg(feature = "ptrhash")]
fn lookup_misses(c: &mut Criterion) {
    for size in [1, 10, 100, 1_000, 10_000] {
        let data = build_lookup_bench(size);
        let mut group = c.benchmark_group(format!("lookup_misses/{size}"));
        group.throughput(Throughput::Elements(data.misses.len() as u64));
        group.bench_function("default", |b| {
            bench_lookup_queries(b, &data, &data.misses, default_lookup)
        });
        group.bench_function("ptrhash", |b| {
            bench_lookup_queries(b, &data, &data.misses, ptrhash_lookup)
        });
        group.finish();
    }
}

#[cfg(feature = "ptrhash")]
criterion_group!(
    benches,
    gen_hash_small,
    gen_hash_med,
    gen_hash_large,
    gen_hash_xlarge,
    gen_hash_compare,
    lookup_hits,
    lookup_misses
);

#[cfg(not(feature = "ptrhash"))]
criterion_group!(
    benches,
    gen_hash_small,
    gen_hash_med,
    gen_hash_large,
    gen_hash_xlarge
);

criterion_main!(benches);
