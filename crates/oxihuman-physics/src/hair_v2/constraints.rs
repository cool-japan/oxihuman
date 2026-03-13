// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! XPBD constraints for Cosserat-rod hair strands.
//!
//! Three constraint types:
//!
//! 1. **Stretch-twist** — couples axial stretch (edge length deviation) with
//!    twist angle between consecutive material frames.
//! 2. **Bend-twist** — limits curvature and relative twist between adjacent
//!    segments by penalizing the Darboux vector deviation from rest.
//! 3. **Length preservation** — simple inextensibility constraint (a subset of
//!    stretch-twist with twist weight = 0, used as fallback).

use super::strand::{
    extract_bend_angles, extract_twist_angle, quat_conj, quat_mul, quat_normalize, quat_rotate,
    v3_add, v3_dot, v3_length, v3_normalize, v3_scale, v3_sub, HairNode, HairStrand,
};

// ── Stretch-twist constraint ────────────────────────────────────────────────

/// Stretch-twist constraint between two adjacent nodes.
///
/// Couples the edge-length deviation (stretch) with the twist angle
/// difference from the rest configuration. The XPBD formulation uses a
/// combined scalar constraint `C = w_s * C_stretch + w_t * C_twist` and
/// computes position/orientation corrections accordingly.
#[derive(Debug, Clone)]
pub struct StretchTwistConstraint {
    /// Index of the first node in the strand.
    pub node_a: usize,
    /// Index of the second node in the strand.
    pub node_b: usize,
    /// Rest length of this edge.
    pub rest_length: f64,
    /// Rest twist angle between the two frames.
    pub rest_twist: f64,
    /// Inverse stiffness (XPBD compliance) for the stretch part.
    pub stretch_compliance: f64,
    /// Inverse stiffness (XPBD compliance) for the twist part.
    pub twist_compliance: f64,
    /// Accumulated Lagrange multiplier for stretch (XPBD warm-starting).
    pub lambda_stretch: f64,
    /// Accumulated Lagrange multiplier for twist.
    pub lambda_twist: f64,
}

impl StretchTwistConstraint {
    /// Create a new stretch-twist constraint from strand data.
    pub fn new(
        node_a: usize,
        node_b: usize,
        rest_length: f64,
        rest_twist: f64,
        stretch_stiffness: f64,
        twist_stiffness: f64,
    ) -> Self {
        // XPBD compliance = 1 / (stiffness * dt^2), but we store the raw
        // compliance and multiply by dt^2 at solve time.
        let stretch_compliance = if stretch_stiffness > 1e-15 {
            1.0 / stretch_stiffness
        } else {
            1e10 // very compliant
        };
        let twist_compliance = if twist_stiffness > 1e-15 {
            1.0 / twist_stiffness
        } else {
            1e10
        };
        Self {
            node_a,
            node_b,
            rest_length,
            rest_twist,
            stretch_compliance,
            twist_compliance,
            lambda_stretch: 0.0,
            lambda_twist: 0.0,
        }
    }

    /// Build all stretch-twist constraints for a strand.
    pub fn build_for_strand(
        strand: &HairStrand,
        stretch_stiffness: f64,
        twist_stiffness: f64,
    ) -> Vec<Self> {
        let seg = strand.segment_count();
        let mut out = Vec::with_capacity(seg);
        for i in 0..seg {
            // Compute rest twist from orientations.
            let dq = quat_mul(
                quat_conj(strand.nodes[i].orientation),
                strand.nodes[i + 1].orientation,
            );
            let rest_twist = extract_twist_angle(dq);
            out.push(Self::new(
                i,
                i + 1,
                strand.rest_lengths[i],
                rest_twist,
                stretch_stiffness,
                twist_stiffness,
            ));
        }
        out
    }

    /// Solve the stretch part of this constraint (position correction).
    ///
    /// Modifies node positions in `nodes` and returns the delta-lambda.
    pub fn solve_stretch(
        &mut self,
        nodes: &mut [HairNode],
        dt: f64,
    ) -> f64 {
        let a = self.node_a;
        let b = self.node_b;
        let edge = v3_sub(nodes[b].position, nodes[a].position);
        let current_length = v3_length(edge);
        if current_length < 1e-15 {
            return 0.0;
        }

        let c = current_length - self.rest_length;
        let n = v3_scale(edge, 1.0 / current_length);

        let w_a = nodes[a].inv_mass;
        let w_b = nodes[b].inv_mass;
        let w_sum = w_a + w_b;
        if w_sum < 1e-30 {
            return 0.0;
        }

        let alpha_tilde = self.stretch_compliance / (dt * dt);
        let denom = w_sum + alpha_tilde;
        if denom.abs() < 1e-30 {
            return 0.0;
        }

        let delta_lambda = (-c - alpha_tilde * self.lambda_stretch) / denom;
        self.lambda_stretch += delta_lambda;

        let correction_a = v3_scale(n, -delta_lambda * w_a);
        let correction_b = v3_scale(n, delta_lambda * w_b);
        nodes[a].position = v3_add(nodes[a].position, correction_a);
        nodes[b].position = v3_add(nodes[b].position, correction_b);

        delta_lambda
    }

    /// Solve the twist part of this constraint (orientation correction).
    ///
    /// Adjusts the orientations of the two nodes to reduce twist deviation.
    pub fn solve_twist(
        &mut self,
        nodes: &mut [HairNode],
        dt: f64,
    ) -> f64 {
        let a = self.node_a;
        let b = self.node_b;

        let dq = quat_mul(quat_conj(nodes[a].orientation), nodes[b].orientation);
        let current_twist = extract_twist_angle(dq);
        let c = current_twist - self.rest_twist;

        let w_a = nodes[a].inv_inertia;
        let w_b = nodes[b].inv_inertia;
        let w_sum = w_a + w_b;
        if w_sum < 1e-30 {
            return 0.0;
        }

        let alpha_tilde = self.twist_compliance / (dt * dt);
        let denom = w_sum + alpha_tilde;
        if denom.abs() < 1e-30 {
            return 0.0;
        }

        let delta_lambda = (-c - alpha_tilde * self.lambda_twist) / denom;
        self.lambda_twist += delta_lambda;

        // Apply angular correction around the edge direction.
        let edge = v3_sub(nodes[b].position, nodes[a].position);
        let axis = v3_normalize(edge);
        if v3_length(axis) < 1e-15 {
            return delta_lambda;
        }

        let angle_a = -delta_lambda * w_a * 0.5;
        let angle_b = delta_lambda * w_b * 0.5;

        apply_twist_correction(&mut nodes[a], axis, angle_a);
        apply_twist_correction(&mut nodes[b], axis, angle_b);

        delta_lambda
    }

    /// Reset accumulated lambdas at the start of a time step.
    pub fn reset_lambdas(&mut self) {
        self.lambda_stretch = 0.0;
        self.lambda_twist = 0.0;
    }
}

/// Apply a small twist rotation around `axis` to a node's orientation.
fn apply_twist_correction(node: &mut HairNode, axis: [f64; 3], half_angle: f64) {
    if half_angle.abs() < 1e-20 {
        return;
    }
    let sin_ha = half_angle.sin();
    let cos_ha = half_angle.cos();
    let dq = [axis[0] * sin_ha, axis[1] * sin_ha, axis[2] * sin_ha, cos_ha];
    node.orientation = quat_normalize(quat_mul(dq, node.orientation));
}

// ── Bend-twist constraint ───────────────────────────────────────────────────

/// Bend-twist constraint between two adjacent edges (three consecutive nodes).
///
/// This constrains the Darboux vector (curvature + twist) between the material
/// frames at node i and node i+1 to stay close to the rest Darboux vector.
/// This couples bending in two directions (curvature_x, curvature_y) and
/// twist (curvature_z).
#[derive(Debug, Clone)]
pub struct BendTwistConstraint {
    /// First node index.
    pub node_a: usize,
    /// Second node index.
    pub node_b: usize,
    /// Rest Darboux vector `[kappa_x, kappa_y, tau]`.
    pub rest_darboux: [f64; 3],
    /// Compliance for bend (kappa_x, kappa_y).
    pub bend_compliance: f64,
    /// Compliance for twist (tau).
    pub twist_compliance: f64,
    /// Accumulated Lagrange multipliers `[lx, ly, lz]`.
    pub lambda: [f64; 3],
}

impl BendTwistConstraint {
    /// Create a new bend-twist constraint.
    pub fn new(
        node_a: usize,
        node_b: usize,
        rest_darboux: [f64; 3],
        bend_stiffness: f64,
        twist_stiffness: f64,
    ) -> Self {
        let bend_compliance = if bend_stiffness > 1e-15 {
            1.0 / bend_stiffness
        } else {
            1e10
        };
        let twist_compliance = if twist_stiffness > 1e-15 {
            1.0 / twist_stiffness
        } else {
            1e10
        };
        Self {
            node_a,
            node_b,
            rest_darboux,
            bend_compliance,
            twist_compliance,
            lambda: [0.0; 3],
        }
    }

    /// Build all bend-twist constraints for a strand.
    pub fn build_for_strand(
        strand: &HairStrand,
        bend_stiffness: f64,
        twist_stiffness: f64,
    ) -> Vec<Self> {
        let seg = strand.segment_count();
        let mut out = Vec::with_capacity(seg);
        for i in 0..seg {
            out.push(Self::new(
                i,
                i + 1,
                strand.rest_curvatures[i],
                bend_stiffness,
                twist_stiffness,
            ));
        }
        out
    }

    /// Solve this bend-twist constraint.
    ///
    /// Computes the current Darboux vector from the two node orientations,
    /// measures the deviation from rest, and applies angular corrections.
    pub fn solve(
        &mut self,
        nodes: &mut [HairNode],
        dt: f64,
    ) -> [f64; 3] {
        let a = self.node_a;
        let b = self.node_b;

        let dq = quat_mul(quat_conj(nodes[a].orientation), nodes[b].orientation);
        let current_darboux = extract_bend_angles(dq);

        // Error vector.
        let error = [
            current_darboux[0] - self.rest_darboux[0],
            current_darboux[1] - self.rest_darboux[1],
            current_darboux[2] - self.rest_darboux[2],
        ];

        let w_a = nodes[a].inv_inertia;
        let w_b = nodes[b].inv_inertia;
        let w_sum = w_a + w_b;
        if w_sum < 1e-30 {
            return [0.0; 3];
        }

        let dt_sq = dt * dt;
        let mut delta_lambda = [0.0_f64; 3];

        // Solve each component (bend_x, bend_y, twist) independently.
        for k in 0..3 {
            let compliance = if k < 2 {
                self.bend_compliance
            } else {
                self.twist_compliance
            };
            let alpha_tilde = compliance / dt_sq;
            let denom = w_sum + alpha_tilde;
            if denom.abs() < 1e-30 {
                continue;
            }
            delta_lambda[k] = (-error[k] - alpha_tilde * self.lambda[k]) / denom;
            self.lambda[k] += delta_lambda[k];
        }

        // Apply angular corrections to both nodes.
        // The correction axis in body-frame corresponds to the Darboux components.
        let correction_a = [
            -delta_lambda[0] * w_a * 0.5,
            -delta_lambda[1] * w_a * 0.5,
            -delta_lambda[2] * w_a * 0.5,
        ];
        let correction_b = [
            delta_lambda[0] * w_b * 0.5,
            delta_lambda[1] * w_b * 0.5,
            delta_lambda[2] * w_b * 0.5,
        ];

        apply_angular_correction(&mut nodes[a], correction_a);
        apply_angular_correction(&mut nodes[b], correction_b);

        delta_lambda
    }

    /// Reset accumulated lambdas.
    pub fn reset_lambdas(&mut self) {
        self.lambda = [0.0; 3];
    }
}

/// Apply a small angular correction (body-frame axis-angle) to a node.
fn apply_angular_correction(node: &mut HairNode, body_axis_angle: [f64; 3]) {
    let angle = v3_length(body_axis_angle);
    if angle < 1e-20 {
        return;
    }
    let axis = v3_scale(body_axis_angle, 1.0 / angle);
    // Convert body-frame axis to world-frame.
    let world_axis = quat_rotate(node.orientation, axis);
    let sin_ha = (angle * 0.5).sin();
    let cos_ha = (angle * 0.5).cos();
    let dq = [
        world_axis[0] * sin_ha,
        world_axis[1] * sin_ha,
        world_axis[2] * sin_ha,
        cos_ha,
    ];
    node.orientation = quat_normalize(quat_mul(dq, node.orientation));
}

// ── Length preservation constraint ──────────────────────────────────────────

/// A simple inextensibility (length) constraint for fallback use.
///
/// This is a simpler version of the stretch constraint that ignores twist,
/// useful as an additional correction pass.
pub fn solve_length_constraint(
    nodes: &mut [HairNode],
    idx_a: usize,
    idx_b: usize,
    rest_length: f64,
    compliance: f64,
    dt: f64,
    accumulated_lambda: &mut f64,
) -> f64 {
    let edge = v3_sub(nodes[idx_b].position, nodes[idx_a].position);
    let current_length = v3_length(edge);
    if current_length < 1e-15 {
        return 0.0;
    }

    let c = current_length - rest_length;
    let n = v3_scale(edge, 1.0 / current_length);

    let w_a = nodes[idx_a].inv_mass;
    let w_b = nodes[idx_b].inv_mass;
    let w_sum = w_a + w_b;
    if w_sum < 1e-30 {
        return 0.0;
    }

    let alpha_tilde = compliance / (dt * dt);
    let denom = w_sum + alpha_tilde;
    if denom.abs() < 1e-30 {
        return 0.0;
    }

    let delta_lambda = (-c - alpha_tilde * *accumulated_lambda) / denom;
    *accumulated_lambda += delta_lambda;

    nodes[idx_a].position = v3_add(nodes[idx_a].position, v3_scale(n, -delta_lambda * w_a));
    nodes[idx_b].position = v3_add(nodes[idx_b].position, v3_scale(n, delta_lambda * w_b));

    delta_lambda
}

// ── Shape matching ──────────────────────────────────────────────────────────

/// Apply shape-matching correction to pull nodes toward their rest-pose
/// configuration, transformed by the current best-fit rigid transformation
/// of the root.
///
/// `stiffness` ∈ [0, 1] controls how strongly nodes are attracted back.
/// This uses a simplified approach: the root orientation defines the
/// reference frame, and each node is blended toward the rest position
/// rotated into the current root frame.
pub fn apply_shape_matching(strand: &mut HairStrand, stiffness: f64) {
    if stiffness < 1e-15 || strand.nodes.is_empty() {
        return;
    }
    let stiffness = stiffness.clamp(0.0, 1.0);

    // Use root node's current orientation and position as the rigid transform.
    let root_pos = strand.nodes[0].position;
    let root_orient = strand.nodes[0].orientation;

    // The rest pose root.
    let rest_root_pos = strand.rest_positions.get(0).copied().unwrap_or(root_pos);
    let rest_root_orient = strand.rest_orientations.get(0).copied().unwrap_or(root_orient);

    // Relative rotation from rest root frame to current root frame.
    let dq = quat_mul(root_orient, quat_conj(rest_root_orient));

    for i in 1..strand.nodes.len() {
        if strand.nodes[i].inv_mass < 1e-30 {
            continue; // skip pinned nodes
        }
        let rest_local = v3_sub(
            strand.rest_positions.get(i).copied().unwrap_or([0.0; 3]),
            rest_root_pos,
        );
        let target_local = quat_rotate(dq, rest_local);
        let target = v3_add(root_pos, target_local);

        // Blend toward target.
        strand.nodes[i].position = v3_add(
            v3_scale(strand.nodes[i].position, 1.0 - stiffness),
            v3_scale(target, stiffness),
        );

        // Also blend orientation toward rest (rotated).
        let target_orient = quat_normalize(quat_mul(
            dq,
            strand.rest_orientations.get(i).copied().unwrap_or(strand.nodes[i].orientation),
        ));
        strand.nodes[i].orientation = slerp(
            strand.nodes[i].orientation,
            target_orient,
            stiffness,
        );
    }
}

/// Spherical linear interpolation between two quaternions.
fn slerp(a: [f64; 4], b: [f64; 4], t: f64) -> [f64; 4] {
    let mut dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    let b = if dot < 0.0 {
        dot = -dot;
        [-b[0], -b[1], -b[2], -b[3]]
    } else {
        b
    };

    if dot > 0.9995 {
        // Close enough — use nlerp.
        let r = [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
            a[3] + (b[3] - a[3]) * t,
        ];
        return quat_normalize(r);
    }

    let theta = dot.clamp(-1.0, 1.0).acos();
    let sin_theta = theta.sin();
    if sin_theta.abs() < 1e-15 {
        return a;
    }
    let s0 = ((1.0 - t) * theta).sin() / sin_theta;
    let s1 = (t * theta).sin() / sin_theta;
    quat_normalize([
        a[0] * s0 + b[0] * s1,
        a[1] * s0 + b[1] * s1,
        a[2] * s0 + b[2] * s1,
        a[3] * s0 + b[3] * s1,
    ])
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_strand() -> HairStrand {
        HairStrand::new([0.0, 1.0, 0.0], [0.0, -1.0, 0.0], 0.5, 5, 0.001)
    }

    #[test]
    fn test_stretch_twist_build() {
        let strand = make_test_strand();
        let constraints =
            StretchTwistConstraint::build_for_strand(&strand, 1.0, 0.5);
        assert_eq!(constraints.len(), 5);
    }

    #[test]
    fn test_stretch_solve_converges() {
        let mut strand = make_test_strand();
        // Perturb a node.
        strand.nodes[3].position[1] -= 0.05;

        let mut constraints =
            StretchTwistConstraint::build_for_strand(&strand, 1.0, 0.5);
        let dt = 1.0 / 60.0;

        for _ in 0..20 {
            for c in &mut constraints {
                c.solve_stretch(&mut strand.nodes, dt);
            }
        }

        // Check lengths are close to rest.
        for (i, c) in constraints.iter().enumerate() {
            let edge = v3_sub(
                strand.nodes[c.node_b].position,
                strand.nodes[c.node_a].position,
            );
            let len = v3_length(edge);
            let err = (len - strand.rest_lengths[i]).abs();
            assert!(err < 0.01, "segment {i} length error: {err}");
        }
    }

    #[test]
    fn test_bend_twist_build() {
        let strand = make_test_strand();
        let constraints = BendTwistConstraint::build_for_strand(&strand, 0.5, 0.3);
        assert_eq!(constraints.len(), 5);
    }

    #[test]
    fn test_length_constraint() {
        let mut strand = make_test_strand();
        strand.nodes[2].position[0] += 0.1;
        let mut lambda = 0.0;
        let dt = 1.0 / 60.0;
        for _ in 0..50 {
            solve_length_constraint(
                &mut strand.nodes,
                1,
                2,
                strand.rest_lengths[1],
                0.0, // zero compliance = infinite stiffness
                dt,
                &mut lambda,
            );
        }
        let edge = v3_sub(strand.nodes[2].position, strand.nodes[1].position);
        let len = v3_length(edge);
        let err = (len - strand.rest_lengths[1]).abs();
        assert!(err < 0.001, "length error: {err}");
    }

    #[test]
    fn test_shape_matching() {
        let mut strand = make_test_strand();
        // Perturb nodes.
        for node in &mut strand.nodes[1..] {
            node.position[0] += 0.1;
        }
        apply_shape_matching(&mut strand, 0.5);
        // Nodes should have moved back toward rest.
        for node in &strand.nodes[1..] {
            assert!(node.position[0].abs() < 0.1);
        }
    }

    #[test]
    fn test_slerp_endpoints() {
        let a = [0.0, 0.0, 0.0, 1.0];
        let b = [0.0, 0.0, 0.3826834, 0.9238795]; // ~45 deg around z
        let r0 = slerp(a, b, 0.0);
        let r1 = slerp(a, b, 1.0);
        for i in 0..4 {
            assert!((r0[i] - a[i]).abs() < 1e-6);
            assert!((r1[i] - b[i]).abs() < 1e-4);
        }
    }
}
