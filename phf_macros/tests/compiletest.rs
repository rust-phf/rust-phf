#[test]
#[ignore] // diagnostics may be different between stable and nightly.
fn compile_test_unicase() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail-unicase/*.rs");
}

#[test]
#[ignore]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
