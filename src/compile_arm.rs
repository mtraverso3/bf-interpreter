use std::{io, process};
use std::fs::File;
use std::io::Write;

use colored::Colorize;

pub fn compile_arm(program: String, output_path: Option<String>) {
    //Create output file if argument is present
    let mut out_writer = match output_path {
        None => Box::new(io::stdout()) as Box<dyn Write>,
        Some(output_path) => {
            let file = File::create(output_path).unwrap_or_else(|err| {
                eprintln!("{}", format!("Error: Unable to create output file: {err}").red());
                process::exit(1);
            });
            Box::new(file) as Box<dyn Write>
        }
    };

    // Write the ARM assembly bss for the memory tape
    writeln!(out_writer, ".bss").unwrap();
    writeln!(out_writer, "tape: .skip 30000").unwrap();

    // Write the ARM assembly text for the program
    writeln!(out_writer, ".text").unwrap();
    writeln!(out_writer, ".global _start").unwrap();
    writeln!(out_writer, "_start:").unwrap();
    writeln!(out_writer, "mov  x3, #0").unwrap();

    let mut jump_markers: Vec<usize> = Vec::new();
    let instructions: Vec<char> = program.chars().collect();

    let mut i = 0;
    loop {
        if i == instructions.len() {
            break;
        }

        let current_character = instructions[i];

        //print the current instruction as comment if not an ignored one:
        if matches!(current_character, '>' | '<' | '+' | '-' | '.' | ',' | '[' | ']') {
            writeln!(out_writer, "// {}", current_character).unwrap();
        }

        //we can use x3 as the data pointer
        match current_character {
            '>' => { // Move the pointer to the right
                writeln!(out_writer, "add  x3, x3, #1").unwrap();

                writeln!(out_writer).unwrap();
            }
            '<' => { // Move the pointer to the left
                writeln!(out_writer, "sub  x3, x3, #1").unwrap();

                writeln!(out_writer).unwrap();
            }
            '+' => { // Increment the memory cell at the pointer
                writeln!(out_writer, "ldr  x0, =tape").unwrap();
                writeln!(out_writer, "add  x0, x0, x3").unwrap();
                writeln!(out_writer, "ldrb w1, [x0]").unwrap();
                writeln!(out_writer, "add  w1, w1, #1").unwrap();
                writeln!(out_writer, "strb w1, [x0]").unwrap();

                writeln!(out_writer).unwrap();
            }
            '-' => { // Decrement the memory cell at the pointer
                writeln!(out_writer, "ldr  x0, =tape").unwrap();
                writeln!(out_writer, "add  x0, x0, x3").unwrap();
                writeln!(out_writer, "ldrb w1, [x0]").unwrap();
                writeln!(out_writer, "sub  w1, w1, #1").unwrap();
                writeln!(out_writer, "strb w1, [x0]").unwrap();

                writeln!(out_writer).unwrap();
            }
            '.' => { // Output the ASCII character at the pointer
                writeln!(out_writer, "ldr  x1, =tape").unwrap();
                writeln!(out_writer, "add  x1, x1, x3").unwrap();
                //the write syscall takes in a char* so we're good to go with the memory address

                // Write the character to stdout using the write syscall
                writeln!(out_writer, "mov  x8, #64").unwrap();
                writeln!(out_writer, "mov  x0, #1").unwrap();
                writeln!(out_writer, "mov  x2, #1").unwrap();
                writeln!(out_writer, "svc  #0").unwrap();

                writeln!(out_writer).unwrap();
            }
            ',' => { // Read a single ASCII character into the memory cell at the pointer
                writeln!(out_writer, "ldr  x1, =tape").unwrap();
                writeln!(out_writer, "add  x1, x1, x3").unwrap();

                // Read a single character from stdin using the read syscall
                writeln!(out_writer, "mov  x8, #63").unwrap();
                writeln!(out_writer, "mov  x0, #0").unwrap();
                writeln!(out_writer, "mov  x2, #1").unwrap();
                writeln!(out_writer, "svc  #0").unwrap();

                writeln!(out_writer).unwrap();
            }
            '[' => { // Jump forward to the command after the matching ] if the memory cell at the pointer is 0
                writeln!(out_writer, ".L{}_{}_start:", jump_markers.len(), i).unwrap();
                writeln!(out_writer, "ldr  x0, =tape").unwrap();
                writeln!(out_writer, "add  x0, x0, x3").unwrap();
                writeln!(out_writer, "ldrb w0, [x0]").unwrap();
                writeln!(out_writer, "cmp  w0, #0").unwrap();
                writeln!(out_writer, "beq  .L{}_{}_end", jump_markers.len(), i).unwrap();
                jump_markers.push(i);
                
                writeln!(out_writer).unwrap();
            }
            ']' => { // Jump back to the command after the matching [ if the memory cell at the pointer is nonzero
                writeln!(out_writer, "ldr  x0, =tape").unwrap();
                writeln!(out_writer, "add  x0, x0, x3").unwrap();
                writeln!(out_writer, "ldrb w0, [x0]").unwrap();
                writeln!(out_writer, "cmp  w0, #0").unwrap();
                let marker_pos = jump_markers.pop().unwrap_or_else(|| {
                    eprintln!("{}", format!("Error: Unmatched ']' at instruction {i}").red());
                    process::exit(1);
                });
                writeln!(out_writer, "bne  .L{}_{}_start", jump_markers.len(), marker_pos).unwrap();
                writeln!(out_writer, ".L{}_{}_end:", jump_markers.len(), marker_pos).unwrap();

                writeln!(out_writer).unwrap();
            }
            _ => {
                // Do nothing, as the character is a comment
            }
        }
        i += 1;
    }

    if let Some(marker_pos) = jump_markers.pop() {
        eprintln!("{}", format!("Error: Unmatched '[' at instruction {marker_pos}").red());
        process::exit(1);
    }

    // Write the ARM assembly text for the exit syscall
    writeln!(out_writer, "mov x0, #0").unwrap();
    writeln!(out_writer, "mov x8, #93").unwrap();
    writeln!(out_writer, "svc #0").unwrap();
}
