extern crate phf;
extern crate unicase;

phf_map! {
    static MAP: phf::Map<UniCase<&'static str>, isize> = ( //~ ERROR duplicate key UniCase("FOO")
        UniCase("FOO") => 42, //~ NOTE one occurrence here
        UniCase("foo") => 42, //~ NOTE one occurrence here
    );
}
