use crate::ir::Instr;
use std::collections::BTreeMap;

/// IDs for optimization passes that can be selected from the CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PassId {
    FoldAddSub,
    FoldMove,
    CanonicalizeTransferLoops,
    CanonicalizeClearLoops,
    RemoveKnownZeroLoops,
}

impl PassId {
    pub fn all() -> Vec<Self> {
        vec![
            Self::FoldAddSub,
            Self::FoldMove,
            Self::CanonicalizeTransferLoops,
            Self::CanonicalizeClearLoops,
            Self::RemoveKnownZeroLoops,
        ]
    }
}

pub trait OptimizationPass {
    fn run(&self, program: &[Instr]) -> Vec<Instr>;
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
                PassId::FoldMove => passes.push(Box::new(FoldMovePass)),
                PassId::CanonicalizeTransferLoops => {
                    passes.push(Box::new(CanonicalizeTransferLoopsPass))
                }
                PassId::CanonicalizeClearLoops => passes.push(Box::new(CanonicalizeClearLoopsPass)),
                PassId::RemoveKnownZeroLoops => passes.push(Box::new(RemoveKnownZeroLoopsPass)),
            }
        }

        Self { passes }
    }

    pub fn run(&self, program: &[Instr]) -> Vec<Instr> {
        let mut current = program.to_vec();

        for pass in &self.passes {
            current = pass.run(&current);
        }

        current
    }
}

pub fn optimize_default(program: &[Instr]) -> Vec<Instr> {
    Pipeline::from_ids(&PassId::all()).run(program)
}

pub fn optimize_with_passes(program: &[Instr], passes: &[PassId]) -> Vec<Instr> {
    Pipeline::from_ids(passes).run(program)
}

struct FoldAddSubPass;
struct FoldMovePass;
struct CanonicalizeTransferLoopsPass;
struct CanonicalizeClearLoopsPass;
struct RemoveKnownZeroLoopsPass;

impl OptimizationPass for FoldAddSubPass {
    fn run(&self, program: &[Instr]) -> Vec<Instr> {
        fold_add_sub(program)
    }
}

impl OptimizationPass for FoldMovePass {
    fn run(&self, program: &[Instr]) -> Vec<Instr> {
        fold_moves(program)
    }
}

impl OptimizationPass for CanonicalizeTransferLoopsPass {
    fn run(&self, program: &[Instr]) -> Vec<Instr> {
        canonicalize_transfer_loops(program)
    }
}

impl OptimizationPass for CanonicalizeClearLoopsPass {
    fn run(&self, program: &[Instr]) -> Vec<Instr> {
        canonicalize_clear_loops(program)
    }
}

impl OptimizationPass for RemoveKnownZeroLoopsPass {
    fn run(&self, program: &[Instr]) -> Vec<Instr> {
        remove_known_zero_loops(program, true)
    }
}

fn fold_add_sub(program: &[Instr]) -> Vec<Instr> {
    let mut out = Vec::new();
    let mut delta: i32 = 0;

    for instr in program {
        match instr {
            Instr::Add(amount) => delta += i32::from(*amount),
            Instr::Loop(body) => {
                flush_add_delta(&mut out, delta);
                delta = 0;
                out.push(Instr::Loop(fold_add_sub(body)));
            }
            _ => {
                flush_add_delta(&mut out, delta);
                delta = 0;
                out.push(instr.clone());
            }
        }
    }

    flush_add_delta(&mut out, delta);
    out
}

fn fold_moves(program: &[Instr]) -> Vec<Instr> {
    let mut out = Vec::new();
    let mut delta: i64 = 0;
    let mut direction: i8 = 0;

    for instr in program {
        match instr {
            Instr::Move(amount) => {
                let next_direction = amount.signum() as i8;
                if direction != 0 && next_direction != 0 && next_direction != direction {
                    flush_move_delta(&mut out, delta);
                    delta = 0;
                }
                if next_direction != 0 {
                    direction = next_direction;
                }
                delta += amount;
            }
            Instr::Loop(body) => {
                flush_move_delta(&mut out, delta);
                delta = 0;
                direction = 0;
                out.push(Instr::Loop(fold_moves(body)));
            }
            _ => {
                flush_move_delta(&mut out, delta);
                delta = 0;
                direction = 0;
                out.push(instr.clone());
            }
        }
    }

    flush_move_delta(&mut out, delta);
    out
}

fn flush_add_delta(out: &mut Vec<Instr>, delta: i32) {
    let normalized = delta.rem_euclid(256);
    if normalized == 0 {
        return;
    }

    let plus_count = normalized;
    let minus_count = 256 - normalized;

    if plus_count <= minus_count {
        out.push(Instr::Add(plus_count as i16));
    } else {
        out.push(Instr::Add(-(minus_count as i16)));
    }
}

fn flush_move_delta(out: &mut Vec<Instr>, delta: i64) {
    if delta != 0 {
        out.push(Instr::Move(delta));
    }
}

fn canonicalize_clear_loops(program: &[Instr]) -> Vec<Instr> {
    program
        .iter()
        .map(|instr| match instr {
            Instr::Loop(body) => {
                let body = canonicalize_clear_loops(body);
                if is_clear_loop_body(&body) {
                    Instr::Clear
                } else {
                    Instr::Loop(body)
                }
            }
            _ => instr.clone(),
        })
        .collect()
}

fn canonicalize_transfer_loops(program: &[Instr]) -> Vec<Instr> {
    program
        .iter()
        .map(|instr| match instr {
            Instr::Loop(body) => {
                let body = canonicalize_transfer_loops(body);
                if let Some(targets) = match_transfer_loop(&body) {
                    Instr::Transfer(targets)
                } else {
                    Instr::Loop(body)
                }
            }
            _ => instr.clone(),
        })
        .collect()
}

fn match_transfer_loop(body: &[Instr]) -> Option<Vec<(i64, i16)>> {
    let mut ptr: i64 = 0;
    let mut source_delta: i32 = 0;
    let mut target_deltas: BTreeMap<i64, i32> = BTreeMap::new();

    for instr in body {
        match instr {
            Instr::Move(delta) => ptr = ptr.checked_add(*delta)?,
            Instr::Add(delta) => {
                if ptr == 0 {
                    source_delta += i32::from(*delta);
                } else {
                    *target_deltas.entry(ptr).or_default() += i32::from(*delta);
                }
            }
            _ => return None,
        }
    }

    if ptr != 0 || source_delta.rem_euclid(256) != 255 {
        return None;
    }

    let mut targets = Vec::new();
    for (offset, delta) in target_deltas {
        let normalized = delta.rem_euclid(256);
        if normalized == 0 {
            continue;
        }

        let plus_count = normalized;
        let minus_count = 256 - normalized;
        let factor = if plus_count <= minus_count {
            plus_count as i16
        } else {
            -(minus_count as i16)
        };
        targets.push((offset, factor));
    }

    if targets.is_empty() {
        return None;
    }

    Some(targets)
}

fn remove_known_zero_loops(program: &[Instr], mut current_cell_known_zero: bool) -> Vec<Instr> {
    let mut out = Vec::new();

    for instr in program {
        match instr {
            Instr::Loop(_) if current_cell_known_zero => continue,
            Instr::Clear => {
                current_cell_known_zero = true;
                out.push(Instr::Clear);
            }
            Instr::Transfer(_) if current_cell_known_zero => continue,
            Instr::Transfer(targets) => {
                current_cell_known_zero = true;
                out.push(Instr::Transfer(targets.clone()));
            }
            Instr::Loop(body) => {
                let optimized_body = remove_known_zero_loops(body, false);
                current_cell_known_zero = false;
                out.push(Instr::Loop(optimized_body));
            }
            Instr::Output => out.push(Instr::Output),
            Instr::Move(_) => {
                current_cell_known_zero = false;
                out.push(instr.clone());
            }
            Instr::Add(_) => {
                current_cell_known_zero = false;
                out.push(instr.clone());
            }
            Instr::Input => {
                current_cell_known_zero = false;
                out.push(Instr::Input);
            }
        }
    }

    out
}

fn is_clear_loop_body(body: &[Instr]) -> bool {
    matches!(body, [Instr::Add(_)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_add_sub_cancels_runs() {
        let program = vec![Instr::Add(1), Instr::Add(1), Instr::Add(1), Instr::Add(-1)];

        let optimized = optimize_with_passes(&program, &[PassId::FoldAddSub]);
        assert_eq!(optimized, vec![Instr::Add(2)]);
    }

    #[test]
    fn fold_add_sub_recurses_into_loops() {
        let program = vec![Instr::Loop(vec![
            Instr::Add(1),
            Instr::Add(-1),
            Instr::Output,
        ])];

        let optimized = optimize_with_passes(&program, &[PassId::FoldAddSub]);
        assert_eq!(optimized, vec![Instr::Loop(vec![Instr::Output])]);
    }

    #[test]
    fn fold_move_coalesces_same_direction_runs() {
        let program = vec![
            Instr::Move(1),
            Instr::Move(1),
            Instr::Move(1),
            Instr::Output,
        ];

        let optimized = optimize_with_passes(&program, &[PassId::FoldMove]);
        assert_eq!(optimized, vec![Instr::Move(3), Instr::Output]);
    }

    #[test]
    fn fold_move_does_not_cancel_opposite_directions() {
        let program = vec![Instr::Move(1), Instr::Move(-1)];

        let optimized = optimize_with_passes(&program, &[PassId::FoldMove]);
        assert_eq!(optimized, program);
    }

    #[test]
    fn canonicalize_clear_loops_rewrites_single_add_loops() {
        let program = vec![Instr::Loop(vec![Instr::Add(1)])];

        let optimized = optimize_with_passes(&program, &[PassId::CanonicalizeClearLoops]);
        assert_eq!(optimized, vec![Instr::Clear]);
    }

    #[test]
    fn canonicalize_transfer_loops_rewrites_simple_transfer() {
        let program = vec![Instr::Loop(vec![
            Instr::Add(-1),
            Instr::Move(1),
            Instr::Add(1),
            Instr::Move(-1),
        ])];

        let optimized = optimize_with_passes(&program, &[PassId::CanonicalizeTransferLoops]);
        assert_eq!(optimized, vec![Instr::Transfer(vec![(1, 1)])]);
    }

    #[test]
    fn canonicalize_transfer_loops_rejects_non_terminating_source_update() {
        let program = vec![Instr::Loop(vec![
            Instr::Add(1),
            Instr::Move(1),
            Instr::Add(1),
            Instr::Move(-1),
        ])];

        let optimized = optimize_with_passes(&program, &[PassId::CanonicalizeTransferLoops]);
        assert_eq!(optimized, program);
    }

    #[test]
    fn selected_passes_can_compose_into_transfer_loops() {
        let program = vec![Instr::Loop(vec![
            Instr::Add(-1),
            Instr::Move(1),
            Instr::Add(1),
            Instr::Add(1),
            Instr::Move(-1),
        ])];

        let optimized = optimize_with_passes(
            &program,
            &[PassId::FoldAddSub, PassId::FoldMove, PassId::CanonicalizeTransferLoops],
        );
        assert_eq!(optimized, vec![Instr::Transfer(vec![(1, 2)])]);
    }

    #[test]
    fn selected_passes_can_compose_into_clear_loops() {
        let program = vec![Instr::Loop(vec![
            Instr::Add(1),
            Instr::Add(1),
            Instr::Add(-1),
        ])];

        let optimized = optimize_with_passes(
            &program,
            &[PassId::FoldAddSub, PassId::CanonicalizeClearLoops],
        );

        assert_eq!(optimized, vec![Instr::Clear]);
    }

    #[test]
    fn remove_known_zero_loops_elides_dead_loops() {
        let program = vec![
            Instr::Loop(vec![Instr::Output]),
            Instr::Add(1),
            Instr::Clear,
            Instr::Loop(vec![Instr::Output]),
        ];

        let optimized = optimize_with_passes(&program, &[PassId::RemoveKnownZeroLoops]);

        assert_eq!(optimized, vec![Instr::Add(1), Instr::Clear]);
    }

    #[test]
    fn remove_known_zero_loops_recurses_inside_live_loops() {
        let program = vec![
            Instr::Add(1),
            Instr::Loop(vec![Instr::Clear, Instr::Loop(vec![Instr::Output])]),
        ];

        let optimized = optimize_with_passes(&program, &[PassId::RemoveKnownZeroLoops]);

        assert_eq!(
            optimized,
            vec![Instr::Add(1), Instr::Loop(vec![Instr::Clear])]
        );
    }
}
