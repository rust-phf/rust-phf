//! A set of macros to generate Rust source for PHF data structures at compile time.
//! See [the `phf` crate's documentation][phf] for details.
//!
//! [phf]: https://docs.rs/phf

#[cfg(not(feature = "ptrhash"))]
use phf_generator::HashState;
#[cfg(feature = "ptrhash")]
use phf_generator::ptrhash::HashState;
use phf_shared::PhfHash;
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use std::collections::HashSet;
use std::hash::Hasher;
use syn::punctuated::Punctuated;
use syn::{BinOp, Error, Expr, ExprLit, Lit, Token, UnOp, parse_macro_input};
#[cfg(feature = "uncased")]
use uncased_::Uncased;
#[cfg(feature = "unicase")]
use unicase_::{Ascii, UniCase};

mod parse;
use parse::AsMapEntry;

#[derive(Hash, PartialEq, Eq)]
enum ParsedKey {
    Str(String),
    Binary(Vec<u8>),
    Char(char),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Isize(isize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Usize(usize),
    Bool(bool),
    Tuple(Vec<ParsedKey>),
    #[cfg(feature = "unicase")]
    UniCase(UniCase<String>),
    #[cfg(feature = "unicase")]
    UniCaseAscii(Ascii<String>),
    #[cfg(feature = "uncased")]
    Uncased(Uncased<'static>),
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
            ParsedKey::Isize(s) => s.phf_hash(state),
            ParsedKey::U8(s) => s.phf_hash(state),
            ParsedKey::U16(s) => s.phf_hash(state),
            ParsedKey::U32(s) => s.phf_hash(state),
            ParsedKey::U64(s) => s.phf_hash(state),
            ParsedKey::U128(s) => s.phf_hash(state),
            ParsedKey::Usize(s) => s.phf_hash(state),
            ParsedKey::Bool(s) => s.phf_hash(state),
            ParsedKey::Tuple(elements) => {
                for element in elements {
                    element.phf_hash(state);
                }
            }
            #[cfg(feature = "unicase")]
            ParsedKey::UniCase(s) => s.phf_hash(state),
            #[cfg(feature = "unicase")]
            ParsedKey::UniCaseAscii(s) => s.phf_hash(state),
            #[cfg(feature = "uncased")]
            ParsedKey::Uncased(s) => s.phf_hash(state),
        }
    }
}

impl ParsedKey {
    fn has_same_type_as(&self, other: &ParsedKey) -> bool {
        match (self, other) {
            (ParsedKey::Str(_), ParsedKey::Str(_))
            | (ParsedKey::Binary(_), ParsedKey::Binary(_))
            | (ParsedKey::Char(_), ParsedKey::Char(_))
            | (ParsedKey::I8(_), ParsedKey::I8(_))
            | (ParsedKey::I16(_), ParsedKey::I16(_))
            | (ParsedKey::I32(_), ParsedKey::I32(_))
            | (ParsedKey::I64(_), ParsedKey::I64(_))
            | (ParsedKey::I128(_), ParsedKey::I128(_))
            | (ParsedKey::Isize(_), ParsedKey::Isize(_))
            | (ParsedKey::U8(_), ParsedKey::U8(_))
            | (ParsedKey::U16(_), ParsedKey::U16(_))
            | (ParsedKey::U32(_), ParsedKey::U32(_))
            | (ParsedKey::U64(_), ParsedKey::U64(_))
            | (ParsedKey::U128(_), ParsedKey::U128(_))
            | (ParsedKey::Usize(_), ParsedKey::Usize(_))
            | (ParsedKey::Bool(_), ParsedKey::Bool(_)) => true,
            (ParsedKey::Tuple(left), ParsedKey::Tuple(right)) => {
                left.len() == right.len()
                    && left
                        .iter()
                        .zip(right)
                        .all(|(left, right)| left.has_same_type_as(right))
            }
            #[cfg(feature = "unicase")]
            (ParsedKey::UniCase(_), ParsedKey::UniCase(_)) => true,
            #[cfg(feature = "unicase")]
            (ParsedKey::UniCaseAscii(_), ParsedKey::UniCaseAscii(_)) => true,
            #[cfg(feature = "uncased")]
            (ParsedKey::Uncased(_), ParsedKey::Uncased(_)) => true,
            _ => false,
        }
    }

    fn from_unsuffixed_int(s: &syn::LitInt, hint: Option<&ParsedKey>) -> syn::Result<ParsedKey> {
        match hint {
            Some(ParsedKey::I8(_)) => Ok(ParsedKey::I8(s.base10_parse::<u8>().unwrap() as i8)),
            Some(ParsedKey::I16(_)) => Ok(ParsedKey::I16(s.base10_parse::<u16>().unwrap() as i16)),
            Some(ParsedKey::I32(_)) => Ok(ParsedKey::I32(s.base10_parse::<u32>().unwrap() as i32)),
            Some(ParsedKey::I64(_)) => Ok(ParsedKey::I64(s.base10_parse::<u64>().unwrap() as i64)),
            Some(ParsedKey::I128(_)) => {
                Ok(ParsedKey::I128(s.base10_parse::<u128>().unwrap() as i128))
            }
            Some(ParsedKey::Isize(_)) => {
                Ok(ParsedKey::Isize(s.base10_parse::<usize>().unwrap() as isize))
            }
            Some(ParsedKey::U8(_)) => Ok(ParsedKey::U8(s.base10_parse::<u8>().unwrap())),
            Some(ParsedKey::U16(_)) => Ok(ParsedKey::U16(s.base10_parse::<u16>().unwrap())),
            Some(ParsedKey::U32(_)) => Ok(ParsedKey::U32(s.base10_parse::<u32>().unwrap())),
            Some(ParsedKey::U64(_)) => Ok(ParsedKey::U64(s.base10_parse::<u64>().unwrap())),
            Some(ParsedKey::U128(_)) => Ok(ParsedKey::U128(s.base10_parse::<u128>().unwrap())),
            Some(ParsedKey::Usize(_)) => Ok(ParsedKey::Usize(s.base10_parse::<usize>().unwrap())),
            Some(_) => Err(Error::new_spanned(
                s,
                "integer key literal type could not be inferred from the first key",
            )),
            None => Err(Error::new_spanned(
                s,
                "integer key literals in the first key must have an explicit type suffix",
            )),
        }
    }

    fn from_expr(expr: &Expr, hint: Option<&ParsedKey>) -> syn::Result<ParsedKey> {
        match ungroup(expr) {
            Expr::Lit(lit) => match &lit.lit {
                Lit::Str(s) => Ok(ParsedKey::Str(s.value())),
                Lit::ByteStr(s) => Ok(ParsedKey::Binary(s.value())),
                Lit::Byte(s) => Ok(ParsedKey::U8(s.value())),
                Lit::Char(s) => Ok(ParsedKey::Char(s.value())),
                Lit::Int(s) => match s.suffix() {
                    // we've lost the sign at this point, so `-128i8` looks like `128i8`,
                    // which doesn't fit in an `i8`; parse it as a `u8` and cast (to `0i8`),
                    // which is handled below, by `Unary`
                    "i8" => Ok(ParsedKey::I8(s.base10_parse::<u8>().unwrap() as i8)),
                    "i16" => Ok(ParsedKey::I16(s.base10_parse::<u16>().unwrap() as i16)),
                    "i32" => Ok(ParsedKey::I32(s.base10_parse::<u32>().unwrap() as i32)),
                    "i64" => Ok(ParsedKey::I64(s.base10_parse::<u64>().unwrap() as i64)),
                    "i128" => Ok(ParsedKey::I128(s.base10_parse::<u128>().unwrap() as i128)),
                    "isize" => Ok(ParsedKey::Isize(s.base10_parse::<usize>().unwrap() as isize)),
                    "u8" => Ok(ParsedKey::U8(s.base10_parse::<u8>().unwrap())),
                    "u16" => Ok(ParsedKey::U16(s.base10_parse::<u16>().unwrap())),
                    "u32" => Ok(ParsedKey::U32(s.base10_parse::<u32>().unwrap())),
                    "u64" => Ok(ParsedKey::U64(s.base10_parse::<u64>().unwrap())),
                    "u128" => Ok(ParsedKey::U128(s.base10_parse::<u128>().unwrap())),
                    "usize" => Ok(ParsedKey::Usize(s.base10_parse::<usize>().unwrap())),
                    "" => ParsedKey::from_unsuffixed_int(s, hint),
                    _ => Err(Error::new_spanned(s, "unsupported integer literal suffix")),
                },
                Lit::Bool(s) => Ok(ParsedKey::Bool(s.value)),
                _ => Err(Error::new_spanned(expr, "unsupported key expression")),
            },
            Expr::Array(array) => {
                let mut buf = vec![];
                let can_infer_u8 = matches!(hint, Some(ParsedKey::Binary(_)));
                for expr in &array.elems {
                    match ungroup(expr) {
                        Expr::Lit(lit) => match &lit.lit {
                            Lit::Int(s) => match s.suffix() {
                                "u8" => buf.push(s.base10_parse::<u8>().unwrap()),
                                "" => {
                                    if can_infer_u8 || !buf.is_empty() {
                                        buf.push(s.base10_parse::<u8>().unwrap());
                                    } else {
                                        return Err(Error::new_spanned(
                                            s,
                                            "integer key literals in the first key must have an explicit type suffix",
                                        ));
                                    }
                                }
                                _ => {
                                    return Err(Error::new_spanned(
                                        s,
                                        "array key literals must be suffixed `u8`",
                                    ));
                                }
                            },
                            _ => {
                                return Err(Error::new_spanned(expr, "unsupported key expression"));
                            }
                        },
                        _ => return Err(Error::new_spanned(expr, "unsupported key expression")),
                    }
                }
                Ok(ParsedKey::Binary(buf))
            }
            Expr::Unary(unary) => {
                // Handle negation for signed integer types
                // If we received an integer literal (always unsigned) greater than i__::max_value()
                // then casting it to a signed integer type of the same width will negate it to
                // the same absolute value so we don't need to negate it here
                macro_rules! try_negate {
                    ($val:expr) => {
                        if $val < 0 { $val } else { -$val }
                    };
                }

                match unary.op {
                    UnOp::Neg(_) => match ParsedKey::from_expr(&unary.expr, hint)? {
                        ParsedKey::I8(v) => Ok(ParsedKey::I8(try_negate!(v))),
                        ParsedKey::I16(v) => Ok(ParsedKey::I16(try_negate!(v))),
                        ParsedKey::I32(v) => Ok(ParsedKey::I32(try_negate!(v))),
                        ParsedKey::I64(v) => Ok(ParsedKey::I64(try_negate!(v))),
                        ParsedKey::I128(v) => Ok(ParsedKey::I128(try_negate!(v))),
                        ParsedKey::Isize(v) => Ok(ParsedKey::Isize(try_negate!(v))),
                        _ => Err(Error::new_spanned(expr, "unsupported key expression")),
                    },
                    UnOp::Deref(_) => match ungroup(&unary.expr) {
                        Expr::Lit(ExprLit {
                            lit: Lit::ByteStr(s),
                            ..
                        }) => Ok(ParsedKey::Binary(s.value())),
                        _ => Err(Error::new_spanned(expr, "unsupported key expression")),
                    },
                    _ => Err(Error::new_spanned(expr, "unsupported key expression")),
                }
            }
            Expr::Tuple(tuple) => {
                let mut elements = Vec::new();
                for (idx, elem) in tuple.elems.iter().enumerate() {
                    let elem_hint = match hint {
                        Some(ParsedKey::Tuple(hints)) => hints.get(idx),
                        _ => None,
                    };
                    elements.push(ParsedKey::from_expr(elem, elem_hint)?);
                }
                Ok(ParsedKey::Tuple(elements))
            }
            Expr::Binary(binary) if matches!(binary.op, BinOp::BitOr(_)) => Err(
                Error::new_spanned(expr, "expected a single key, found an OR pattern"),
            ),
            Expr::Call(call) if call.args.len() == 1 => {
                let last;
                let last_ahead;

                if let Expr::Path(ep) = call.func.as_ref() {
                    let mut segments = ep.path.segments.iter();
                    last = segments
                        .next_back()
                        .ok_or_else(|| Error::new_spanned(expr, "unsupported key expression"))?
                        .ident
                        .to_string();
                    last_ahead = segments
                        .next_back()
                        .ok_or_else(|| Error::new_spanned(expr, "unsupported key expression"))?
                        .ident
                        .to_string();
                } else {
                    return Err(Error::new_spanned(expr, "unsupported key expression"));
                }

                let _value = match ungroup(&call.args[0]) {
                    Expr::Lit(ExprLit {
                        attrs: _,
                        lit: Lit::Str(s),
                    }) => s.value(),
                    _ => {
                        return Err(Error::new_spanned(expr, "unsupported key expression"));
                    }
                };

                match (&*last_ahead, &*last) {
                    #[cfg(feature = "unicase")]
                    ("UniCase", "unicode") => Ok(ParsedKey::UniCase(UniCase::unicode(_value))),
                    #[cfg(feature = "unicase")]
                    ("UniCase", "ascii") => Ok(ParsedKey::UniCase(UniCase::ascii(_value))),
                    #[cfg(feature = "unicase")]
                    ("Ascii", "new") => Ok(ParsedKey::UniCaseAscii(Ascii::new(_value))),
                    #[cfg(feature = "uncased")]
                    ("UncasedStr", "new") => Ok(ParsedKey::Uncased(Uncased::new(_value))),
                    _ => Err(Error::new_spanned(expr, "unsupported key expression")),
                }
            }
            _ => Err(Error::new_spanned(expr, "unsupported key expression")),
        }
    }
}

fn ungroup(mut expr: &Expr) -> &Expr {
    loop {
        expr = match expr {
            Expr::Group(group) => &group.expr,
            Expr::Paren(paren) => &paren.expr,
            _ => return expr,
        };
    }
}

fn generate_hash_state<H: PhfHash>(entries: &[H]) -> HashState {
    #[cfg(not(feature = "ptrhash"))]
    {
        phf_generator::generate_hash(entries)
    }

    #[cfg(feature = "ptrhash")]
    {
        phf_generator::ptrhash::generate_hash(entries)
    }
}

fn expand_or_patterns(key: &Expr) -> Vec<Expr> {
    match key {
        Expr::Binary(binary) if matches!(binary.op, BinOp::BitOr(_)) => {
            let mut variants = expand_or_patterns(&binary.left);
            variants.extend(expand_or_patterns(&binary.right));
            variants
        }
        Expr::Tuple(tuple) => expand_elements(&tuple.elems)
            .into_iter()
            .map(|elems| {
                Expr::Tuple(syn::ExprTuple {
                    elems,
                    ..tuple.clone()
                })
            })
            .collect(),
        Expr::Array(array) => expand_elements(&array.elems)
            .into_iter()
            .map(|elems| {
                Expr::Array(syn::ExprArray {
                    elems,
                    ..array.clone()
                })
            })
            .collect(),
        Expr::Call(call) if call.args.len() == 1 => expand_or_patterns(&call.args[0])
            .into_iter()
            .map(|arg| {
                let mut call = call.clone();
                call.args[0] = arg;
                Expr::Call(call)
            })
            .collect(),
        Expr::Unary(unary) => expand_or_patterns(&unary.expr)
            .into_iter()
            .map(|expr| {
                Expr::Unary(syn::ExprUnary {
                    expr: Box::new(expr),
                    ..unary.clone()
                })
            })
            .collect(),
        Expr::Group(group) => expand_or_patterns(&group.expr),
        Expr::Paren(paren) => expand_or_patterns(&paren.expr),
        _ => vec![key.clone()],
    }
}

fn expand_elements(elements: &Punctuated<Expr, Token![,]>) -> Vec<Punctuated<Expr, Token![,]>> {
    let mut variants = vec![elements.clone()];
    for (index, element) in elements.iter().enumerate() {
        let expanded = expand_or_patterns(element);
        if let [element] = expanded.as_slice() {
            for variant in &mut variants {
                variant[index] = element.clone();
            }
            continue;
        }
        // Form the Cartesian product, with earlier elements varying more slowly.
        variants = variants
            .into_iter()
            .flat_map(|elements| {
                expanded.iter().map(move |element| {
                    let mut elements = elements.clone();
                    elements[index] = element.clone();
                    elements
                })
            })
            .collect();
    }
    variants
}

// Parse a column of keys with the first key as the type hint for that column.
fn hash_keys<'a>(
    exprs: impl Iterator<Item = &'a Expr> + Clone,
    name: &str,
) -> syn::Result<HashState> {
    let mut keys = Vec::with_capacity(exprs.size_hint().0);
    for expr in exprs.clone() {
        let first = keys.first();
        let key = ParsedKey::from_expr(expr, first)?;
        if first.is_some_and(|first| !key.has_same_type_as(first)) {
            return Err(Error::new_spanned(
                expr,
                format!("{name} type does not match the first {name}"),
            ));
        }
        keys.push(key);
    }

    let mut seen = HashSet::with_capacity(keys.len());
    for (key, expr) in keys.iter().zip(exprs) {
        if !seen.insert(key) {
            return Err(Error::new_spanned(expr, format!("duplicate {name}")));
        }
    }
    drop(seen);
    Ok(generate_hash_state(&keys))
}

fn hash_fields(state: &HashState) -> proc_macro2::TokenStream {
    #[cfg(not(feature = "ptrhash"))]
    {
        let key = state.key;
        let disps = state.disps.iter().map(|(d1, d2)| quote!((#d1, #d2)));
        quote!(key: #key, disps: &[#(#disps),*],)
    }
    #[cfg(feature = "ptrhash")]
    {
        let key = state.seed;
        let pilots = &state.pilots;
        quote!(key: #key, pilots: &[#(#pilots),*],)
    }
}

fn entry_tokens((key, value): &(Expr, Expr)) -> proc_macro2::TokenStream {
    quote!((#key, #value))
}

fn build_map(entries: &[(Expr, Expr)], state: &HashState) -> proc_macro2::TokenStream {
    let fields = hash_fields(state);
    #[cfg(feature = "ptrhash")]
    let padding = {
        let empty_slots = &state.empty_slots;
        quote!(empty_slots: &[#(#empty_slots),*],)
    };
    #[cfg(not(feature = "ptrhash"))]
    let padding = quote!();
    let entries = state.map.iter().map(|&idx| entry_tokens(&entries[idx]));
    quote!(phf::Map { #fields #padding entries: &[#(#entries),*] })
}

fn build_index(state: &HashState) -> proc_macro2::TokenStream {
    let fields = hash_fields(state);
    let idxs = state
        .map
        .iter()
        .copied()
        .map(proc_macro2::Literal::usize_unsuffixed);
    quote!(phf::HashIndex { #fields idxs: &[#(#idxs),*] })
}

fn build_ordered_map(entries: &[(Expr, Expr)], state: &HashState) -> proc_macro2::TokenStream {
    let fields = hash_fields(state);
    let idxs = &state.map;
    let entries = entries.iter().map(entry_tokens);
    quote!(phf::OrderedMap { #fields idxs: &[#(#idxs),*], entries: &[#(#entries),*] })
}

fn resolve_cfg<T: AsMapEntry + ToTokens>(
    macro_name: impl ToTokens,
    entries: Punctuated<T, Token![,]>,
    generate: impl FnOnce(Punctuated<T, Token![,]>) -> syn::Result<proc_macro2::TokenStream>,
) -> TokenStream {
    let has_cfg = |entry: &T| {
        entry
            .key()
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("cfg"))
    };
    if !entries.iter().any(has_cfg) {
        return generate(entries)
            .unwrap_or_else(Error::into_compile_error)
            .into();
    }

    let mut cfg_args = quote! { #macro_name [] };

    // Wrap conditional entries and groups of unconditional entries in { ... }.
    // Grouping avoids unnecessarily hitting macro recursion limit. Entries are
    // not reordered to handle ordered maps correctly (see #395).
    let mut unconditional = Vec::new();
    for pair in entries.pairs() {
        let entry = pair.value();
        if has_cfg(entry) {
            // Pushing groups unconditionally simplifies the decl macro side.
            quote! { { #(#unconditional)* } }.to_tokens(&mut cfg_args);
            unconditional.clear();
            quote! { { #pair } }.to_tokens(&mut cfg_args);
        } else {
            unconditional.push(pair);
        }
    }
    quote! { { #(#unconditional)* } }.to_tokens(&mut cfg_args);

    quote! {
        // We generate code including paths like `phf::Map`, so accessing macros
        // from `phf` like this should be alright.
        phf::__resolve_cfg! {
            #cfg_args
        }
    }
    .into()
}

fn emit_code(
    macro_name: impl ToTokens,
    entries: Punctuated<impl AsMapEntry + ToTokens, Token![,]>,
    builder: fn(&[(Expr, Expr)], &HashState) -> proc_macro2::TokenStream,
) -> TokenStream {
    resolve_cfg(macro_name, entries, |input| {
        let mut entries = Vec::with_capacity(input.len());
        for entry in input {
            let (key, value) = entry.into_pair();
            for key in expand_or_patterns(&key) {
                entries.push((key, value.clone()));
            }
        }
        let state = hash_keys(entries.iter().map(|(key, _)| key), "key")?;
        Ok(builder(&entries, &state))
    })
}

#[proc_macro]
pub fn phf_bimap(input: TokenStream) -> TokenStream {
    let map = parse_macro_input!(input as parse::Map);
    resolve_cfg(quote!(phf_bimap), map.entries, |input| {
        let entries: Vec<_> = input.into_iter().map(AsMapEntry::into_pair).collect();
        let left = hash_keys(entries.iter().map(|(l, _)| l), "left key")?;
        let right = hash_keys(entries.iter().map(|(_, r)| r), "right key")?;
        let left = build_index(&left);
        let right = build_index(&right);
        let entries = entries.iter().map(entry_tokens);
        Ok(quote!(phf::BiMap { left: #left, right: #right, entries: &[#(#entries),*] }))
    })
}

#[proc_macro]
pub fn phf_map(input: TokenStream) -> TokenStream {
    let map = parse_macro_input!(input as parse::Map);
    emit_code(quote! { phf_map }, map.entries, build_map)
}

#[proc_macro]
pub fn phf_set(input: TokenStream) -> TokenStream {
    let set = parse_macro_input!(input as parse::Set);
    emit_code(quote! { phf_set }, set.keys, |entries, state| {
        let map = build_map(entries, state);
        quote!(phf::Set { map: #map })
    })
}

#[proc_macro]
pub fn phf_ordered_map(input: TokenStream) -> TokenStream {
    let map = parse_macro_input!(input as parse::Map);
    emit_code(quote! { phf_ordered_map }, map.entries, build_ordered_map)
}

#[proc_macro]
pub fn phf_ordered_set(input: TokenStream) -> TokenStream {
    let set = parse_macro_input!(input as parse::Set);
    emit_code(quote! { phf_ordered_set }, set.keys, |entries, state| {
        let map = build_ordered_map(entries, state);
        quote!(phf::OrderedSet { map: #map })
    })
}
