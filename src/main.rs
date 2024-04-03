use std::fs::File;
use std::io;
use std::io::{Read, Write};
use clap::Parser;



/// A simple program to interpret Brainfuck programs.
#[derive(Parser)]
#[command(about, version)]
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
}
