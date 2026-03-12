use std::io::Write;

use crate::common::create_output_writer;
use crate::syntax::Node;

pub fn compile_arm(nodes: &[Node], output_path: Option<String>, tape_size: usize, wrapping: bool) {
    let mut out_writer = create_output_writer(output_path);

    // BSS section — memory tape
    writeln!(out_writer, ".bss").unwrap();
    writeln!(out_writer, "tape: .skip {tape_size}").unwrap();

    // Text section — entry point
    writeln!(out_writer, ".text").unwrap();
    writeln!(out_writer, ".global _start").unwrap();
    writeln!(out_writer, "_start:").unwrap();
    writeln!(out_writer, "mov  x3, #0").unwrap();
    writeln!(out_writer).unwrap();

    // Emit instructions; label counter is threaded through to keep labels unique
    let mut label_counter: usize = 0;
    emit(nodes, &mut *out_writer, &mut label_counter, tape_size, wrapping);

    if !wrapping {
        writeln!(out_writer, ".L_oob:").unwrap();
        writeln!(out_writer, "mov x0, #1").unwrap();
        writeln!(out_writer, "mov x8, #93").unwrap();
        writeln!(out_writer, "svc #0").unwrap();
        writeln!(out_writer).unwrap();
    }

    // Exit syscall
    writeln!(out_writer, "mov x0, #0").unwrap();
    writeln!(out_writer, "mov x8, #93").unwrap();
    writeln!(out_writer, "svc #0").unwrap();
}

fn emit(nodes: &[Node], out: &mut dyn Write, counter: &mut usize, tape_size: usize, wrapping: bool) {
    for node in nodes {
        match node {
            Node::MoveRight => {
                writeln!(out, "// {}", node.symbol()).unwrap();
                if wrapping {
                    let label = *counter;
                    *counter += 1;
                    writeln!(out, "ldr  x4, ={}", tape_size - 1).unwrap();
                    writeln!(out, "cmp  x3, x4").unwrap();
                    writeln!(out, "beq  .Lwrap_r_{label}").unwrap();
                    writeln!(out, "add  x3, x3, #1").unwrap();
                    writeln!(out, "b    .Lwrap_r_done_{label}").unwrap();
                    writeln!(out, ".Lwrap_r_{label}:").unwrap();
                    writeln!(out, "mov  x3, #0").unwrap();
                    writeln!(out, ".Lwrap_r_done_{label}:").unwrap();
                } else {
                    writeln!(out, "ldr  x4, ={}", tape_size - 1).unwrap();
                    writeln!(out, "cmp  x3, x4").unwrap();
                    writeln!(out, "beq  .L_oob").unwrap();
                    writeln!(out, "add  x3, x3, #1").unwrap();
                }
                writeln!(out).unwrap();
            }
            Node::MoveLeft => {
                writeln!(out, "// {}", node.symbol()).unwrap();
                if wrapping {
                    let label = *counter;
                    *counter += 1;
                    writeln!(out, "cmp  x3, #0").unwrap();
                    writeln!(out, "beq  .Lwrap_l_{label}").unwrap();
                    writeln!(out, "sub  x3, x3, #1").unwrap();
                    writeln!(out, "b    .Lwrap_l_done_{label}").unwrap();
                    writeln!(out, ".Lwrap_l_{label}:").unwrap();
                    writeln!(out, "ldr  x4, ={}", tape_size - 1).unwrap();
                    writeln!(out, "mov  x3, x4").unwrap();
                    writeln!(out, ".Lwrap_l_done_{label}:").unwrap();
                } else {
                    writeln!(out, "cmp  x3, #0").unwrap();
                    writeln!(out, "beq  .L_oob").unwrap();
                    writeln!(out, "sub  x3, x3, #1").unwrap();
                }
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
                let label = *counter;
                *counter += 1;
                writeln!(out, "ldr  x1, =tape").unwrap();
                writeln!(out, "add  x1, x1, x3").unwrap();
                writeln!(out, "mov  x8, #63").unwrap();
                writeln!(out, "mov  x0, #0").unwrap();
                writeln!(out, "mov  x2, #1").unwrap();
                writeln!(out, "svc  #0").unwrap();
                writeln!(out, "cmp  x0, #0").unwrap();
                writeln!(out, "bne  .Lin_ok_{label}").unwrap();
                writeln!(out, "mov  w5, #0").unwrap();
                writeln!(out, "strb w5, [x1]").unwrap();
                writeln!(out, ".Lin_ok_{label}:").unwrap();
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
                emit(body, out, counter, tape_size, wrapping);

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

