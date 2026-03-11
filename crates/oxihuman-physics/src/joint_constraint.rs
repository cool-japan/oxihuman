// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Joint constraint system for articulated bodies using PBD.

#[allow(dead_code)]
pub enum JointType {
    Fixed,
    Hinge {
        axis: [f32; 3],
    },
    Ball {
        radius: f32,
    },
    Slider {
        axis: [f32; 3],
        min_dist: f32,
        max_dist: f32,
    },
    Universal {
        axis1: [f32; 3],
        axis2: [f32; 3],
    },
}

#[allow(dead_code)]
pub struct JointConstraint {
    pub body_a: usize,
    pub body_b: usize,
    pub anchor_a: [f32; 3],
    pub anchor_b: [f32; 3],
    pub joint_type: JointType,
    pub stiffness: f32,
    pub damping: f32,
    pub broken: bool,
    pub max_force: f32,
}

#[allow(dead_code)]
pub struct ConstraintSolver {
    pub positions: Vec<[f32; 3]>,
    pub velocities: Vec<[f32; 3]>,
    pub masses: Vec<f32>,
    pub constraints: Vec<JointConstraint>,
}

// ── helpers ───────────────────────────────────────────────────────────────────

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn len3(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a);
    if l < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        scale3(a, 1.0 / l)
    }
}

// ── public API ────────────────────────────────────────────────────────────────

/// Create a new constraint solver with `n_bodies` bodies at the origin.
#[allow(dead_code)]
pub fn new_constraint_solver(n_bodies: usize) -> ConstraintSolver {
    ConstraintSolver {
        positions: vec![[0.0; 3]; n_bodies],
        velocities: vec![[0.0; 3]; n_bodies],
        masses: vec![1.0; n_bodies],
        constraints: Vec::new(),
    }
}

/// Add a Fixed joint between bodies `a` and `b`. Returns constraint index.
#[allow(dead_code)]
pub fn add_fixed_joint(solver: &mut ConstraintSolver, a: usize, b: usize, stiffness: f32) -> usize {
    let idx = solver.constraints.len();
    let anchor_a = [0.0f32; 3];
    let anchor_b = [0.0f32; 3];
    solver.constraints.push(JointConstraint {
        body_a: a,
        body_b: b,
        anchor_a,
        anchor_b,
        joint_type: JointType::Fixed,
        stiffness,
        damping: 0.0,
        broken: false,
        max_force: f32::MAX,
    });
    idx
}

/// Add a Hinge joint. Returns constraint index.
#[allow(dead_code)]
pub fn add_hinge_joint(
    solver: &mut ConstraintSolver,
    a: usize,
    b: usize,
    axis: [f32; 3],
    stiffness: f32,
) -> usize {
    let idx = solver.constraints.len();
    let anchor_a = [0.0f32; 3];
    let anchor_b = [0.0f32; 3];
    solver.constraints.push(JointConstraint {
        body_a: a,
        body_b: b,
        anchor_a,
        anchor_b,
        joint_type: JointType::Hinge { axis },
        stiffness,
        damping: 0.0,
        broken: false,
        max_force: f32::MAX,
    });
    idx
}

/// Add a Ball joint with a maximum radius. Returns constraint index.
#[allow(dead_code)]
pub fn add_ball_joint(
    solver: &mut ConstraintSolver,
    a: usize,
    b: usize,
    radius: f32,
    stiffness: f32,
) -> usize {
    let idx = solver.constraints.len();
    let anchor_a = [0.0f32; 3];
    let anchor_b = [0.0f32; 3];
    solver.constraints.push(JointConstraint {
        body_a: a,
        body_b: b,
        anchor_a,
        anchor_b,
        joint_type: JointType::Ball { radius },
        stiffness,
        damping: 0.0,
        broken: false,
        max_force: f32::MAX,
    });
    idx
}

/// PBD constraint solving: iteratively correct body positions.
#[allow(dead_code)]
pub fn solve_constraints(solver: &mut ConstraintSolver, dt: f32, iterations: u32) {
    let _ = dt;
    for _ in 0..iterations {
        let n = solver.constraints.len();
        for ci in 0..n {
            if solver.constraints[ci].broken {
                continue;
            }

            let a = solver.constraints[ci].body_a;
            let b = solver.constraints[ci].body_b;

            if a >= solver.positions.len() || b >= solver.positions.len() {
                continue;
            }

            let pa = solver.positions[a];
            let pb = solver.positions[b];
            let anchor_a = solver.constraints[ci].anchor_a;
            let anchor_b = solver.constraints[ci].anchor_b;
            let stiff = solver.constraints[ci].stiffness;

            // World-space anchor positions (simplified: body pos + offset)
            let world_a = add3(pa, anchor_a);
            let world_b = add3(pb, anchor_b);
            let delta = sub3(world_b, world_a);
            let dist = len3(delta);

            let violation = match &solver.constraints[ci].joint_type {
                JointType::Fixed => {
                    // Drive distance to zero
                    dist
                }
                JointType::Ball { radius } => {
                    let r = *radius;
                    if dist > r {
                        dist - r
                    } else {
                        0.0
                    }
                }
                JointType::Hinge { axis } => {
                    // Project delta onto plane perpendicular to axis, constrain that
                    let ax = normalize3(*axis);
                    let proj = dot3(delta, ax);
                    let along = scale3(ax, proj);
                    let perp = sub3(delta, along);
                    len3(perp)
                }
                JointType::Slider {
                    axis,
                    min_dist,
                    max_dist,
                } => {
                    let ax = normalize3(*axis);
                    let proj = dot3(delta, ax);
                    let clamped = proj.clamp(*min_dist, *max_dist);
                    (proj - clamped).abs()
                }
                JointType::Universal { axis1, axis2 } => {
                    let ax1 = normalize3(*axis1);
                    let ax2 = normalize3(*axis2);
                    let v1 = dot3(delta, ax1);
                    let v2 = dot3(delta, ax2);
                    (v1 * v1 + v2 * v2).sqrt()
                }
            };

            // Check max_force
            let max_f = solver.constraints[ci].max_force;
            if violation * stiff > max_f && max_f < f32::MAX {
                solver.constraints[ci].broken = true;
                continue;
            }

            if violation < 1e-10 || dist < 1e-10 {
                continue;
            }

            let correction_dir = scale3(delta, 1.0 / dist);
            let correction = scale3(correction_dir, violation * stiff * 0.5);

            let ma = solver.masses.get(a).copied().unwrap_or(1.0);
            let mb = solver.masses.get(b).copied().unwrap_or(1.0);
            let total_inv = (1.0 / ma + 1.0 / mb).max(1e-10);
            let w_a = (1.0 / ma) / total_inv;
            let w_b = (1.0 / mb) / total_inv;

            solver.positions[a] = add3(pa, scale3(correction, w_a));
            solver.positions[b] = sub3(pb, scale3(correction, w_b));
        }
    }
}

/// Apply an impulse to both bodies of a constraint.
#[allow(dead_code)]
pub fn apply_joint_impulse(
    solver: &mut ConstraintSolver,
    constraint_idx: usize,
    impulse: [f32; 3],
) {
    if let Some(c) = solver.constraints.get(constraint_idx) {
        let a = c.body_a;
        let b = c.body_b;
        if a < solver.velocities.len() {
            let ma = solver.masses.get(a).copied().unwrap_or(1.0);
            solver.velocities[a] = add3(solver.velocities[a], scale3(impulse, 1.0 / ma));
        }
        if b < solver.velocities.len() {
            let mb = solver.masses.get(b).copied().unwrap_or(1.0);
            solver.velocities[b] = sub3(solver.velocities[b], scale3(impulse, 1.0 / mb));
        }
    }
}

/// Mark a constraint as broken (permanently disabled).
#[allow(dead_code)]
pub fn break_joint(solver: &mut ConstraintSolver, constraint_idx: usize) {
    if let Some(c) = solver.constraints.get_mut(constraint_idx) {
        c.broken = true;
    }
}

/// Distance error for a constraint (current violation).
#[allow(dead_code)]
pub fn joint_violation(solver: &ConstraintSolver, constraint_idx: usize) -> f32 {
    let c = match solver.constraints.get(constraint_idx) {
        Some(c) => c,
        None => return 0.0,
    };
    let a = c.body_a;
    let b = c.body_b;
    if a >= solver.positions.len() || b >= solver.positions.len() {
        return 0.0;
    }
    let pa = solver.positions[a];
    let pb = solver.positions[b];
    let world_a = add3(pa, c.anchor_a);
    let world_b = add3(pb, c.anchor_b);
    len3(sub3(world_b, world_a))
}

/// Count constraints that are not broken.
#[allow(dead_code)]
pub fn count_active_joints(solver: &ConstraintSolver) -> usize {
    solver.constraints.iter().filter(|c| !c.broken).count()
}

/// BFS from `root` body, following constraint edges, returning (body_id, position).
#[allow(dead_code)]
pub fn compute_chain_positions(solver: &ConstraintSolver, root: usize) -> Vec<(usize, [f32; 3])> {
    let n = solver.positions.len();
    if root >= n {
        return vec![];
    }

    // Build adjacency
    let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
    for c in &solver.constraints {
        if !c.broken && c.body_a < n && c.body_b < n {
            adj[c.body_a].push(c.body_b);
            adj[c.body_b].push(c.body_a);
        }
    }

    let mut visited = vec![false; n];
    let mut result = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(root);
    visited[root] = true;

    while let Some(node) = queue.pop_front() {
        let pos = solver.positions.get(node).copied().unwrap_or([0.0; 3]);
        result.push((node, pos));
        for &nb in &adj[node] {
            if !visited[nb] {
                visited[nb] = true;
                queue.push_back(nb);
            }
        }
    }

    result
}

/// Sum of squared violations across all active constraints.
#[allow(dead_code)]
pub fn constraint_energy(solver: &ConstraintSolver) -> f32 {
    (0..solver.constraints.len())
        .filter(|&i| !solver.constraints[i].broken)
        .map(|i| {
            let v = joint_violation(solver, i);
            v * v
        })
        .sum()
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // 1
    #[test]
    fn new_solver_sizes() {
        let s = new_constraint_solver(5);
        assert_eq!(s.positions.len(), 5);
        assert_eq!(s.velocities.len(), 5);
        assert_eq!(s.masses.len(), 5);
        assert!(s.constraints.is_empty());
    }

    // 2
    #[test]
    fn add_fixed_joint_increments_count() {
        let mut s = new_constraint_solver(3);
        let idx = add_fixed_joint(&mut s, 0, 1, 1.0);
        assert_eq!(idx, 0);
        assert_eq!(s.constraints.len(), 1);
    }

    // 3
    #[test]
    fn add_hinge_joint_returns_correct_index() {
        let mut s = new_constraint_solver(3);
        add_fixed_joint(&mut s, 0, 1, 1.0);
        let idx = add_hinge_joint(&mut s, 1, 2, [0.0, 1.0, 0.0], 0.8);
        assert_eq!(idx, 1);
    }

    // 4
    #[test]
    fn add_ball_joint_correct_type() {
        let mut s = new_constraint_solver(2);
        add_ball_joint(&mut s, 0, 1, 0.5, 1.0);
        assert!(matches!(
            s.constraints[0].joint_type,
            JointType::Ball { .. }
        ));
    }

    // 5
    #[test]
    fn count_active_joints_all_active() {
        let mut s = new_constraint_solver(4);
        add_fixed_joint(&mut s, 0, 1, 1.0);
        add_fixed_joint(&mut s, 1, 2, 1.0);
        assert_eq!(count_active_joints(&s), 2);
    }

    // 6
    #[test]
    fn break_joint_decrements_active() {
        let mut s = new_constraint_solver(3);
        add_fixed_joint(&mut s, 0, 1, 1.0);
        add_fixed_joint(&mut s, 1, 2, 1.0);
        break_joint(&mut s, 0);
        assert_eq!(count_active_joints(&s), 1);
    }

    // 7
    #[test]
    fn joint_violation_zero_at_origin() {
        let mut s = new_constraint_solver(2);
        add_fixed_joint(&mut s, 0, 1, 1.0);
        // Both at origin, anchors at origin → violation = 0
        let v = joint_violation(&s, 0);
        assert!(v < 1e-5, "expected ~0 violation, got {v}");
    }

    // 8
    #[test]
    fn joint_violation_nonzero_when_separated() {
        let mut s = new_constraint_solver(2);
        s.positions[1] = [3.0, 4.0, 0.0]; // dist = 5
        add_fixed_joint(&mut s, 0, 1, 1.0);
        let v = joint_violation(&s, 0);
        assert!((v - 5.0).abs() < 1e-3, "expected 5, got {v}");
    }

    // 9
    #[test]
    fn solve_constraints_reduces_violation() {
        let mut s = new_constraint_solver(2);
        s.positions[1] = [2.0, 0.0, 0.0];
        add_fixed_joint(&mut s, 0, 1, 1.0);
        let before = joint_violation(&s, 0);
        solve_constraints(&mut s, 0.016, 10);
        let after = joint_violation(&s, 0);
        assert!(after < before, "violation should decrease");
    }

    // 10
    #[test]
    fn apply_joint_impulse_changes_velocity() {
        let mut s = new_constraint_solver(2);
        add_fixed_joint(&mut s, 0, 1, 1.0);
        let vx_before = s.velocities[0][0];
        apply_joint_impulse(&mut s, 0, [1.0, 0.0, 0.0]);
        assert!((s.velocities[0][0] - vx_before).abs() > 1e-6);
    }

    // 11
    #[test]
    fn compute_chain_positions_single_body() {
        let s = new_constraint_solver(3);
        let chain = compute_chain_positions(&s, 0);
        // No constraints → only root
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].0, 0);
    }

    // 12
    #[test]
    fn compute_chain_positions_connected() {
        let mut s = new_constraint_solver(3);
        add_fixed_joint(&mut s, 0, 1, 1.0);
        add_fixed_joint(&mut s, 1, 2, 1.0);
        let chain = compute_chain_positions(&s, 0);
        assert_eq!(chain.len(), 3);
    }

    // 13
    #[test]
    fn constraint_energy_zero_at_origin() {
        let mut s = new_constraint_solver(2);
        add_fixed_joint(&mut s, 0, 1, 1.0);
        let e = constraint_energy(&s);
        assert!(e < 1e-8, "energy should be ~0 at co-located bodies");
    }

    // 14
    #[test]
    fn constraint_energy_nonzero_when_separated() {
        let mut s = new_constraint_solver(2);
        s.positions[1] = [1.0, 0.0, 0.0];
        add_fixed_joint(&mut s, 0, 1, 1.0);
        let e = constraint_energy(&s);
        assert!(e > 0.0);
    }

    // 15
    #[test]
    fn ball_joint_no_correction_within_radius() {
        let mut s = new_constraint_solver(2);
        s.positions[1] = [0.3, 0.0, 0.0]; // dist=0.3 < radius=0.5
        add_ball_joint(&mut s, 0, 1, 0.5, 1.0);
        let before = s.positions[1];
        solve_constraints(&mut s, 0.016, 10);
        // Should not move significantly since within radius
        let after = s.positions[1];
        let moved = ((after[0] - before[0]).powi(2) + (after[1] - before[1]).powi(2)).sqrt();
        assert!(
            moved < 0.01,
            "ball joint within radius should not move, moved={moved}"
        );
    }
}
