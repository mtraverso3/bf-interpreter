use std::io::Write;

use crate::common::create_output_writer;
use crate::syntax::Node;

mod optimizer;

pub use optimizer::PassId;

pub enum OptimizeConfig {
    Default,
    None,
    Selected(Vec<PassId>),
}

pub fn compress(nodes: &[Node], output_path: Option<String>, config: OptimizeConfig) {
    let mut out_writer = create_output_writer(output_path);
    let optimized = optimize_ast(nodes, config);
    let compressed = emit_nodes(&optimized);

    out_writer
        .write_all(compressed.as_bytes())
        .unwrap_or_else(|err| {
            eprintln!("Error: Unable to write output: {err}");
            std::process::exit(1);
        });
}

pub fn optimize_ast(nodes: &[Node], config: OptimizeConfig) -> Vec<Node> {
    match config {
        OptimizeConfig::Default => optimizer::optimize_default(nodes),
        OptimizeConfig::None => nodes.to_vec(),
        OptimizeConfig::Selected(passes) => optimizer::optimize_with_passes(nodes, &passes),
    }
}

fn emit_nodes(nodes: &[Node]) -> String {
    let mut out = String::new();

    for node in nodes {
        match node {
            Node::MoveRight => out.push('>'),
            Node::MoveLeft => out.push('<'),
            Node::Increment => out.push('+'),
            Node::Decrement => out.push('-'),
            Node::Output => out.push('.'),
            Node::Input => out.push(','),
            Node::Loop(body) => {
                out.push('[');
                out.push_str(&emit_nodes(body));
                out.push(']');
            }
        }
    }

    out
}


