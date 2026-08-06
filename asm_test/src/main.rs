use phf::phf_map;

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

#[cfg(not(feature = "ptrhash"))]
fn print_info() {
    println!(
        "TINY_MAP:  disps.len={}, entries={}",
        TINY_MAP.disps.len(),
        TINY_MAP.len()
    );
    println!(
        "SMALL_MAP: disps.len={}, entries={}",
        SMALL_MAP.disps.len(),
        SMALL_MAP.len()
    );
    println!(
        "MED_MAP:   disps.len={}, entries={}",
        MED_MAP.disps.len(),
        MED_MAP.len()
    );
    println!(
        "LARGE_MAP: disps.len={}, entries={}",
        LARGE_MAP.disps.len(),
        LARGE_MAP.len()
    );
}

#[cfg(feature = "ptrhash")]
fn print_info() {
    println!("TINY_MAP:  entries={}", TINY_MAP.len());
    println!("SMALL_MAP: entries={}", SMALL_MAP.len());
    println!("MED_MAP:   entries={}", MED_MAP.len());
    println!("LARGE_MAP: entries={}", LARGE_MAP.len());
}

fn main() {
    print_info();
    println!();
    // 验证 HITS 里的 key 确实都存在
    for k in &["key00001", "key00002", "key00003"] {
        println!("TINY_MAP.get({k:?}) = {:?}", TINY_MAP.get(k));
    }
    // 验证 MISSES 确实不存在
    println!(
        "TINY_MAP.get(\"mis00001\") = {:?}",
        TINY_MAP.get("mis00001")
    );
}
