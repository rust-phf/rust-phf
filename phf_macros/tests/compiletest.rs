extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

#[allow(dead_code)]
fn run_mode(directory: &'static str, mode: &'static str) {
    let mut config = compiletest::default_config();
    let cfg_mode = mode.parse().ok().expect("Invalid mode");

    config.mode = cfg_mode;
    config.target_rustcflags = Some("-L target/debug/".to_owned());
    config.src_base = PathBuf::from(format!("tests/{}", directory));

    compiletest::run_tests(&config);
}

// #[test]
// fn compile_test() {
//     run_mode("compile-fail", "compile-fail");
// }

#[cfg(feature = "unicase_support")]
#[test]
fn compile_test_unicase() {
    run_mode("compile-fail-unicase", "compile-fail");
}
