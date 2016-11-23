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
#![doc(html_root_url="http://sfackler.github.io/rust-phf/doc/v0.7.19")]
#![feature(plugin_registrar, quote, rustc_private)]

extern crate syntax;
#[cfg(feature = "stats")]
extern crate time;
extern crate rustc_plugin;
extern crate phf_shared;
extern crate phf_generator;
#[cfg(feature = "unicase_support")]
extern crate unicase;

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use syntax::ast::{self, Expr, ExprKind, Mutability, TyKind};
use syntax::tokenstream::TokenTree;
use syntax::codemap::Span;
use syntax::ext::base::{DummyResult, ExtCtxt, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::fold::Folder;
use syntax::parse;
use syntax::parse::token::{Comma, Eof, FatArrow};
use syntax::print::pprust;
use syntax::ptr::P;
use syntax::symbol::Symbol;
use rustc_plugin::Registry;
use phf_generator::HashState;
use std::env;
#[cfg(feature = "unicase_support")]
use unicase::UniCase;

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
    fn precise_time_s() -> f64 {
        0.
    }

    let start = precise_time_s();
    let state = phf_generator::generate_hash(entries);
    let time = precise_time_s() - start;

    if cfg!(feature = "stats") && env::var_os("PHF_STATS").is_some() {
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

fn expand_phf_ordered_map(cx: &mut ExtCtxt,
                          sp: Span,
                          tts: &[TokenTree])
                          -> Box<MacResult + 'static> {
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

fn expand_phf_ordered_set(cx: &mut ExtCtxt,
                          sp: Span,
                          tts: &[TokenTree])
                          -> Box<MacResult + 'static> {
    let entries = match parse_set(cx, tts) {
        Some(entries) => entries,
        None => return DummyResult::expr(sp),
    };

    if has_duplicates(cx, sp, &*entries) {
        return DummyResult::expr(sp);
    }

    let state = generate_hash(cx, sp, &*entries);

    create_ordered_set(cx, sp, entries, state)
}

fn parse_map(cx: &mut ExtCtxt, tts: &[TokenTree]) -> Option<Vec<Entry>> {
    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), tts.to_vec());
    let mut entries = Vec::new();

    let mut bad = false;
    while parser.token != Eof {
        let key = cx.expander().fold_expr(parser.parse_expr().unwrap());
        let key_contents = parse_key(cx, &*key).unwrap_or_else(|| {
            bad = true;
            Key::Str(Symbol::intern("").as_str())
        });
        let key = adjust_key(cx, key);

        if !parser.eat(&FatArrow) {
            cx.span_err(parser.span, "expected `=>`");
            return None;
        }

        let value = parser.parse_expr().unwrap();

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
        let key = cx.expander().fold_expr(parser.parse_expr().unwrap());
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
        ExprKind::Lit(ref lit) => {
            match lit.node {
                ast::LitKind::Str(ref s, _) => Some(Key::Str(s.as_str())),
                ast::LitKind::ByteStr(ref b) => Some(Key::Binary(b.clone())),
                ast::LitKind::Byte(b) => Some(Key::U8(b)),
                ast::LitKind::Char(c) => Some(Key::Char(c)),
                ast::LitKind::Int(i, ast::LitIntType::Signed(ast::IntTy::I8)) =>
                    Some(Key::I8(i as i8)),
                ast::LitKind::Int(i, ast::LitIntType::Signed(ast::IntTy::I16)) =>
                    Some(Key::I16(i as i16)),
                ast::LitKind::Int(i, ast::LitIntType::Signed(ast::IntTy::I32)) =>
                    Some(Key::I32(i as i32)),
                ast::LitKind::Int(i, ast::LitIntType::Signed(ast::IntTy::I64)) =>
                    Some(Key::I64(i as i64)),
                ast::LitKind::Int(i, ast::LitIntType::Unsigned(ast::UintTy::U8)) =>
                    Some(Key::U8(i as u8)),
                ast::LitKind::Int(i, ast::LitIntType::Unsigned(ast::UintTy::U16)) =>
                    Some(Key::U16(i as u16)),
                ast::LitKind::Int(i, ast::LitIntType::Unsigned(ast::UintTy::U32)) =>
                    Some(Key::U32(i as u32)),
                ast::LitKind::Int(i, ast::LitIntType::Unsigned(ast::UintTy::U64)) =>
                    Some(Key::U64(i as u64)),
                ast::LitKind::Bool(b) => Some(Key::Bool(b)),
                _ => {
                    cx.span_err(e.span, "unsupported literal type");
                    None
                }
            }
        }
        ExprKind::Vec(ref v) => {
            let bytes: Vec<Option<u8>> = v.iter().map(|expr|
                if let ExprKind::Lit(ref p) = expr.node {
                    match p.node {
                        ast::LitKind::Int(val, ast::LitIntType::Unsigned(ast::UintTy::U8)) if val < 256 =>
                            Some(val as u8),
                        ast::LitKind::Int(val, ast::LitIntType::Unsuffixed) if val < 256 =>
                            Some(val as u8),
                        _ => None,
                    }
                } else {
                    None
            }).collect();
            if bytes.iter().all(|x| x.is_some()) {
                Some(Key::Binary(std::rc::Rc::new(bytes.iter().map(|x| x.unwrap()).collect())))
            } else {
                cx.span_err(e.span,
                            "not all elements of an expected u8 array literal were u8 literals");
                None
            }
        }
        #[cfg(feature = "unicase_support")]
        ExprKind::Call(ref f, ref args) => {
            if let ExprKind::Path(_, ref path) = f.node {
                if &*path.segments.last().unwrap().identifier.name.as_str() == "UniCase" {
                    if args.len() == 1 {
                        if let ExprKind::Lit(ref lit) = args.first().unwrap().node {
                            if let ast::LitKind::Str(ref s, _) = lit.node {
                                return Some(Key::UniCase(UniCase(s.to_string())));
                            } else {
                                cx.span_err(e.span, "only a str literal is allowed in UniCase");
                                return None;
                            }
                        }
                    } else {
                        cx.span_err(e.span, "only one str literal is allowed in UniCase");
                        return None;
                    }
                }
            }
            cx.span_err(e.span, "only UniCase is allowed besides literals");
            None
        },
        _ => {
            cx.span_err(e.span, "expected a literal (or a UniCase if the unicase_support feature is enabled)");
            None
        }
    }
}

fn adjust_key(cx: &mut ExtCtxt, e: P<Expr>) -> P<Expr> {
    let coerce_as_slice = match e.node {
        ExprKind::Lit(ref lit) => {
            match lit.node {
                ast::LitKind::ByteStr(_) => true,
                _ => false,
            }
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
        let mut err = cx.struct_span_err(sp, &*format!("duplicate key {}",
                                                       pprust::expr_to_string(&**key)));
        for span in spans.iter() {
            err.span_note(*span, "one occurrence here");
        }
        err.emit();
    }

    dups
}
