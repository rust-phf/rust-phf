use std::os;
use std::rc::Rc;
use std::hash;
use std::hash::Hash;

use syntax::ast::Expr;
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt,
                        MacResult,
                        MacExpr};
use syntax::ext::build::AstBuilder;
use syntax::parse::token::InternedString;
use syntax::ptr::P;
use rand::{Rng, SeedableRng, XorShiftRng};

use shared::PhfHash;

use time;

use self::Key::{KeyStr, KeyBinary, KeyChar, KeyU8, KeyI8, KeyU16};
use self::Key::{KeyI16, KeyU32, KeyI32, KeyU64, KeyI64, KeyBool};

static DEFAULT_LAMBDA: uint = 5;

static FIXED_SEED: [u32, ..4] = [3141592653, 589793238, 462643383, 2795028841];

#[deriving(PartialEq, Eq, Clone)]
pub enum Key {
    KeyStr(InternedString),
    KeyBinary(Rc<Vec<u8>>),
    KeyChar(char),
    KeyU8(u8),
    KeyI8(i8),
    KeyU16(u16),
    KeyI16(i16),
    KeyU32(u32),
    KeyI32(i32),
    KeyU64(u64),
    KeyI64(i64),
    KeyBool(bool),
}

impl<S> Hash<S> for Key where S: hash::Writer {
    fn hash(&self, state: &mut S) {
        match *self {
            KeyStr(ref s) => s.get().hash(state),
            KeyBinary(ref b) => b.hash(state),
            KeyChar(c) => c.hash(state),
            KeyU8(b) => b.hash(state),
            KeyI8(b) => b.hash(state),
            KeyU16(b) => b.hash(state),
            KeyI16(b) => b.hash(state),
            KeyU32(b) => b.hash(state),
            KeyI32(b) => b.hash(state),
            KeyU64(b) => b.hash(state),
            KeyI64(b) => b.hash(state),
            KeyBool(b) => b.hash(state),
        }
    }
}

impl PhfHash for Key {
    fn phf_hash(&self, key: u64) -> (u32, u32, u32) {
        match *self {
            KeyStr(ref s) => s.get().phf_hash(key),
            KeyBinary(ref b) => (**b)[].phf_hash(key),
            KeyChar(c) => c.phf_hash(key),
            KeyU8(b) => b.phf_hash(key),
            KeyI8(b) => b.phf_hash(key),
            KeyU16(b) => b.phf_hash(key),
            KeyI16(b) => b.phf_hash(key),
            KeyU32(b) => b.phf_hash(key),
            KeyI32(b) => b.phf_hash(key),
            KeyU64(b) => b.phf_hash(key),
            KeyI64(b) => b.phf_hash(key),
            KeyBool(b) => b.phf_hash(key),
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
    map: Vec<uint>,
}

pub fn generate_hash(cx: &mut ExtCtxt, sp: Span, entries: &[Entry]) -> HashState {
    let mut rng: XorShiftRng = SeedableRng::from_seed(FIXED_SEED);
    let start = time::precise_time_s();
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
    let time = time::precise_time_s() - start;
    if os::getenv("PHF_STATS").is_some() {
        cx.span_note(sp, format!("PHF generation took {} seconds", time)[]);
    }

    state
}

pub fn try_generate_hash(entries: &[Entry], rng: &mut XorShiftRng) -> Option<HashState> {
    struct Bucket {
        idx: uint,
        keys: Vec<uint>,
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
    let mut buckets = Vec::from_fn(buckets_len, |i| Bucket { idx: i, keys: vec![] });

    for (i, hash) in hashes.iter().enumerate() {
        buckets[(hash.g % (buckets_len as u32)) as uint].keys.push(i);
    }

    // Sort descending
    buckets.sort_by(|a, b| a.keys.len().cmp(&b.keys.len()).reverse());

    let table_len = entries.len();
    let mut map = Vec::from_elem(table_len, None);
    let mut disps = Vec::from_elem(buckets_len, (0u32, 0u32));

    // store whether an element from the bucket being placed is
    // located at a certain position, to allow for efficient overlap
    // checks. It works by storing the generation in each cell and
    // each new placement-attempt is a new generation, so you can tell
    // if this is legitimately full by checking that the generations
    // are equal. (A u64 is far too large to overflow in a reasonable
    // time for current hardware.)
    let mut try_map = Vec::from_elem(table_len, 0u64);
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
                    let idx = (::shared::displace(hashes[key].f1, hashes[key].f2, d1, d2)
                                % (table_len as u32)) as uint;
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
