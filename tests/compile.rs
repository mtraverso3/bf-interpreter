use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

// ── arm ───────────────────────────────────────────────────────────────────────

#[test]
fn arm_emits_valid_assembly_structure() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bf-tools")?;
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
fn arm_output_to_file() -> Result<(), Box<dyn std::error::Error>> {
    let out = assert_fs::NamedTempFile::new("out.s")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
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
fn arm_honors_size_and_non_wrapping_checks() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args([
        "compile", "--target", "arm",
        "--input", "tests/programs/print_no_loop.bf",
        "--size", "64",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tape: .skip 64"))
        .stdout(predicate::str::contains(".L_oob:"));
    Ok(())
}

#[test]
fn arm_wrapping_omits_oob_handler() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args([
        "compile", "--target", "arm",
        "--input", "tests/programs/print_no_loop.bf",
        "--wrapping", "--size", "64",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tape: .skip 64"))
        .stdout(predicate::str::contains(".L_oob:").not());
    Ok(())
}

#[test]
fn arm_sibling_loop_labels_are_unique() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("two-loops.bf")?;
    file.write_str("+[.-]+[.-]")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["compile", "--target", "arm", "--input", file.path().to_str().unwrap()]);
    let stdout = String::from_utf8(cmd.output()?.stdout)?;

    assert!(stdout.contains(".L0_start:"), "expected .L0_start label");
    assert!(stdout.contains(".L1_start:"), "expected .L1_start label");
    Ok(())
}

#[test]
fn arm_nested_loop_labels_are_unique() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("nested-loops.bf")?;
    file.write_str("+[>[.-]<-]")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["compile", "--target", "arm", "--input", file.path().to_str().unwrap()]);
    let stdout = String::from_utf8(cmd.output()?.stdout)?;

    assert!(stdout.contains(".L0_start:"), "expected .L0_start label");
    assert!(stdout.contains(".L1_start:"), "expected .L1_start label");
    Ok(())
}

#[test]
fn arm_compiles_counted_moves_from_ir() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("moves.bf")?;
    file.write_str(">>>")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["compile", "--target", "arm", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("// move 3"));
    Ok(())
}

#[test]
fn arm_compiles_clear_loops_as_clear_ir() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("clear.bf")?;
    file.write_str("[-]")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["compile", "--target", "arm", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("// clear"))
        .stdout(predicate::str::contains("_start:").and(predicate::str::contains(".L0_start:").not()));
    Ok(())
}

// ── llvm ──────────────────────────────────────────────────────────────────────

#[test]
fn llvm_emits_valid_ir_structure() -> Result<(), Box<dyn std::error::Error>> {
    // Default target is llvm — no --target flag needed
    let mut cmd = Command::cargo_bin("bf-tools")?;
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
fn llvm_output_to_file() -> Result<(), Box<dyn std::error::Error>> {
    let out = assert_fs::NamedTempFile::new("out.ll")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
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
fn llvm_honors_size_and_non_wrapping_checks() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args([
        "compile",
        "--input", "tests/programs/print_no_loop.bf",
        "--size", "64",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("@tape = global [64 x i8] zeroinitializer"))
        .stdout(predicate::str::contains("oob:"));
    Ok(())
}

#[test]
fn llvm_wrapping_omits_oob_handler() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args([
        "compile",
        "--input", "tests/programs/print_no_loop.bf",
        "--wrapping", "--size", "64",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("@tape = global [64 x i8] zeroinitializer"))
        .stdout(predicate::str::contains("oob:").not());
    Ok(())
}

#[test]
fn llvm_sibling_loop_labels_are_unique() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("two-loops.bf")?;
    file.write_str("+[.-]+[.-]")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["compile", "--input", file.path().to_str().unwrap()]);
    let stdout = String::from_utf8(cmd.output()?.stdout)?;

    let check_count = stdout.matches("_check:").count();
    assert!(check_count >= 2, "expected at least 2 loop check labels, found {check_count}");
    Ok(())
}

#[test]
fn llvm_nested_loop_labels_are_unique() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("nested-loops.bf")?;
    file.write_str("+[>[.-]<-]")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["compile", "--input", file.path().to_str().unwrap()]);
    let stdout = String::from_utf8(cmd.output()?.stdout)?;

    let check_count = stdout.matches("_check:").count();
    assert!(check_count >= 2, "expected at least 2 loop check labels, found {check_count}");
    assert!(stdout.contains("entry:"), "expected entry: label");
    Ok(())
}

#[test]
fn llvm_dp_initialised_to_zero() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["compile", "--input", "tests/programs/print_no_loop.bf"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("store i64 0, ptr %dp"));
    Ok(())
}

#[test]
fn llvm_output_uses_putchar() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("dot.bf")?;
    file.write_str(".")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["compile", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("call i32 @putchar"));
    Ok(())
}

#[test]
fn llvm_input_uses_getchar() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("comma.bf")?;
    file.write_str(",")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["compile", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("call i32 @getchar"));
    Ok(())
}

#[test]
fn llvm_input_maps_eof_to_zero() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("comma.bf")?;
    file.write_str(",")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["compile", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("select i1").and(predicate::str::contains("i8 0")));
    Ok(())
}

#[test]
fn llvm_compiles_counted_moves_from_ir() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("moves.bf")?;
    file.write_str(">>>")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["compile", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("; move 3"));
    Ok(())
}

#[test]
fn llvm_compiles_clear_loops_as_clear_ir() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("clear.bf")?;
    file.write_str("[-]")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["compile", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("; clear"))
        .stdout(predicate::str::contains("entry:").and(predicate::str::contains("loop_0_check:").not()));
    Ok(())
}

// ── shared error handling ─────────────────────────────────────────────────────

#[test]
fn unmatched_open_bracket_fails() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("bad-open.bf")?;
    file.write_str("+[")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["compile", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unclosed"));
    Ok(())
}

#[test]
fn unmatched_close_bracket_fails() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("bad-close.bf")?;
    file.write_str("+]")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["compile", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unmatched"));
    Ok(())
}

#[test]
fn size_zero_fails_validation() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args([
        "compile",
        "--input", "tests/programs/print_no_loop.bf",
        "--size", "0",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Tape size must be greater than 0"));
    Ok(())
}

