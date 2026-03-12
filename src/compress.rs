use std::io::Write;

use crate::io_utils::create_output_writer;
use crate::parser::Node;

pub fn compress(nodes: &[Node], output_path: Option<String>) {
    let mut out_writer = create_output_writer(output_path);
    let compressed = emit_nodes(nodes);

    out_writer
        .write_all(compressed.as_bytes())
        .unwrap_or_else(|err| {
            eprintln!("Error: Unable to write output: {err}");
            std::process::exit(1);
        });
}

fn emit_nodes(nodes: &[Node]) -> String {
    let mut out = String::new();
    let mut delta: i32 = 0;

    for node in nodes {
        match node {
            Node::Increment => delta += 1,
            Node::Decrement => delta -= 1,
            _ => {
                flush_delta(&mut out, delta);
                delta = 0;

                match node {
                    Node::MoveRight => out.push('>'),
                    Node::MoveLeft => out.push('<'),
                    Node::Output => out.push('.'),
                    Node::Input => out.push(','),
                    Node::Loop(body) => {
                        out.push('[');
                        out.push_str(&emit_nodes(body));
                        out.push(']');
                    }
                    Node::Increment | Node::Decrement => unreachable!(),
                }
            }
        }
    }

    flush_delta(&mut out, delta);
    out
}

fn flush_delta(out: &mut String, delta: i32) {
    // Cell math is on u8 values, so +/- runs can be reduced modulo 256.
    let normalized = delta.rem_euclid(256) as usize;

    if normalized == 0 {
        return;
    }

    let plus_count = normalized;
    let minus_count = 256 - normalized;

    if plus_count <= minus_count {
        out.push_str(&"+".repeat(plus_count));
    } else {
        out.push_str(&"-".repeat(minus_count));
    }
}

