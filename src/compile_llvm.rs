use std::fs::File;
use std::io::Write;
use std::{io, process};

use colored::Colorize;

use crate::parser::Node;

pub fn compile_llvm(nodes: &[Node], output_path: Option<String>) {
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

    let w = &mut *out_writer;

    // module level declarations
    writeln!(w, "; Brainfuck compiled to LLVM IR").unwrap();
    writeln!(w, "; Compile with: clang -O2 -o program out.ll").unwrap();
    writeln!(w).unwrap();
    writeln!(w, "declare i32 @putchar(i32)").unwrap();
    writeln!(w, "declare i32 @getchar()").unwrap();
    writeln!(w).unwrap();
    writeln!(w, "@tape = global [30000 x i8] zeroinitializer").unwrap();
    writeln!(w).unwrap();

    // main entry point
    // %dp holds the current tape index as an i64 on the stack. all accesses
    // load/store through it so we never need phi nodes.
    writeln!(w, "define i32 @main() {{").unwrap();
    writeln!(w, "entry:").unwrap();
    writeln!(w, "  %dp = alloca i64, align 8").unwrap();
    writeln!(w, "  store i64 0, ptr %dp, align 8").unwrap();

    let mut counter: usize = 0;
    emit(nodes, w, &mut counter);

    writeln!(w, "  ret i32 0").unwrap();
    writeln!(w, "}}").unwrap();
}

// Returns the next unique SSA value id and advances the counter.
fn next(c: &mut usize) -> usize {
    let id = *c;
    *c += 1;
    id
}

fn emit(nodes: &[Node], out: &mut dyn Write, c: &mut usize) {
    for node in nodes {
        match node {
            Node::MoveRight => {
                let (t0, t1) = (next(c), next(c));
                writeln!(out, "  ; >").unwrap();
                writeln!(out, "  %t{t0} = load i64, ptr %dp, align 8").unwrap();
                writeln!(out, "  %t{t1} = add i64 %t{t0}, 1").unwrap();
                writeln!(out, "  store i64 %t{t1}, ptr %dp, align 8").unwrap();
            }

            Node::MoveLeft => {
                let (t0, t1) = (next(c), next(c));
                writeln!(out, "  ; <").unwrap();
                writeln!(out, "  %t{t0} = load i64, ptr %dp, align 8").unwrap();
                writeln!(out, "  %t{t1} = sub i64 %t{t0}, 1").unwrap();
                writeln!(out, "  store i64 %t{t1}, ptr %dp, align 8").unwrap();
            }

            Node::Increment => {
                let (t0, t1, t2, t3) = (next(c), next(c), next(c), next(c));
                writeln!(out, "  ; +").unwrap();
                writeln!(out, "  %t{t0} = load i64, ptr %dp, align 8").unwrap();
                writeln!(
                    out,
                    "  %t{t1} = getelementptr inbounds [30000 x i8], ptr @tape, i64 0, i64 %t{t0}"
                )
                .unwrap();
                writeln!(out, "  %t{t2} = load i8, ptr %t{t1}").unwrap();
                writeln!(out, "  %t{t3} = add i8 %t{t2}, 1").unwrap();
                writeln!(out, "  store i8 %t{t3}, ptr %t{t1}").unwrap();
            }

            Node::Decrement => {
                let (t0, t1, t2, t3) = (next(c), next(c), next(c), next(c));
                writeln!(out, "  ; -").unwrap();
                writeln!(out, "  %t{t0} = load i64, ptr %dp, align 8").unwrap();
                writeln!(
                    out,
                    "  %t{t1} = getelementptr inbounds [30000 x i8], ptr @tape, i64 0, i64 %t{t0}"
                )
                .unwrap();
                writeln!(out, "  %t{t2} = load i8, ptr %t{t1}").unwrap();
                writeln!(out, "  %t{t3} = sub i8 %t{t2}, 1").unwrap();
                writeln!(out, "  store i8 %t{t3}, ptr %t{t1}").unwrap();
            }

            Node::Output => {
                let (t0, t1, t2, t3) = (next(c), next(c), next(c), next(c));
                writeln!(out, "  ; .").unwrap();
                writeln!(out, "  %t{t0} = load i64, ptr %dp, align 8").unwrap();
                writeln!(
                    out,
                    "  %t{t1} = getelementptr inbounds [30000 x i8], ptr @tape, i64 0, i64 %t{t0}"
                )
                .unwrap();
                writeln!(out, "  %t{t2} = load i8, ptr %t{t1}").unwrap();
                writeln!(out, "  %t{t3} = zext i8 %t{t2} to i32").unwrap();
                writeln!(out, "  call i32 @putchar(i32 %t{t3})").unwrap();
            }

            Node::Input => {
                let (t0, t1, t2, t3) = (next(c), next(c), next(c), next(c));
                writeln!(out, "  ; ,").unwrap();
                writeln!(out, "  %t{t0} = load i64, ptr %dp, align 8").unwrap();
                writeln!(
                    out,
                    "  %t{t1} = getelementptr inbounds [30000 x i8], ptr @tape, i64 0, i64 %t{t0}"
                )
                .unwrap();
                writeln!(out, "  %t{t2} = call i32 @getchar()").unwrap();
                writeln!(out, "  %t{t3} = trunc i32 %t{t2} to i8").unwrap();
                writeln!(out, "  store i8 %t{t3}, ptr %t{t1}").unwrap();
            }

            Node::Loop(body) => {
                let lbl = next(c);
                let (t0, t1, t2, t3) = (next(c), next(c), next(c), next(c));

                writeln!(out, "  ; [").unwrap();
                writeln!(out, "  br label %loop_{lbl}_check").unwrap();

                writeln!(out, "loop_{lbl}_check:").unwrap();
                writeln!(out, "  %t{t0} = load i64, ptr %dp, align 8").unwrap();
                writeln!(
                    out,
                    "  %t{t1} = getelementptr inbounds [30000 x i8], ptr @tape, i64 0, i64 %t{t0}"
                )
                .unwrap();
                writeln!(out, "  %t{t2} = load i8, ptr %t{t1}").unwrap();
                writeln!(out, "  %t{t3} = icmp ne i8 %t{t2}, 0").unwrap();
                writeln!(
                    out,
                    "  br i1 %t{t3}, label %loop_{lbl}_body, label %loop_{lbl}_end"
                )
                .unwrap();

                writeln!(out, "loop_{lbl}_body:").unwrap();
                emit(body, out, c);

                writeln!(out, "  ; ]").unwrap();
                writeln!(out, "  br label %loop_{lbl}_check").unwrap();

                writeln!(out, "loop_{lbl}_end:").unwrap();
            }
        }
    }
}
