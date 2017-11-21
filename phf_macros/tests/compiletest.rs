extern crate compiletest_rs as compiletest;

use std::path::Path;

#[allow(dead_code)]
fn run_mode(directory: &'static str, mode: &'static str) {
    let mut config = compiletest::default_config();
    let cfg_mode = mode.parse().ok().expect("Invalid mode");

    config.mode = cfg_mode;
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    config.target_rustcflags = Some(format!("-L {}", dir.join("../target/debug/deps").display()));
    config.src_base = dir.join("tests").join(directory);

    compiletest::run_tests(&config);
}

#[test]
fn compile_fail() {
    run_mode("compile-fail", "compile-fail");
}