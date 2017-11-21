use std::rc::Rc;
use std::hash::{Hash, Hasher};

use syntax::ast::Expr;
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult, MacEager};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;
use syntax::symbol::InternedString;

use phf_shared::PhfHash;
use phf_generator::HashState;

#[derive(PartialEq, Eq, Clone)]
pub enum Key {
    Str(InternedString),
    Binary(Rc<Vec<u8>>),
    Char(char),
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    Bool(bool),
}

impl Hash for Key {
    fn hash<S: Hasher>(&self, state: &mut S) {
        match *self {
            Key::Str(ref s) => s.hash(state),
            Key::Binary(ref b) => b.hash(state),
            Key::Char(c) => c.hash(state),
            Key::U8(b) => b.hash(state),
            Key::I8(b) => b.hash(state),
            Key::U16(b) => b.hash(state),
            Key::I16(b) => b.hash(state),
            Key::U32(b) => b.hash(state),
            Key::I32(b) => b.hash(state),
            Key::U64(b) => b.hash(state),
            Key::I64(b) => b.hash(state),
            Key::Bool(b) => b.hash(state),
        }
    }
}

impl PhfHash for Key {
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            Key::Str(ref s) => s.phf_hash(state),
            Key::Binary(ref b) => b.phf_hash(state),
            Key::Char(c) => c.phf_hash(state),
            Key::U8(b) => b.phf_hash(state),
            Key::I8(b) => b.phf_hash(state),
            Key::U16(b) => b.phf_hash(state),
            Key::I16(b) => b.phf_hash(state),
            Key::U32(b) => b.phf_hash(state),
            Key::I32(b) => b.phf_hash(state),
            Key::U64(b) => b.phf_hash(state),
            Key::I64(b) => b.phf_hash(state),
            Key::Bool(b) => b.phf_hash(state),
        }
    }
}

pub struct Entry {
    pub key_contents: Key,
    pub key: P<Expr>,
    pub value: P<Expr>,
}

impl PhfHash for Entry {
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        self.key_contents.phf_hash(state)
    }
}

pub fn create_map(cx: &mut ExtCtxt,
                  sp: Span,
                  entries: Vec<Entry>,
                  state: HashState)
                  -> Box<MacResult + 'static> {
    let disps = state.disps
                     .iter()
                     .map(|&(d1, d2)| quote_expr!(&*cx, ($d1, $d2)))
                     .collect();
    let disps = cx.expr_vec(sp, disps);

    let entries = state.map
                       .iter()
                       .map(|&idx| {
                           let &Entry { ref key, ref value, .. } = &entries[idx];
                           quote_expr!(&*cx, ($key, $value))
                       })
                       .collect();
    let entries = cx.expr_vec(sp, entries);

    let key = state.key;
    MacEager::expr(quote_expr!(cx, ::phf::Map {
        key: $key,
        disps: ::phf::Slice::Static(&$disps),
        entries: ::phf::Slice::Static(&$entries),
    }))
}

pub fn create_set(cx: &mut ExtCtxt,
                  sp: Span,
                  entries: Vec<Entry>,
                  state: HashState)
                  -> Box<MacResult + 'static> {
    let map = create_map(cx, sp, entries, state).make_expr().unwrap();
    MacEager::expr(quote_expr!(cx, ::phf::Set { map: $map }))
}

pub fn create_ordered_map(cx: &mut ExtCtxt,
                          sp: Span,
                          entries: Vec<Entry>,
                          state: HashState)
                          -> Box<MacResult + 'static> {
    let disps = state.disps
                     .iter()
                     .map(|&(d1, d2)| quote_expr!(&*cx, ($d1, $d2)))
                     .collect();
    let disps = cx.expr_vec(sp, disps);

    let idxs = state.map.iter().map(|&idx| quote_expr!(&*cx, $idx)).collect();
    let idxs = cx.expr_vec(sp, idxs);

    let entries = entries.iter()
                         .map(|&Entry { ref key, ref value, .. }| {
                             quote_expr!(&*cx, ($key, $value))
                         })
                         .collect();
    let entries = cx.expr_vec(sp, entries);

    let key = state.key;
    MacEager::expr(quote_expr!(cx, ::phf::OrderedMap {
        key: $key,
        disps: ::phf::Slice::Static(&$disps),
        idxs: ::phf::Slice::Static(&$idxs),
        entries: ::phf::Slice::Static(&$entries),
    }))
}

pub fn create_ordered_set(cx: &mut ExtCtxt,
                          sp: Span,
                          entries: Vec<Entry>,
                          state: HashState)
                          -> Box<MacResult + 'static> {
    let map = create_ordered_map(cx, sp, entries, state).make_expr().unwrap();
    MacEager::expr(quote_expr!(cx, ::phf::OrderedSet { map: $map }))
}
