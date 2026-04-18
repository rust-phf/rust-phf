//! `syn` types for macro input.

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Result, Token};

#[derive(Clone)]
pub struct Key {
    pub attrs: Vec<syn::Attribute>,
    pub expr: Expr,
}

impl Parse for Key {
    fn parse(input: ParseStream<'_>) -> Result<Key> {
        Ok(Key {
            attrs: input.call(syn::Attribute::parse_outer)?,
            expr: input.parse()?,
        })
    }
}

impl ToTokens for Key {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for attr in &self.attrs {
            attr.to_tokens(tokens);
        }
        self.expr.to_tokens(tokens);
    }
}

// Attributes on entries are handled as attributed on the corresponding `Key`.
#[derive(Clone)]
pub struct Entry {
    pub key: Key,
    pub arrow: Token![=>],
    pub value: Expr,
}

impl Parse for Entry {
    fn parse(input: ParseStream<'_>) -> Result<Entry> {
        Ok(Entry {
            key: input.parse()?,
            arrow: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl ToTokens for Entry {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.key.to_tokens(tokens);
        self.arrow.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}

pub struct Map {
    pub entries: Punctuated<Entry, Token![,]>,
}

impl Parse for Map {
    fn parse(input: ParseStream<'_>) -> Result<Map> {
        Ok(Map {
            entries: Punctuated::parse_terminated(input)?,
        })
    }
}

pub struct Set {
    pub keys: Punctuated<Key, Token![,]>,
}

impl Parse for Set {
    fn parse(input: ParseStream<'_>) -> Result<Set> {
        Ok(Set {
            keys: Punctuated::parse_terminated(input)?,
        })
    }
}

pub trait AsMapEntry {
    fn key(&self) -> &Key;
    fn value(&self) -> Expr;
}

impl AsMapEntry for Key {
    fn key(&self) -> &Key {
        self
    }
    fn value(&self) -> Expr {
        syn::parse_str("()").expect("Failed to parse unit value")
    }
}

impl AsMapEntry for Entry {
    fn key(&self) -> &Key {
        &self.key
    }
    fn value(&self) -> Expr {
        self.value.clone()
    }
}
