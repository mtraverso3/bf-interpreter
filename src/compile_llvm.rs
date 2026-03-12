use std::io::Write;

use crate::io_utils::create_output_writer;
use crate::parser::Node;

pub fn compile_llvm(nodes: &[Node], output_path: Option<String>, tape_size: usize, wrapping: bool) {
    let mut out_writer = create_output_writer(output_path);

    let w = &mut *out_writer;

    // module level declarations
    writeln!(w, "; Brainfuck compiled to LLVM IR").unwrap();
    writeln!(w, "; Compile with: clang -O2 -o program out.ll").unwrap();
    writeln!(w).unwrap();
    writeln!(w, "declare i32 @putchar(i32)").unwrap();
    writeln!(w, "declare i32 @getchar()").unwrap();
    writeln!(w).unwrap();
    writeln!(w, "@tape = global [{tape_size} x i8] zeroinitializer").unwrap();
    writeln!(w).unwrap();

    // main entry point
    // %dp holds the current tape index as an i64 on the stack. all accesses
    // load/store through it so we never need phi nodes.
    writeln!(w, "define i32 @main() {{").unwrap();
    writeln!(w, "entry:").unwrap();
    writeln!(w, "  %dp = alloca i64, align 8").unwrap();
    writeln!(w, "  store i64 0, ptr %dp, align 8").unwrap();

    let mut counter: usize = 0;
    emit(nodes, w, &mut counter, tape_size, wrapping);

    writeln!(w, "  ret i32 0").unwrap();
    if !wrapping {
        writeln!(w, "oob:").unwrap();
        writeln!(w, "  ret i32 1").unwrap();
    }
    writeln!(w, "}}").unwrap();
}

// Returns the next unique SSA value id and advances the counter.
fn next(c: &mut usize) -> usize {
    let id = *c;
    *c += 1;
    id
}

fn emit(nodes: &[Node], out: &mut dyn Write, c: &mut usize, tape_size: usize, wrapping: bool) {
    for node in nodes {
        match node {
            Node::MoveRight => {
                let (t0, t1) = (next(c), next(c));
                writeln!(out, "  ; >").unwrap();
                writeln!(out, "  %t{t0} = load i64, ptr %dp, align 8").unwrap();
                if wrapping {
                    let (t2, t3) = (next(c), next(c));
                    writeln!(out, "  %t{t1} = add i64 %t{t0}, 1").unwrap();
                    writeln!(out, "  %t{t2} = icmp eq i64 %t{t1}, {tape_size}").unwrap();
                    writeln!(out, "  %t{t3} = select i1 %t{t2}, i64 0, i64 %t{t1}").unwrap();
                    writeln!(out, "  store i64 %t{t3}, ptr %dp, align 8").unwrap();
                } else {
                    let label = next(c);
                    writeln!(out, "  %t{t1} = icmp eq i64 %t{t0}, {}", tape_size - 1).unwrap();
                    writeln!(out, "  br i1 %t{t1}, label %oob, label %move_r_ok_{label}").unwrap();
                    writeln!(out, "move_r_ok_{label}:").unwrap();
                    let t2 = next(c);
                    writeln!(out, "  %t{t2} = add i64 %t{t0}, 1").unwrap();
                    writeln!(out, "  store i64 %t{t2}, ptr %dp, align 8").unwrap();
                }
            }

            Node::MoveLeft => {
                let (t0, t1) = (next(c), next(c));
                writeln!(out, "  ; <").unwrap();
                writeln!(out, "  %t{t0} = load i64, ptr %dp, align 8").unwrap();
                if wrapping {
                    let (t2, t3, t4) = (next(c), next(c), next(c));
                    writeln!(out, "  %t{t1} = icmp eq i64 %t{t0}, 0").unwrap();
                    writeln!(out, "  %t{t2} = sub i64 {tape_size}, 1").unwrap();
                    writeln!(out, "  %t{t3} = sub i64 %t{t0}, 1").unwrap();
                    writeln!(out, "  %t{t4} = select i1 %t{t1}, i64 %t{t2}, i64 %t{t3}").unwrap();
                    writeln!(out, "  store i64 %t{t4}, ptr %dp, align 8").unwrap();
                } else {
                    let label = next(c);
                    writeln!(out, "  %t{t1} = icmp eq i64 %t{t0}, 0").unwrap();
                    writeln!(out, "  br i1 %t{t1}, label %oob, label %move_l_ok_{label}").unwrap();
                    writeln!(out, "move_l_ok_{label}:").unwrap();
                    let t2 = next(c);
                    writeln!(out, "  %t{t2} = sub i64 %t{t0}, 1").unwrap();
                    writeln!(out, "  store i64 %t{t2}, ptr %dp, align 8").unwrap();
                }
            }

            Node::Increment => {
                let (t0, t1, t2, t3) = (next(c), next(c), next(c), next(c));
                writeln!(out, "  ; +").unwrap();
                writeln!(out, "  %t{t0} = load i64, ptr %dp, align 8").unwrap();
                writeln!(
                    out,
                    "  %t{t1} = getelementptr inbounds [{tape_size} x i8], ptr @tape, i64 0, i64 %t{t0}"
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
                    "  %t{t1} = getelementptr inbounds [{tape_size} x i8], ptr @tape, i64 0, i64 %t{t0}"
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
                    "  %t{t1} = getelementptr inbounds [{tape_size} x i8], ptr @tape, i64 0, i64 %t{t0}"
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
                    "  %t{t1} = getelementptr inbounds [{tape_size} x i8], ptr @tape, i64 0, i64 %t{t0}"
                )
                .unwrap();
                writeln!(out, "  %t{t2} = call i32 @getchar()").unwrap();
                let (t4, t5) = (next(c), next(c));
                writeln!(out, "  %t{t3} = icmp eq i32 %t{t2}, -1").unwrap();
                writeln!(out, "  %t{t4} = trunc i32 %t{t2} to i8").unwrap();
                writeln!(out, "  %t{t5} = select i1 %t{t3}, i8 0, i8 %t{t4}").unwrap();
                writeln!(out, "  store i8 %t{t5}, ptr %t{t1}").unwrap();
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
                    "  %t{t1} = getelementptr inbounds [{tape_size} x i8], ptr @tape, i64 0, i64 %t{t0}"
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
                emit(body, out, c, tape_size, wrapping);

                writeln!(out, "  ; ]").unwrap();
                writeln!(out, "  br label %loop_{lbl}_check").unwrap();

                writeln!(out, "loop_{lbl}_end:").unwrap();
            }
        }
    }
}
