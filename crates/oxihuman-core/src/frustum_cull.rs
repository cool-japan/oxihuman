#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! View frustum culling.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Plane {
    pub normal: [f32; 3],
    pub d: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

fn plane_distance(p: &Plane, pt: [f32; 3]) -> f32 {
    p.normal[0] * pt[0] + p.normal[1] * pt[1] + p.normal[2] * pt[2] + p.d
}

fn extract_plane(row0: [f32; 4], row1: [f32; 4], sign: f32) -> Plane {
    let nx = row0[3] + sign * row0[0];
    let ny = row0[3] + sign * row0[1];
    let nz = row0[3] + sign * row0[2];
    let d = row1[3] + sign * row1[3];
    let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-10);
    Plane {
        normal: [nx / len, ny / len, nz / len],
        d: d / len,
    }
}

#[allow(dead_code)]
pub fn frustum_from_mat4(m: [[f32; 4]; 4]) -> Frustum {
    // Extract 6 planes from a combined view-projection matrix.
    let make = |a: usize, s: f32| {
        let nx = m[3][0] + s * m[a][0];
        let ny = m[3][1] + s * m[a][1];
        let nz = m[3][2] + s * m[a][2];
        let d = m[3][3] + s * m[a][3];
        let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-10);
        Plane {
            normal: [nx / len, ny / len, nz / len],
            d: d / len,
        }
    };
    // Suppress unused warning for extract_plane helper
    let _ = extract_plane([0.0; 4], [0.0; 4], 1.0);
    Frustum {
        planes: [
            make(0, 1.0),  // left
            make(0, -1.0), // right
            make(1, 1.0),  // bottom
            make(1, -1.0), // top
            make(2, 1.0),  // near
            make(2, -1.0), // far
        ],
    }
}

#[allow(dead_code)]
pub fn sphere_in_frustum(f: &Frustum, center: [f32; 3], radius: f32) -> bool {
    for plane in &f.planes {
        if plane_distance(plane, center) < -radius {
            return false;
        }
    }
    true
}

#[allow(dead_code)]
pub fn aabb_in_frustum(f: &Frustum, min: [f32; 3], max: [f32; 3]) -> bool {
    for plane in &f.planes {
        let p_vertex = [
            if plane.normal[0] >= 0.0 { max[0] } else { min[0] },
            if plane.normal[1] >= 0.0 { max[1] } else { min[1] },
            if plane.normal[2] >= 0.0 { max[2] } else { min[2] },
        ];
        if plane_distance(plane, p_vertex) < 0.0 {
            return false;
        }
    }
    true
}

#[allow(dead_code)]
pub fn point_in_frustum(f: &Frustum, pt: [f32; 3]) -> bool {
    for plane in &f.planes {
        if plane_distance(plane, pt) < 0.0 {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_identity_frustum() -> Frustum {
        // Simple axis-aligned frustum: planes bounding the unit cube [-1,1]^3
        let make_plane = |nx: f32, ny: f32, nz: f32, d: f32| Plane {
            normal: [nx, ny, nz],
            d,
        };
        Frustum {
            planes: [
                make_plane(1.0, 0.0, 0.0, 1.0),  // x >= -1
                make_plane(-1.0, 0.0, 0.0, 1.0), // x <= 1
                make_plane(0.0, 1.0, 0.0, 1.0),  // y >= -1
                make_plane(0.0, -1.0, 0.0, 1.0), // y <= 1
                make_plane(0.0, 0.0, 1.0, 1.0),  // z >= -1
                make_plane(0.0, 0.0, -1.0, 1.0), // z <= 1
            ],
        }
    }

    #[test]
    fn test_point_inside() {
        let f = make_identity_frustum();
        assert!(point_in_frustum(&f, [0.0, 0.0, 0.0]));
    }

    #[test]
    fn test_point_outside() {
        let f = make_identity_frustum();
        assert!(!point_in_frustum(&f, [2.0, 0.0, 0.0]));
    }

    #[test]
    fn test_sphere_inside() {
        let f = make_identity_frustum();
        assert!(sphere_in_frustum(&f, [0.0, 0.0, 0.0], 0.5));
    }

    #[test]
    fn test_sphere_outside() {
        let f = make_identity_frustum();
        assert!(!sphere_in_frustum(&f, [5.0, 0.0, 0.0], 0.1));
    }

    #[test]
    fn test_sphere_intersecting() {
        let f = make_identity_frustum();
        // Sphere centered just outside but overlapping
        assert!(sphere_in_frustum(&f, [1.5, 0.0, 0.0], 1.0));
    }

    #[test]
    fn test_aabb_inside() {
        let f = make_identity_frustum();
        assert!(aabb_in_frustum(&f, [-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]));
    }

    #[test]
    fn test_aabb_outside() {
        let f = make_identity_frustum();
        assert!(!aabb_in_frustum(&f, [2.0, 2.0, 2.0], [3.0, 3.0, 3.0]));
    }

    #[test]
    fn test_plane_distance_positive() {
        let p = Plane { normal: [1.0, 0.0, 0.0], d: 0.0 };
        assert!(plane_distance(&p, [1.0, 0.0, 0.0]) > 0.0);
    }

    #[test]
    fn test_plane_distance_negative() {
        let p = Plane { normal: [1.0, 0.0, 0.0], d: 0.0 };
        assert!(plane_distance(&p, [-1.0, 0.0, 0.0]) < 0.0);
    }
}
