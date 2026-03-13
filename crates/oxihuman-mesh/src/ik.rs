// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Inverse Kinematics solvers: 2-bone analytical IK and multi-bone FABRIK.

// ─── Private math helpers ──────────────────────────────────────────────────

#[allow(dead_code)]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[allow(dead_code)]
fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = vec3_len(v);
    if len < 1e-8 {
        [0.0, 1.0, 0.0]
    } else {
        vec3_scale(v, 1.0 / len)
    }
}

#[allow(dead_code)]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

// ─── 2-bone IK ─────────────────────────────────────────────────────────────

/// Result of a 2-bone IK solve.
#[derive(Debug, Clone)]
pub struct IkResult {
    /// Position of the mid joint (elbow/knee).
    pub mid_pos: [f32; 3],
    /// Position of the end effector (reached or clamped).
    pub end_pos: [f32; 3],
    /// Whether the target was reachable.
    pub reached: bool,
    /// Angle at the root joint (radians).
    pub root_angle: f32,
    /// Angle at the mid joint (radians).
    pub mid_angle: f32,
}

/// Solve 2-bone IK (e.g., arm: shoulder → elbow → wrist).
///
/// Parameters:
/// - `root`: position of root joint (shoulder)
/// - `target`: desired position of end effector (wrist target)
/// - `upper_len`: length of upper bone (upper arm)
/// - `lower_len`: length of lower bone (forearm)
/// - `hint`: optional hint direction for the pole vector (elbow direction
///   preference). If `None`, defaults to `[0, 1, 0]` (bend upward).
///
/// Uses the law of cosines to find the elbow position analytically.
pub fn solve_2bone_ik(
    root: [f32; 3],
    target: [f32; 3],
    upper_len: f32,
    lower_len: f32,
    hint: Option<[f32; 3]>,
) -> IkResult {
    let to_target = vec3_sub(target, root);
    let original_d = vec3_len(to_target);

    let total_len = upper_len + lower_len;
    let min_len = (upper_len - lower_len).abs();

    // Clamp d to [min_len, total_len]
    let d = original_d.clamp(min_len, total_len);
    let reached = original_d <= total_len;

    // Direction from root toward target (or clamped)
    let dir = if original_d < 1e-8 {
        [0.0, 0.0, 1.0]
    } else {
        vec3_normalize(to_target)
    };

    // Law of cosines: angle at root between root→target and root→mid
    // cos(alpha) = (upper² + d² - lower²) / (2 * upper * d)
    let cos_alpha = ((upper_len * upper_len + d * d - lower_len * lower_len)
        / (2.0 * upper_len * d))
        .clamp(-1.0, 1.0);
    let alpha = cos_alpha.acos(); // angle at root joint

    // Angle at mid joint using law of cosines
    let cos_mid = ((upper_len * upper_len + lower_len * lower_len - d * d)
        / (2.0 * upper_len * lower_len))
        .clamp(-1.0, 1.0);
    let mid_angle = cos_mid.acos();

    // Build a perpendicular vector for the bend plane using the hint
    let pole = hint.unwrap_or([0.0, 1.0, 0.0]);
    let pole_norm = vec3_normalize(pole);

    // Gram-Schmidt: remove component parallel to dir
    let dot = vec3_dot(pole_norm, dir);
    let perp_raw = vec3_sub(pole_norm, vec3_scale(dir, dot));
    let perp = if vec3_len(perp_raw) < 1e-6 {
        // hint is parallel to dir — choose an arbitrary perpendicular
        let alt = if dir[0].abs() < 0.9 {
            [1.0_f32, 0.0, 0.0]
        } else {
            [0.0_f32, 1.0, 0.0]
        };
        let dot2 = vec3_dot(alt, dir);
        vec3_normalize(vec3_sub(alt, vec3_scale(dir, dot2)))
    } else {
        vec3_normalize(perp_raw)
    };

    // Mid joint position:
    // project along dir by (upper * cos_alpha), then along perp by (upper * sin_alpha)
    let mid_along = vec3_scale(dir, upper_len * cos_alpha);
    let mid_perp = vec3_scale(perp, upper_len * alpha.sin());
    let mid_pos = vec3_add(root, vec3_add(mid_along, mid_perp));

    // End effector: clamped or exact target
    let end_pos = if reached {
        target
    } else {
        // Stretch as far as possible along dir from root
        vec3_add(root, vec3_scale(dir, total_len))
    };

    IkResult {
        mid_pos,
        end_pos,
        reached,
        root_angle: alpha,
        mid_angle,
    }
}

// ─── FABRIK ────────────────────────────────────────────────────────────────

/// A joint for multi-bone FABRIK.
#[derive(Debug, Clone)]
pub struct IkJoint {
    /// World-space position of this joint.
    pub position: [f32; 3],
    /// Distance from this joint to the next one in the chain.
    pub bone_length: f32,
}

/// FABRIK (Forward And Backward Reaching IK) solver for N joints.
///
/// - `joints`: chain of joints (first = root, last = end effector)
/// - `target`: desired end effector position
/// - `iterations`: max iterations (e.g., 10)
/// - `tolerance`: convergence tolerance (e.g., 0.001)
///
/// Returns a `Vec` of updated joint positions (same length as `joints`).
pub fn fabrik_solve(
    joints: &[IkJoint],
    target: [f32; 3],
    iterations: usize,
    tolerance: f32,
) -> Vec<[f32; 3]> {
    if joints.is_empty() {
        return Vec::new();
    }
    if joints.len() == 1 {
        return vec![joints[0].position];
    }

    let n = joints.len();
    let mut positions: Vec<[f32; 3]> = joints.iter().map(|j| j.position).collect();
    let root = positions[0];

    // Bone lengths: joints[i].bone_length is the distance from joint i to joint i+1
    // The last joint's bone_length is unused (no joint after it)
    let bone_lengths: Vec<f32> = joints.iter().map(|j| j.bone_length).collect();

    // Check total reach
    let total_len: f32 = bone_lengths[..n - 1].iter().sum();
    let to_target = vec3_sub(target, root);
    let dist_to_target = vec3_len(to_target);

    if dist_to_target > total_len {
        // Target unreachable — stretch chain toward target
        let dir = vec3_normalize(to_target);
        let mut acc = 0.0_f32;
        positions[0] = root;
        for i in 1..n {
            acc += bone_lengths[i - 1];
            positions[i] = vec3_add(root, vec3_scale(dir, acc));
        }
        return positions;
    }

    for _iter in 0..iterations {
        // ── Backward pass: pull from target ──────────────────────────────
        positions[n - 1] = target;
        for i in (0..n - 1).rev() {
            let dir = vec3_normalize(vec3_sub(positions[i], positions[i + 1]));
            positions[i] = vec3_add(positions[i + 1], vec3_scale(dir, bone_lengths[i]));
        }

        // ── Forward pass: push from root ──────────────────────────────────
        positions[0] = root;
        for i in 0..n - 1 {
            let dir = vec3_normalize(vec3_sub(positions[i + 1], positions[i]));
            positions[i + 1] = vec3_add(positions[i], vec3_scale(dir, bone_lengths[i]));
        }

        // ── Convergence check ─────────────────────────────────────────────
        let end_dist = vec3_len(vec3_sub(positions[n - 1], target));
        if end_dist < tolerance {
            break;
        }
    }

    positions
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-3;

    // ── 2-bone IK tests ────────────────────────────────────────────────────

    #[test]
    fn two_bone_ik_straight_reach() {
        // Target exactly at upper+lower — chain should be fully extended
        let root = [0.0, 0.0, 0.0];
        let target = [5.0, 0.0, 0.0];
        let result = solve_2bone_ik(root, target, 3.0, 2.0, None);
        assert!(result.reached, "should be reachable");
        // end_pos should be near target
        let dx = result.end_pos[0] - target[0];
        let dy = result.end_pos[1] - target[1];
        let dz = result.end_pos[2] - target[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        assert!(dist < EPS, "end_pos should match target; dist={dist}");
    }

    #[test]
    fn two_bone_ik_reachable_target() {
        let root = [0.0, 0.0, 0.0];
        let target = [2.0, 0.0, 0.0];
        let result = solve_2bone_ik(root, target, 3.0, 2.0, None);
        assert!(result.reached, "target at distance 2 should be reachable");
    }

    #[test]
    fn two_bone_ik_unreachable_target() {
        let root = [0.0, 0.0, 0.0];
        let target = [100.0, 0.0, 0.0]; // way beyond 3+2=5
        let result = solve_2bone_ik(root, target, 3.0, 2.0, None);
        assert!(!result.reached, "target at 100 should be unreachable");

        // end_pos should be clamped (at total_len from root)
        let d = vec3_len(vec3_sub(result.end_pos, root));
        assert!(
            (d - 5.0).abs() < EPS,
            "end_pos should be at total length (5); d={d}"
        );
    }

    #[test]
    fn two_bone_ik_end_pos_near_target_when_reachable() {
        let root = [0.0, 0.0, 0.0];
        let target = [3.0, 2.0, 1.0];
        let result = solve_2bone_ik(root, target, 4.0, 3.0, None);
        assert!(result.reached);
        let dist = vec3_len(vec3_sub(result.end_pos, target));
        assert!(
            dist < EPS,
            "end_pos should equal target when reachable; dist={dist}"
        );
    }

    #[test]
    fn two_bone_ik_mid_pos_correct_distance_from_root() {
        let root = [0.0, 0.0, 0.0];
        let target = [3.0, 0.0, 0.0];
        let upper = 3.0_f32;
        let lower = 2.0_f32;
        let result = solve_2bone_ik(root, target, upper, lower, None);
        let d_root = vec3_len(vec3_sub(result.mid_pos, root));
        assert!(
            (d_root - upper).abs() < EPS,
            "mid_pos should be upper_len from root; d={d_root}"
        );
    }

    #[test]
    fn two_bone_ik_mid_pos_correct_distance_from_end() {
        let root = [0.0, 0.0, 0.0];
        let target = [3.0, 0.0, 0.0];
        let upper = 3.0_f32;
        let lower = 2.0_f32;
        let result = solve_2bone_ik(root, target, upper, lower, None);
        let d_end = vec3_len(vec3_sub(result.mid_pos, result.end_pos));
        assert!(
            (d_end - lower).abs() < EPS,
            "mid_pos should be lower_len from end; d={d_end}"
        );
    }

    #[test]
    fn two_bone_ik_equal_bones() {
        // upper == lower == 2, target at distance 3 (reachable since 2+2=4 >= 3)
        let root = [0.0, 0.0, 0.0];
        let target = [3.0, 0.0, 0.0];
        let result = solve_2bone_ik(root, target, 2.0, 2.0, None);
        assert!(result.reached, "distance 3 should be reachable with 2+2=4");

        let d_root = vec3_len(vec3_sub(result.mid_pos, root));
        let d_end = vec3_len(vec3_sub(result.mid_pos, result.end_pos));
        assert!(
            (d_root - 2.0).abs() < EPS,
            "upper bone length wrong; d={d_root}"
        );
        assert!(
            (d_end - 2.0).abs() < EPS,
            "lower bone length wrong; d={d_end}"
        );
    }

    #[test]
    fn two_bone_ik_with_hint_bends_correctly() {
        // With hint=[0,0,1], the elbow should be displaced in +Z direction
        let root = [0.0, 0.0, 0.0];
        let target = [4.0, 0.0, 0.0];
        let result_up = solve_2bone_ik(root, target, 3.0, 2.0, Some([0.0, 1.0, 0.0]));
        let result_fwd = solve_2bone_ik(root, target, 3.0, 2.0, Some([0.0, 0.0, 1.0]));

        // Both should be reachable
        assert!(result_up.reached);
        assert!(result_fwd.reached);

        // Elbow with Y-hint should have positive Y component
        assert!(
            result_up.mid_pos[1] > 0.0,
            "Y-hint should push elbow in +Y; mid_pos={:?}",
            result_up.mid_pos
        );

        // Elbow with Z-hint should have positive Z component
        assert!(
            result_fwd.mid_pos[2] > 0.0,
            "Z-hint should push elbow in +Z; mid_pos={:?}",
            result_fwd.mid_pos
        );
    }

    // ── FABRIK tests ───────────────────────────────────────────────────────

    fn make_joints(positions: &[[f32; 3]], bone_len: f32) -> Vec<IkJoint> {
        positions
            .iter()
            .map(|&position| IkJoint {
                position,
                bone_length: bone_len,
            })
            .collect()
    }

    #[test]
    fn fabrik_solve_two_joints_reaches_target() {
        // 3 joints (2 bones of 0.5 each): root=[0,0,0], mid=[0.5,0,0], end=[1,0,0]
        // Target at [0.6, 0.4, 0]: distance ~0.72 < total 1.0, so reachable via bending
        let joints = make_joints(&[[0.0, 0.0, 0.0], [0.5, 0.0, 0.0], [1.0, 0.0, 0.0]], 0.5);
        let target = [0.6, 0.4, 0.0];
        let result = fabrik_solve(&joints, target, 30, 1e-4);
        let end_dist = vec3_len(vec3_sub(*result.last().expect("should succeed"), target));
        assert!(
            end_dist < 0.01,
            "2-segment FABRIK should reach target; dist={end_dist}"
        );
    }

    #[test]
    fn fabrik_solve_three_joints_near_target() {
        let joints = make_joints(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]], 1.0);
        let target = [1.5, 0.5, 0.0];
        let result = fabrik_solve(&joints, target, 20, 1e-4);
        let end_dist = vec3_len(vec3_sub(*result.last().expect("should succeed"), target));
        assert!(
            end_dist < 0.01,
            "3-joint FABRIK should reach target; dist={end_dist}"
        );
    }

    #[test]
    fn fabrik_solve_unreachable_target_returns_stretched() {
        // Total chain length = 2.0 (two bones of 1.0 each)
        let joints = make_joints(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]], 1.0);
        let target = [100.0, 0.0, 0.0];
        let result = fabrik_solve(&joints, target, 10, 1e-4);
        // End effector should be stretched toward target at total length
        let end = result.last().expect("should succeed");
        let d_from_root = vec3_len(vec3_sub(*end, result[0]));
        assert!(
            (d_from_root - 2.0).abs() < EPS,
            "stretched chain should reach total length from root; d={d_from_root}"
        );
        // Should be pointing toward target (positive X)
        assert!(end[0] > 0.0, "stretched chain should point toward target");
    }

    #[test]
    fn fabrik_solve_preserves_bone_lengths() {
        let joints = make_joints(
            &[
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
            ],
            1.0,
        );
        let target = [1.5, 1.5, 0.0];
        let result = fabrik_solve(&joints, target, 20, 1e-5);
        for i in 0..result.len() - 1 {
            let d = vec3_len(vec3_sub(result[i + 1], result[i]));
            assert!(
                (d - 1.0).abs() < EPS,
                "bone length between joints {i} and {} should be 1.0; d={d}",
                i + 1
            );
        }
    }

    #[test]
    fn fabrik_solve_result_length_matches_joints() {
        let joints = make_joints(
            &[
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
            ],
            1.0,
        );
        let target = [2.0, 2.0, 0.0];
        let result = fabrik_solve(&joints, target, 10, 1e-4);
        assert_eq!(
            result.len(),
            joints.len(),
            "result length should match joints length"
        );
    }
}
