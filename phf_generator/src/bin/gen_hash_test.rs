use criterion::*;

use phf_generator::{generate_hash, rng::Rng};

fn gen_vec(len: usize) -> Vec<String> {
    let mut rng = Rng::new(0xAAAAAAAAAAAAAAAA);

    (0..len)
        .map(move |_| {
            let mut str = String::with_capacity(64);
            (0..64).for_each(|_| str.push(rng.generate_alphanumeric()));
            str
        })
        .collect()
}

fn main() {
    let data = black_box(gen_vec(1_000_000));
    black_box(generate_hash(&data));
}
