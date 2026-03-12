use std::io::Write;

use crate::common::create_output_writer;
use crate::ir::Instr;

pub fn compile_arm(
    program: &[Instr],
    output_path: Option<String>,
    tape_size: usize,
    wrapping: bool,
) {
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

    // Emit instructions; label counter is threaded through to keep labels unique.
    let mut label_counter: usize = 0;
    emit(
        program,
        &mut *out_writer,
        &mut label_counter,
        tape_size,
        wrapping,
    );

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

fn emit(
    program: &[Instr],
    out: &mut dyn Write,
    counter: &mut usize,
    tape_size: usize,
    wrapping: bool,
) {
    for instr in program {
        match instr {
            Instr::Move(delta) => emit_move(*delta, out, counter, tape_size, wrapping),
            Instr::Add(delta) => emit_add(*delta, out),
            Instr::Output => {
                writeln!(out, "// .").unwrap();
                writeln!(out, "ldr  x1, =tape").unwrap();
                writeln!(out, "add  x1, x1, x3").unwrap();
                writeln!(out, "mov  x8, #64").unwrap();
                writeln!(out, "mov  x0, #1").unwrap();
                writeln!(out, "mov  x2, #1").unwrap();
                writeln!(out, "svc  #0").unwrap();
                writeln!(out).unwrap();
            }
            Instr::Input => {
                writeln!(out, "// ,").unwrap();
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
            Instr::Clear => emit_clear(out),
            Instr::Loop(body) => {
                let label = *counter;
                *counter += 1;

                writeln!(out, "// loop start").unwrap();
                writeln!(out, ".L{label}_start:").unwrap();
                writeln!(out, "ldr  x0, =tape").unwrap();
                writeln!(out, "add  x0, x0, x3").unwrap();
                writeln!(out, "ldrb w0, [x0]").unwrap();
                writeln!(out, "cmp  w0, #0").unwrap();
                writeln!(out, "beq  .L{label}_end").unwrap();
                writeln!(out).unwrap();

                emit(body, out, counter, tape_size, wrapping);

                writeln!(out, "// loop end").unwrap();
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

fn emit_move(
    delta: i64,
    out: &mut dyn Write,
    counter: &mut usize,
    tape_size: usize,
    wrapping: bool,
) {
    if delta == 0 {
        return;
    }

    writeln!(out, "// move {delta}").unwrap();

    if wrapping {
        let shift = delta.rem_euclid(tape_size as i64);
        if shift == 0 {
            return;
        }

        let label = *counter;
        *counter += 1;
        writeln!(out, "ldr  x4, ={shift}").unwrap();
        writeln!(out, "ldr  x5, ={tape_size}").unwrap();
        writeln!(out, "add  x3, x3, x4").unwrap();
        writeln!(out, "cmp  x3, x5").unwrap();
        writeln!(out, "blo  .Lmove_done_{label}").unwrap();
        writeln!(out, "sub  x3, x3, x5").unwrap();
        writeln!(out, ".Lmove_done_{label}:").unwrap();
        writeln!(out).unwrap();
        return;
    }

    if delta > 0 {
        let amount = delta as usize;
        if amount >= tape_size {
            let label = *counter;
            *counter += 1;
            writeln!(out, "b    .L_oob").unwrap();
            writeln!(out, ".Lmove_after_oob_{label}:").unwrap();
            writeln!(out).unwrap();
            return;
        }

        writeln!(out, "ldr  x4, ={}", tape_size - amount).unwrap();
        writeln!(out, "cmp  x3, x4").unwrap();
        writeln!(out, "bhs  .L_oob").unwrap();
        writeln!(out, "ldr  x4, ={amount}").unwrap();
        writeln!(out, "add  x3, x3, x4").unwrap();
        writeln!(out).unwrap();
    } else {
        let amount = delta.unsigned_abs() as usize;
        if amount >= tape_size {
            let label = *counter;
            *counter += 1;
            writeln!(out, "b    .L_oob").unwrap();
            writeln!(out, ".Lmove_after_oob_{label}:").unwrap();
            writeln!(out).unwrap();
            return;
        }

        writeln!(out, "ldr  x4, ={amount}").unwrap();
        writeln!(out, "cmp  x3, x4").unwrap();
        writeln!(out, "blo  .L_oob").unwrap();
        writeln!(out, "sub  x3, x3, x4").unwrap();
        writeln!(out).unwrap();
    }
}

fn emit_add(delta: i16, out: &mut dyn Write) {
    let normalized = i32::from(delta).rem_euclid(256);
    if normalized == 0 {
        return;
    }

    let (op, amount) = if normalized <= 128 {
        ("add", normalized)
    } else {
        ("sub", 256 - normalized)
    };

    writeln!(out, "// {op} {amount}").unwrap();
    writeln!(out, "ldr  x0, =tape").unwrap();
    writeln!(out, "add  x0, x0, x3").unwrap();
    writeln!(out, "ldrb w1, [x0]").unwrap();
    writeln!(out, "{op}  w1, w1, #{amount}").unwrap();
    writeln!(out, "strb w1, [x0]").unwrap();
    writeln!(out).unwrap();
}

fn emit_clear(out: &mut dyn Write) {
    writeln!(out, "// clear").unwrap();
    writeln!(out, "ldr  x0, =tape").unwrap();
    writeln!(out, "add  x0, x0, x3").unwrap();
    writeln!(out, "mov  w1, #0").unwrap();
    writeln!(out, "strb w1, [x0]").unwrap();
    writeln!(out).unwrap();
}
