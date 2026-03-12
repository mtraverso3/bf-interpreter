use std::io::Write;

use crate::common::create_output_writer;
use crate::ir;
use crate::syntax::Node;

pub use crate::ir::PassId;

pub enum OptimizeConfig {
    Default,
    None,
    Selected(Vec<PassId>),
}

pub fn compress(nodes: &[Node], output_path: Option<String>, config: OptimizeConfig) {
    let mut out_writer = create_output_writer(output_path);
    let program = ir::lower(nodes);
    let optimized = optimize_program(&program, config);
    let compressed = ir::emit_brainfuck(&optimized);

    out_writer
        .write_all(compressed.as_bytes())
        .unwrap_or_else(|err| {
            eprintln!("Error: Unable to write output: {err}");
            std::process::exit(1);
        });
}

pub fn optimize_program(program: &[ir::Instr], config: OptimizeConfig) -> Vec<ir::Instr> {
    match config {
        OptimizeConfig::Default => ir::optimize_default(program),
        OptimizeConfig::None => program.to_vec(),
        OptimizeConfig::Selected(passes) => ir::optimize_with_passes(program, &passes),
    }
}
