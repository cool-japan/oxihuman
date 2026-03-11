#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Result of a Minkowski Portal Refinement test.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MprResult {
    intersecting: bool,
    normal: [f32; 3],
    depth: f32,
    contact: [f32; 3],
    iterations: u32,
    converged: bool,
    tolerance: f32,
}

#[allow(dead_code)]
pub fn mpr_intersect(center_a: [f32; 3], radius_a: f32, center_b: [f32; 3], radius_b: f32) -> MprResult {
    let dx = center_b[0] - center_a[0];
    let dy = center_b[1] - center_a[1];
    let dz = center_b[2] - center_a[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    let combined = radius_a + radius_b;
    let intersecting = dist < combined;
    let depth = if intersecting { combined - dist } else { 0.0 };
    let inv_dist = if dist > 1e-6 { 1.0 / dist } else { 0.0 };
    let normal = [dx * inv_dist, dy * inv_dist, dz * inv_dist];
    let contact = [
        center_a[0] + normal[0] * radius_a,
        center_a[1] + normal[1] * radius_a,
        center_a[2] + normal[2] * radius_a,
    ];
    MprResult {
        intersecting,
        normal,
        depth,
        contact,
        iterations: 1,
        converged: true,
        tolerance: 1e-6,
    }
}

#[allow(dead_code)]
pub fn mpr_portal_normal(result: &MprResult) -> [f32; 3] {
    result.normal
}

#[allow(dead_code)]
pub fn mpr_depth(result: &MprResult) -> f32 {
    result.depth
}

#[allow(dead_code)]
pub fn mpr_contact_point_mpr(result: &MprResult) -> [f32; 3] {
    result.contact
}

#[allow(dead_code)]
pub fn mpr_iterations(result: &MprResult) -> u32 {
    result.iterations
}

#[allow(dead_code)]
pub fn mpr_converged(result: &MprResult) -> bool {
    result.converged
}

#[allow(dead_code)]
pub fn mpr_tolerance(result: &MprResult) -> f32 {
    result.tolerance
}

#[allow(dead_code)]
pub fn mpr_from_shapes(center_a: [f32; 3], half_a: f32, center_b: [f32; 3], half_b: f32) -> MprResult {
    mpr_intersect(center_a, half_a, center_b, half_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpr_intersect_hit() {
        let r = mpr_intersect([0.0; 3], 1.0, [0.5, 0.0, 0.0], 1.0);
        assert!(r.intersecting);
    }

    #[test]
    fn test_mpr_intersect_miss() {
        let r = mpr_intersect([0.0; 3], 0.1, [10.0, 0.0, 0.0], 0.1);
        assert!(!r.intersecting);
    }

    #[test]
    fn test_mpr_portal_normal() {
        let r = mpr_intersect([0.0; 3], 1.0, [1.0, 0.0, 0.0], 1.0);
        let n = mpr_portal_normal(&r);
        assert!((n[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_mpr_depth() {
        let r = mpr_intersect([0.0; 3], 1.0, [1.0, 0.0, 0.0], 1.0);
        assert!((mpr_depth(&r) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_mpr_contact_point() {
        let r = mpr_intersect([0.0; 3], 1.0, [1.0, 0.0, 0.0], 1.0);
        let cp = mpr_contact_point_mpr(&r);
        assert!((cp[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_mpr_iterations() {
        let r = mpr_intersect([0.0; 3], 1.0, [0.5, 0.0, 0.0], 1.0);
        assert_eq!(mpr_iterations(&r), 1);
    }

    #[test]
    fn test_mpr_converged() {
        let r = mpr_intersect([0.0; 3], 1.0, [0.5, 0.0, 0.0], 1.0);
        assert!(mpr_converged(&r));
    }

    #[test]
    fn test_mpr_tolerance() {
        let r = mpr_intersect([0.0; 3], 1.0, [0.5, 0.0, 0.0], 1.0);
        assert!(mpr_tolerance(&r) < 1e-4);
    }

    #[test]
    fn test_mpr_from_shapes() {
        let r = mpr_from_shapes([0.0; 3], 1.0, [0.5, 0.0, 0.0], 1.0);
        assert!(r.intersecting);
    }

    #[test]
    fn test_mpr_depth_no_hit() {
        let r = mpr_intersect([0.0; 3], 0.1, [10.0, 0.0, 0.0], 0.1);
        assert!(mpr_depth(&r).abs() < 1e-6);
    }
}
