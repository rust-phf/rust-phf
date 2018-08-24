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
#![doc(html_root_url = "https://docs.rs/phf_macros/0.7.20")]
#![feature(plugin_registrar, quote, rustc_private)]

#[macro_use]
extern crate syntax;
extern crate syntax_pos;
extern crate phf_generator;
extern crate phf_shared;
extern crate rustc_plugin;

use phf_generator::HashState;
use rustc_plugin::Registry;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::env;
use std::time::Instant;
use syntax::ast::{self, Expr, ExprKind, Mutability, TyKind};
use syntax_pos::Span;
use syntax::ext::base::{DummyResult, ExtCtxt, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::fold::Folder;
use syntax::parse;
use syntax::parse::token::{Comma, Eof, FatArrow};
use syntax::print::pprust;
use syntax::ptr::P;
use syntax::symbol::Symbol;
use syntax::tokenstream::TokenTree;

use util::{create_map, create_set};
use util::{Entry, Key};

mod macros;
pub mod util;

mod errors {
    pub use syntax::errors::*;
}

#[plugin_registrar]
#[doc(hidden)]
pub fn macro_registrar(reg: &mut Registry) {
    reg.register_macro("phf_map", expand_phf_map);
    reg.register_macro("phf_set", expand_phf_set);
}

fn generate_hash(cx: &mut ExtCtxt, sp: Span, entries: &[Entry]) -> HashState {
    let start = Instant::now();
    let state = phf_generator::generate_hash(entries);
    let time = Instant::now() - start;

    if env::var_os("PHF_STATS").is_some() {
        let time = time.as_secs() as f64 + (time.subsec_nanos() as f64 / 1_000_000_000.);
        cx.parse_sess
            .span_diagnostic
            .span_note_without_error(sp, &format!("PHF generation took {} seconds", time));
    }

    state
}

fn expand_phf_map(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult + 'static> {
    let entries = match parse_map(cx, tts) {
        Some(entries) => entries,
        None => return DummyResult::expr(sp),
    };

    if has_duplicates(cx, sp, &*entries) {
        return DummyResult::expr(sp);
    }

    let state = generate_hash(cx, sp, &*entries);

    create_map(cx, sp, entries, state)
}

fn expand_phf_set(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult + 'static> {
    let entries = match parse_set(cx, tts) {
        Some(entries) => entries,
        None => return DummyResult::expr(sp),
    };

    if has_duplicates(cx, sp, &*entries) {
        return DummyResult::expr(sp);
    }

    let state = generate_hash(cx, sp, &*entries);

    create_set(cx, sp, entries, state)
}

fn parse_map(cx: &mut ExtCtxt, tts: &[TokenTree]) -> Option<Vec<Entry>> {
    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), tts.to_vec());
    let mut entries = Vec::new();

    let mut bad = false;
    while parser.token != Eof {
        let key = cx.expander().fold_expr(panictry!(parser.parse_expr()));
        let key_contents = parse_key(cx, &*key).unwrap_or_else(|| {
            bad = true;
            Key::Str(Symbol::intern("").as_str())
        });
        let key = adjust_key(cx, key);

        if !parser.eat(&FatArrow) {
            cx.span_err(parser.span, "expected `=>`");
            return None;
        }

        let value = panictry!(parser.parse_expr());

        entries.push(Entry {
            key_contents: key_contents,
            key: key,
            value: value,
        });

        if !parser.eat(&Comma) && parser.token != Eof {
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
    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), tts.to_vec());
    let mut entries = Vec::new();
    let value = quote_expr!(&*cx, ());

    let mut bad = false;
    while parser.token != Eof {
        let key = cx.expander().fold_expr(panictry!(parser.parse_expr()));
        let key_contents = parse_key(cx, &*key).unwrap_or_else(|| {
            bad = true;
            Key::Str(Symbol::intern("").as_str())
        });
        let key = adjust_key(cx, key);

        entries.push(Entry {
            key_contents: key_contents,
            key: key,
            value: value.clone(),
        });

        if !parser.eat(&Comma) && parser.token != Eof {
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
        ExprKind::Lit(ref lit) => match lit.node {
            ast::LitKind::Str(ref s, _) => Some(Key::Str(s.as_str())),
            ast::LitKind::ByteStr(ref b) => Some(Key::Binary(b.clone())),
            _ => {
                cx.span_err(e.span, "unsupported literal type");
                None
            }
        },
        ExprKind::Array(ref v) => {
            let bytes: Vec<Option<u8>> = v.iter()
                .map(|expr| {
                    if let ExprKind::Lit(ref p) = expr.node {
                        match p.node {
                            ast::LitKind::Int(val, ast::LitIntType::Unsigned(ast::UintTy::U8))
                                if val < 256 =>
                            {
                                Some(val as u8)
                            }
                            ast::LitKind::Int(val, ast::LitIntType::Unsuffixed) if val < 256 => {
                                Some(val as u8)
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                })
                .collect();
            if bytes.iter().all(|x| x.is_some()) {
                Some(Key::Binary(std::rc::Rc::new(
                    bytes.iter().map(|x| x.unwrap()).collect(),
                )))
            } else {
                cx.span_err(
                    e.span,
                    "not all elements of an expected u8 array literal were u8 literals",
                );
                None
            }
        }
        _ => {
            cx.span_err(e.span, "expected a literal");
            None
        }
    }
}

fn adjust_key(cx: &mut ExtCtxt, e: P<Expr>) -> P<Expr> {
    let coerce_as_slice = match e.node {
        ExprKind::Lit(ref lit) => match lit.node {
            ast::LitKind::ByteStr(_) => true,
            _ => false,
        },
        _ => false,
    };
    if coerce_as_slice {
        let u8_type = cx.ty_path(cx.path_ident(e.span, cx.ident_of("u8")));
        let array_type = cx.ty(e.span, TyKind::Slice(u8_type));
        let slice_type = cx.ty_rptr(e.span, array_type, None, Mutability::Immutable);
        cx.expr_cast(e.span, e, slice_type)
    } else {
        e
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
        let mut err = cx.struct_span_err(
            sp,
            &*format!("duplicate key {}", pprust::expr_to_string(&**key)),
        );
        for span in spans.iter() {
            err.span_note(*span, "one occurrence here");
        }
        err.emit();
    }

    dups
}
