extern crate phf;

use phf::phf_map;

phf_map! {
    static MAP: phf::Map<u32, u32> = {
        Signature::
        => //~ ERROR expected identifier
        ()
    };
}
