use std::fs::File;
use std::io::Write;
use std::{io, process};

use colored::Colorize;

pub fn create_output_writer(output_path: Option<String>) -> Box<dyn Write> {
    match output_path {
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
    }
}


