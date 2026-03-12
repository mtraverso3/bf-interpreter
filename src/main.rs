use std::fs::File;
use std::io::Read;
use std::process;

use clap::{Parser, Subcommand};
use colored::Colorize;

mod codegen;
mod common;
mod minify;
mod runtime;
mod syntax;

/// Compilation output target.
#[derive(clap::ValueEnum, Clone, Debug)]
enum Target {
    /// LLVM IR (.ll) — compile with: clang -O2 -o program out.ll
    Llvm,
    /// AArch64 Linux assembly (.s) — assemble with: as out.s -o out.o && ld out.o -o program
    Arm,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum OptimizePass {
    /// Fold contiguous +/- runs modulo 256 and keep the shorter direction.
    FoldAddSub,
    /// Canonicalize `[+]` and `[-]` style zeroing loops to `[-]`.
    CanonicalizeClearLoops,
    /// Remove loops that are provably dead because the current cell is known to be zero.
    RemoveKnownZeroLoops,
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

        /// Enable wrapping of the data pointer at tape boundaries.
        #[arg(short, long)]
        wrapping: bool,

        /// Size of the memory tape in cells.
        #[arg(short, long, default_value_t = 30000)]
        size: usize,

        /// Output target format.
        #[arg(short, long, value_enum, default_value = "llvm")]
        target: Target,
    },

    /// Minify a Brainfuck source program by removing non-instruction characters
    /// and applying selected AST optimization passes.
    #[command(alias = "compress")]
    Minify {
        /// Path to the Brainfuck source file.
        #[arg(short, long)]
        input: String,

        /// Optional file to write the minified output to. Defaults to stdout.
        #[arg(short, long)]
        output: Option<String>,

        /// Disable all optimization passes (still strips comments/whitespace).
        #[arg(long, conflicts_with = "pass")]
        no_optimize: bool,

        /// Optimization passes to run. Repeat this flag to run multiple passes.
        #[arg(long = "pass", value_enum)]
        pass: Vec<OptimizePass>,
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
            validate_size(size);
            let source = read_program(&input);
            let nodes = parse(&source);
            runtime::interpret(&nodes, output, wrapping, size, debug);
        }
        Command::Compile {
            input,
            output,
            wrapping,
            size,
            target,
        } => {
            validate_size(size);
            let source = read_program(&input);
            let nodes = parse(&source);
            match target {
                Target::Llvm => codegen::llvm::compile_llvm(&nodes, output, size, wrapping),
                Target::Arm => codegen::arm::compile_arm(&nodes, output, size, wrapping),
            }
        }
        Command::Minify {
            input,
            output,
            no_optimize,
            pass,
        } => {
            let source = read_program(&input);
            let nodes = parse(&source);

            let config = if no_optimize {
                minify::OptimizeConfig::None
            } else if pass.is_empty() {
                minify::OptimizeConfig::Default
            } else {
                let selected = pass
                    .into_iter()
                    .map(|p| match p {
                        OptimizePass::FoldAddSub => minify::PassId::FoldAddSub,
                        OptimizePass::CanonicalizeClearLoops => {
                            minify::PassId::CanonicalizeClearLoops
                        }
                        OptimizePass::RemoveKnownZeroLoops => {
                            minify::PassId::RemoveKnownZeroLoops
                        }
                    })
                    .collect();
                minify::OptimizeConfig::Selected(selected)
            };

            minify::compress(&nodes, output, config);
        }
    }
}

fn validate_size(size: usize) {
    if size == 0 {
        eprintln!("{}", "Error: Tape size must be greater than 0".red());
        process::exit(1);
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

fn parse(source: &str) -> Vec<syntax::Node> {
    syntax::parse(source).unwrap_or_else(|err| {
        eprintln!("{}", format!("Error: {err}").red());
        process::exit(1);
    })
}
