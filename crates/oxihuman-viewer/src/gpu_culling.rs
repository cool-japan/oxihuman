// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! GPU frustum culling stub.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [[f32; 4]; 6],
}

#[allow(dead_code)]
pub fn new_frustum_from_vp(fov_y: f32, aspect: f32, near: f32, far: f32) -> Frustum {
    /* stub: generate simple axis-aligned planes from near/far */
    let _ = fov_y;
    let _ = aspect;
    Frustum {
        planes: [
            [0.0, 0.0, 1.0, near],   /* near */
            [0.0, 0.0, -1.0, far],   /* far */
            [1.0, 0.0, 0.0, 1.0],    /* left */
            [-1.0, 0.0, 0.0, 1.0],   /* right */
            [0.0, 1.0, 0.0, 1.0],    /* bottom */
            [0.0, -1.0, 0.0, 1.0],   /* top */
        ],
    }
}

fn plane_dist(plane: [f32; 4], center: [f32; 3]) -> f32 {
    plane[0] * center[0] + plane[1] * center[1] + plane[2] * center[2] + plane[3]
}

#[allow(dead_code)]
pub fn fc_test_sphere(frustum: &Frustum, center: [f32; 3], radius: f32) -> bool {
    for plane in &frustum.planes {
        if plane_dist(*plane, center) < -radius {
            return false;
        }
    }
    true
}

#[allow(dead_code)]
pub fn fc_test_aabb(frustum: &Frustum, min: [f32; 3], max: [f32; 3]) -> bool {
    let cx = (min[0] + max[0]) * 0.5;
    let cy = (min[1] + max[1]) * 0.5;
    let cz = (min[2] + max[2]) * 0.5;
    let rx = (max[0] - min[0]) * 0.5;
    let ry = (max[1] - min[1]) * 0.5;
    let rz = (max[2] - min[2]) * 0.5;
    for plane in &frustum.planes {
        let r = rx * plane[0].abs() + ry * plane[1].abs() + rz * plane[2].abs();
        if plane_dist(*plane, [cx, cy, cz]) < -r {
            return false;
        }
    }
    true
}

#[allow(dead_code)]
pub fn fc_cull_spheres(frustum: &Frustum, spheres: &[([f32; 3], f32)]) -> Vec<bool> {
    spheres.iter().map(|&(c, r)| fc_test_sphere(frustum, c, r)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frustum_from_params() {
        let f = new_frustum_from_vp(0.785, 1.777, 0.1, 1000.0);
        assert_eq!(f.planes.len(), 6);
    }

    #[test]
    fn test_test_sphere_visible() {
        let f = new_frustum_from_vp(0.785, 1.777, 0.1, 100.0);
        /* sphere at origin with large radius should be visible */
        let visible = fc_test_sphere(&f, [0.0, 0.0, 0.0], 5.0);
        assert!(visible);
    }

    #[test]
    fn test_test_aabb_visible() {
        let f = new_frustum_from_vp(0.785, 1.777, 0.1, 100.0);
        let visible = fc_test_aabb(&f, [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        assert!(visible);
    }

    #[test]
    fn test_cull_spheres_returns_correct_length() {
        let f = new_frustum_from_vp(0.785, 1.777, 0.1, 100.0);
        let spheres = vec![
            ([0.0, 0.0, 0.0], 1.0),
            ([0.0, 0.0, 0.0], 2.0),
            ([0.0, 0.0, 0.0], 0.5),
        ];
        let result = fc_cull_spheres(&f, &spheres);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_cull_spheres_empty() {
        let f = new_frustum_from_vp(0.785, 1.777, 0.1, 100.0);
        let result = fc_cull_spheres(&f, &[]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_plane_count_is_six() {
        let f = new_frustum_from_vp(1.0, 1.0, 1.0, 10.0);
        assert_eq!(f.planes.len(), 6);
    }

    #[test]
    fn test_test_sphere_large_radius_visible() {
        let f = new_frustum_from_vp(0.785, 1.0, 0.1, 100.0);
        let result = fc_test_sphere(&f, [0.0, 0.0, 0.0], 200.0);
        assert!(result);
    }

    #[test]
    fn test_test_aabb_unit_cube() {
        let f = new_frustum_from_vp(0.785, 1.0, 0.1, 100.0);
        let result = fc_test_aabb(&f, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(result);
    }
}
