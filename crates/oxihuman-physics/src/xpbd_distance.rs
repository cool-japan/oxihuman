// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct XPBDDistanceConstraint {
    pub pa: usize,
    pub pb: usize,
    pub rest_length: f32,
    pub compliance: f32,
}

#[allow(dead_code)]
pub fn new_xpbd_distance_constraint(pa: usize, pb: usize, rest_length: f32, compliance: f32) -> XPBDDistanceConstraint {
    XPBDDistanceConstraint { pa, pb, rest_length, compliance }
}

#[allow(dead_code)]
pub fn xpbd_dist_project(c: &XPBDDistanceConstraint, positions: &mut [[f32; 3]], inv_masses: &[f32], dt: f32) {
    let pa = c.pa;
    let pb = c.pb;
    let dx = positions[pb][0] - positions[pa][0];
    let dy = positions[pb][1] - positions[pa][1];
    let dz = positions[pb][2] - positions[pa][2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    if dist < 1e-10 {
        return;
    }
    let constraint = dist - c.rest_length;
    let w_a = if pa < inv_masses.len() { inv_masses[pa] } else { 0.0 };
    let w_b = if pb < inv_masses.len() { inv_masses[pb] } else { 0.0 };
    let w_sum = w_a + w_b;
    if w_sum < 1e-10 {
        return;
    }
    let alpha = c.compliance / (dt * dt);
    let lambda = -constraint / (w_sum + alpha);
    let nx = dx / dist;
    let ny = dy / dist;
    let nz = dz / dist;
    positions[pa][0] -= w_a * lambda * nx;
    positions[pa][1] -= w_a * lambda * ny;
    positions[pa][2] -= w_a * lambda * nz;
    positions[pb][0] += w_b * lambda * nx;
    positions[pb][1] += w_b * lambda * ny;
    positions[pb][2] += w_b * lambda * nz;
}

#[allow(dead_code)]
pub fn xpbd_dist_residual(c: &XPBDDistanceConstraint, positions: &[[f32; 3]]) -> f32 {
    let pa = &positions[c.pa];
    let pb = &positions[c.pb];
    let dx = pb[0] - pa[0];
    let dy = pb[1] - pa[1];
    let dz = pb[2] - pa[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    (dist - c.rest_length).abs()
}

#[allow(dead_code)]
pub fn xpbd_dist_rest_length(c: &XPBDDistanceConstraint) -> f32 {
    c.rest_length
}

#[allow(dead_code)]
pub fn xpbd_dist_compliance(c: &XPBDDistanceConstraint) -> f32 {
    c.compliance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let c = new_xpbd_distance_constraint(0, 1, 1.0, 0.0);
        assert_eq!(c.pa, 0);
        assert_eq!(c.pb, 1);
    }

    #[test]
    fn test_rest_length() {
        let c = new_xpbd_distance_constraint(0, 1, 2.5, 0.0);
        assert!((xpbd_dist_rest_length(&c) - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_compliance() {
        let c = new_xpbd_distance_constraint(0, 1, 1.0, 0.01);
        assert!((xpbd_dist_compliance(&c) - 0.01).abs() < 1e-7);
    }

    #[test]
    fn test_residual_at_rest() {
        let c = new_xpbd_distance_constraint(0, 1, 1.0, 0.0);
        let positions = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let r = xpbd_dist_residual(&c, &positions);
        assert!(r < 1e-5);
    }

    #[test]
    fn test_residual_stretched() {
        let c = new_xpbd_distance_constraint(0, 1, 1.0, 0.0);
        let positions = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let r = xpbd_dist_residual(&c, &positions);
        assert!((r - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_project_no_crash() {
        let c = new_xpbd_distance_constraint(0, 1, 1.0, 0.0);
        let mut positions = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let inv_masses = [1.0, 1.0];
        xpbd_dist_project(&c, &mut positions, &inv_masses, 0.016);
        assert!(positions[0][0].is_finite());
    }

    #[test]
    fn test_project_reduces_residual() {
        let c = new_xpbd_distance_constraint(0, 1, 1.0, 0.0);
        let mut positions = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let inv_masses = [1.0, 1.0];
        let before = xpbd_dist_residual(&c, &positions);
        xpbd_dist_project(&c, &mut positions, &inv_masses, 0.016);
        let after = xpbd_dist_residual(&c, &positions);
        assert!(after < before);
    }

    #[test]
    fn test_project_pinned_particle() {
        let c = new_xpbd_distance_constraint(0, 1, 1.0, 0.0);
        let mut positions = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let inv_masses = [0.0, 1.0]; /* pin particle 0 */
        xpbd_dist_project(&c, &mut positions, &inv_masses, 0.016);
        assert!((positions[0][0]).abs() < 1e-6);
    }
}
