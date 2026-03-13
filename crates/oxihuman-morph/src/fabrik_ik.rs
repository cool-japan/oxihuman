// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! FABRIK (Forward And Backward Reaching Inverse Kinematics) solver.
//!
//! Provides a two-pass iterative IK solver for articulated chains, with an
//! optional constrained variant that supports pole-vector hints and per-joint
//! cone angle limits.
//!
//! All `vec3` helpers are inlined to avoid extra dependencies.

// ── inline vec3 helpers ───────────────────────────────────────────────────────

#[inline]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn vec3_scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn vec3_len(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

#[inline]
fn vec3_dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    vec3_len(vec3_sub(a, b))
}

/// Returns `None` if the vector is degenerate (near-zero length).
/// Callers should use `.unwrap_or([0.0, 1.0, 0.0])` as a safe fallback.
#[inline]
fn vec3_normalize(a: [f32; 3]) -> Option<[f32; 3]> {
    let len = vec3_len(a);
    if len < 1e-8 {
        None
    } else {
        Some([a[0] / len, a[1] / len, a[2] / len])
    }
}

// ── IkChain ───────────────────────────────────────────────────────────────────

/// A linear chain of joints used as input/output for the FABRIK solver.
///
/// `joints[0]` is the **root** (fixed during backward pass).
/// `joints[n-1]` is the **end-effector** (tip).
/// `lengths[i]` stores the rest distance between `joints[i]` and `joints[i+1]`.
#[derive(Clone, Debug)]
pub struct IkChain {
    /// World-space positions of each joint.  `joints[0]` = root (fixed).
    pub joints: Vec<[f32; 3]>,
    /// Bone lengths: `lengths[i] = dist(joints[i], joints[i+1])`.
    pub lengths: Vec<f32>,
}

impl IkChain {
    /// Constructs a chain from an ordered slice of world-space positions.
    ///
    /// # Panics
    /// Does not panic; however, a chain with fewer than 2 positions produces
    /// a solver that immediately returns without moving anything.
    pub fn from_positions(positions: &[[f32; 3]]) -> Self {
        let n = positions.len();
        let lengths = if n < 2 {
            vec![]
        } else {
            (0..n - 1)
                .map(|i| vec3_dist(positions[i], positions[i + 1]))
                .collect()
        };
        Self {
            joints: positions.to_vec(),
            lengths,
        }
    }

    /// The maximum reach of the chain (sum of all bone lengths).
    pub fn total_reach(&self) -> f32 {
        self.lengths.iter().sum()
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    fn n(&self) -> usize {
        self.joints.len()
    }

    /// Single forward pass (tip → root).
    fn forward_pass(&mut self, target: [f32; 3]) {
        let n = self.n();
        if n < 2 {
            return;
        }
        self.joints[n - 1] = target;
        for i in (0..n - 1).rev() {
            let dir = vec3_normalize(vec3_sub(self.joints[i], self.joints[i + 1]))
                .unwrap_or([0.0, 1.0, 0.0]);
            self.joints[i] = vec3_add(self.joints[i + 1], vec3_scale(dir, self.lengths[i]));
        }
    }

    /// Single backward pass (root → tip).  Restores root to `root_pos`.
    fn backward_pass(&mut self, root_pos: [f32; 3]) {
        let n = self.n();
        if n < 2 {
            return;
        }
        self.joints[0] = root_pos;
        for i in 0..n - 1 {
            let dir = vec3_normalize(vec3_sub(self.joints[i + 1], self.joints[i]))
                .unwrap_or([0.0, 1.0, 0.0]);
            self.joints[i + 1] = vec3_add(self.joints[i], vec3_scale(dir, self.lengths[i]));
        }
    }

    // ── public solvers ───────────────────────────────────────────────────────

    /// Two-pass FABRIK solver.
    ///
    /// Returns the number of iterations actually performed (≥ 1).  Exits early
    /// once the end-effector is within `tolerance` of `target`.
    ///
    /// If the target is further than `total_reach()` the chain is straightened
    /// toward the target and the function returns `1`.
    pub fn solve_fabrik(&mut self, target: [f32; 3], tolerance: f32, max_iter: u32) -> u32 {
        let n = self.n();
        if n < 2 {
            return 0;
        }

        let root_pos = self.joints[0];

        // Unreachable: extend linearly toward target.
        if vec3_dist(root_pos, target) > self.total_reach() {
            let dir = vec3_normalize(vec3_sub(target, root_pos)).unwrap_or([0.0, 1.0, 0.0]);
            let mut cursor = root_pos;
            for i in 0..n - 1 {
                self.joints[i] = cursor;
                cursor = vec3_add(cursor, vec3_scale(dir, self.lengths[i]));
            }
            self.joints[n - 1] = cursor;
            return 1;
        }

        for iter in 1..=max_iter {
            self.forward_pass(target);
            self.backward_pass(root_pos);

            if vec3_dist(self.joints[n - 1], target) < tolerance {
                return iter;
            }
        }
        max_iter
    }

    /// Constrained FABRIK with a pole-vector hint and per-joint cone angle limit.
    ///
    /// After each backward pass the following post-processing is applied:
    ///
    /// 1. **Cone angle limit** – each joint `i` (for `i` in `1..n-1`) has its
    ///    direction relative to the parent bone clamped to `angle_limit_deg`.
    /// 2. **Pole vector hint** – intermediate joints are nudged toward the
    ///    plane defined by (root → tip, root → pole) so the chain bends
    ///    consistently.
    ///
    /// Returns the number of iterations performed (≥ 1).
    pub fn solve_constrained_fabrik(
        &mut self,
        target: [f32; 3],
        pole_vector: [f32; 3],
        angle_limit_deg: f32,
        tolerance: f32,
        max_iter: u32,
    ) -> u32 {
        let n = self.n();
        if n < 2 {
            return 0;
        }

        let root_pos = self.joints[0];
        let angle_limit_cos = angle_limit_deg.to_radians().cos();

        // Unreachable: straighten and return.
        if vec3_dist(root_pos, target) > self.total_reach() {
            let dir = vec3_normalize(vec3_sub(target, root_pos)).unwrap_or([0.0, 1.0, 0.0]);
            let mut cursor = root_pos;
            for i in 0..n - 1 {
                self.joints[i] = cursor;
                cursor = vec3_add(cursor, vec3_scale(dir, self.lengths[i]));
            }
            self.joints[n - 1] = cursor;
            return 1;
        }

        for iter in 1..=max_iter {
            self.forward_pass(target);
            self.backward_pass(root_pos);

            // Post-process: cone angle limit on intermediate joints.
            apply_cone_limits(&mut self.joints, &self.lengths, angle_limit_cos);

            // Post-process: pole vector hint on intermediate joints.
            apply_pole_hint(
                &mut self.joints,
                &self.lengths,
                root_pos,
                target,
                pole_vector,
            );

            if vec3_dist(self.joints[n - 1], target) < tolerance {
                return iter;
            }
        }
        max_iter
    }
}

// ── constraint helpers ────────────────────────────────────────────────────────

/// Clamps each bone direction to within `angle_limit_cos` of the previous bone
/// direction.  Works in place; `lengths` is used to re-place joints after clamping.
fn apply_cone_limits(joints: &mut [[f32; 3]], lengths: &[f32], angle_limit_cos: f32) {
    let n = joints.len();
    if n < 3 {
        return;
    }

    for i in 1..n - 1 {
        // Direction of the parent bone (bone i-1 → bone i).
        let parent_dir =
            vec3_normalize(vec3_sub(joints[i], joints[i - 1])).unwrap_or([0.0, 1.0, 0.0]);

        // Direction of the child bone (bone i → bone i+1).
        let child_dir =
            vec3_normalize(vec3_sub(joints[i + 1], joints[i])).unwrap_or([0.0, 1.0, 0.0]);

        let cos_angle = vec3_dot(parent_dir, child_dir).clamp(-1.0, 1.0);

        if cos_angle < angle_limit_cos {
            // Clamp: rotate child_dir toward parent_dir so the cone is respected.
            // We slerp between child_dir and parent_dir to sit on the cone boundary.
            let clamped_dir = slerp_dir(child_dir, parent_dir, cos_angle, angle_limit_cos);
            joints[i + 1] = vec3_add(joints[i], vec3_scale(clamped_dir, lengths[i]));
        }
    }
}

/// Spherical-linear interpolation between two unit vectors so that the result
/// sits exactly at `target_cos` angle from `b`.  Returns a unit vector.
fn slerp_dir(a: [f32; 3], b: [f32; 3], current_cos: f32, target_cos: f32) -> [f32; 3] {
    // We need the result `r` such that dot(r, b) == target_cos and r lies in the
    // same plane as a and b.
    // r = sin(target_angle) * perp + cos(target_angle) * b
    // where perp is the component of a perpendicular to b, normalised.
    let target_sin = (1.0_f32 - target_cos * target_cos).max(0.0).sqrt();

    // Perpendicular component of `a` w.r.t. `b`.
    let a_perp = vec3_sub(a, vec3_scale(b, current_cos));
    let perp_dir = vec3_normalize(a_perp).unwrap_or_else(|| {
        // a and b are collinear — pick an arbitrary perpendicular.
        arbitrary_perp(b)
    });

    vec3_add(vec3_scale(b, target_cos), vec3_scale(perp_dir, target_sin))
}

/// Returns an arbitrary unit vector perpendicular to `v`.
fn arbitrary_perp(v: [f32; 3]) -> [f32; 3] {
    let candidate = if v[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    vec3_normalize(vec3_cross(v, candidate)).unwrap_or([0.0, 1.0, 0.0])
}

/// Nudges intermediate joints toward the pole-vector plane.
///
/// The plane is spanned by (root→tip) and (root→pole).  For each middle joint
/// we project its deviation from the root→tip axis onto the pole side, giving
/// a gentle hint without hard constraints.
fn apply_pole_hint(
    joints: &mut [[f32; 3]],
    lengths: &[f32],
    root_pos: [f32; 3],
    target: [f32; 3],
    pole_vector: [f32; 3],
) {
    let n = joints.len();
    if n < 3 {
        return;
    }

    let chain_dir = vec3_normalize(vec3_sub(target, root_pos)).unwrap_or([0.0, 1.0, 0.0]);

    // Pole direction projected out of chain_dir.
    let pole_from_root = vec3_sub(pole_vector, root_pos);
    let pole_proj = vec3_sub(
        pole_from_root,
        vec3_scale(chain_dir, vec3_dot(pole_from_root, chain_dir)),
    );
    let pole_side = vec3_normalize(pole_proj).unwrap_or_else(|| arbitrary_perp(chain_dir));

    // For each intermediate joint, compute where it sits along the chain axis
    // and nudge it toward the pole side using a small fraction of the bone length.
    let total_len: f32 = lengths.iter().sum::<f32>().max(1e-8);
    let mut accumulated = 0.0_f32;

    for i in 1..n - 1 {
        accumulated += lengths[i - 1];
        let t = accumulated / total_len;

        // Lerp position along root→tip line.
        let on_axis = vec3_add(root_pos, vec3_scale(vec3_sub(target, root_pos), t));

        // Current offset from axis.
        let offset = vec3_sub(joints[i], on_axis);
        let offset_along_pole = vec3_dot(offset, pole_side);

        // If joint is already on the pole side, do nothing; otherwise flip gently.
        if offset_along_pole < 0.0 {
            // Reflect the perpendicular component toward pole side.
            let perp_correction = vec3_scale(pole_side, -2.0 * offset_along_pole);
            let nudged = vec3_add(joints[i], vec3_scale(perp_correction, 0.5));

            // Re-normalise distances to parent and child to preserve bone lengths.
            let dir_from_prev =
                vec3_normalize(vec3_sub(nudged, joints[i - 1])).unwrap_or([0.0, 1.0, 0.0]);
            joints[i] = vec3_add(joints[i - 1], vec3_scale(dir_from_prev, lengths[i - 1]));

            if i + 1 < n {
                let dir_to_next =
                    vec3_normalize(vec3_sub(joints[i + 1], joints[i])).unwrap_or([0.0, 1.0, 0.0]);
                joints[i + 1] = vec3_add(joints[i], vec3_scale(dir_to_next, lengths[i]));
            }
        }
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f32 = 0.01;
    const MAX_ITER: u32 = 50;

    // ── test 1: single-bone reach ────────────────────────────────────────────

    #[test]
    fn test_single_bone_reach() {
        // 2 joints, bone length = 1.0.
        // A single-bone FABRIK chain can only place the tip on the unit sphere
        // centred at the root.  We use a target that is exactly on that sphere
        // (at 45° in the XY plane) so FABRIK converges to within tolerance.
        let positions: &[[f32; 3]] = &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let mut chain = IkChain::from_positions(positions);
        let s = std::f32::consts::FRAC_1_SQRT_2; // 1/√2 ≈ 0.7071
        let target = [s, s, 0.0]; // unit vector at 45°

        chain.solve_fabrik(target, TOL, MAX_ITER);

        let tip = chain.joints[1];
        assert!(
            vec3_dist(tip, target) < TOL,
            "tip={tip:?} should be within {TOL} of target={target:?}"
        );
    }

    // ── test 2: 3-joint arm convergence ─────────────────────────────────────

    #[test]
    fn test_three_joint_arm_convergence() {
        // 3 joints (2 bones), each length 1.0.
        // Target at [2.5, 0, 0] — just beyond a straight extension of 2.0,
        // so the solver will converge to the best reachable point.
        // Hmm, total_reach = 2.0 and dist(root, target) = 2.5 > 2.0 →
        // unreachable case: chain extends straight, returns in 1 iteration.
        // Use a reachable target instead: [1.8, 0.2, 0].
        let positions: &[[f32; 3]] = &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let mut chain = IkChain::from_positions(positions);
        let target = [1.8_f32, 0.2, 0.0];

        let iters = chain.solve_fabrik(target, TOL, MAX_ITER);

        assert!(
            iters <= MAX_ITER,
            "should converge within {MAX_ITER} iterations, used {iters}"
        );
        let tip = chain.joints[2];
        assert!(
            vec3_dist(tip, target) < TOL,
            "tip={tip:?} should be within {TOL} of target={target:?}"
        );
    }

    // ── test 3: unreachable target → straight line ───────────────────────────

    #[test]
    fn test_unreachable_target_straight_line() {
        // 2 joints (1 bone of length 1.0).  Target at [100, 0, 0].
        let positions: &[[f32; 3]] = &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let mut chain = IkChain::from_positions(positions);
        let target = [100.0_f32, 0.0, 0.0];

        let iters = chain.solve_fabrik(target, TOL, MAX_ITER);

        assert_eq!(
            iters, 1,
            "unreachable target should return in exactly 1 iteration"
        );

        // Joints should be collinear along the X axis toward target.
        let root = chain.joints[0];
        let tip = chain.joints[1];

        // Root should not have moved.
        assert!(
            vec3_dist(root, [0.0, 0.0, 0.0]) < 1e-5,
            "root moved: {root:?}"
        );

        // Tip should be at [1, 0, 0] (full bone length in direction of target).
        assert!(
            (tip[0] - 1.0).abs() < 1e-4 && tip[1].abs() < 1e-5 && tip[2].abs() < 1e-5,
            "expected tip≈[1,0,0] got {tip:?}"
        );
    }

    // ── test 4: already-at-target ────────────────────────────────────────────

    #[test]
    fn test_already_at_target() {
        // Target equals the current tip → should converge in 0 or 1 iterations.
        let positions: &[[f32; 3]] = &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let mut chain = IkChain::from_positions(positions);
        let target = chain.joints[2]; // already at tip

        let iters = chain.solve_fabrik(target, TOL, MAX_ITER);

        // The tip should still be (nearly) at target.
        let tip = chain.joints[2];
        assert!(
            vec3_dist(tip, target) < TOL,
            "tip={tip:?} drifted from target={target:?} after {iters} iterations"
        );
        assert!(
            iters <= 2,
            "should converge almost immediately, used {iters}"
        );
    }

    // ── test 5: constrained FABRIK produces finite results ───────────────────

    #[test]
    fn test_constrained_fabrik_finite() {
        // 4-joint chain (3 bones of length 1.0).
        let positions: &[[f32; 3]] = &[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ];
        let mut chain = IkChain::from_positions(positions);
        let target = [2.5_f32, 0.5, 0.5];
        let pole = [1.5_f32, 1.0, 0.0];

        chain.solve_constrained_fabrik(target, pole, 45.0, TOL, MAX_ITER);

        for (i, joint) in chain.joints.iter().enumerate() {
            assert!(
                joint[0].is_finite() && joint[1].is_finite() && joint[2].is_finite(),
                "joint[{i}] contains non-finite value: {joint:?}"
            );
        }
    }
}
