use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::io::Write;
use std::process::Command;

// ── interpret ─────────────────────────────────────────────────────────────────

#[test]
fn interpret_hello_world() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["interpret", "--input", "tests/programs/hello_world.bf"]);
    cmd.assert()
        .stdout(predicate::eq("Hello World!\n"))
        .success();
    Ok(())
}

#[test]
fn interpret_print_no_loop() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["interpret", "--input", "tests/programs/print_no_loop.bf"]);
    cmd.assert().stdout(predicate::eq("A")).success();
    Ok(())
}

#[test]
fn interpret_simple_loop_skipped_when_cell_is_zero() -> Result<(), Box<dyn std::error::Error>> {
    // [[]] — outer loop is never entered because cell 0 starts at 0; no output
    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["interpret", "--input", "tests/programs/simple_loop.bf"]);
    cmd.assert().stdout(predicate::str::is_empty()).success();
    Ok(())
}

#[test]
fn interpret_output_to_file() -> Result<(), Box<dyn std::error::Error>> {
    let out = assert_fs::NamedTempFile::new("out.txt")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args([
        "interpret",
        "--input",
        "tests/programs/print_no_loop.bf",
        "--output",
        out.path().to_str().unwrap(),
    ]);
    cmd.assert().stdout(predicate::str::is_empty()).success();

    out.assert(predicate::eq("A"));
    Ok(())
}

#[test]
fn interpret_pointer_underflow_fails_without_wrapping() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("underflow.bf")?;
    file.write_str("<")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["interpret", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("underflow"));
    Ok(())
}

#[test]
fn interpret_pointer_overflow_fails_without_wrapping() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("overflow.bf")?;
    // size defaults to 30000; move past the end
    file.write_str(&format!("{}>", ">".repeat(30000)))?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["interpret", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("overflow"));
    Ok(())
}

#[test]
fn interpret_pointer_wraps_with_wrapping_enabled() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("ptr-wrap.bf")?;
    // Move to cell 2, set it to 'A' (65), then wrap pointer back to it with size=3
    file.write_str(&format!(">>{}<<<.", "+".repeat(65)))?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args([
        "interpret",
        "--input",
        file.path().to_str().unwrap(),
        "--wrapping",
        "--size",
        "3",
    ]);
    cmd.assert().stdout(predicate::eq("A")).success();
    Ok(())
}

#[test]
fn interpret_cell_value_wraps_on_overflow() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("cell-wrap.bf")?;
    // 257 increments on a u8 starting at 0: wraps through 256→0, then 257→1; print \x01
    file.write_str(&format!("{}.", "+".repeat(257)))?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["interpret", "--input", file.path().to_str().unwrap()]);
    cmd.assert().stdout(predicate::eq("\x01")).success();
    Ok(())
}

#[test]
fn interpret_unmatched_open_bracket_fails() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("unmatched-open.bf")?;
    file.write_str("+[")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["interpret", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unclosed"));
    Ok(())
}

#[test]
fn interpret_unmatched_close_bracket_fails() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("unmatched-close.bf")?;
    file.write_str("+]")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["interpret", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unmatched"));
    Ok(())
}

#[test]
fn interpret_debug_flag_does_not_crash() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args([
        "interpret",
        "--input",
        "tests/programs/print_no_loop.bf",
        "--debug",
    ]);
    // debug prints to stdout (node trace) so just verify it exits cleanly
    cmd.assert().success();
    Ok(())
}

#[test]
fn interpret_input_reads_single_byte() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("read-byte.bf")?;
    file.write_str(",.")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["interpret", "--input", file.path().to_str().unwrap()]);
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;
    child.stdin.take().unwrap().write_all(b"AB\n")?;
    let output = child.wait_with_output()?;

    assert!(output.status.success());
    assert_eq!(output.stdout, b"A");
    Ok(())
}

#[test]
fn interpret_size_zero_fails_validation() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args([
        "interpret",
        "--input",
        "tests/programs/print_no_loop.bf",
        "--size",
        "0",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Tape size must be greater than 0"));
    Ok(())
}

// ── compile --target arm ──────────────────────────────────────────────────────

#[test]
fn compile_arm_emits_valid_assembly_structure() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["compile", "--target", "arm", "--input", "tests/programs/hello_world.bf"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(".global _start"))
        .stdout(predicate::str::contains("_start:"))
        .stdout(predicate::str::contains("tape:"))
        .stdout(predicate::str::contains("svc  #0"));
    Ok(())
}

#[test]
fn compile_arm_output_to_file() -> Result<(), Box<dyn std::error::Error>> {
    let out = assert_fs::NamedTempFile::new("out.s")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args([
        "compile", "--target", "arm",
        "--input",  "tests/programs/hello_world.bf",
        "--output", out.path().to_str().unwrap(),
    ]);
    cmd.assert().stdout(predicate::str::is_empty()).success();

    out.assert(predicate::str::contains(".global _start"));
    out.assert(predicate::str::contains("tape:"));
    Ok(())
}

#[test]
fn compile_arm_honors_size_and_non_wrapping_checks() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args([
        "compile",
        "--target",
        "arm",
        "--input",
        "tests/programs/print_no_loop.bf",
        "--size",
        "64",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tape: .skip 64"))
        .stdout(predicate::str::contains(".L_oob:"));
    Ok(())
}

#[test]
fn compile_arm_wrapping_omits_oob_handler() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args([
        "compile",
        "--target",
        "arm",
        "--input",
        "tests/programs/print_no_loop.bf",
        "--wrapping",
        "--size",
        "64",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tape: .skip 64"))
        .stdout(predicate::str::contains(".L_oob:").not());
    Ok(())
}

#[test]
fn compile_arm_sibling_loop_labels_are_unique() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("two-loops.bf")?;
    file.write_str("[-][+]")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["compile", "--target", "arm", "--input", file.path().to_str().unwrap()]);
    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains(".L0_start:"), "expected .L0_start label");
    assert!(stdout.contains(".L1_start:"), "expected .L1_start label");
    Ok(())
}

#[test]
fn compile_arm_nested_loop_labels_are_unique() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("nested-loops.bf")?;
    file.write_str("[[-]]")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["compile", "--target", "arm", "--input", file.path().to_str().unwrap()]);
    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains(".L0_start:"), "expected .L0_start label");
    assert!(stdout.contains(".L1_start:"), "expected .L1_start label");
    Ok(())
}

// ── compile --target llvm (default) ──────────────────────────────────────────

#[test]
fn compile_llvm_emits_valid_ir_structure() -> Result<(), Box<dyn std::error::Error>> {
    // Default target is llvm — no --target flag needed
    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["compile", "--input", "tests/programs/hello_world.bf"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("declare i32 @putchar(i32)"))
        .stdout(predicate::str::contains("declare i32 @getchar()"))
        .stdout(predicate::str::contains("@tape = global [30000 x i8] zeroinitializer"))
        .stdout(predicate::str::contains("define i32 @main()"))
        .stdout(predicate::str::contains("ret i32 0"));
    Ok(())
}

#[test]
fn compile_llvm_output_to_file() -> Result<(), Box<dyn std::error::Error>> {
    let out = assert_fs::NamedTempFile::new("out.ll")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args([
        "compile",
        "--input",  "tests/programs/hello_world.bf",
        "--output", out.path().to_str().unwrap(),
    ]);
    cmd.assert().stdout(predicate::str::is_empty()).success();

    out.assert(predicate::str::contains("define i32 @main()"));
    out.assert(predicate::str::contains("@tape = global [30000 x i8] zeroinitializer"));
    Ok(())
}

#[test]
fn compile_llvm_honors_size_and_non_wrapping_checks() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args([
        "compile",
        "--input",
        "tests/programs/print_no_loop.bf",
        "--size",
        "64",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("@tape = global [64 x i8] zeroinitializer"))
        .stdout(predicate::str::contains("oob:"));
    Ok(())
}

#[test]
fn compile_llvm_wrapping_omits_oob_handler() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args([
        "compile",
        "--input",
        "tests/programs/print_no_loop.bf",
        "--wrapping",
        "--size",
        "64",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("@tape = global [64 x i8] zeroinitializer"))
        .stdout(predicate::str::contains("oob:").not());
    Ok(())
}

#[test]
fn compile_llvm_sibling_loop_labels_are_unique() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("two-loops.bf")?;
    file.write_str("[-][+]")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["compile", "--input", file.path().to_str().unwrap()]);
    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout)?;

    let check_count = stdout.matches("_check:").count();
    assert!(check_count >= 2, "expected at least 2 loop check labels, found {check_count}");
    Ok(())
}

#[test]
fn compile_llvm_nested_loop_labels_are_unique() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("nested-loops.bf")?;
    file.write_str("[[-]]")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["compile", "--input", file.path().to_str().unwrap()]);
    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout)?;

    let check_count = stdout.matches("_check:").count();
    assert!(check_count >= 2, "expected at least 2 loop check labels, found {check_count}");
    assert!(stdout.contains("entry:"), "expected entry: label");
    Ok(())
}

#[test]
fn compile_llvm_dp_initialised_to_zero() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["compile", "--input", "tests/programs/print_no_loop.bf"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("store i64 0, ptr %dp"));
    Ok(())
}

#[test]
fn compile_llvm_output_uses_putchar() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("dot.bf")?;
    file.write_str(".")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["compile", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("call i32 @putchar"));
    Ok(())
}

#[test]
fn compile_llvm_input_uses_getchar() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("comma.bf")?;
    file.write_str(",")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["compile", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("call i32 @getchar"));
    Ok(())
}

#[test]
fn compile_llvm_input_maps_eof_to_zero() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("comma.bf")?;
    file.write_str(",")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["compile", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("select i1").and(predicate::str::contains("i8 0")));
    Ok(())
}

// ── compile — shared error handling ──────────────────────────────────────────

#[test]
fn compile_unmatched_open_bracket_fails() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("bad-open.bf")?;
    file.write_str("+[")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["compile", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unclosed"));
    Ok(())
}

#[test]
fn compile_unmatched_close_bracket_fails() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("bad-close.bf")?;
    file.write_str("+]")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args(["compile", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unmatched"));
    Ok(())
}

#[test]
fn compile_size_zero_fails_validation() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.args([
        "compile",
        "--input",
        "tests/programs/print_no_loop.bf",
        "--size",
        "0",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Tape size must be greater than 0"));
    Ok(())
}

