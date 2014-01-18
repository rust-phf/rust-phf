#[crate_id="github.com/sfackler/rust-phf#phf:0.0"];
#[crate_type="lib"];
#[feature(managed_boxes, macro_registrar)];

extern mod syntax;

use std::vec::VecIterator;

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

pub struct PhfMap<T> {
    #[doc(hidden)]
    entries: &'static [(&'static str, T)],
}

impl<T> Container for PhfMap<T> {
    fn len(&self) -> uint {
        self.entries.len()
    }
}

impl<T> Map<&'static str, T> for PhfMap<T> {
    fn find<'a>(&'a self, key: & &'static str) -> Option<&'a T> {
        self.entries.bsearch(|&(val, _)| val.cmp(key)).map(|idx| {
            let (_, ref val) = self.entries[idx];
            val
        })
    }
}

impl<T> PhfMap<T> {
    pub fn entries<'a>(&'a self) -> PhfMapEntries<'a, T> {
        PhfMapEntries { iter: self.entries.iter() }
    }

    pub fn keys<'a>(&'a self) -> PhfMapKeys<'a, T> {
        PhfMapKeys { iter: self.entries() }
    }

    pub fn values<'a>(&'a self) -> PhfMapValues<'a, T> {
        PhfMapValues { iter: self.entries() }
    }
}

pub struct PhfMapEntries<'a, T> {
    priv iter: VecIterator<'a, (&'static str, T)>,
}

impl<'a, T> Iterator<(&'static str, &'a T)> for PhfMapEntries<'a, T> {
    fn next(&mut self) -> Option<(&'static str, &'a T)> {
        self.iter.next().map(|&(key, ref value)| (key, value))
    }
}

pub struct PhfMapKeys<'a, T> {
    priv iter: PhfMapEntries<'a, T>,
}

impl<'a, T> Iterator<&'static str> for PhfMapKeys<'a, T> {
    fn next(&mut self) -> Option<&'static str> {
        self.iter.next().map(|(key, _)| key)
    }
}

pub struct PhfMapValues<'a, T> {
    priv iter: PhfMapEntries<'a, T>,
}

impl<'a, T> Iterator<&'a T> for PhfMapValues<'a, T> {
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next().map(|(_, value)| value)
    }
}

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

    for window in pairs.windows(2) {
        let (ref a, ref a_expr, _) = window[0];
        let (ref b, ref b_expr, _) = window[1];
        if a == b {
            cx.span_err(sp, format!("duplicate key \"{}\"", *a));
            cx.span_err(a_expr.span, "one occurrence here");
            cx.span_err(b_expr.span, "one occurrence here");
        }
    }
    cx.parse_sess().span_diagnostic.handler().abort_if_errors();

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
