use std::rc::Rc;
use std::hash::{Hash, Hasher};

use syntax::ast::{Arm, Expr};
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult, MacEager};
use syntax::ext::build::AstBuilder;
use syntax::parse::token::InternedString;
use syntax::ptr::P;

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
    pub value: P<Expr>
}

impl PhfHash for Entry {
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        self.key_contents.phf_hash(state)
    }
}

pub fn create_map(cx: &mut ExtCtxt, sp: Span, entries: Vec<Entry>, state: HashState)
                  -> Box<MacResult+'static> {
    let disps = state.disps.iter().map(|&(d1, d2)| {
        quote_expr!(&*cx, ($d1, $d2))
    }).collect();
    let disps = cx.expr_vec(sp, disps);

    let entries = state.map.iter().map(|&idx| {
        let &Entry { ref key, ref value, .. } = &entries[idx];
        quote_expr!(&*cx, ($key, $value))
    }).collect();
    let entries = cx.expr_vec(sp, entries);

    let key = state.key;
    MacEager::expr(quote_expr!(cx, ::phf::Map {
        key: $key,
        disps: &$disps,
        entries: &$entries,
    }))
}

pub fn create_set(cx: &mut ExtCtxt, sp: Span, entries: Vec<Entry>, state: HashState)
              -> Box<MacResult+'static> {
    let map = create_map(cx, sp, entries, state).make_expr().unwrap();
    MacEager::expr(quote_expr!(cx, ::phf::Set { map: $map }))
}

pub fn create_ordered_map(cx: &mut ExtCtxt, sp: Span, entries: Vec<Entry>, state: HashState)
                          -> Box<MacResult+'static> {
    let disps = state.disps.iter().map(|&(d1, d2)| {
        quote_expr!(&*cx, ($d1, $d2))
    }).collect();
    let disps = cx.expr_vec(sp, disps);

    let idxs = state.map.iter().map(|&idx| quote_expr!(&*cx, $idx)).collect();
    let idxs = cx.expr_vec(sp, idxs);

    let entries = entries.iter().map(|&Entry { ref key, ref value, .. }| {
        quote_expr!(&*cx, ($key, $value))
    }).collect();
    let entries = cx.expr_vec(sp, entries);

    let key = state.key;
    MacEager::expr(quote_expr!(cx, ::phf::OrderedMap {
        key: $key,
        disps: &$disps,
        idxs: &$idxs,
        entries: &$entries,
    }))
}

pub fn create_ordered_set(cx: &mut ExtCtxt, sp: Span, entries: Vec<Entry>, state: HashState)
                          -> Box<MacResult+'static> {
    let map = create_ordered_map(cx, sp, entries, state).make_expr().unwrap();
    MacEager::expr(quote_expr!(cx, ::phf::OrderedSet { map: $map }))
}

pub fn create_match(cx: &mut ExtCtxt,
                    sp: Span,
                    discriminant: P<Expr>,
                    entries: Vec<Entry>,
                    wild: Option<Arm>,
                    state: HashState)
                  -> Box<MacResult+'static> {
    let len = entries.len();

    // Don't bother generating a hash for 2 items or less.
    if len <= 2 {
        let mut arms: Vec<Arm> = state.map.iter().map(|&idx| {
            let &Entry { ref key, ref value, .. } = &entries[idx];
            quote_arm!(&*cx, $key => $value,)
        }).collect();

        match wild {
            Some(wild) => { arms.push(wild); }
            None => {}
        }

        MacEager::expr(quote_expr!(cx,
            match $discriminant {
                $arms
            }
        ))
    } else {
        let disps = state.disps.iter().map(|&(d1, d2)| {
            quote_expr!(&*cx, ($d1, $d2))
        }).collect();
        let disps = cx.expr_vec(sp, disps);

        let mut arms: Vec<Arm> = state.map.iter().enumerate().map(|(i, &idx)| {
            let i = i as u32;
            let &Entry { ref key, ref value, .. } = &entries[idx];
            quote_arm!(&*cx, $i if $discriminant == $key => $value,)
        }).collect();

        match wild {
            Some(wild) => { arms.push(wild); }
            None => {}
        }

        let key = state.key;

        MacEager::expr(quote_expr!(cx,
            match ::phf_shared::get_index(::phf_shared::hash($discriminant, $key), &$disps, $len) {
                $arms
            }
        ))
    }
}
