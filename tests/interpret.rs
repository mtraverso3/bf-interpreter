use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::io::Write;
use std::process::Command;

#[test]
fn hello_world() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["interpret", "--input", "tests/programs/hello_world.bf"]);
    cmd.assert()
        .stdout(predicate::eq("Hello World!\n"))
        .success();
    Ok(())
}

#[test]
fn print_no_loop() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["interpret", "--input", "tests/programs/print_no_loop.bf"]);
    cmd.assert().stdout(predicate::eq("A")).success();
    Ok(())
}

#[test]
fn simple_loop_skipped_when_cell_is_zero() -> Result<(), Box<dyn std::error::Error>> {
    // [[]] — outer loop is never entered because cell 0 starts at 0; no output
    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["interpret", "--input", "tests/programs/simple_loop.bf"]);
    cmd.assert().stdout(predicate::str::is_empty()).success();
    Ok(())
}

#[test]
fn output_to_file() -> Result<(), Box<dyn std::error::Error>> {
    let out = assert_fs::NamedTempFile::new("out.txt")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
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
fn pointer_underflow_fails_without_wrapping() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("underflow.bf")?;
    file.write_str("<")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["interpret", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("underflow"));
    Ok(())
}

#[test]
fn pointer_overflow_fails_without_wrapping() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("overflow.bf")?;
    // size defaults to 30000; move past the end
    file.write_str(&format!("{}>", ">".repeat(30000)))?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["interpret", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("overflow"));
    Ok(())
}

#[test]
fn pointer_wraps_with_wrapping_enabled() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("ptr-wrap.bf")?;
    // Move to cell 2, set it to 'A' (65), then wrap pointer back to it with size=3
    file.write_str(&format!(">>{}<<<.", "+".repeat(65)))?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
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
fn cell_value_wraps_on_overflow() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("cell-wrap.bf")?;
    // 257 increments on a u8 starting at 0: wraps through 256→0, then 257→1; print \x01
    file.write_str(&format!("{}.", "+".repeat(257)))?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["interpret", "--input", file.path().to_str().unwrap()]);
    cmd.assert().stdout(predicate::eq("\x01")).success();
    Ok(())
}

#[test]
fn unmatched_open_bracket_fails() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("unmatched-open.bf")?;
    file.write_str("+[")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["interpret", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unclosed"));
    Ok(())
}

#[test]
fn unmatched_close_bracket_fails() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("unmatched-close.bf")?;
    file.write_str("+]")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["interpret", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unmatched"));
    Ok(())
}

#[test]
fn debug_flag_does_not_crash() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bf-tools")?;
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
fn input_reads_single_byte() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("read-byte.bf")?;
    file.write_str(",.")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
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
fn size_zero_fails_validation() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bf-tools")?;
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

