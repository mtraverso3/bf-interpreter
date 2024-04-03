use std::fs::File;
use std::io::Read;
use std::process;

use clap::Parser;
use colored::Colorize;

mod interpreter;

/// A simple program to interpret Brainfuck programs.
#[derive(Parser)]
#[command(about, version, author)]
struct Cli {
    /// A Brainfuck program to interpret
    #[arg(short, long)]
    input: String,
    /// An optional file to output to. If not specified, the program uses stdout.
    #[arg(short, long)]
    output: Option<String>,
    /// Enables or disables wrapping of the data pointer.
    #[arg(short, long)]
    wrapping: bool,
    /// The size of the memory tape.
    #[arg(short, long, default_value_t = 30000)]
    size: usize,
    /// Enable debug information. This will print the memory tape at each instruction.
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let args: Cli = Cli::parse();

    // Try to open and read passed in file
    let mut file = File::open(args.input).unwrap_or_else(|err| {
        eprintln!("{}", format!("Error: Unable to open input file: {err}").red());
        process::exit(1);
    });

    let mut program = String::new();
    file.read_to_string(&mut program).unwrap_or_else(|err| {
        eprintln!("{}", format!("Error: Unable to read input file: {err}").red());
        process::exit(1);
    });

    //make program immutable
    let program = program;

    interpreter::interpret(program, args.output, args.wrapping, args.size, args.debug);
}
