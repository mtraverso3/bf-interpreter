use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use assert_fs::prelude::*;

#[test]
fn test_hello_world() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.arg("--input").arg("tests/programs/hello_world.b");

    cmd.assert().stdout(predicate::eq("Hello World!\n")).success();
    Ok(())
}

#[test]
fn test_print_no_loop() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.arg("--input").arg("tests/programs/print_no_loop.b");

    cmd.assert().stdout(predicate::eq("A")).success();
    Ok(())
}

#[test]
fn test_wrapping_off() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("wrapping-off.b")?;
    file.write_str("<")?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.arg("--input").arg(file.path());

    cmd.assert().failure();
    Ok(())
}

#[test]
fn test_wrapping_on() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("wrapping-on.b")?;

    // set a cell to a value and print it by wrapping back around to it
    let a_sum = String::from_utf8(vec![b'+'; 65]);
    let program = format!(">>{:?} << <.", a_sum);
    file.write_str(&program)?;

    let mut cmd = Command::cargo_bin("brainfuck-interpreter")?;
    cmd.arg("--input").arg(file.path()).arg("--wrapping").arg("--size").arg("3");

    cmd.assert().stdout(predicate::eq("A")).success();
    Ok(())
}