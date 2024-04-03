use std::{io, process};
use std::fs::File;
use std::io::Write;

use colored::Colorize;

pub fn interpret(program: String, output_path: Option<String>, wrapping: bool, size: usize, debug: bool) {
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


    let mut memory: Vec<u8> = vec![0; size];
    let mut data_pointer: usize = 0;
    let mut jump_markers: Vec<usize> = Vec::new();

    let mut i = 0;
    loop {
        if i == program.len() {
            break;
        }

        let current_character = program.chars().nth(i).unwrap();

        //print current state (ignore comments)
        if debug && (current_character == '>' || current_character == '<' || current_character == '+' || current_character == '-' || current_character == '.' || current_character == ',' || current_character == '[' || current_character == ']') {
            println!("Currently at: {} - {} -- {:?}", i, current_character, memory);
        }


        match current_character {
            '>' => { // Move the pointer to the right, wrapping if it is enabled
                data_pointer = if data_pointer == size - 1 {
                    if wrapping {
                        0
                    } else {
                        eprintln!("{}", format!("Error: Data pointer out of bounds (overflow) at position {i}").red());
                        process::exit(1);
                    }
                } else {
                    data_pointer + 1
                };
            }
            '<' => { // Move the pointer to the left, wrapping if it is enabled
                data_pointer = if data_pointer == 0 {
                    if wrapping {
                        size - 1
                    } else {
                        eprintln!("{}", format!("Error: Data pointer out of bounds (underflow) at position {i}").red());
                        process::exit(1);
                    }
                } else {
                    data_pointer - 1
                };
            }
            '+' => { // Increment the memory cell at the pointer, wrapping around on overflow
                memory[data_pointer] = memory[data_pointer].wrapping_add(1);
            }
            '-' => { // Decrement the memory cell at the pointer, wrapping around on underflow
                memory[data_pointer] = memory[data_pointer].wrapping_sub(1);
            }
            '.' => { // Output the character at the memory cell at the pointer
                let output = memory[data_pointer] as char;
                out_writer.write(&[output as u8]).unwrap_or_else(|err| {
                    eprintln!("{}", format!("Error: Unable to write to output file: {err}").red());
                    process::exit(1);
                });
            }
            ',' => { // Input a character and store it in the memory cell at the pointer
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap_or_else(|err| {
                    eprintln!("{}", format!("Error: Unable to read from stdin: {err}").red());
                    process::exit(1);
                });
                memory[data_pointer] = input.chars().next().unwrap() as u8;
            }
            '[' => {
                // Jump forward to the command after the matching ] if the memory cell at the pointer is 0
                if memory[data_pointer] == 0 {
                    let mut count = 1;
                    while count > 0 {
                        i += 1;
                        if program.chars().nth(i).unwrap() == '[' {
                            count += 1;
                        } else if program.chars().nth(i).unwrap() == ']' {
                            count -= 1;
                        }
                    }
                    i += 1;
                    continue;
                } else {
                    jump_markers.push(i + 1); //i+1 to jump to the command after the matching ]
                }
            }
            ']' => {
                // Jump back to the command after the matching [ if the memory cell at the pointer is nonzero
                if memory[data_pointer] != 0 {
                    i = jump_markers[jump_markers.len() - 1];
                    continue;
                } else {
                    jump_markers.pop();
                }
            }
            _ => {
                // Do nothing, as the character is a comment
            }
        }
        i += 1;
    }
}