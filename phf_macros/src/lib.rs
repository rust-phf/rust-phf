extern crate proc_macro;

use phf_generator::HashState;
use phf_shared::PhfHash;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use std::hash::Hasher;
use syn::parse::{self, Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Error, Expr, IntSuffix, Lit, Token, UnOp};

#[derive(Hash, PartialEq, Eq, Clone)]
enum ParsedKey {
    Str(String),
    Binary(Vec<u8>),
    Char(char),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Bool(bool),
}

impl PhfHash for ParsedKey {
    fn phf_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
    {
        match self {
            ParsedKey::Str(s) => s.phf_hash(state),
            ParsedKey::Binary(s) => s.phf_hash(state),
            ParsedKey::Char(s) => s.phf_hash(state),
            ParsedKey::I8(s) => s.phf_hash(state),
            ParsedKey::I16(s) => s.phf_hash(state),
            ParsedKey::I32(s) => s.phf_hash(state),
            ParsedKey::I64(s) => s.phf_hash(state),
            ParsedKey::I128(s) => s.phf_hash(state),
            ParsedKey::U8(s) => s.phf_hash(state),
            ParsedKey::U16(s) => s.phf_hash(state),
            ParsedKey::U32(s) => s.phf_hash(state),
            ParsedKey::U64(s) => s.phf_hash(state),
            ParsedKey::U128(s) => s.phf_hash(state),
            ParsedKey::Bool(s) => s.phf_hash(state),
        }
    }
}

impl ParsedKey {
    fn from_expr(expr: &Expr) -> Option<ParsedKey> {
        match expr {
            Expr::Lit(lit) => match &lit.lit {
                Lit::Str(s) => Some(ParsedKey::Str(s.value())),
                Lit::ByteStr(s) => Some(ParsedKey::Binary(s.value())),
                Lit::Byte(s) => Some(ParsedKey::U8(s.value())),
                Lit::Char(s) => Some(ParsedKey::Char(s.value())),
                Lit::Int(s) => match s.suffix() {
                    IntSuffix::I8 => Some(ParsedKey::I8(s.value() as i8)),
                    IntSuffix::I16 => Some(ParsedKey::I16(s.value() as i16)),
                    IntSuffix::I32 => Some(ParsedKey::I32(s.value() as i32)),
                    IntSuffix::I64 => Some(ParsedKey::I64(s.value() as i64)),
                    IntSuffix::I128 => Some(ParsedKey::I128(s.value() as i128)),
                    IntSuffix::U8 => Some(ParsedKey::U8(s.value() as u8)),
                    IntSuffix::U16 => Some(ParsedKey::U16(s.value() as u16)),
                    IntSuffix::U32 => Some(ParsedKey::U32(s.value() as u32)),
                    IntSuffix::U64 => Some(ParsedKey::U64(s.value())),
                    IntSuffix::U128 => Some(ParsedKey::U128(s.value() as u128)),
                    _ => None,
                },
                // `syn` doesn't bother parsing integer literals larger than 64 bits so that's on us
                Lit::Verbatim(verb) => {
                    let lit_str = verb.token.to_string();

                    // check the literal suffix manually
                    if lit_str.ends_with("u128") {
                        Some(ParsedKey::U128(lit_str[..lit_str.len() - 4].parse::<u128>().unwrap()))
                    } else if lit_str.ends_with("i128") {
                        Some(ParsedKey::I128(
                            // parse as u128 so we get the full integer literal range
                            // then the cast will negate it to the same absolute value
                            // we receive the negation as a separate token so this will always
                            // appear as a non-negative value
                            lit_str[..lit_str.len() - 4].parse::<u128>().unwrap() as i128))
                    } else {
                        None
                    }
                }
                Lit::Bool(s) => Some(ParsedKey::Bool(s.value)),
                _ => None,
            },
            Expr::Array(array) => {
                let mut buf = vec![];
                for expr in &array.elems {
                    match expr {
                        Expr::Lit(lit) => match &lit.lit {
                            Lit::Int(s) => match s.suffix() {
                                IntSuffix::U8 | IntSuffix::None => buf.push(s.value() as u8),
                                _ => return None,
                            },
                            _ => return None,
                        },
                        _ => return None,
                    }
                }
                Some(ParsedKey::Binary(buf))
            }
            Expr::Unary(unary) => {
                // if we received an integer literal (always unsigned) greater than i__::max_value()
                // then casting it to a signed integer type of the same width will negate it to
                // the same absolute value so we don't need to negate it here
                macro_rules! try_negate (
                    ($val:expr) => {if $val < 0 { $val } else { -$val }}
                );

                match unary.op {
                    UnOp::Neg(_) => match ParsedKey::from_expr(&unary.expr)? {
                        ParsedKey::I8(v) => Some(ParsedKey::I8(try_negate!(v))),
                        ParsedKey::I16(v) => Some(ParsedKey::I16(try_negate!(v))),
                        ParsedKey::I32(v) => Some(ParsedKey::I32(try_negate!(v))),
                        ParsedKey::I64(v) => Some(ParsedKey::I64(try_negate!(v))),
                        ParsedKey::I128(v) => Some(ParsedKey::I128(try_negate!(v))),
                        _ => None,
                    },
                    _ => None,
                }
            }
            Expr::Group(group) => ParsedKey::from_expr(&group.expr),
            _ => None,
        }
    }
}

struct Key {
    parsed: ParsedKey,
    expr: Expr,
}

impl PhfHash for Key {
    fn phf_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
    {
        self.parsed.phf_hash(state)
    }
}

impl Parse for Key {
    fn parse(input: ParseStream) -> parse::Result<Key> {
        let expr = input.parse()?;
        let parsed = ParsedKey::from_expr(&expr)
            .ok_or_else(|| Error::new_spanned(&expr, "unsupported key expression"))?;

        Ok(Key { parsed, expr })
    }
}

struct Entry {
    key: Key,
    value: Expr,
}

impl PhfHash for Entry {
    fn phf_hash<H>(&self, state: &mut H)
        where
            H: Hasher,
    {
        self.key.phf_hash(state)
    }
}

impl Parse for Entry {
    fn parse(input: ParseStream) -> parse::Result<Entry> {
        let key = input.parse()?;
        input.parse::<Token![=>]>()?;
        let value = input.parse()?;
        Ok(Entry { key, value })
    }
}

struct Map(Vec<Entry>);

impl Parse for Map {
    fn parse(input: ParseStream) -> parse::Result<Map> {
        let parsed = Punctuated::<Entry, Token![,]>::parse_terminated(input)?;
        let map = parsed.into_iter().collect::<Vec<_>>();
        check_duplicates(&map)?;
        Ok(Map(map))
    }
}

struct Set(Vec<Entry>);

impl Parse for Set {
    fn parse(input: ParseStream) -> parse::Result<Set> {
        let parsed = Punctuated::<Key, Token![,]>::parse_terminated(input)?;
        let set = parsed
            .into_iter()
            .map(|key| Entry {
                key,
                value: syn::parse_str("()").unwrap(),
            })
            .collect::<Vec<_>>();
        check_duplicates(&set)?;
        Ok(Set(set))
    }
}

fn check_duplicates(entries: &[Entry]) -> parse::Result<()> {
    let mut keys = HashSet::new();
    for entry in entries {
        if !keys.insert(&entry.key.parsed) {
            return Err(Error::new_spanned(&entry.key.expr, "duplicate key"));
        }
    }
    Ok(())
}

fn build_map(entries: &[Entry], state: HashState) -> proc_macro2::TokenStream {
    let key = state.key;
    let disps = state.disps.iter().map(|&(d1, d2)| quote!((#d1, #d2)));
    let entries = state.map.iter().map(|&idx| {
        let key = &entries[idx].key.expr;
        let value = &entries[idx].value;
        quote!((#key, #value))
    });

    quote! {
        phf::Map {
            key: #key,
            disps: phf::Slice::Static(&[#(#disps),*]),
            entries: phf::Slice::Static(&[#(#entries),*]),
        }
    }
}

#[::proc_macro_hack::proc_macro_hack]
pub fn phf_map(input: TokenStream) -> TokenStream {
    let map = parse_macro_input!(input as Map);
    let state = phf_generator::generate_hash(&map.0);

    build_map(&map.0, state).into()
}

#[::proc_macro_hack::proc_macro_hack]
pub fn phf_set(input: TokenStream) -> TokenStream {
    let set = parse_macro_input!(input as Set);
    let state = phf_generator::generate_hash(&set.0);

    let map = build_map(&set.0, state);
    quote!(phf::Set { map: #map }).into()
}
