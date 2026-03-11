// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cloth constraint: distance and bend constraints for PBD cloth simulation.

use std::f32::consts::PI;

/// A distance constraint between two particles.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClothDistConstraint {
    pub i: usize,
    pub j: usize,
    pub rest_length: f32,
    pub compliance: f32,
}

/// A bend (dihedral) constraint between four particles.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClothBendConstraint {
    pub i: usize,
    pub j: usize,
    pub k: usize,
    pub l: usize,
    pub rest_angle: f32,
    pub compliance: f32,
}

/// Create a distance constraint.
#[allow(dead_code)]
pub fn new_dist_constraint(
    i: usize,
    j: usize,
    rest_length: f32,
    compliance: f32,
) -> ClothDistConstraint {
    ClothDistConstraint {
        i,
        j,
        rest_length,
        compliance,
    }
}

/// Create a bend constraint.
#[allow(dead_code)]
pub fn new_bend_constraint(
    i: usize,
    j: usize,
    k: usize,
    l: usize,
    rest_angle: f32,
    compliance: f32,
) -> ClothBendConstraint {
    ClothBendConstraint {
        i,
        j,
        k,
        l,
        rest_angle,
        compliance,
    }
}

/// Compute distance constraint correction for particle i and j.
/// Returns (delta_i, delta_j) in the direction of the segment.
#[allow(dead_code)]
pub fn dist_constraint_delta(
    c: &ClothDistConstraint,
    pi: [f32; 3],
    pj: [f32; 3],
    wi: f32,
    wj: f32,
    dt: f32,
) -> ([f32; 3], [f32; 3]) {
    let dx = [pj[0] - pi[0], pj[1] - pi[1], pj[2] - pi[2]];
    let dist = (dx[0] * dx[0] + dx[1] * dx[1] + dx[2] * dx[2]).sqrt();
    if dist < 1e-12 {
        return ([0.0; 3], [0.0; 3]);
    }
    let constraint = dist - c.rest_length;
    let alpha = c.compliance / (dt * dt);
    let w_sum = wi + wj;
    if w_sum < 1e-12 {
        return ([0.0; 3], [0.0; 3]);
    }
    let lambda = -constraint / (w_sum + alpha);
    let n = [dx[0] / dist, dx[1] / dist, dx[2] / dist];
    let di = [
        -wi * lambda * n[0],
        -wi * lambda * n[1],
        -wi * lambda * n[2],
    ];
    let dj = [wj * lambda * n[0], wj * lambda * n[1], wj * lambda * n[2]];
    (di, dj)
}

/// Constraint violation (stretch ratio − 1).
#[allow(dead_code)]
pub fn dist_constraint_violation(c: &ClothDistConstraint, pi: [f32; 3], pj: [f32; 3]) -> f32 {
    let dx = [pj[0] - pi[0], pj[1] - pi[1], pj[2] - pi[2]];
    let dist = (dx[0] * dx[0] + dx[1] * dx[1] + dx[2] * dx[2]).sqrt();
    dist - c.rest_length
}

/// Whether a distance constraint is satisfied within tolerance.
#[allow(dead_code)]
pub fn dist_satisfied(c: &ClothDistConstraint, pi: [f32; 3], pj: [f32; 3], tol: f32) -> bool {
    dist_constraint_violation(c, pi, pj).abs() < tol
}

/// Constraint stiffness from compliance and timestep.
#[allow(dead_code)]
pub fn stiffness_from_compliance(compliance: f32, dt: f32) -> f32 {
    if compliance < 1e-12 {
        return f32::MAX;
    }
    dt * dt / compliance
}

/// Bend angle between two triangles sharing edge (j,k), using normals.
#[allow(dead_code)]
pub fn bend_angle(pi: [f32; 3], pj: [f32; 3], pk: [f32; 3], pl: [f32; 3]) -> f32 {
    let cross = |a: [f32; 3], b: [f32; 3]| -> [f32; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let norm = |v: [f32; 3]| -> [f32; 3] {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l < 1e-12 {
            return [0.0, 0.0, 1.0];
        }
        [v[0] / l, v[1] / l, v[2] / l]
    };
    let dotf = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let jk = [pk[0] - pj[0], pk[1] - pj[1], pk[2] - pj[2]];
    let ji = [pi[0] - pj[0], pi[1] - pj[1], pi[2] - pj[2]];
    let jl = [pl[0] - pj[0], pl[1] - pj[1], pl[2] - pj[2]];
    let n1 = norm(cross(jk, ji));
    let n2 = norm(cross(jk, jl));
    dotf(n1, n2).clamp(-1.0, 1.0).acos()
}

/// Rest angle for a flat sheet.
#[allow(dead_code)]
pub fn flat_rest_angle() -> f32 {
    PI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dist_constraint_violation_zero() {
        let c = new_dist_constraint(0, 1, 2.0, 0.0);
        let pi = [0.0, 0.0, 0.0];
        let pj = [2.0, 0.0, 0.0];
        assert!(dist_constraint_violation(&c, pi, pj).abs() < 1e-5);
    }

    #[test]
    fn test_dist_constraint_violation_positive() {
        let c = new_dist_constraint(0, 1, 1.0, 0.0);
        let pi = [0.0, 0.0, 0.0];
        let pj = [3.0, 0.0, 0.0];
        assert!((dist_constraint_violation(&c, pi, pj) - 2.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_dist_satisfied() {
        let c = new_dist_constraint(0, 1, 1.0, 0.0);
        let pi = [0.0, 0.0, 0.0];
        let pj = [1.0, 0.0, 0.0];
        assert!(dist_satisfied(&c, pi, pj, 0.01));
    }

    #[test]
    fn test_delta_direction() {
        let c = new_dist_constraint(0, 1, 1.0, 0.001);
        let pi = [0.0, 0.0, 0.0];
        let pj = [2.0, 0.0, 0.0];
        let (di, dj) = dist_constraint_delta(&c, pi, pj, 1.0, 1.0, 0.01);
        // i should move toward j (positive x), j should move toward i (negative x)
        assert!(di[0] > 0.0);
        assert!(dj[0] < 0.0);
    }

    #[test]
    fn test_stiffness_from_compliance() {
        let s = stiffness_from_compliance(0.01, 0.1);
        assert!((s - 1.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_zero_compliance_infinite_stiffness() {
        let s = stiffness_from_compliance(0.0, 0.1);
        assert_eq!(s, f32::MAX);
    }

    #[test]
    fn test_flat_rest_angle() {
        assert!((flat_rest_angle() - PI).abs() < 1e-5);
    }

    #[test]
    fn test_bend_angle_flat() {
        // Flat configuration: all in XY plane
        let pi = [0.0, 1.0, 0.0];
        let pj = [0.0, 0.0, 0.0];
        let pk = [1.0, 0.0, 0.0];
        let pl = [0.0, -1.0, 0.0];
        let angle = bend_angle(pi, pj, pk, pl);
        assert!(angle > 0.0 && angle <= PI + 1e-4);
    }

    #[test]
    fn test_delta_equal_weights() {
        let c = new_dist_constraint(0, 1, 1.0, 0.0);
        let pi = [0.0, 0.0, 0.0];
        let pj = [3.0, 0.0, 0.0];
        let (di, dj) = dist_constraint_delta(&c, pi, pj, 0.5, 0.5, 0.1);
        // Both corrections equal magnitude
        assert!((di[0].abs() - dj[0].abs()).abs() < 1e-5);
    }
}
