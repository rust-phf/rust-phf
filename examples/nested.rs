use cphf::{phf_ordered_map, OrderedMap};

pub enum Keyword {
    Loop,
    Continue,
    Break,
    Fn,
    Extern,
}

#[allow(long_running_const_eval)]
static KEYWORDS: OrderedMap<u32, OrderedMap<&str, Keyword>> = phf_ordered_map! {u32, OrderedMap<&'static str, Keyword>;
    0 => phf_ordered_map! {&'static str, Keyword;
        "loop" => Keyword::Loop,
        "continue" => Keyword::Continue,
    },
    2 => phf_ordered_map! {&'static str, Keyword;
        "break" => Keyword::Break,
        "fn" => Keyword::Fn,
    },
    9 => phf_ordered_map! {&'static str, Keyword;
        "break" => Keyword::Break,
        "fn" => Keyword::Fn,
        "extern" => Keyword::Extern,
    },
};

fn main() {
    let v = KEYWORDS.get(&2).unwrap().get("break");

    let Some(Keyword::Break) = v else {
        panic!("failed")
    };
}
