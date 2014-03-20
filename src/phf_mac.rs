//! Compiler plugin for Rust-PHF
//!
//! See the documentation for the `phf` crate for more details.
#[crate_id="github.com/sfackler/rust-phf/phf_mac"];
#[crate_type="dylib"];
#[doc(html_root_url="http://www.rust-ci.org/sfackler/rust-phf/doc")];
#[feature(managed_boxes, macro_registrar, quote)];

extern crate collections;
extern crate rand;
extern crate syntax;
extern crate time;
extern crate phf;

use collections::HashMap;
use std::os;
use std::vec_ng::Vec;
use syntax::ast;
use syntax::ast::{Name, TokenTree, LitStr, MutImmutable, Expr, ExprVec, ExprLit};
use syntax::codemap::Span;
use syntax::ext::base::{SyntaxExtension,
                        ExtCtxt,
                        MacResult,
                        MRExpr,
                        NormalTT,
                        BasicMacroExpander};
use syntax::parse;
use syntax::parse::token;
use syntax::parse::token::{InternedString, COMMA, EOF, FAT_ARROW};
use rand::{Rng, SeedableRng, XorShiftRng};

static DEFAULT_LAMBDA: uint = 5;

static FIXED_SEED: [u32, ..4] = [3141592653, 589793238, 462643383, 2795028841];

#[macro_registrar]
#[doc(hidden)]
pub fn macro_registrar(register: |Name, SyntaxExtension|) {
    register(token::intern("phf_map"),
             NormalTT(~BasicMacroExpander {
                expander: expand_mphf_map,
                span: None
             },
             None));
}

struct Entry {
    key_str: InternedString,
    key: @Expr,
    value: @Expr
}

fn expand_mphf_map(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> MacResult {
    let entries = match parse_entries(cx, tts) {
        Some(entries) => entries,
        None => return MacResult::dummy_expr(sp)
    };

    if has_duplicates(cx, sp, entries.as_slice()) {
        return MacResult::dummy_expr(sp);
    }

    let mut rng: XorShiftRng = SeedableRng::from_seed(FIXED_SEED);
    let start = time::precise_time_s();
    let state;
    loop {
        match generate_hash(entries.as_slice(), &mut rng) {
            Some(s) => {
                state = s;
                break;
            }
            None => {}
        }
    }
    let time = time::precise_time_s() - start;
    if os::getenv("PHF_STATS").is_some() {
        cx.span_note(sp, format!("PHF generation took {} seconds", time));
    }

    create_map(cx, sp, entries, state)
}

fn parse_entries(cx: &mut ExtCtxt, tts: &[TokenTree]) -> Option<Vec<Entry>> {
    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(),
                                                tts.iter().map(|x| x.clone())
                                                    .collect());
    let mut entries = Vec::new();

    let mut bad = false;
    while parser.token != EOF {
        let key = cx.expand_expr(parser.parse_expr());

        let key_str = match key.node {
            ExprLit(lit) => {
                match lit.node {
                    LitStr(ref s, _) => s.clone(),
                    _ => {
                        cx.span_err(key.span, "expected string literal");
                        bad = true;
                        InternedString::new("")
                    }
                }
            }
            _ => {
                cx.span_err(key.span, "expected string literal");
                bad = true;
                InternedString::new("")
            }
        };

        if !parser.eat(&FAT_ARROW) {
            cx.span_err(parser.span, "expected `=>`");
            return None;
        }

        let value = parser.parse_expr();

        entries.push(Entry {
            key_str: key_str,
            key: key,
            value: value
        });

        if !parser.eat(&COMMA) && parser.token != EOF {
            cx.span_err(parser.span, "expected `,`");
            return None;
        }
    }

    if entries.len() > phf::MAX_SIZE {
        cx.span_err(parser.span,
                    format!("maps with more than {} entries are not supported",
                            phf::MAX_SIZE));
        return None;
    }

    if bad {
        return None;
    }

    Some(entries)
}

fn has_duplicates(cx: &mut ExtCtxt, sp: Span, entries: &[Entry]) -> bool {
    let mut dups = false;
    let mut strings = HashMap::new();
    for entry in entries.iter() {
        strings.insert_or_update_with(entry.key_str.clone(), (entry, true),
                                      |_, &(orig, ref mut first)| {
                if *first {
                    cx.span_err(sp, format!("duplicate key \"{}\"", entry.key_str));
                    cx.span_note(orig.key.span, "one occurrence here");
                    *first = false;
                }
                cx.span_note(entry.key.span, "one occurrence here");
                dups = true;
            });
    }

    dups
}

struct HashState {
    k1: u64,
    k2: u64,
    disps: Vec<(uint, uint)>,
    map: Vec<Option<uint>>,
}

fn generate_hash(entries: &[Entry], rng: &mut XorShiftRng) -> Option<HashState> {
    struct Bucket {
        idx: uint,
        keys: Vec<uint>,
    }

    struct Hashes {
        g: uint,
        f1: uint,
        f2: uint,
    }

    let k1 = rng.gen();
    let k2 = rng.gen();

    let hashes: Vec<Hashes> = entries.iter().map(|entry| {
            let (g, f1, f2) = phf::hash(entry.key_str.get(), k1, k2);
            Hashes {
                g: g,
                f1: f1,
                f2: f2
            }
        }).collect();

    let buckets_len = (entries.len() + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA;
    let mut buckets = Vec::from_fn(buckets_len,
                                   |i| Bucket { idx: i, keys: Vec::new() });

    for (i, hash) in hashes.iter().enumerate() {
        buckets.get_mut(hash.g % buckets_len).keys.push(i);
    }

    // Sort descending
    buckets.sort_by(|a, b| b.keys.len().cmp(&a.keys.len()));

    let table_len = entries.len();
    let mut map = Vec::from_elem(table_len, None);
    let mut disps = Vec::from_elem(buckets_len, None);
    let mut try_map = HashMap::new();
    'buckets: for bucket in buckets.iter() {
        for d1 in range(0, table_len) {
            'disps_l: for d2 in range(0, table_len) {
                try_map.clear();
                for &key in bucket.keys.iter() {
                    let idx = phf::displace(hashes.get(key).f1,
                                            hashes.get(key).f2,
                                            d1,
                                            d2) % table_len;
                    if try_map.find(&idx).is_some() || map.get(idx).is_some() {
                        continue 'disps_l;
                    }
                    try_map.insert(idx, key);
                }

                // We've picked a good set of disps
                *disps.get_mut(bucket.idx) = Some((d1, d2));
                for (&idx, &key) in try_map.iter() {
                    *map.get_mut(idx) = Some(key);
                }
                continue 'buckets;
            }
        }

        // Unable to find displacements for a bucket
        return None;
    }

    let disps = disps.move_iter().map(|i| i.expect("should have a bucket"))
            .collect();

    Some(HashState {
        k1: k1,
        k2: k2,
        disps: disps,
        map: map,
    })
}

fn create_map(cx: &mut ExtCtxt, sp: Span, entries: Vec<Entry>, state: HashState)
              -> MacResult {
    let len = entries.len();
    let k1 = state.k1;
    let k2 = state.k2;
    let disps = state.disps.iter().map(|&(d1, d2)| {
            quote_expr!(&*cx, ($d1, $d2))
        }).collect();
    let disps = @Expr {
        id: ast::DUMMY_NODE_ID,
        node: ExprVec(disps, MutImmutable),
        span: sp,
    };
    let entries = state.map.iter().map(|&idx| {
            match idx {
                Some(idx) => {
                    let &Entry { key, value, .. } = entries.get(idx);
                    quote_expr!(&*cx, Some(($key, $value)))
                }
                None => quote_expr!(&*cx, None),
            }
        }).collect();
    let entries = @Expr {
        id: ast::DUMMY_NODE_ID,
        node: ExprVec(entries, MutImmutable),
        span: sp,
    };

    MRExpr(quote_expr!(cx, PhfMap {
        len: $len,
        k1: $k1,
        k2: $k2,
        disps: &'static $disps,
        entries: &'static $entries,
    }))
}
