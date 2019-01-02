#![feature(proc_macro_hygiene)]

extern crate phf;
extern crate phf_macros;

use phf_macros::phf_map;

static MAP: phf::Map<u32, u32> = phf_map! {
    Signature::
    => //~ ERROR expected identifier
    ()
};
