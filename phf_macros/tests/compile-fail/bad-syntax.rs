#![feature(plugin)]
#![plugin(phf_macros)]

extern crate phf;

static MAP: phf::Map<u32, u32> = phf_map! {
    Signature:: => () //~ ERROR expected identifier, found `=>`
};
