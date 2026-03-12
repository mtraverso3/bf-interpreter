use std::fs::File;
use std::io::Write;
use std::{io, process};

use colored::Colorize;

use crate::parser::Node;

pub fn compile_arm(nodes: &[Node], output_path: Option<String>) {
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

    // BSS section — memory tape
    writeln!(out_writer, ".bss").unwrap();
    writeln!(out_writer, "tape: .skip 30000").unwrap();

    // Text section — entry point
    writeln!(out_writer, ".text").unwrap();
    writeln!(out_writer, ".global _start").unwrap();
    writeln!(out_writer, "_start:").unwrap();
    writeln!(out_writer, "mov  x3, #0").unwrap();
    writeln!(out_writer).unwrap();

    // Emit instructions; label counter is threaded through to keep labels unique
    let mut label_counter: usize = 0;
    emit(nodes, &mut *out_writer, &mut label_counter);

    // Exit syscall
    writeln!(out_writer, "mov x0, #0").unwrap();
    writeln!(out_writer, "mov x8, #93").unwrap();
    writeln!(out_writer, "svc #0").unwrap();
}

fn emit(nodes: &[Node], out: &mut dyn Write, counter: &mut usize) {
    for node in nodes {
        match node {
            Node::MoveRight => {
                writeln!(out, "// {}", node.symbol()).unwrap();
                writeln!(out, "add  x3, x3, #1").unwrap();
                writeln!(out).unwrap();
            }
            Node::MoveLeft => {
                writeln!(out, "// {}", node.symbol()).unwrap();
                writeln!(out, "sub  x3, x3, #1").unwrap();
                writeln!(out).unwrap();
            }
            Node::Increment => {
                writeln!(out, "// {}", node.symbol()).unwrap();
                writeln!(out, "ldr  x0, =tape").unwrap();
                writeln!(out, "add  x0, x0, x3").unwrap();
                writeln!(out, "ldrb w1, [x0]").unwrap();
                writeln!(out, "add  w1, w1, #1").unwrap();
                writeln!(out, "strb w1, [x0]").unwrap();
                writeln!(out).unwrap();
            }
            Node::Decrement => {
                writeln!(out, "// {}", node.symbol()).unwrap();
                writeln!(out, "ldr  x0, =tape").unwrap();
                writeln!(out, "add  x0, x0, x3").unwrap();
                writeln!(out, "ldrb w1, [x0]").unwrap();
                writeln!(out, "sub  w1, w1, #1").unwrap();
                writeln!(out, "strb w1, [x0]").unwrap();
                writeln!(out).unwrap();
            }
            Node::Output => {
                writeln!(out, "// {}", node.symbol()).unwrap();
                writeln!(out, "ldr  x1, =tape").unwrap();
                writeln!(out, "add  x1, x1, x3").unwrap();
                writeln!(out, "mov  x8, #64").unwrap();
                writeln!(out, "mov  x0, #1").unwrap();
                writeln!(out, "mov  x2, #1").unwrap();
                writeln!(out, "svc  #0").unwrap();
                writeln!(out).unwrap();
            }
            Node::Input => {
                writeln!(out, "// {}", node.symbol()).unwrap();
                writeln!(out, "ldr  x1, =tape").unwrap();
                writeln!(out, "add  x1, x1, x3").unwrap();
                writeln!(out, "mov  x8, #63").unwrap();
                writeln!(out, "mov  x0, #0").unwrap();
                writeln!(out, "mov  x2, #1").unwrap();
                writeln!(out, "svc  #0").unwrap();
                writeln!(out).unwrap();
            }
            Node::Loop(body) => {
                let label = *counter;
                *counter += 1;

                // Loop header — skip body if current cell is zero
                writeln!(out, "// [").unwrap();
                writeln!(out, ".L{label}_start:").unwrap();
                writeln!(out, "ldr  x0, =tape").unwrap();
                writeln!(out, "add  x0, x0, x3").unwrap();
                writeln!(out, "ldrb w0, [x0]").unwrap();
                writeln!(out, "cmp  w0, #0").unwrap();
                writeln!(out, "beq  .L{label}_end").unwrap();
                writeln!(out).unwrap();

                // Loop body
                emit(body, out, counter);

                // Loop footer — jump back if current cell is still nonzero
                writeln!(out, "// ]").unwrap();
                writeln!(out, "ldr  x0, =tape").unwrap();
                writeln!(out, "add  x0, x0, x3").unwrap();
                writeln!(out, "ldrb w0, [x0]").unwrap();
                writeln!(out, "cmp  w0, #0").unwrap();
                writeln!(out, "bne  .L{label}_start").unwrap();
                writeln!(out, ".L{label}_end:").unwrap();
                writeln!(out).unwrap();
            }
        }
    }
}
