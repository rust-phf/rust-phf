// The diagnostics may be different between stable and nightly, so
// we mark them as `#[ignore]` and invoke with `--ignored` explicitly
// when testing.

#[test]
#[ignore]
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
