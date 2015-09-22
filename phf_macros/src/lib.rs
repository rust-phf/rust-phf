//! Compiler plugin defining macros that create PHF data structures.
//!
//! # Example
//!
//! ```rust
//! #![feature(plugin, core)]
//! #![plugin(phf_macros)]
//!
//! extern crate phf;
//!
//! #[derive(Clone)]
//! pub enum Keyword {
//!     Loop,
//!     Continue,
//!     Break,
//!     Fn,
//!     Extern,
//! }
//!
//! static KEYWORDS: phf::Map<&'static str, Keyword> = phf_map! {
//!     "loop" => Keyword::Loop,
//!     "continue" => Keyword::Continue,
//!     "break" => Keyword::Break,
//!     "fn" => Keyword::Fn,
//!     "extern" => Keyword::Extern,
//! };
//!
//! pub fn parse_keyword(keyword: &str) -> Option<Keyword> {
//!     KEYWORDS.get(keyword).cloned()
//! }
//! # fn main() {}
//! ```
#![doc(html_root_url="http://sfackler.github.io/rust-phf/doc/v0.7.5")]
#![feature(plugin_registrar, quote, rustc_private)]

extern crate syntax;
#[cfg(feature = "stats")]
extern crate time;
extern crate rustc;
extern crate phf_shared;
extern crate phf_generator;

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use syntax::ast::{self, TokenTree, LitStr, LitByteStr, LitByte, LitChar, Expr, ExprLit, ExprVec};
use syntax::codemap::{Span, Spanned};
use syntax::ext::base::{DummyResult,
                        ExtCtxt,
                        MacResult};
use syntax::fold::Folder;
use syntax::parse;
use syntax::parse::token::{InternedString, Comma, Eof, FatArrow};
use syntax::print::pprust;
use rustc::plugin::Registry;
use phf_generator::HashState;
use std::env;

use util::{Entry, Key};
use util::{create_map, create_set, create_ordered_map, create_ordered_set};

pub mod util;
mod macros;

#[plugin_registrar]
#[doc(hidden)]
pub fn macro_registrar(reg: &mut Registry) {
    reg.register_macro("phf_map", expand_phf_map);
    reg.register_macro("phf_set", expand_phf_set);
    reg.register_macro("phf_ordered_map", expand_phf_ordered_map);
    reg.register_macro("phf_ordered_set", expand_phf_ordered_set);
}

fn generate_hash(cx: &mut ExtCtxt, sp: Span, entries: &[Entry]) -> HashState {
    #[cfg(feature = "stats")]
    use time::precise_time_s;
    #[cfg(not(feature = "stats"))]
    fn precise_time_s() -> f64 { 0. }

    let start = precise_time_s();
    let state = phf_generator::generate_hash(entries);
    let time = precise_time_s() - start;

    if cfg!(feature = "stats") && env::var_os("PHF_STATS").is_some() {
        cx.span_note(sp, &format!("PHF generation took {} seconds", time));
    }

    state
}

fn expand_phf_map(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult+'static> {
    let entries = match parse_map(cx, tts) {
        Some(entries) => entries,
        None => return DummyResult::expr(sp)
    };

    if has_duplicates(cx, sp, &*entries) {
        return DummyResult::expr(sp);
    }

    let state = generate_hash(cx, sp, &*entries);

    create_map(cx, sp, entries, state)
}

fn expand_phf_set(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult+'static> {
    let entries = match parse_set(cx, tts) {
        Some(entries) => entries,
        None => return DummyResult::expr(sp)
    };

    if has_duplicates(cx, sp, &*entries) {
        return DummyResult::expr(sp);
    }

    let state = generate_hash(cx, sp, &*entries);

    create_set(cx, sp, entries, state)
}

fn expand_phf_ordered_map(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult+'static> {
    let entries = match parse_map(cx, tts) {
        Some(entries) => entries,
        None => return DummyResult::expr(sp),
    };

    if has_duplicates(cx, sp, &*entries) {
        return DummyResult::expr(sp);
    }

    let state = generate_hash(cx, sp, &*entries);

    create_ordered_map(cx, sp, entries, state)
}

fn expand_phf_ordered_set(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult+'static> {
    let entries = match parse_set(cx, tts) {
        Some(entries) => entries,
        None => return DummyResult::expr(sp)
    };

    if has_duplicates(cx, sp, &*entries) {
        return DummyResult::expr(sp);
    }

    let state = generate_hash(cx, sp, &*entries);

    create_ordered_set(cx, sp, entries, state)
}

fn parse_map(cx: &mut ExtCtxt, tts: &[TokenTree]) -> Option<Vec<Entry>> {
    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(), tts.to_vec());
    let mut entries = Vec::new();

    let mut bad = false;
    while parser.token != Eof {
        let key = cx.expander().fold_expr(parser.parse_expr());
        let key_contents = parse_key(cx, &*key).unwrap_or_else(|| {
            bad = true;
            Key::Str(InternedString::new(""))
        });

        if !parser.eat(&FatArrow).ok().unwrap() {
            cx.span_err(parser.span, "expected `=>`");
            return None;
        }

        let value = parser.parse_expr();

        entries.push(Entry {
            key_contents: key_contents,
            key: key,
            value: value
        });

        if !parser.eat(&Comma).ok().unwrap() && parser.token != Eof {
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
    while parser.token != Eof {
        let key = cx.expander().fold_expr(parser.parse_expr());
        let key_contents = parse_key(cx, &*key).unwrap_or_else(|| {
            bad = true;
            Key::Str(InternedString::new(""))
        });

        entries.push(Entry {
            key_contents: key_contents,
            key: key,
            value: value.clone(),
        });

        if !parser.eat(&Comma).ok().unwrap() && parser.token != Eof {
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
                ast::LitStr(ref s, _) => Some(Key::Str(s.clone())),
                ast::LitByteStr(ref b) => Some(Key::Binary(b.clone())),
                ast::LitByte(b) => Some(Key::U8(b)),
                ast::LitChar(c) => Some(Key::Char(c)),
                ast::LitInt(i, ast::SignedIntLit(ast::TyI8, ast::Plus)) => Some(Key::I8(i as i8)),
                ast::LitInt(i, ast::SignedIntLit(ast::TyI8, ast::Minus)) => Some(Key::I8(-(i as i8))),
                ast::LitInt(i, ast::SignedIntLit(ast::TyI16, ast::Plus)) => Some(Key::I16(i as i16)),
                ast::LitInt(i, ast::SignedIntLit(ast::TyI16, ast::Minus)) => Some(Key::I16(-(i as i16))),
                ast::LitInt(i, ast::SignedIntLit(ast::TyI32, ast::Plus)) => Some(Key::I32(i as i32)),
                ast::LitInt(i, ast::SignedIntLit(ast::TyI32, ast::Minus)) => Some(Key::I32(-(i as i32))),
                ast::LitInt(i, ast::SignedIntLit(ast::TyI64, ast::Plus)) => Some(Key::I64(i as i64)),
                ast::LitInt(i, ast::SignedIntLit(ast::TyI64, ast::Minus)) => Some(Key::I64(-(i as i64))),
                ast::LitInt(i, ast::UnsignedIntLit(ast::TyU8)) => Some(Key::U8(i as u8)),
                ast::LitInt(i, ast::UnsignedIntLit(ast::TyU16)) => Some(Key::U16(i as u16)),
                ast::LitInt(i, ast::UnsignedIntLit(ast::TyU32)) => Some(Key::U32(i as u32)),
                ast::LitInt(i, ast::UnsignedIntLit(ast::TyU64)) => Some(Key::U64(i as u64)),
                ast::LitBool(b) => Some(Key::Bool(b)),
                _ => {
                    cx.span_err(e.span, "unsupported literal type");
                    None
                }
            }
        }
        ExprVec(ref v) => {
            let bytes: Vec<Option<u8>> = v.iter().map(|expr|
                if let ExprLit(ref p) = expr.node {
                    match **p {
                        Spanned {node: ast::LitInt(val, ast::UnsignedIntLit(ast::UintTy::TyU8)), ..} if val < 256 => Some(val as u8),
                        Spanned {node: ast::LitInt(val, ast::UnsuffixedIntLit(ast::Plus)), ..} if val < 256 => Some(val as u8),
                        _ => None,
                    }
                } else {
                    None
            }).collect();
            if bytes.iter().all(|x| x.is_some()) {
                Some(Key::Binary(std::rc::Rc::new(bytes.iter().map(|x| x.unwrap()).collect())))
            } else {
                cx.span_err(e.span, "not all elements of an expected u8 array literal were u8 literals");
                None
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
        let &mut (ref mut spans, _) = match strings.entry(&entry.key_contents) {
            Occupied(e) => e.into_mut(),
            Vacant(e) => e.insert((vec![], &entry.key)),
        };
        spans.push(entry.key.span);
    }

    for &(ref spans, key) in strings.values() {
        if spans.len() == 1 {
            continue;
        }

        dups = true;
        cx.span_err(sp, &*format!("duplicate key {}", pprust::expr_to_string(&**key)));
        for span in spans.iter() {
            cx.span_note(*span, "one occurrence here");
        }
    }

    dups
}
