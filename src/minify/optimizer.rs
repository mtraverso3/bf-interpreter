use crate::syntax::Node;

/// IDs for optimization passes that can be selected from the CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PassId {
    FoldAddSub,
    CanonicalizeClearLoops,
    RemoveKnownZeroLoops,
}

impl PassId {
    pub fn all() -> Vec<Self> {
        vec![
            Self::FoldAddSub,
            Self::CanonicalizeClearLoops,
            Self::RemoveKnownZeroLoops,
        ]
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
                PassId::CanonicalizeClearLoops => passes.push(Box::new(CanonicalizeClearLoopsPass)),
                PassId::RemoveKnownZeroLoops => passes.push(Box::new(RemoveKnownZeroLoopsPass)),
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
struct CanonicalizeClearLoopsPass;
struct RemoveKnownZeroLoopsPass;

impl OptimizationPass for FoldAddSubPass {
    fn run(&self, nodes: &[Node]) -> Vec<Node> {
        fold_add_sub(nodes)
    }
}

impl OptimizationPass for CanonicalizeClearLoopsPass {
    fn run(&self, nodes: &[Node]) -> Vec<Node> {
        canonicalize_clear_loops(nodes)
    }
}

impl OptimizationPass for RemoveKnownZeroLoopsPass {
    fn run(&self, nodes: &[Node]) -> Vec<Node> {
        remove_known_zero_loops(nodes, true)
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

fn canonicalize_clear_loops(nodes: &[Node]) -> Vec<Node> {
    nodes
        .iter()
        .map(|node| match node {
            Node::Loop(body) => {
                let body = canonicalize_clear_loops(body);
                if is_clear_loop_body(&body) {
                    Node::Loop(vec![Node::Decrement])
                } else {
                    Node::Loop(body)
                }
            }
            _ => node.clone(),
        })
        .collect()
}

fn remove_known_zero_loops(nodes: &[Node], mut current_cell_known_zero: bool) -> Vec<Node> {
    let mut out = Vec::new();

    for node in nodes {
        match node {
            Node::Loop(_) if current_cell_known_zero => continue,
            Node::Loop(body) => {
                let optimized_body = remove_known_zero_loops(body, false);
                let optimized_loop = Node::Loop(optimized_body);
                current_cell_known_zero = is_clear_loop(&optimized_loop);
                out.push(optimized_loop);
            }
            Node::Output => out.push(Node::Output),
            Node::MoveRight => {
                current_cell_known_zero = false;
                out.push(Node::MoveRight);
            }
            Node::MoveLeft => {
                current_cell_known_zero = false;
                out.push(Node::MoveLeft);
            }
            Node::Increment => {
                current_cell_known_zero = false;
                out.push(Node::Increment);
            }
            Node::Decrement => {
                current_cell_known_zero = false;
                out.push(Node::Decrement);
            }
            Node::Input => {
                current_cell_known_zero = false;
                out.push(Node::Input);
            }
        }
    }

    out
}

fn is_clear_loop(node: &Node) -> bool {
    match node {
        Node::Loop(body) => is_clear_loop_body(body),
        _ => false,
    }
}

fn is_clear_loop_body(body: &[Node]) -> bool {
    matches!(body, [Node::Increment] | [Node::Decrement])
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

        let optimized = optimize_with_passes(&nodes, &[PassId::FoldAddSub]);
        assert_eq!(optimized.len(), 1);

        match &optimized[0] {
            Node::Loop(body) => {
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Node::Output));
            }
            _ => panic!("expected loop"),
        }
    }

    #[test]
    fn canonicalize_clear_loops_rewrites_plus_loops() {
        let nodes = vec![Node::Loop(vec![Node::Increment])];

        let optimized = optimize_with_passes(&nodes, &[PassId::CanonicalizeClearLoops]);

        assert_eq!(optimized.len(), 1);
        assert!(matches!(optimized[0], Node::Loop(_)));
        match &optimized[0] {
            Node::Loop(body) => {
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Node::Decrement));
            }
            _ => panic!("expected loop"),
        }
    }

    #[test]
    fn selected_passes_can_compose_into_clear_loops() {
        let nodes = vec![Node::Loop(vec![
            Node::Increment,
            Node::Increment,
            Node::Decrement,
        ])];

        let optimized = optimize_with_passes(
            &nodes,
            &[PassId::FoldAddSub, PassId::CanonicalizeClearLoops],
        );

        match &optimized[0] {
            Node::Loop(body) => {
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Node::Decrement));
            }
            _ => panic!("expected loop"),
        }
    }

    #[test]
    fn remove_known_zero_loops_elides_dead_loops() {
        let nodes = vec![
            Node::Loop(vec![Node::Output]),
            Node::Increment,
            Node::Loop(vec![Node::Decrement]),
            Node::Loop(vec![Node::Output]),
        ];

        let optimized = optimize_with_passes(&nodes, &[PassId::RemoveKnownZeroLoops]);

        assert_eq!(optimized.len(), 2);
        assert!(matches!(optimized[0], Node::Increment));
        match &optimized[1] {
            Node::Loop(body) => {
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Node::Decrement));
            }
            _ => panic!("expected clear loop"),
        }
    }

    #[test]
    fn remove_known_zero_loops_recurses_inside_live_loops() {
        let nodes = vec![
            Node::Increment,
            Node::Loop(vec![
                Node::Loop(vec![Node::Decrement]),
                Node::Loop(vec![Node::Output]),
            ]),
        ];

        let optimized = optimize_with_passes(&nodes, &[PassId::RemoveKnownZeroLoops]);

        assert_eq!(optimized.len(), 2);
        assert!(matches!(optimized[0], Node::Increment));

        match &optimized[1] {
            Node::Loop(body) => {
                assert_eq!(body.len(), 1);
                match &body[0] {
                    Node::Loop(inner_body) => {
                        assert_eq!(inner_body.len(), 1);
                        assert!(matches!(inner_body[0], Node::Decrement));
                    }
                    _ => panic!("expected inner clear loop"),
                }
            }
            _ => panic!("expected outer loop"),
        }
    }
}
