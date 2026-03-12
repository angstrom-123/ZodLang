use std::process::Command;

use super::*;

static TEST_DIR: &str = "./language_tests/";
static EXPECTED_EXT: &str = ".expected";
static LANGUAGE_EXT: &str = ".lang";

// Attemtps to compile a file at ./language_tests/test_name.lang and runs it.
// The output of the file is compared to the file at ./language_tests/test_name.expected.
// If the output matches, the test passes, otherwise the test fails.
fn run_test(test_name: &str) {
    let mut src_path: String = String::new();
    src_path.push_str(TEST_DIR);
    src_path.push_str(test_name);
    src_path.push_str(LANGUAGE_EXT);

    let mut exp_path: String = String::new();
    exp_path.push_str(TEST_DIR);
    exp_path.push_str(test_name);
    exp_path.push_str(EXPECTED_EXT);

    let mut res_path: String = String::new();
    res_path.push_str("./");
    res_path.push_str(test_name);

    let src: Vec<u8> = fs::read(src_path.clone()).expect("Error: Test failed to read source file");
    let exp: Vec<u8> = fs::read(exp_path.clone()).expect("Error: Test failed to read expected file");

    let compiler: Compiler = Compiler::new();
    compiler.compile(src, src_path.clone(), res_path.clone());
    let run = Command::new(res_path.clone()).output().expect("Error: Failed to run executable");
    let stdout_str: String = String::from_utf8(run.stdout.clone()).expect("Error: Failed to convert stdout to string");
    let exp_str: String = String::from_utf8(exp.clone()).expect("Error: Failed to convert expected to string");
    assert_eq!(exp, run.stdout, "{} Error: Unexpected Program output.\nExpected:\n{}\n\nGot:\n{}", src_path, exp_str, stdout_str);

    let _ = Command::new("rm").arg(res_path.clone()).output().expect("Error: Failed to delete compiled executable");
}

#[test]
fn test_arithmetic() { run_test("arithmetic"); }
#[test]
fn test_conditional() { run_test("conditional"); }
#[test]
fn test_variable() { run_test("variable"); }
#[test]
fn test_function() { run_test("function"); }
#[test]
fn test_redeclare() { run_test("redeclare"); }
#[test]
fn test_loop() { run_test("loop"); }
#[test]
fn test_comment() { run_test("comment"); }
#[test]
fn test_scope_break() { run_test("scope_break"); }
#[test]
fn test_nest() { run_test("nest"); }
#[test]
fn test_memory() { run_test("memory"); }
#[test]
fn test_index() { run_test("index"); }
#[test]
fn test_string() { run_test("string"); }
#[test]
fn test_rule110() { run_test("rule110"); }
