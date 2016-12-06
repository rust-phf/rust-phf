extern crate compiletest_rs as compiletest;

use std::path::Path;

#[allow(dead_code)]
fn run_mode(directory: &'static str, mode: &'static str) {
    let mut config = compiletest::default_config();
    let cfg_mode = mode.parse().ok().expect("Invalid mode");

    config.mode = cfg_mode;
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let flags = format!("-L {}", dir.join("../target/debug/deps").display());
    config.target_rustcflags = Some(flags);
    config.src_base = dir.join("tests").join(directory);

    compiletest::run_tests(&config);
}

#[cfg(feature = "unicase_support")]
#[test]
fn compile_test_unicase() {
    run_mode("compile-fail-unicase", "compile-fail");
}
