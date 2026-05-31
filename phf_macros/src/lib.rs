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
        match expr {
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
                    match expr {
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
                    UnOp::Deref(_) => {
                        let mut expr = &*unary.expr;
                        while let Expr::Group(group) = expr {
                            expr = &*group.expr;
                        }
                        match expr {
                            Expr::Lit(ExprLit {
                                lit: Lit::ByteStr(s),
                                ..
                            }) => Ok(ParsedKey::Binary(s.value())),
                            _ => Err(Error::new_spanned(expr, "unsupported key expression")),
                        }
                    }
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
            Expr::Group(group) => ParsedKey::from_expr(&group.expr, hint),
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

                let mut arg = call.args.first().unwrap();

                while let Expr::Group(group) = arg {
                    arg = &group.expr;
                }

                let _value = match arg {
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

#[derive(Clone)]
struct Entry {
    parsed_key: ParsedKey,
    key_expr: Expr,
    value_expr: Expr,
}

impl PhfHash for Entry {
    fn phf_hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.parsed_key.phf_hash(state)
    }
}

struct Map {
    entries: Vec<Entry>,
    key_hint: Option<ParsedKey>,
}

impl Map {
    fn from_parsed(entries: Punctuated<impl AsMapEntry, Token![,]>) -> syn::Result<Self> {
        let mut map = Self {
            entries: Vec::new(),
            key_hint: None,
        };
        for entry in entries {
            map.add_variants_from(&entry.key().expr, &entry.value())?;
        }
        map.check_duplicates()?;
        Ok(map)
    }

    /// Handle OR patterns within the key expression
    fn add_variants_from(&mut self, key: &Expr, value: &Expr) -> syn::Result<()> {
        if let Expr::Binary(binary) = key {
            if let BinOp::BitOr(_) = binary.op {
                // Handle OR pattern: left | right
                self.add_variants_from(&binary.left, value)?;
                self.add_variants_from(&binary.right, value)?;
                return Ok(());
            }
        }
        // Single key
        let parsed_key = ParsedKey::from_expr(key, self.key_hint.as_ref())?;
        if let Some(key_hint) = &self.key_hint {
            if !parsed_key.has_same_type_as(key_hint) {
                return Err(Error::new_spanned(
                    key,
                    "key type does not match the first key",
                ));
            }
        } else {
            self.key_hint = Some(parsed_key.clone());
        }
        self.entries.push(Entry {
            parsed_key,
            key_expr: key.clone(),
            value_expr: value.clone(),
        });
        Ok(())
    }

    fn check_duplicates(&self) -> syn::Result<()> {
        let mut keys = HashSet::new();
        for entry in &self.entries {
            if !keys.insert(&entry.parsed_key) {
                return Err(Error::new_spanned(&entry.key_expr, "duplicate key"));
            }
        }
        Ok(())
    }
}

fn key_has_cfg_attr(key: &parse::Key) -> bool {
    key.attrs.iter().any(|attr| attr.path().is_ident("cfg"))
}

fn build_map(entries: &[Entry], state: HashState) -> proc_macro2::TokenStream {
    #[cfg(not(feature = "ptrhash"))]
    {
        let key = state.key;
        let disps = state.disps.iter().map(|&(d1, d2)| quote!((#d1, #d2)));
        let entries = state.map.iter().map(|&idx| {
            let entry = &entries[idx];
            let key = &entry.key_expr;
            let value = &entry.value_expr;
            quote!((#key, #value))
        });

        quote! {
            phf::Map {
                key: #key,
                disps: &[#(#disps),*],
                entries: &[#(#entries),*],
            }
        }
    }

    #[cfg(feature = "ptrhash")]
    {
        let key = state.seed;
        let pilots = state.pilots.iter().map(|pilot| quote!(#pilot));
        let remap = state.remap.iter().map(|index| quote!(#index));
        let entries = state.map.iter().map(|&idx| {
            let entry = &entries[idx];
            let key = &entry.key_expr;
            let value = &entry.value_expr;
            quote!((#key, #value))
        });

        quote! {
            phf::Map {
                key: #key,
                pilots: &[#(#pilots),*],
                remap: &[#(#remap),*],
                entries: &[#(#entries),*],
            }
        }
    }
}

fn build_ordered_map(entries: &[Entry], state: HashState) -> proc_macro2::TokenStream {
    #[cfg(not(feature = "ptrhash"))]
    {
        let key = state.key;
        let disps = state.disps.iter().map(|&(d1, d2)| quote!((#d1, #d2)));
        let idxs = state.map.iter().map(|idx| quote!(#idx));
        let entries = entries.iter().map(|entry| {
            let key = &entry.key_expr;
            let value = &entry.value_expr;
            quote!((#key, #value))
        });

        quote! {
            phf::OrderedMap {
                key: #key,
                disps: &[#(#disps),*],
                idxs: &[#(#idxs),*],
                entries: &[#(#entries),*],
            }
        }
    }

    #[cfg(feature = "ptrhash")]
    {
        let key = state.seed;
        let pilots = state.pilots.iter().map(|pilot| quote!(#pilot));
        let remap = state.remap.iter().map(|index| quote!(#index));
        let idxs = state.map.iter().map(|idx| quote!(#idx));
        let entries = entries.iter().map(|entry| {
            let key = &entry.key_expr;
            let value = &entry.value_expr;
            quote!((#key, #value))
        });

        quote! {
            phf::OrderedMap {
                key: #key,
                pilots: &[#(#pilots),*],
                remap: &[#(#remap),*],
                idxs: &[#(#idxs),*],
                entries: &[#(#entries),*],
            }
        }
    }
}

fn resolve_cfg<T: AsMapEntry + ToTokens>(
    macro_name: impl ToTokens,
    entries: Punctuated<T, Token![,]>,
) -> TokenStream {
    let mut cfg_args = quote! { #macro_name [] };

    // Wrap conditional entries and groups of unconditional entries in { ... }.
    // Grouping avoids unnecessarily hitting macro recursion limit. Entries are
    // not reordered to handle ordered maps correctly (see #395).
    let mut unconditional = Vec::new();
    for pair in entries.pairs() {
        let entry = pair.value();
        if key_has_cfg_attr(entry.key()) {
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
    builder: fn(&[Entry], HashState) -> proc_macro2::TokenStream,
) -> TokenStream {
    // If any entries have cfg attributes, resolve them via decl macro
    let has_cfg_attrs = entries.iter().any(|entry| key_has_cfg_attr(entry.key()));
    if has_cfg_attrs {
        return resolve_cfg(macro_name, entries);
    }

    // No cfg attributes - generate code directly
    match Map::from_parsed(entries) {
        Ok(map) => {
            let state = generate_hash_state(&map.entries);
            builder(&map.entries, state).into()
        }
        Err(err) => err.to_compile_error().into(),
    }
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
