use crate::syntax::Node;

mod optimizer;

pub use optimizer::PassId;

pub type Program = Vec<Instr>;

/// A richer Brainfuck IR that can represent the original program and optimized forms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instr {
    /// Move the data pointer by the signed delta.
    Move(i64),
    /// Add the signed delta to the current cell (modulo 256 at runtime).
    Add(i16),
    Output,
    Input,
    Loop(Program),
    /// Set the current cell to zero.
    Clear,
    /// Distribute the current cell value to relative offsets, then clear current cell.
    ///
    /// Each pair is (offset, factor). For example, `[->++<]` lowers to `Transfer([(1, 2)])`.
    Transfer(Vec<(i64, i16)>),
}

/// Lower the parsed Brainfuck AST into the richer IR without changing semantics.
pub fn lower(nodes: &[Node]) -> Program {
    nodes.iter().map(lower_node).collect()
}

pub fn optimize_default(program: &[Instr]) -> Program {
    optimizer::optimize_default(program)
}

pub fn optimize_with_passes(program: &[Instr], passes: &[PassId]) -> Program {
    optimizer::optimize_with_passes(program, passes)
}

/// Emit Brainfuck source from IR.
pub fn emit_brainfuck(program: &[Instr]) -> String {
    let mut out = String::new();
    emit_into(program, &mut out);
    out
}

fn lower_node(node: &Node) -> Instr {
    match node {
        Node::MoveRight => Instr::Move(1),
        Node::MoveLeft => Instr::Move(-1),
        Node::Increment => Instr::Add(1),
        Node::Decrement => Instr::Add(-1),
        Node::Output => Instr::Output,
        Node::Input => Instr::Input,
        Node::Loop(body) => Instr::Loop(lower(body)),
    }
}

fn emit_into(program: &[Instr], out: &mut String) {
    for instr in program {
        match instr {
            Instr::Move(delta) => {
                if *delta > 0 {
                    out.extend(std::iter::repeat_n('>', *delta as usize));
                } else if *delta < 0 {
                    out.extend(std::iter::repeat_n('<', delta.unsigned_abs() as usize));
                }
            }
            Instr::Add(delta) => emit_add(*delta, out),
            Instr::Output => out.push('.'),
            Instr::Input => out.push(','),
            Instr::Loop(body) => {
                out.push('[');
                emit_into(body, out);
                out.push(']');
            }
            Instr::Clear => out.push_str("[-]"),
            Instr::Transfer(targets) => emit_transfer(targets, out),
        }
    }
}

fn emit_transfer(targets: &[(i64, i16)], out: &mut String) {
    out.push('[');
    out.push('-');

    for (offset, factor) in targets {
        emit_move(*offset, out);
        emit_add(*factor, out);
        emit_move(-*offset, out);
    }

    out.push(']');
}

fn emit_move(delta: i64, out: &mut String) {
    if delta > 0 {
        out.extend(std::iter::repeat_n('>', delta as usize));
    } else if delta < 0 {
        out.extend(std::iter::repeat_n('<', delta.unsigned_abs() as usize));
    }
}

fn emit_add(delta: i16, out: &mut String) {
    let normalized = (delta as i32).rem_euclid(256) as usize;
    if normalized == 0 {
        return;
    }

    let plus_count = normalized;
    let minus_count = 256 - normalized;

    if plus_count <= minus_count {
        out.extend(std::iter::repeat_n('+', plus_count));
    } else {
        out.extend(std::iter::repeat_n('-', minus_count));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowering_preserves_original_program_shape() {
        let ast = vec![
            Node::MoveRight,
            Node::Increment,
            Node::Loop(vec![Node::Decrement, Node::Output]),
        ];

        let ir = lower(&ast);

        assert_eq!(
            ir,
            vec![
                Instr::Move(1),
                Instr::Add(1),
                Instr::Loop(vec![Instr::Add(-1), Instr::Output]),
            ]
        );
    }

    #[test]
    fn emit_brainfuck_round_trips_exact_lowering() {
        let ast = vec![
            Node::MoveRight,
            Node::MoveLeft,
            Node::Increment,
            Node::Decrement,
            Node::Loop(vec![Node::Increment]),
        ];

        let ir = lower(&ast);

        assert_eq!(emit_brainfuck(&ir), "><+-[+]");
    }

    #[test]
    fn emit_brainfuck_uses_clear_canonical_form() {
        assert_eq!(emit_brainfuck(&[Instr::Clear]), "[-]");
    }

    #[test]
    fn emit_brainfuck_emits_transfer_loop() {
        let ir = vec![Instr::Transfer(vec![(1, 2), (-1, -1)])];

        assert_eq!(emit_brainfuck(&ir), "[->++<<->]");
    }
}
