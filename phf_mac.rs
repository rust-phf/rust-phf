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

fn expand_mphf_map(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> MacResult {
    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(),
                                                tts.to_owned());
    let mut pairs = ~[];

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

        pairs.push((key_str, key, value));

        if !parser.eat(&COMMA) && parser.token != EOF {
            cx.span_fatal(parser.span, "expected `,`");
        }
    }

    pairs.sort_by(|&(ref a, _, _), &(ref b, _, _)| a.cmp(b));
    check_for_duplicates(cx, sp, pairs);

    let entries = pairs.move_iter()
        .map(|(_, key, value)| quote_expr!(&*cx, ($key, $value)))
        .collect();
    let entries = @Expr {
        id: ast::DUMMY_NODE_ID,
        node: ExprVec(entries, MutImmutable),
        span: sp,
    };

    MRExpr(quote_expr!(cx, PhfMap { entries: &'static $entries }))
}

fn check_for_duplicates(cx: &mut ExtCtxt, sp: Span, entries: &[(@str, @Expr, @Expr)]) {
    let mut in_dup = false;
    for window in entries.windows(2) {
        let (a, a_expr, _) = window[0];
        let (b, b_expr, _) = window[1];
        if a == b {
            if !in_dup {
                cx.span_err(sp, format!("duplicate key \"{}\"", a));
                cx.span_err(a_expr.span, "one occurrence here");
                in_dup = true;
            }
            cx.span_err(b_expr.span, "one occurrence here");
        } else {
            in_dup = false;
        }
    }
    cx.parse_sess().span_diagnostic.handler().abort_if_errors();
}
