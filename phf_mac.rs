//! Compiler plugin for Rust-PHF
//!
//! See the documentation for the `phf` crate for more details.
#[crate_id="github.com/sfackler/rust-phf/phf_mac"];
#[crate_type="dylib"];
#[doc(html_root_url="http://www.rust-ci.org/sfackler/rust-phf/doc")];
#[feature(managed_boxes, macro_registrar)];

extern mod extra;
extern mod syntax;
extern mod phf;

use extra::time;
use std::hashmap::HashMap;
use std::rand;
use std::os;
use std::vec;
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

static DEFAULT_LAMBDA: uint = 5;

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
    let mut entries = match parse_entries(cx, tts) {
        Some(entries) => entries,
        None => return MacResult::dummy_expr()
    };

    entries.sort_by(|a, b| a.key_str.cmp(&b.key_str));
    if check_for_duplicates(cx, sp, entries) {
        return MacResult::dummy_expr();
    }

    let start = time::precise_time_s();
    let state;
    loop {
        match generate_hash(entries) {
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

fn parse_entries(cx: &mut ExtCtxt, tts: &[TokenTree]) -> Option<~[Entry]> {
    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(),
                                                tts.to_owned());
    let mut entries = ~[];

    let mut bad = false;
    while parser.token != EOF {
        let key = parser.parse_expr();

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

fn check_for_duplicates(cx: &mut ExtCtxt, sp: Span, entries: &[Entry]) -> bool {
    let mut dups = false;
    let mut in_dup = false;
    for window in entries.windows(2) {
        let ref a = window[0];
        let ref b = window[1];
        if a.key_str == b.key_str {
            dups = true;
            if !in_dup {
                cx.span_err(sp, format!("duplicate key \"{}\"", a.key_str));
                cx.span_note(a.key.span, "one occurrence here");
                in_dup = true;
            }
            cx.span_note(b.key.span, "one occurrence here");
        } else {
            in_dup = false;
        }
    }

    dups
}

struct HashState {
    k1: u64,
    k2: u64,
    disps: ~[(uint, uint)],
    map: ~[Option<uint>],
}

fn generate_hash(entries: &[Entry]) -> Option<HashState> {
    struct Bucket {
        idx: uint,
        keys: ~[uint],
    }

    struct Hashes {
        g: uint,
        f1: uint,
        f2: uint,
    }

    let k1 = rand::random();
    let k2 = rand::random();

    let hashes = entries.iter().map(|entry| {
            let (g, f1, f2) = phf::hash(entry.key_str.get(), k1, k2);
            Hashes {
                g: g,
                f1: f1,
                f2: f2
            }
        }).to_owned_vec();

    let buckets_len = (entries.len() + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA;
    let mut buckets = vec::from_fn(buckets_len,
                                   |i| Bucket { idx: i, keys: ~[] });

    for (i, hash) in hashes.iter().enumerate() {
        buckets[hash.g % buckets_len].keys.push(i);
    }

    // Sort descending
    buckets.sort_by(|a, b| b.keys.len().cmp(&a.keys.len()));

    let table_len = entries.len();
    let mut map = vec::from_elem(table_len, None);
    let mut disps = vec::from_elem(buckets_len, None);
    let mut try_map = HashMap::new();
    'buckets: for bucket in buckets.iter() {
        for d1 in range(0, table_len) {
            'disps: for d2 in range(0, table_len) {
                try_map.clear();
                for &key in bucket.keys.iter() {
                    let idx = phf::displace(hashes[key].f1, hashes[key].f2,
                                            d1, d2) % table_len;
                    if try_map.find(&idx).is_some() || map[idx].is_some() {
                        continue 'disps;
                    }
                    try_map.insert(idx, key);
                }

                // We've picked a good set of disps
                disps[bucket.idx] = Some((d1, d2));
                for (&idx, &key) in try_map.iter() {
                    map[idx] = Some(key);
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

fn create_map(cx: &mut ExtCtxt, sp: Span, entries: ~[Entry], state: HashState)
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
                    let Entry { key, value, .. } = entries[idx];
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
