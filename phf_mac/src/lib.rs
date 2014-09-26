//! Compiler plugin for Rust-PHF
//!
//! See the documentation for the `phf` crate for more details.
#![doc(html_root_url="http://sfackler.github.io/rust-phf/doc")]
#![feature(plugin_registrar, quote, default_type_params, macro_rules)]

extern crate rand;
extern crate syntax;
extern crate time;
extern crate rustc;

use std::collections::HashMap;
use std::os;
use std::rc::Rc;
use std::hash;
use std::hash::Hash;
use syntax::ast;
use syntax::ast::{TokenTree, LitStr, LitBinary, LitByte, LitChar, Expr, ExprLit};
use syntax::codemap::Span;
use syntax::ext::base::{DummyResult,
                        ExtCtxt,
                        MacResult,
                        MacExpr};
use syntax::ext::build::AstBuilder;
use syntax::fold::Folder;
use syntax::parse;
use syntax::parse::token::{InternedString, COMMA, EOF, FAT_ARROW};
use syntax::print::pprust;
use syntax::ptr::P;
use rand::{Rng, SeedableRng, XorShiftRng};
use rustc::plugin::Registry;

use shared::PhfHash;

#[path="../../shared/mod.rs"]
mod shared;

static DEFAULT_LAMBDA: uint = 5;

static FIXED_SEED: [u32, ..4] = [3141592653, 589793238, 462643383, 2795028841];

#[plugin_registrar]
#[doc(hidden)]
pub fn macro_registrar(reg: &mut Registry) {
    reg.register_macro("phf_map", expand_phf_map);
    reg.register_macro("phf_set", expand_phf_set);
    reg.register_macro("phf_ordered_map", expand_phf_ordered_map);
    reg.register_macro("phf_ordered_set", expand_phf_ordered_set);
}

#[deriving(PartialEq, Eq, Clone)]
enum Key {
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
            KeyBinary(ref b) => b.as_slice().phf_hash(key),
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

struct Entry {
    key_contents: Key,
    key: P<Expr>,
    value: P<Expr>
}

struct HashState {
    key: u64,
    disps: Vec<(u32, u32)>,
    map: Vec<uint>,
}

fn expand_phf_map(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult+'static> {
    let entries = match parse_map(cx, tts) {
        Some(entries) => entries,
        None => return DummyResult::expr(sp)
    };

    if has_duplicates(cx, sp, entries.as_slice()) {
        return DummyResult::expr(sp);
    }

    let state = generate_hash(cx, sp, entries.as_slice());

    create_map(cx, sp, entries, state)
}

fn expand_phf_set(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult+'static> {
    let entries = match parse_set(cx, tts) {
        Some(entries) => entries,
        None => return DummyResult::expr(sp)
    };

    if has_duplicates(cx, sp, entries.as_slice()) {
        return DummyResult::expr(sp);
    }

    let state = generate_hash(cx, sp, entries.as_slice());

    create_set(cx, sp, entries, state)
}

fn expand_phf_ordered_map(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult+'static> {
    let entries = match parse_map(cx, tts) {
        Some(entries) => entries,
        None => return DummyResult::expr(sp),
    };

    if has_duplicates(cx, sp, entries.as_slice()) {
        return DummyResult::expr(sp);
    }

    let state = generate_hash(cx, sp, entries.as_slice());

    create_ordered_map(cx, sp, entries, state)
}

fn expand_phf_ordered_set(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult+'static> {
    let entries = match parse_set(cx, tts) {
        Some(entries) => entries,
        None => return DummyResult::expr(sp)
    };

    if has_duplicates(cx, sp, entries.as_slice()) {
        return DummyResult::expr(sp);
    }

    let state = generate_hash(cx, sp, entries.as_slice());

    create_ordered_set(cx, sp, entries, state)
}

fn parse_map(cx: &mut ExtCtxt, tts: &[TokenTree]) -> Option<Vec<Entry>> {
    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(), tts.to_vec());
    let mut entries = Vec::new();

    let mut bad = false;
    while parser.token != EOF {
        let key = cx.expander().fold_expr(parser.parse_expr());
        let key_contents = parse_key(cx, &*key).unwrap_or_else(|| {
            bad = true;
            KeyStr(InternedString::new(""))
        });

        if !parser.eat(&FAT_ARROW) {
            cx.span_err(parser.span, "expected `=>`");
            return None;
        }

        let value = parser.parse_expr();

        entries.push(Entry {
            key_contents: key_contents,
            key: key,
            value: value
        });

        if !parser.eat(&COMMA) && parser.token != EOF {
            cx.span_err(parser.span, "expected `,`");
            return None;
        }
    }

    if bad {
        return None;
    }

    Some(entries)
}

fn parse_set(cx: &mut ExtCtxt, tts: &[TokenTree]) -> Option<Vec<Entry>> {
    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(), tts.to_vec());
    let mut entries = Vec::new();
    let value = quote_expr!(&*cx, ());

    let mut bad = false;
    while parser.token != EOF {
        let key = cx.expander().fold_expr(parser.parse_expr());
        let key_contents = parse_key(cx, &*key).unwrap_or_else(|| {
            bad = true;
            KeyStr(InternedString::new(""))
        });

        entries.push(Entry {
            key_contents: key_contents,
            key: key,
            value: value.clone(),
        });

        if !parser.eat(&COMMA) && parser.token != EOF {
            cx.span_err(parser.span, "expected `,`");
            return None;
        }
    }

    if bad {
        return None;
    }

    Some(entries)
}

fn parse_key(cx: &mut ExtCtxt, e: &Expr) -> Option<Key> {
    match e.node {
        ExprLit(ref lit) => {
            match lit.node {
                ast::LitStr(ref s, _) => Some(KeyStr(s.clone())),
                ast::LitBinary(ref b) => Some(KeyBinary(b.clone())),
                ast::LitByte(b) => Some(KeyU8(b)),
                ast::LitChar(c) => Some(KeyChar(c)),
                ast::LitInt(i, ast::SignedIntLit(ast::TyI8, ast::Plus)) => Some(KeyI8(i as i8)),
                ast::LitInt(i, ast::SignedIntLit(ast::TyI8, ast::Minus)) => Some(KeyI8(-(i as i8))),
                ast::LitInt(i, ast::SignedIntLit(ast::TyI16, ast::Plus)) => Some(KeyI16(i as i16)),
                ast::LitInt(i, ast::SignedIntLit(ast::TyI16, ast::Minus)) => Some(KeyI16(-(i as i16))),
                ast::LitInt(i, ast::SignedIntLit(ast::TyI32, ast::Plus)) => Some(KeyI32(i as i32)),
                ast::LitInt(i, ast::SignedIntLit(ast::TyI32, ast::Minus)) => Some(KeyI32(-(i as i32))),
                ast::LitInt(i, ast::SignedIntLit(ast::TyI64, ast::Plus)) => Some(KeyI64(i as i64)),
                ast::LitInt(i, ast::SignedIntLit(ast::TyI64, ast::Minus)) => Some(KeyI64(-(i as i64))),
                ast::LitInt(i, ast::UnsignedIntLit(ast::TyU8)) => Some(KeyU8(i as u8)),
                ast::LitInt(i, ast::UnsignedIntLit(ast::TyU16)) => Some(KeyU16(i as u16)),
                ast::LitInt(i, ast::UnsignedIntLit(ast::TyU32)) => Some(KeyU32(i as u32)),
                ast::LitInt(i, ast::UnsignedIntLit(ast::TyU64)) => Some(KeyU64(i as u64)),
                ast::LitBool(b) => Some(KeyBool(b)),
                _ => {
                    cx.span_err(e.span, "unsupported literal type");
                    None
                }
            }
        }
        _ => {
            cx.span_err(e.span, "expected a literal");
            None
        }
    }
}

fn has_duplicates(cx: &mut ExtCtxt, sp: Span, entries: &[Entry]) -> bool {
    let mut dups = false;
    let mut strings = HashMap::new();
    for entry in entries.iter() {
        let &(ref mut spans, _) = strings.find_or_insert(&entry.key_contents,
                                                         (vec![], &entry.key));
        spans.push(entry.key.span);
    }

    for &(ref spans, key) in strings.values() {
        if spans.len() == 1 {
            continue;
        }

        dups = true;
        cx.span_err(sp, format!("duplicate key {}", pprust::expr_to_string(&**key)).as_slice());
        for span in spans.iter() {
            cx.span_note(*span, "one occurrence here");
        }
    }

    dups
}

fn generate_hash(cx: &mut ExtCtxt, sp: Span, entries: &[Entry]) -> HashState {
    let mut rng: XorShiftRng = SeedableRng::from_seed(FIXED_SEED);
    let start = time::precise_time_s();
    let state;
    loop {
        match try_generate_hash(entries.as_slice(), &mut rng) {
            Some(s) => {
                state = s;
                break;
            }
            None => {}
        }
    }
    let time = time::precise_time_s() - start;
    if os::getenv("PHF_STATS").is_some() {
        cx.span_note(sp, format!("PHF generation took {} seconds", time).as_slice());
    }

    state
}

fn try_generate_hash(entries: &[Entry], rng: &mut XorShiftRng) -> Option<HashState> {
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

    let hashes: Vec<Hashes> = entries.iter().map(|entry| {
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
        buckets.get_mut((hash.g % (buckets_len as u32)) as uint).keys.push(i);
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
                    let idx = (shared::displace(hashes[key].f1, hashes[key].f2, d1, d2)
                                % (table_len as u32)) as uint;
                    if map[idx].is_some() || try_map[idx] == generation {
                        continue 'disps;
                    }
                    *try_map.get_mut(idx) = generation;
                    values_to_add.push((idx, key));
                }

                // We've picked a good set of disps
                *disps.get_mut(bucket.idx) = (d1, d2);
                for &(idx, key) in values_to_add.iter() {
                    *map.get_mut(idx) = Some(key);
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

fn create_map(cx: &mut ExtCtxt, sp: Span, entries: Vec<Entry>, state: HashState)
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
    MacExpr::new(quote_expr!(cx, ::phf::PhfMap {
        key: $key,
        disps: &$disps,
        entries: &$entries,
    }))
}

fn create_set(cx: &mut ExtCtxt, sp: Span, entries: Vec<Entry>, state: HashState)
              -> Box<MacResult+'static> {
    let map = create_map(cx, sp, entries, state).make_expr().unwrap();
    MacExpr::new(quote_expr!(cx, ::phf::PhfSet { map: $map }))
}

fn create_ordered_map(cx: &mut ExtCtxt, sp: Span, entries: Vec<Entry>, state: HashState)
                      -> Box<MacResult+'static> {
    let disps = state.disps.iter().map(|&(d1, d2)| {
        quote_expr!(&*cx, ($d1, $d2))
    }).collect();
    let disps = cx.expr_vec(sp, disps);

    let idxs = state.map.iter().map(|&idx| quote_expr!(&*cx, $idx)).collect();
    let idxs =cx.expr_vec(sp, idxs);

    let entries = entries.iter().map(|&Entry { ref key, ref value, .. }| {
        quote_expr!(&*cx, ($key, $value))
    }).collect();
    let entries = cx.expr_vec(sp, entries);

    let key = state.key;
    MacExpr::new(quote_expr!(cx, ::phf::PhfOrderedMap {
        key: $key,
        disps: &$disps,
        idxs: &$idxs,
        entries: &$entries,
    }))
}

fn create_ordered_set(cx: &mut ExtCtxt, sp: Span, entries: Vec<Entry>, state: HashState)
                      -> Box<MacResult+'static> {
    let map = create_ordered_map(cx, sp, entries, state).make_expr().unwrap();
    MacExpr::new(quote_expr!(cx, ::phf::PhfOrderedSet { map: $map }))
}
