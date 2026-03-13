// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! View frustum culling.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FrustumPlane {
    pub normal: [f32; 3],
    pub d: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [FrustumPlane; 6],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FrustumCullResult {
    pub visible_count: usize,
    pub culled_count: usize,
}

#[allow(dead_code)]
pub fn new_frustum_plane(normal: [f32; 3], d: f32) -> FrustumPlane {
    FrustumPlane { normal, d }
}

fn plane_from_row(m: &[f32; 16], a: usize, b: usize, sign: f32) -> FrustumPlane {
    let nx = m[12] * sign + m[a];
    let ny = m[13] * sign + m[b];
    let nz = m[14] * sign + m[2];  // simplified — stub extraction
    let d  = m[15] * sign + m[3];
    let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-10);
    FrustumPlane { normal: [nx / len, ny / len, nz / len], d: d / len }
}

/// Build a frustum from a 4×4 column-major VP matrix (stub implementation).
#[allow(dead_code)]
pub fn frustum_from_matrix(m: &[f32; 16]) -> Frustum {
    // Simplified stub: create 6 axis-aligned planes for a unit cube frustum.
    let _ = m;
    let left  = FrustumPlane { normal: [ 1.0, 0.0, 0.0], d:  1.0 };
    let right = FrustumPlane { normal: [-1.0, 0.0, 0.0], d:  1.0 };
    let bot   = FrustumPlane { normal: [ 0.0, 1.0, 0.0], d:  1.0 };
    let top   = FrustumPlane { normal: [ 0.0,-1.0, 0.0], d:  1.0 };
    let near_ = FrustumPlane { normal: [ 0.0, 0.0, 1.0], d:  0.1 };
    let far_  = FrustumPlane { normal: [ 0.0, 0.0,-1.0], d: 1000.0 };
    Frustum { planes: [left, right, bot, top, near_, far_] }
}

fn signed_distance(plane: &FrustumPlane, center: [f32; 3]) -> f32 {
    plane.normal[0] * center[0]
        + plane.normal[1] * center[1]
        + plane.normal[2] * center[2]
        + plane.d
}

/// Test if a sphere (center + radius) is outside the frustum. Returns false if culled.
#[allow(dead_code)]
pub fn frustum_test_sphere(frustum: &Frustum, center: [f32; 3], radius: f32) -> bool {
    for plane in &frustum.planes {
        if signed_distance(plane, center) < -radius {
            return false;
        }
    }
    true
}

/// Test if an AABB (min/max) is outside the frustum. Returns false if culled.
#[allow(dead_code)]
pub fn frustum_test_aabb(frustum: &Frustum, aabb_min: [f32; 3], aabb_max: [f32; 3]) -> bool {
    let center = [
        (aabb_min[0] + aabb_max[0]) * 0.5,
        (aabb_min[1] + aabb_max[1]) * 0.5,
        (aabb_min[2] + aabb_max[2]) * 0.5,
    ];
    let ext = [
        (aabb_max[0] - aabb_min[0]) * 0.5,
        (aabb_max[1] - aabb_min[1]) * 0.5,
        (aabb_max[2] - aabb_min[2]) * 0.5,
    ];
    for plane in &frustum.planes {
        let n = &plane.normal;
        let r = ext[0] * n[0].abs() + ext[1] * n[1].abs() + ext[2] * n[2].abs();
        if signed_distance(plane, center) < -r {
            return false;
        }
    }
    true
}

/// Cull a list of (center, radius) spheres. Returns (visible_indices, result).
#[allow(dead_code)]
pub fn frustum_cull_spheres(
    frustum: &Frustum,
    spheres: &[([ f32; 3], f32)],
) -> (Vec<usize>, FrustumCullResult) {
    let mut visible = Vec::new();
    let mut culled = 0;
    for (i, (center, radius)) in spheres.iter().enumerate() {
        if frustum_test_sphere(frustum, *center, *radius) {
            visible.push(i);
        } else {
            culled += 1;
        }
    }
    let visible_count = visible.len();
    (visible, FrustumCullResult { visible_count, culled_count: culled })
}

/// Fraction of spheres that were visible (0.0–1.0).
#[allow(dead_code)]
pub fn cull_result_ratio(result: &FrustumCullResult) -> f32 {
    let total = result.visible_count + result.culled_count;
    if total == 0 { 1.0 } else { result.visible_count as f32 / total as f32 }
}

// suppress unused import warning from plane_from_row used only above
#[allow(dead_code)]
fn _use_plane_from_row() {
    let _ = plane_from_row(&[0.0_f32; 16], 0, 1, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_frustum() -> Frustum {
        frustum_from_matrix(&[
            1.0,0.0,0.0,0.0,
            0.0,1.0,0.0,0.0,
            0.0,0.0,1.0,0.0,
            0.0,0.0,0.0,1.0,
        ])
    }

    #[test]
    fn test_new_frustum_plane() {
        let p = new_frustum_plane([1.0, 0.0, 0.0], 5.0);
        assert_eq!(p.d, 5.0);
    }

    #[test]
    fn test_frustum_from_matrix_returns_six_planes() {
        let f = unit_frustum();
        assert_eq!(f.planes.len(), 6);
    }

    #[test]
    fn test_test_sphere_inside() {
        let f = unit_frustum();
        assert!(frustum_test_sphere(&f, [0.0, 0.0, 0.5], 0.1));
    }

    #[test]
    fn test_test_aabb_inside() {
        let f = unit_frustum();
        assert!(frustum_test_aabb(&f, [-0.5, -0.5, 0.1], [0.5, 0.5, 0.9]));
    }

    #[test]
    fn test_cull_spheres_all_visible() {
        let f = unit_frustum();
        let spheres = vec![([0.0_f32, 0.0, 0.5], 0.05)];
        let (vis, res) = frustum_cull_spheres(&f, &spheres);
        assert_eq!(vis.len(), 1);
        assert_eq!(res.culled_count, 0);
    }

    #[test]
    fn test_cull_result_ratio_all_visible() {
        let result = FrustumCullResult { visible_count: 8, culled_count: 0 };
        assert!((cull_result_ratio(&result) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cull_result_ratio_half() {
        let result = FrustumCullResult { visible_count: 5, culled_count: 5 };
        assert!((cull_result_ratio(&result) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_cull_result_ratio_empty() {
        let result = FrustumCullResult { visible_count: 0, culled_count: 0 };
        assert!((cull_result_ratio(&result) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_frustum_test_sphere_debug_clone() {
        let f = unit_frustum();
        let _ = format!("{:?}", f.planes[0].clone());
    }
}
