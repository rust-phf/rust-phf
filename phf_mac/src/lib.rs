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
use syntax::ast::{mod, TokenTree, LitStr, LitBinary, LitByte, LitChar, Expr, ExprLit};
use syntax::codemap::Span;
use syntax::ext::base::{DummyResult,
                        ExtCtxt,
                        MacResult};
use syntax::fold::Folder;
use syntax::parse;
use syntax::parse::token::{InternedString, COMMA, EOF, FAT_ARROW};
use syntax::print::pprust;
use rustc::plugin::Registry;

use util::{Entry, Key, KeyStr, KeyBinary, KeyChar, KeyU8, KeyI8, KeyU16};
use util::{KeyI16, KeyU32, KeyI32, KeyU64, KeyI64, KeyBool};
use util::{generate_hash, create_map, create_set, create_ordered_map, create_ordered_set};

#[path="../../shared/mod.rs"]
mod shared;
pub mod util;

#[plugin_registrar]
#[doc(hidden)]
pub fn macro_registrar(reg: &mut Registry) {
    reg.register_macro("phf_map", expand_phf_map);
    reg.register_macro("phf_set", expand_phf_set);
    reg.register_macro("phf_ordered_map", expand_phf_ordered_map);
    reg.register_macro("phf_ordered_set", expand_phf_ordered_set);
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
