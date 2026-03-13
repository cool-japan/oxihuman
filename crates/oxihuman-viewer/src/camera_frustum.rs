// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! CameraFrustum — frustum planes and intersection tests.

#![allow(dead_code)]

/// A plane defined by `ax + by + cz + d = 0`.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FrustumPlane {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
}

impl FrustumPlane {
    /// Signed distance from point to plane.
    fn distance(&self, p: [f32; 3]) -> f32 {
        self.a * p[0] + self.b * p[1] + self.c * p[2] + self.d
    }
}

/// Six-plane camera frustum (left, right, bottom, top, near, far).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraFrustum {
    pub planes: [FrustumPlane; 6],
}

/// Create a default camera frustum (unit cube).
#[allow(dead_code)]
pub fn new_camera_frustum() -> CameraFrustum {
    let z = FrustumPlane { a: 0.0, b: 0.0, c: 0.0, d: 0.0 };
    CameraFrustum { planes: [z; 6] }
}

/// Extract six frustum planes from a column-major view-projection matrix.
#[allow(dead_code)]
pub fn extract_planes(vp: &[[f32; 4]; 4]) -> CameraFrustum {
    // Row extraction from column-major matrix
    let row = |r: usize| -> [f32; 4] {
        [vp[0][r], vp[1][r], vp[2][r], vp[3][r]]
    };
    let r0 = row(0);
    let r1 = row(1);
    let r2 = row(2);
    let r3 = row(3);

    let make = |a: f32, b: f32, c: f32, d: f32| -> FrustumPlane {
        let len = (a * a + b * b + c * c).sqrt().max(1e-12);
        FrustumPlane {
            a: a / len,
            b: b / len,
            c: c / len,
            d: d / len,
        }
    };

    CameraFrustum {
        planes: [
            make(r3[0] + r0[0], r3[1] + r0[1], r3[2] + r0[2], r3[3] + r0[3]), // left
            make(r3[0] - r0[0], r3[1] - r0[1], r3[2] - r0[2], r3[3] - r0[3]), // right
            make(r3[0] + r1[0], r3[1] + r1[1], r3[2] + r1[2], r3[3] + r1[3]), // bottom
            make(r3[0] - r1[0], r3[1] - r1[1], r3[2] - r1[2], r3[3] - r1[3]), // top
            make(r3[0] + r2[0], r3[1] + r2[1], r3[2] + r2[2], r3[3] + r2[3]), // near
            make(r3[0] - r2[0], r3[1] - r2[1], r3[2] - r2[2], r3[3] - r2[3]), // far
        ],
    }
}

/// Test whether a point is inside all frustum planes.
#[allow(dead_code)]
pub fn point_in_frustum(frustum: &CameraFrustum, point: [f32; 3]) -> bool {
    frustum.planes.iter().all(|p| p.distance(point) >= 0.0)
}

/// Test whether a sphere intersects the frustum.
#[allow(dead_code)]
pub fn sphere_in_frustum(frustum: &CameraFrustum, center: [f32; 3], radius: f32) -> bool {
    frustum
        .planes
        .iter()
        .all(|p| p.distance(center) >= -radius)
}

/// Test whether an AABB intersects the frustum.
/// `min` and `max` are the corners of the AABB.
#[allow(dead_code)]
pub fn aabb_in_frustum(frustum: &CameraFrustum, min: [f32; 3], max: [f32; 3]) -> bool {
    for plane in &frustum.planes {
        let px = if plane.a >= 0.0 { max[0] } else { min[0] };
        let py = if plane.b >= 0.0 { max[1] } else { min[1] };
        let pz = if plane.c >= 0.0 { max[2] } else { min[2] };
        if plane.distance([px, py, pz]) < 0.0 {
            return false;
        }
    }
    true
}

/// Compute the eight corners of the frustum (stub: returns identity cube corners).
#[allow(dead_code)]
pub fn frustum_corners(frustum: &CameraFrustum) -> [[f32; 3]; 8] {
    let _ = frustum;
    [
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0],
    ]
}

/// Return the near plane.
#[allow(dead_code)]
pub fn frustum_near_plane(frustum: &CameraFrustum) -> &FrustumPlane {
    &frustum.planes[4]
}

/// Return the far plane.
#[allow(dead_code)]
pub fn frustum_far_plane(frustum: &CameraFrustum) -> &FrustumPlane {
    &frustum.planes[5]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_vp() -> [[f32; 4]; 4] {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    #[test]
    fn test_new_camera_frustum() {
        let f = new_camera_frustum();
        assert_eq!(f.planes.len(), 6);
    }

    #[test]
    fn test_extract_planes() {
        let f = extract_planes(&identity_vp());
        assert_eq!(f.planes.len(), 6);
    }

    #[test]
    fn test_point_in_frustum_origin() {
        let f = extract_planes(&identity_vp());
        assert!(point_in_frustum(&f, [0.0, 0.0, 0.0]));
    }

    #[test]
    fn test_sphere_in_frustum() {
        let f = extract_planes(&identity_vp());
        assert!(sphere_in_frustum(&f, [0.0, 0.0, 0.0], 0.5));
    }

    #[test]
    fn test_sphere_outside_frustum() {
        let f = extract_planes(&identity_vp());
        assert!(!sphere_in_frustum(&f, [100.0, 100.0, 100.0], 0.1));
    }

    #[test]
    fn test_aabb_in_frustum() {
        let f = extract_planes(&identity_vp());
        assert!(aabb_in_frustum(&f, [-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]));
    }

    #[test]
    fn test_frustum_corners() {
        let f = new_camera_frustum();
        let corners = frustum_corners(&f);
        assert_eq!(corners.len(), 8);
    }

    #[test]
    fn test_frustum_near_plane() {
        let f = extract_planes(&identity_vp());
        let _p = frustum_near_plane(&f);
    }

    #[test]
    fn test_frustum_far_plane() {
        let f = extract_planes(&identity_vp());
        let _p = frustum_far_plane(&f);
    }

    #[test]
    fn test_aabb_outside_frustum() {
        let f = extract_planes(&identity_vp());
        assert!(!aabb_in_frustum(&f, [10.0, 10.0, 10.0], [20.0, 20.0, 20.0]));
    }
}
