use crate::syntax::Node;

/// IDs for optimization passes that can be selected from the CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PassId {
    FoldAddSub,
}

impl PassId {
    pub fn all() -> Vec<Self> {
        vec![Self::FoldAddSub]
    }
}

pub trait OptimizationPass {
    fn run(&self, nodes: &[Node]) -> Vec<Node>;
}

pub struct Pipeline {
    passes: Vec<Box<dyn OptimizationPass>>,
}

impl Pipeline {
    pub fn from_ids(ids: &[PassId]) -> Self {
        let mut passes: Vec<Box<dyn OptimizationPass>> = Vec::new();

        for id in ids {
            match id {
                PassId::FoldAddSub => passes.push(Box::new(FoldAddSubPass)),
            }
        }

        Self { passes }
    }

    pub fn run(&self, nodes: &[Node]) -> Vec<Node> {
        let mut current = nodes.to_vec();

        for pass in &self.passes {
            current = pass.run(&current);
        }

        current
    }
}

pub fn optimize_default(nodes: &[Node]) -> Vec<Node> {
    Pipeline::from_ids(&PassId::all()).run(nodes)
}

pub fn optimize_with_passes(nodes: &[Node], passes: &[PassId]) -> Vec<Node> {
    Pipeline::from_ids(passes).run(nodes)
}

struct FoldAddSubPass;

impl OptimizationPass for FoldAddSubPass {
    fn run(&self, nodes: &[Node]) -> Vec<Node> {
        fold_add_sub(nodes)
    }
}

fn fold_add_sub(nodes: &[Node]) -> Vec<Node> {
    let mut out = Vec::new();
    let mut delta: i32 = 0;

    for node in nodes {
        match node {
            Node::Increment => delta += 1,
            Node::Decrement => delta -= 1,
            Node::Loop(body) => {
                flush_delta(&mut out, delta);
                delta = 0;
                out.push(Node::Loop(fold_add_sub(body)));
            }
            _ => {
                flush_delta(&mut out, delta);
                delta = 0;
                out.push(node.clone());
            }
        }
    }

    flush_delta(&mut out, delta);
    out
}

fn flush_delta(out: &mut Vec<Node>, delta: i32) {
    // Cell math is on u8 values, so +/- runs can be reduced modulo 256.
    let normalized = delta.rem_euclid(256) as usize;

    if normalized == 0 {
        return;
    }

    let plus_count = normalized;
    let minus_count = 256 - normalized;

    if plus_count <= minus_count {
        out.extend(std::iter::repeat(Node::Increment).take(plus_count));
    } else {
        out.extend(std::iter::repeat(Node::Decrement).take(minus_count));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_add_sub_cancels_runs() {
        let nodes = vec![
            Node::Increment,
            Node::Increment,
            Node::Increment,
            Node::Decrement,
        ];

        let optimized = optimize_default(&nodes);
        assert_eq!(optimized.len(), 2);
        assert!(matches!(optimized[0], Node::Increment));
        assert!(matches!(optimized[1], Node::Increment));
    }

    #[test]
    fn fold_add_sub_recurses_into_loops() {
        let nodes = vec![Node::Loop(vec![
            Node::Increment,
            Node::Decrement,
            Node::Output,
        ])];

        let optimized = optimize_default(&nodes);
        assert_eq!(optimized.len(), 1);

        match &optimized[0] {
            Node::Loop(body) => {
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Node::Output));
            }
            _ => panic!("expected loop"),
        }
    }
}
