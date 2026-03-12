use std::fs::File;
use std::io::Read;
use std::process;

use clap::{Parser, Subcommand};
use colored::Colorize;

mod compile_arm;
mod compile_llvm;
mod interpreter;
mod parser;

/// Compilation output target.
#[derive(clap::ValueEnum, Clone, Debug)]
enum Target {
    /// LLVM IR (.ll) — compile with: clang -O2 -o program out.ll
    Llvm,
    /// AArch64 Linux assembly (.s) — assemble with: as out.s -o out.o && ld out.o -o program
    Arm,
}

/// A Brainfuck interpreter and AArch64 / LLVM IR compiler.
#[derive(Parser)]
#[command(about, version, author)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Interpret a Brainfuck program directly.
    Interpret {
        /// Path to the Brainfuck source file.
        #[arg(short, long)]
        input: String,

        /// Optional file to write output to. Defaults to stdout.
        #[arg(short, long)]
        output: Option<String>,

        /// Enable wrapping of the data pointer at tape boundaries.
        #[arg(short, long)]
        wrapping: bool,

        /// Size of the memory tape in cells.
        #[arg(short, long, default_value_t = 30000)]
        size: usize,

        /// Print the memory tape state at each instruction (debug mode).
        #[arg(short, long)]
        debug: bool,
    },

    /// Compile a Brainfuck program to the chosen target.
    Compile {
        /// Path to the Brainfuck source file.
        #[arg(short, long)]
        input: String,

        /// Optional file to write the compiled output to. Defaults to stdout.
        #[arg(short, long)]
        output: Option<String>,

        /// Output target format.
        #[arg(short, long, value_enum, default_value = "llvm")]
        target: Target,
    },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Command::Interpret {
            input,
            output,
            wrapping,
            size,
            debug,
        } => {
            let source = read_program(&input);
            let nodes = parse(&source);
            interpreter::interpret(&nodes, output, wrapping, size, debug);
        }
        Command::Compile {
            input,
            output,
            target,
        } => {
            let source = read_program(&input);
            let nodes = parse(&source);
            match target {
                Target::Llvm => compile_llvm::compile_llvm(&nodes, output),
                Target::Arm => compile_arm::compile_arm(&nodes, output),
            }
        }
    }
}

fn read_program(path: &str) -> String {
    let mut file = File::open(path).unwrap_or_else(|err| {
        eprintln!(
            "{}",
            format!("Error: Unable to open input file: {err}").red()
        );
        process::exit(1);
    });

    let mut program = String::new();
    file.read_to_string(&mut program).unwrap_or_else(|err| {
        eprintln!(
            "{}",
            format!("Error: Unable to read input file: {err}").red()
        );
        process::exit(1);
    });

    program
}

fn parse(source: &str) -> Vec<parser::Node> {
    parser::parse(source).unwrap_or_else(|err| {
        eprintln!("{}", format!("Error: {err}").red());
        process::exit(1);
    })
}
