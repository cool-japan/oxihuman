// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! FABRIK (Forward And Backward Reaching IK) solver.

// ── Configuration ─────────────────────────────────────────────────────────────

/// Solver configuration for a FABRIK IK chain.
#[allow(dead_code)]
pub struct IkConfig {
    /// Maximum number of FABRIK iterations per solve call.
    pub max_iterations: usize,
    /// Convergence threshold: stop when end-effector error < tolerance.
    pub tolerance: f32,
    /// Number of joints in the chain (informational; not enforced).
    pub chain_length: usize,
}

// ── Data types ────────────────────────────────────────────────────────────────

/// A single joint in an IK chain.
#[allow(dead_code)]
pub struct IkJoint {
    /// Current world-space position of the joint.
    pub position: [f32; 3],
    /// Minimum allowed angle (radians) at this joint.
    pub min_angle: f32,
    /// Maximum allowed angle (radians) at this joint.
    pub max_angle: f32,
    /// Length of the bone segment *from* this joint to the next.
    pub length: f32,
}

/// An ordered sequence of joints forming an IK chain.
#[allow(dead_code)]
pub struct IkChain {
    /// Joints ordered from root (index 0) to end-effector (last index).
    pub joints: Vec<IkJoint>,
    /// When `true`, the root joint is pinned and must not move.
    pub base_fixed: bool,
}

/// Result returned after one [`fabrik_solve`] call.
#[allow(dead_code)]
pub struct IkResult {
    /// `true` when the end-effector reached the target within tolerance.
    pub solved: bool,
    /// Number of FABRIK iterations actually executed.
    pub iterations: usize,
    /// Final world-space position of the end-effector joint.
    pub end_effector: [f32; 3],
    /// Euclidean distance between the end-effector and the target.
    pub error: f32,
}

// ── Vector helpers ─────────────────────────────────────────────────────────────

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
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
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    len3(sub3(a, b))
}

/// Normalise `a`; returns `[0,1,0]` if near-zero.
#[inline]
fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a);
    if l < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        scale3(a, 1.0 / l)
    }
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// Build a default [`IkConfig`] for a chain with `chain_length` joints.
#[allow(dead_code)]
pub fn default_ik_config(chain_length: usize) -> IkConfig {
    IkConfig {
        max_iterations: 32,
        tolerance: 1e-4,
        chain_length,
    }
}

/// Create a new joint at `pos` with bone length `length`.
///
/// Angle limits default to `[−π, π]`.
#[allow(dead_code)]
pub fn new_ik_joint(pos: [f32; 3], length: f32) -> IkJoint {
    IkJoint {
        position: pos,
        min_angle: -std::f32::consts::PI,
        max_angle: std::f32::consts::PI,
        length,
    }
}

/// Wrap a `Vec<IkJoint>` into an [`IkChain`] with `base_fixed = true`.
#[allow(dead_code)]
pub fn new_ik_chain(joints: Vec<IkJoint>) -> IkChain {
    IkChain {
        joints,
        base_fixed: true,
    }
}

/// Run the FABRIK algorithm to move the chain's end-effector toward `target`.
///
/// Returns an [`IkResult`] describing the final state.
#[allow(dead_code)]
pub fn fabrik_solve(chain: &mut IkChain, target: [f32; 3], cfg: &IkConfig) -> IkResult {
    let n = chain.joints.len();
    if n == 0 {
        return IkResult {
            solved: false,
            iterations: 0,
            end_effector: [0.0; 3],
            error: f32::MAX,
        };
    }

    // If target is unreachable (too far), stretch chain toward it.
    let total_len = ik_chain_total_length(chain);
    let root_pos = chain.joints[0].position;
    let root_to_target = dist3(root_pos, target);

    if root_to_target >= total_len {
        // Fully extend chain toward target.
        for i in 1..n {
            let dir = normalize3(sub3(target, chain.joints[i - 1].position));
            let bone_len = chain.joints[i - 1].length;
            chain.joints[i].position = add3(chain.joints[i - 1].position, scale3(dir, bone_len));
        }
        let ee = chain.joints[n - 1].position;
        let err = dist3(ee, target);
        return IkResult {
            solved: false,
            iterations: 1,
            end_effector: ee,
            error: err,
        };
    }

    let mut iterations = 0usize;

    for _iter in 0..cfg.max_iterations {
        iterations += 1;

        // ── Backward pass ─────────────────────────────────────────────────────
        chain.joints[n - 1].position = target;
        for i in (0..n - 1).rev() {
            let dir = normalize3(sub3(chain.joints[i].position, chain.joints[i + 1].position));
            let bone_len = chain.joints[i].length;
            chain.joints[i].position = add3(chain.joints[i + 1].position, scale3(dir, bone_len));
        }

        // ── Forward pass ──────────────────────────────────────────────────────
        if chain.base_fixed {
            chain.joints[0].position = root_pos;
        }
        for i in 0..n - 1 {
            let dir = normalize3(sub3(chain.joints[i + 1].position, chain.joints[i].position));
            let bone_len = chain.joints[i].length;
            chain.joints[i + 1].position = add3(chain.joints[i].position, scale3(dir, bone_len));
        }

        // ── Convergence check ─────────────────────────────────────────────────
        let err = dist3(chain.joints[n - 1].position, target);
        if err < cfg.tolerance {
            return IkResult {
                solved: true,
                iterations,
                end_effector: chain.joints[n - 1].position,
                error: err,
            };
        }
    }

    let ee = chain.joints[n - 1].position;
    IkResult {
        solved: false,
        iterations,
        end_effector: ee,
        error: dist3(ee, target),
    }
}

/// Return the sum of all bone lengths in `chain`.
#[allow(dead_code)]
pub fn ik_chain_total_length(chain: &IkChain) -> f32 {
    chain.joints.iter().map(|j| j.length).sum()
}

/// Return the number of joints in `chain`.
#[allow(dead_code)]
pub fn ik_joint_count(chain: &IkChain) -> usize {
    chain.joints.len()
}

/// Return the current position of the end-effector (last joint).
///
/// Returns `[0,0,0]` for an empty chain.
#[allow(dead_code)]
pub fn ik_end_effector(chain: &IkChain) -> [f32; 3] {
    chain.joints.last().map(|j| j.position).unwrap_or([0.0; 3])
}

/// Serialise the chain to a compact JSON string.
#[allow(dead_code)]
pub fn ik_chain_to_json(chain: &IkChain) -> String {
    let joints_str: Vec<String> = chain
        .joints
        .iter()
        .map(|j| {
            format!(
                r#"{{"position":[{:.4},{:.4},{:.4}],"length":{:.4}}}"#,
                j.position[0], j.position[1], j.position[2], j.length
            )
        })
        .collect();
    format!(
        r#"{{"base_fixed":{},"joints":[{}]}}"#,
        chain.base_fixed,
        joints_str.join(",")
    )
}

/// Serialise an [`IkResult`] to a compact JSON string.
#[allow(dead_code)]
pub fn ik_result_to_json(r: &IkResult) -> String {
    format!(
        r#"{{"solved":{},"iterations":{},"end_effector":[{:.4},{:.4},{:.4}],"error":{:.6}}}"#,
        r.solved,
        r.iterations,
        r.end_effector[0],
        r.end_effector[1],
        r.end_effector[2],
        r.error,
    )
}

/// Clamp `angle` to `[joint.min_angle, joint.max_angle]`, write it back, and
/// return the clamped value.
#[allow(dead_code)]
pub fn clamp_joint_angle(joint: &mut IkJoint, angle: f32) -> f32 {
    angle.clamp(joint.min_angle, joint.max_angle)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn two_bone_chain() -> IkChain {
        // Root at origin, two bones of length 1.0 each.
        new_ik_chain(vec![
            new_ik_joint([0.0, 0.0, 0.0], 1.0),
            new_ik_joint([1.0, 0.0, 0.0], 1.0),
            new_ik_joint([2.0, 0.0, 0.0], 0.0),
        ])
    }

    #[test]
    fn default_config_chain_length() {
        let cfg = default_ik_config(4);
        assert_eq!(cfg.chain_length, 4);
        assert_eq!(cfg.max_iterations, 32);
    }

    #[test]
    fn new_ik_joint_stores_values() {
        let j = new_ik_joint([1.0, 2.0, 3.0], 0.5);
        assert!((j.position[0] - 1.0).abs() < 1e-6);
        assert!((j.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn ik_joint_count_correct() {
        let chain = two_bone_chain();
        assert_eq!(ik_joint_count(&chain), 3);
    }

    #[test]
    fn ik_chain_total_length_correct() {
        let chain = two_bone_chain();
        // lengths: 1.0 + 1.0 + 0.0 = 2.0
        let total = ik_chain_total_length(&chain);
        assert!((total - 2.0).abs() < 1e-5, "expected 2.0, got {total}");
    }

    #[test]
    fn ik_end_effector_last_joint() {
        let chain = two_bone_chain();
        let ee = ik_end_effector(&chain);
        assert!((ee[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn fabrik_solve_reachable_target() {
        let mut chain = two_bone_chain();
        let cfg = default_ik_config(3);
        // Target at (1, 1, 0) is within total reach of 2.0.
        let result = fabrik_solve(&mut chain, [1.0, 1.0, 0.0], &cfg);
        assert!(result.error < 1e-2, "error too large: {}", result.error);
    }

    #[test]
    fn fabrik_solve_unreachable_target_extends_chain() {
        let mut chain = two_bone_chain();
        let cfg = default_ik_config(3);
        // Target at (100, 0, 0) is far beyond total reach of 2.0.
        let result = fabrik_solve(&mut chain, [100.0, 0.0, 0.0], &cfg);
        // Chain should be stretched; solved = false.
        assert!(!result.solved);
        // End effector should have moved toward target.
        assert!(result.end_effector[0] > 0.0);
    }

    #[test]
    fn fabrik_solve_empty_chain() {
        let mut chain = new_ik_chain(vec![]);
        let cfg = default_ik_config(0);
        let result = fabrik_solve(&mut chain, [1.0, 0.0, 0.0], &cfg);
        assert!(!result.solved);
        assert_eq!(result.iterations, 0);
    }

    #[test]
    fn clamp_joint_angle_within_range() {
        let mut j = new_ik_joint([0.0; 3], 1.0);
        j.min_angle = -1.0;
        j.max_angle = 1.0;
        let v = clamp_joint_angle(&mut j, 0.5);
        assert!((v - 0.5).abs() < 1e-6);
    }

    #[test]
    fn clamp_joint_angle_clamps_above_max() {
        let mut j = new_ik_joint([0.0; 3], 1.0);
        j.min_angle = -1.0;
        j.max_angle = 1.0;
        let v = clamp_joint_angle(&mut j, 5.0);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ik_chain_to_json_contains_base_fixed() {
        let chain = two_bone_chain();
        let json = ik_chain_to_json(&chain);
        assert!(json.contains("base_fixed"));
        assert!(json.contains("true"));
        assert!(json.contains("joints"));
    }

    #[test]
    fn ik_result_to_json_fields() {
        let r = IkResult {
            solved: true,
            iterations: 5,
            end_effector: [1.0, 2.0, 3.0],
            error: 0.001,
        };
        let json = ik_result_to_json(&r);
        assert!(json.contains("\"solved\":true"));
        assert!(json.contains("\"iterations\":5"));
        assert!(json.contains("\"error\":"));
    }

    #[test]
    fn ik_end_effector_empty_chain() {
        let chain = new_ik_chain(vec![]);
        let ee = ik_end_effector(&chain);
        assert_eq!(ee, [0.0, 0.0, 0.0]);
    }
}
