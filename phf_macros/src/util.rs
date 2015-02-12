use std::env;
use std::rc::Rc;
use std::hash::{self, Hash, Hasher};
use std::iter::repeat;

use syntax::ast::Expr;
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult, MacExpr};
use syntax::ext::build::AstBuilder;
use syntax::parse::token::InternedString;
use syntax::ptr::P;
use rand::{Rng, SeedableRng, XorShiftRng};

use phf_shared::{self, PhfHash};

const DEFAULT_LAMBDA: usize = 5;

const FIXED_SEED: [u32; 4] = [3141592653, 589793238, 462643383, 2795028841];

#[derive(PartialEq, Eq, Clone)]
pub enum Key {
    Str(InternedString),
    Binary(Rc<Vec<u8>>),
    Char(char),
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    Bool(bool),
}

impl<S> Hash<S> for Key where S: Hasher + hash::Writer {
    fn hash(&self, state: &mut S) {
        match *self {
            Key::Str(ref s) => s.hash(state),
            Key::Binary(ref b) => b.hash(state),
            Key::Char(c) => c.hash(state),
            Key::U8(b) => b.hash(state),
            Key::I8(b) => b.hash(state),
            Key::U16(b) => b.hash(state),
            Key::I16(b) => b.hash(state),
            Key::U32(b) => b.hash(state),
            Key::I32(b) => b.hash(state),
            Key::U64(b) => b.hash(state),
            Key::I64(b) => b.hash(state),
            Key::Bool(b) => b.hash(state),
        }
    }
}

impl PhfHash for Key {
    fn phf_hash(&self, key: u64) -> (u32, u32, u32) {
        match *self {
            Key::Str(ref s) => s.phf_hash(key),
            Key::Binary(ref b) => b.phf_hash(key),
            Key::Char(c) => c.phf_hash(key),
            Key::U8(b) => b.phf_hash(key),
            Key::I8(b) => b.phf_hash(key),
            Key::U16(b) => b.phf_hash(key),
            Key::I16(b) => b.phf_hash(key),
            Key::U32(b) => b.phf_hash(key),
            Key::I32(b) => b.phf_hash(key),
            Key::U64(b) => b.phf_hash(key),
            Key::I64(b) => b.phf_hash(key),
            Key::Bool(b) => b.phf_hash(key),
        }
    }
}

pub struct Entry {
    pub key_contents: Key,
    pub key: P<Expr>,
    pub value: P<Expr>
}

pub struct HashState {
    key: u64,
    disps: Vec<(u32, u32)>,
    map: Vec<usize>,
}

pub fn generate_hash(cx: &mut ExtCtxt, sp: Span, entries: &[Entry]) -> HashState {
    #[cfg(feature = "stats")]
    use time::precise_time_s;
    #[cfg(not(feature = "stats"))]
    fn precise_time_s() -> f64 { 0.0 }

    let mut rng: XorShiftRng = SeedableRng::from_seed(FIXED_SEED);
    let start = precise_time_s();
    let state;
    loop {
        match try_generate_hash(entries, &mut rng) {
            Some(s) => {
                state = s;
                break;
            }
            None => {}
        }
    }
    let time = precise_time_s() - start;
    if cfg!(feature = "stats") && env::var_os("PHF_STATS").is_some() {
        cx.span_note(sp, &*format!("PHF generation took {} seconds", time));
    }

    state
}

pub fn try_generate_hash(entries: &[Entry], rng: &mut XorShiftRng) -> Option<HashState> {
    struct Bucket {
        idx: usize,
        keys: Vec<usize>,
    }

    struct Hashes {
        g: u32,
        f1: u32,
        f2: u32,
    }

    let key = rng.gen();

    let hashes: Vec<_> = entries.iter().map(|entry| {
        let (g, f1, f2) = entry.key_contents.phf_hash(key);
        Hashes {
            g: g,
            f1: f1,
            f2: f2
        }
    }).collect();

    let buckets_len = (entries.len() + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA;
    let mut buckets = range(0, buckets_len)
        .map(|i| Bucket { idx: i, keys: vec![] })
        .collect::<Vec<_>>();

    for (i, hash) in hashes.iter().enumerate() {
        buckets[(hash.g % (buckets_len as u32)) as usize].keys.push(i);
    }

    // Sort descending
    buckets.sort_by(|a, b| a.keys.len().cmp(&b.keys.len()).reverse());

    let table_len = entries.len();
    let mut map = repeat(None).take(table_len).collect::<Vec<_>>();
    let mut disps = repeat((0u32, 0u32)).take(buckets_len).collect::<Vec<_>>();

    // store whether an element from the bucket being placed is
    // located at a certain position, to allow for efficient overlap
    // checks. It works by storing the generation in each cell and
    // each new placement-attempt is a new generation, so you can tell
    // if this is legitimately full by checking that the generations
    // are equal. (A u64 is far too large to overflow in a reasonable
    // time for current hardware.)
    let mut try_map = repeat(0u64).take(table_len).collect::<Vec<_>>();
    let mut generation = 0u64;

    // the actual values corresponding to the markers above, as
    // (index, key) pairs, for adding to the main map once we've
    // chosen the right disps.
    let mut values_to_add = vec![];

    'buckets: for bucket in buckets.iter() {
        for d1 in range(0, table_len as u32) {
            'disps: for d2 in range(0, table_len as u32) {
                values_to_add.clear();
                generation += 1;

                for &key in bucket.keys.iter() {
                    let idx = (phf_shared::displace(hashes[key].f1, hashes[key].f2, d1, d2)
                                % (table_len as u32)) as usize;
                    if map[idx].is_some() || try_map[idx] == generation {
                        continue 'disps;
                    }
                    try_map[idx] = generation;
                    values_to_add.push((idx, key));
                }

                // We've picked a good set of disps
                disps[bucket.idx] = (d1, d2);
                for &(idx, key) in values_to_add.iter() {
                    map[idx] = Some(key);
                }
                continue 'buckets;
            }
        }

        // Unable to find displacements for a bucket
        return None;
    }

    Some(HashState {
        key: key,
        disps: disps,
        map: map.into_iter().map(|i| i.unwrap()).collect(),
    })
}

pub fn create_map(cx: &mut ExtCtxt, sp: Span, entries: Vec<Entry>, state: HashState)
                  -> Box<MacResult+'static> {
    let disps = state.disps.iter().map(|&(d1, d2)| {
        quote_expr!(&*cx, ($d1, $d2))
    }).collect();
    let disps = cx.expr_vec(sp, disps);

    let entries = state.map.iter().map(|&idx| {
        let &Entry { ref key, ref value, .. } = &entries[idx];
        quote_expr!(&*cx, ($key, $value))
    }).collect();
    let entries = cx.expr_vec(sp, entries);

    let key = state.key;
    MacExpr::new(quote_expr!(cx, ::phf::Map {
        key: $key,
        disps: &$disps,
        entries: &$entries,
    }))
}

pub fn create_set(cx: &mut ExtCtxt, sp: Span, entries: Vec<Entry>, state: HashState)
              -> Box<MacResult+'static> {
    let map = create_map(cx, sp, entries, state).make_expr().unwrap();
    MacExpr::new(quote_expr!(cx, ::phf::Set { map: $map }))
}

pub fn create_ordered_map(cx: &mut ExtCtxt, sp: Span, entries: Vec<Entry>, state: HashState)
                          -> Box<MacResult+'static> {
    let disps = state.disps.iter().map(|&(d1, d2)| {
        quote_expr!(&*cx, ($d1, $d2))
    }).collect();
    let disps = cx.expr_vec(sp, disps);

    let idxs = state.map.iter().map(|&idx| quote_expr!(&*cx, $idx)).collect();
    let idxs = cx.expr_vec(sp, idxs);

    let entries = entries.iter().map(|&Entry { ref key, ref value, .. }| {
        quote_expr!(&*cx, ($key, $value))
    }).collect();
    let entries = cx.expr_vec(sp, entries);

    let key = state.key;
    MacExpr::new(quote_expr!(cx, ::phf::OrderedMap {
        key: $key,
        disps: &$disps,
        idxs: &$idxs,
        entries: &$entries,
    }))
}

pub fn create_ordered_set(cx: &mut ExtCtxt, sp: Span, entries: Vec<Entry>, state: HashState)
                          -> Box<MacResult+'static> {
    let map = create_ordered_map(cx, sp, entries, state).make_expr().unwrap();
    MacExpr::new(quote_expr!(cx, ::phf::OrderedSet { map: $map }))
}
