use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn strips_non_instruction_characters() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("comments.bf")?;
    file.write_str("hello + world > [ - ] .")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["minify", "--input", file.path().to_str().unwrap()]);
    cmd.assert().success().stdout(predicate::eq("+>[-]."));
    Ok(())
}

#[test]
fn compress_alias_works() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("alias.bf")?;
    file.write_str("hello + world")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["compress", "--input", file.path().to_str().unwrap()]);
    cmd.assert().success().stdout(predicate::eq("+"));
    Ok(())
}

#[test]
fn reduces_redundant_plus_minus_runs() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("reduce.bf")?;
    file.write_str("+++-")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["minify", "--input", file.path().to_str().unwrap()]);
    cmd.assert().success().stdout(predicate::eq("++"));
    Ok(())
}

#[test]
fn chooses_shorter_direction_for_large_delta() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("large-delta.bf")?;
    file.write_str(&"+".repeat(200))?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["minify", "--input", file.path().to_str().unwrap()]);
    cmd.assert().success().stdout(predicate::eq("-".repeat(56)));
    Ok(())
}

#[test]
fn output_to_file() -> Result<(), Box<dyn std::error::Error>> {
    let input = assert_fs::NamedTempFile::new("input.bf")?;
    input.write_str("abc++--.xyz")?;

    let out = assert_fs::NamedTempFile::new("out.bf")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args([
        "minify",
        "--input",  input.path().to_str().unwrap(),
        "--output", out.path().to_str().unwrap(),
    ]);
    cmd.assert().stdout(predicate::str::is_empty()).success();

    out.assert(predicate::eq("."));
    Ok(())
}

#[test]
fn no_optimize_keeps_instruction_order() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("no-opt.bf")?;
    file.write_str("a+++-b")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["minify", "--input", file.path().to_str().unwrap(), "--no-optimize"]);
    cmd.assert().success().stdout(predicate::eq("+++-"));
    Ok(())
}

#[test]
fn selected_passes_are_applied() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("selected-pass.bf")?;
    file.write_str("a+++-b")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args([
        "minify",
        "--input", file.path().to_str().unwrap(),
        "--pass", "fold-add-sub",
    ]);
    cmd.assert().success().stdout(predicate::eq("++"));
    Ok(())
}

#[test]
fn unmatched_open_bracket_fails() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("bad-open-minify.bf")?;
    file.write_str("+[")?;

    let mut cmd = Command::cargo_bin("bf-tools")?;
    cmd.args(["minify", "--input", file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unclosed"));
    Ok(())
}

