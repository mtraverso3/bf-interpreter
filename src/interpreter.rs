use std::fs::File;
use std::io::Write;
use std::{io, process};

use colored::Colorize;

use crate::parser::Node;

pub fn interpret(
    nodes: &[Node],
    output_path: Option<String>,
    wrapping: bool,
    size: usize,
    debug: bool,
) {
    let mut out_writer: Box<dyn Write> = match output_path {
        None => Box::new(io::stdout()),
        Some(path) => {
            let file = File::create(path).unwrap_or_else(|err| {
                eprintln!(
                    "{}",
                    format!("Error: Unable to create output file: {err}").red()
                );
                process::exit(1);
            });
            Box::new(file)
        }
    };

    let mut memory: Vec<u8> = vec![0; size];
    let mut dp: usize = 0;

    run(
        nodes,
        &mut memory,
        &mut dp,
        &mut *out_writer,
        wrapping,
        size,
        debug,
    );
}

fn run(
    nodes: &[Node],
    memory: &mut Vec<u8>,
    dp: &mut usize,
    out: &mut dyn Write,
    wrapping: bool,
    size: usize,
    debug: bool,
) {
    for node in nodes {
        if debug {
            println!(
                "Node: '{}', dp: {}, cell: {}, memory: {:?}",
                node.symbol(),
                dp,
                memory[*dp],
                memory
            );
        }

        match node {
            Node::MoveRight => {
                *dp = if *dp == size - 1 {
                    if wrapping {
                        0
                    } else {
                        eprintln!("{}", "Error: Data pointer out of bounds (overflow)".red());
                        process::exit(1);
                    }
                } else {
                    *dp + 1
                };
            }
            Node::MoveLeft => {
                *dp = if *dp == 0 {
                    if wrapping {
                        size - 1
                    } else {
                        eprintln!("{}", "Error: Data pointer out of bounds (underflow)".red());
                        process::exit(1);
                    }
                } else {
                    *dp - 1
                };
            }
            Node::Increment => {
                memory[*dp] = memory[*dp].wrapping_add(1);
            }
            Node::Decrement => {
                memory[*dp] = memory[*dp].wrapping_sub(1);
            }
            Node::Output => {
                out.write_all(&[memory[*dp]]).unwrap_or_else(|err| {
                    eprintln!("{}", format!("Error: Unable to write output: {err}").red());
                    process::exit(1);
                });
            }
            Node::Input => {
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap_or_else(|err| {
                    eprintln!(
                        "{}",
                        format!("Error: Unable to read from stdin: {err}").red()
                    );
                    process::exit(1);
                });
                memory[*dp] = input.bytes().next().unwrap_or(0);
            }
            Node::Loop(body) => {
                while memory[*dp] != 0 {
                    run(body, memory, dp, out, wrapping, size, debug);
                }
            }
        }
    }
}
