use cphf::{phf_ordered_map, OrderedMap};

pub enum Keyword {
    Loop,
    Continue,
    Break,
    Fn,
    Extern,
}

static KEYWORDS: OrderedMap<&'static str, Keyword> = phf_ordered_map! {&'static str, Keyword;
    "loop" => Keyword::Loop,
    "continue" => Keyword::Continue,
    "break" => Keyword::Break,
    "fn" => Keyword::Fn,
    "extern" => Keyword::Extern,
};

fn main() {
    let v = KEYWORDS.get("loop");

    let Some(Keyword::Loop) = v else {
        panic!("failed")
    };
}
