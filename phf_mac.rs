//! Compiler plugin for Rust-Phf
//!
//! See the documentation for the `phf` crate for more details.
#[crate_id="github.com/sfackler/rust-phf/phf_mac"];
#[crate_type="lib"];
#[doc(html_root_url="http://www.rust-ci.org/sfackler/rust-phf/doc")];
#[feature(managed_boxes, macro_registrar)];

extern mod syntax;

use syntax::ast;
use syntax::ast::{Name, TokenTree, LitStr, MutImmutable, Expr, ExprVec, ExprLit};
use syntax::codemap::Span;
use syntax::ext::base::{SyntaxExtension,
                        ExtCtxt,
                        MacResult,
                        MRExpr,
                        NormalTT,
                        SyntaxExpanderTT,
                        SyntaxExpanderTTExpanderWithoutContext};
use syntax::parse;
use syntax::parse::token;
use syntax::parse::token::{COMMA, EOF, FAT_ARROW};

#[macro_registrar]
#[doc(hidden)]
pub fn macro_registrar(register: |Name, SyntaxExtension|) {
    register(token::intern("phf_map"),
             NormalTT(~SyntaxExpanderTT {
                expander: SyntaxExpanderTTExpanderWithoutContext(expand_mphf_map),
                span: None
             },
             None));
}

struct Entry {
    key_str: @str,
    key: @Expr,
    value: @Expr
}

fn expand_mphf_map(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> MacResult {
    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(),
                                                tts.to_owned());
    let mut entries = ~[];

    while parser.token != EOF {
        let key = parser.parse_expr();

        let key_str = match key.node {
            ExprLit(lit) => {
                match lit.node {
                    LitStr(s, _) => s,
                    _ => cx.span_fatal(key.span, "expected string literal"),
                }
            }
            _ => cx.span_fatal(key.span, "expected string literal"),
        };

        if !parser.eat(&FAT_ARROW) {
            cx.span_fatal(parser.span, "expected `=>`");
        }

        let value = parser.parse_expr();

        entries.push(Entry {
            key_str: key_str,
            key: key,
            value: value
        });

        if !parser.eat(&COMMA) && parser.token != EOF {
            cx.span_fatal(parser.span, "expected `,`");
        }
    }

    entries.sort_by(|a, b| a.key_str.cmp(&b.key_str));
    check_for_duplicates(cx, sp, entries);

    let entries = entries.move_iter()
        .map(|Entry { key, value, .. }| quote_expr!(&*cx, ($key, $value)))
        .collect();
    let entries = @Expr {
        id: ast::DUMMY_NODE_ID,
        node: ExprVec(entries, MutImmutable),
        span: sp,
    };

    MRExpr(quote_expr!(cx, PhfMap { entries: &'static $entries }))
}

fn check_for_duplicates(cx: &mut ExtCtxt, sp: Span, entries: &[Entry]) {
    let mut in_dup = false;
    for window in entries.windows(2) {
        let ref a = window[0];
        let ref b = window[1];
        if a.key_str == b.key_str {
            if !in_dup {
                cx.span_err(sp, format!("duplicate key \"{}\"", a.key_str));
                cx.span_err(a.key.span, "one occurrence here");
                in_dup = true;
            }
            cx.span_err(b.key.span, "one occurrence here");
        } else {
            in_dup = false;
        }
    }
    cx.parse_sess().span_diagnostic.handler().abort_if_errors();
}
