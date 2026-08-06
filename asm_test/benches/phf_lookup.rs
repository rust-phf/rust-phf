use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use phf::phf_map;
use std::time::Duration;

// ── 所有 map 使用统一 8 字节 key，只改变条目数 ──────────────────────────────
//
// tiny  ( 3 entries): disps.len == 1  ← 优化命中
// small ( 6 entries): disps.len == 2
// med   (12 entries): disps.len == 4
// large (30 entries): disps.len == 10

static TINY_MAP: phf::Map<&'static str, u32> = phf_map! {
    "key00001" => 1, "key00002" => 2, "key00003" => 3,
};

static SMALL_MAP: phf::Map<&'static str, u32> = phf_map! {
    "key00001" => 1, "key00002" => 2, "key00003" => 3,
    "key00004" => 4, "key00005" => 5, "key00006" => 6,
};

static MED_MAP: phf::Map<&'static str, u32> = phf_map! {
    "key00001" =>  1, "key00002" =>  2, "key00003" =>  3,
    "key00004" =>  4, "key00005" =>  5, "key00006" =>  6,
    "key00007" =>  7, "key00008" =>  8, "key00009" =>  9,
    "key00010" => 10, "key00011" => 11, "key00012" => 12,
};

static LARGE_MAP: phf::Map<&'static str, u32> = phf_map! {
    "key00001" =>  1, "key00002" =>  2, "key00003" =>  3,
    "key00004" =>  4, "key00005" =>  5, "key00006" =>  6,
    "key00007" =>  7, "key00008" =>  8, "key00009" =>  9,
    "key00010" => 10, "key00011" => 11, "key00012" => 12,
    "key00013" => 13, "key00014" => 14, "key00015" => 15,
    "key00016" => 16, "key00017" => 17, "key00018" => 18,
    "key00019" => 19, "key00020" => 20, "key00021" => 21,
    "key00022" => 22, "key00023" => 23, "key00024" => 24,
    "key00025" => 25, "key00026" => 26, "key00027" => 27,
    "key00028" => 28, "key00029" => 29, "key00030" => 30,
};

// ── 批量查询集合（各 32 条，循环覆盖所有 key，消除单 key 缓存偏差）─────────
const HITS: &[&str] = &[
    "key00001", "key00002", "key00003", "key00001", "key00002", "key00003", "key00001", "key00002",
    "key00003", "key00001", "key00002", "key00003", "key00001", "key00002", "key00003", "key00001",
    "key00002", "key00003", "key00001", "key00002", "key00003", "key00001", "key00002", "key00003",
    "key00001", "key00002", "key00003", "key00001", "key00002", "key00003", "key00001", "key00002",
];

// miss key 与 map key 等长（8字节），保证哈希成本相同
const MISSES: &[&str] = &[
    "mis00001", "mis00002", "mis00003", "mis00004", "mis00005", "mis00006", "mis00007", "mis00008",
    "mis00009", "mis00010", "mis00011", "mis00012", "mis00013", "mis00014", "mis00015", "mis00016",
    "mis00017", "mis00018", "mis00019", "mis00020", "mis00021", "mis00022", "mis00023", "mis00024",
    "mis00025", "mis00026", "mis00027", "mis00028", "mis00029", "mis00030", "mis00031", "mis00032",
];

// ── 通用 bench helper ────────────────────────────────────────────────────────

fn bench_map<F>(c: &mut Criterion, group_name: &str, disps_len: usize, mut lookup: F)
where
    F: FnMut(&str) -> Option<u32>,
{
    let label = format!("{group_name} (disps.len={disps_len})");
    let mut g = c.benchmark_group(&label);
    g.warm_up_time(Duration::from_secs(5));
    g.measurement_time(Duration::from_secs(10));
    g.sample_size(500);

    // throughput: 每次迭代处理 N 条查询
    g.throughput(Throughput::Elements(HITS.len() as u64));

    g.bench_function("hit_batch", |b| {
        b.iter(|| {
            let mut sum = 0u32;
            for key in HITS {
                sum = sum.wrapping_add(lookup(black_box(key)).unwrap_or(0));
            }
            black_box(sum)
        })
    });

    g.throughput(Throughput::Elements(MISSES.len() as u64));

    g.bench_function("miss_batch", |b| {
        b.iter(|| {
            let mut found = 0u32;
            for key in MISSES {
                found = found.wrapping_add(lookup(black_box(key)).is_some() as u32);
            }
            black_box(found)
        })
    });

    g.finish();
}

fn bench_tiny(c: &mut Criterion) {
    bench_map(c, "tiny", TINY_MAP.disps.len(), |k| {
        TINY_MAP.get(k).copied()
    });
}

fn bench_small(c: &mut Criterion) {
    bench_map(c, "small", SMALL_MAP.disps.len(), |k| {
        SMALL_MAP.get(k).copied()
    });
}

fn bench_med(c: &mut Criterion) {
    bench_map(c, "med", MED_MAP.disps.len(), |k| MED_MAP.get(k).copied());
}

fn bench_large(c: &mut Criterion) {
    bench_map(c, "large", LARGE_MAP.disps.len(), |k| {
        LARGE_MAP.get(k).copied()
    });
}

criterion_group!(benches, bench_tiny, bench_small, bench_med, bench_large);
criterion_main!(benches);
