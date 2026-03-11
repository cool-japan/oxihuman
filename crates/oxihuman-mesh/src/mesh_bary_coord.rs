// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Barycentric coordinate computation and utilities.

/// Barycentric coordinates (u, v, w) where u + v + w = 1.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BaryCoord {
    pub u: f32,
    pub v: f32,
    pub w: f32,
}

/// Compute barycentric coordinates of point `p` in triangle (v0, v1, v2).
#[allow(dead_code)]
pub fn compute_bary(p: [f32; 3], v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> BaryCoord {
    let e0 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e1 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let ep = [p[0] - v0[0], p[1] - v0[1], p[2] - v0[2]];
    let d00 = dot3(e0, e0);
    let d01 = dot3(e0, e1);
    let d11 = dot3(e1, e1);
    let d20 = dot3(ep, e0);
    let d21 = dot3(ep, e1);
    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < 1e-12 {
        return BaryCoord {
            u: 1.0,
            v: 0.0,
            w: 0.0,
        };
    }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;
    BaryCoord { u, v, w }
}

/// Dot product.
#[allow(dead_code)]
pub fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Check if barycentric coordinates are inside the triangle (all >= 0).
#[allow(dead_code)]
pub fn is_inside_triangle(bc: &BaryCoord) -> bool {
    bc.u >= -1e-6 && bc.v >= -1e-6 && bc.w >= -1e-6
}

/// Interpolate a value using barycentric coordinates.
#[allow(dead_code)]
pub fn bary_interpolate(bc: &BaryCoord, a: f32, b: f32, c: f32) -> f32 {
    bc.u * a + bc.v * b + bc.w * c
}

/// Interpolate a 3D position using barycentric coordinates.
#[allow(dead_code)]
pub fn bary_interpolate3(bc: &BaryCoord, a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    [
        bc.u * a[0] + bc.v * b[0] + bc.w * c[0],
        bc.u * a[1] + bc.v * b[1] + bc.w * c[1],
        bc.u * a[2] + bc.v * b[2] + bc.w * c[2],
    ]
}

/// Clamp barycentric coords to valid range and re-normalize.
#[allow(dead_code)]
pub fn clamp_bary(bc: &BaryCoord) -> BaryCoord {
    let u = bc.u.max(0.0);
    let v = bc.v.max(0.0);
    let w = bc.w.max(0.0);
    let sum = u + v + w;
    if sum < 1e-12 {
        return BaryCoord {
            u: 1.0 / 3.0,
            v: 1.0 / 3.0,
            w: 1.0 / 3.0,
        };
    }
    BaryCoord {
        u: u / sum,
        v: v / sum,
        w: w / sum,
    }
}

/// Distance from barycentric coords to triangle center.
#[allow(dead_code)]
pub fn bary_distance_to_center(bc: &BaryCoord) -> f32 {
    let third = 1.0 / 3.0;
    let du = bc.u - third;
    let dv = bc.v - third;
    let dw = bc.w - third;
    (du * du + dv * dv + dw * dw).sqrt()
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn bary_to_json(bc: &BaryCoord) -> String {
    format!("{{\"u\":{:.6},\"v\":{:.6},\"w\":{:.6}}}", bc.u, bc.v, bc.w)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> ([f32; 3], [f32; 3], [f32; 3]) {
        ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0])
    }

    #[test]
    fn test_bary_at_vertex() {
        let (v0, v1, v2) = triangle();
        let bc = compute_bary(v0, v0, v1, v2);
        assert!((bc.u - 1.0).abs() < 1e-5);
        assert!((bc.v).abs() < 1e-5);
    }

    #[test]
    fn test_bary_center() {
        let (v0, v1, v2) = triangle();
        let center = [1.0 / 3.0, 1.0 / 3.0, 0.0];
        let bc = compute_bary(center, v0, v1, v2);
        assert!((bc.u - 1.0 / 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_is_inside() {
        let bc = BaryCoord {
            u: 0.3,
            v: 0.3,
            w: 0.4,
        };
        assert!(is_inside_triangle(&bc));
    }

    #[test]
    fn test_is_outside() {
        let bc = BaryCoord {
            u: -0.1,
            v: 0.5,
            w: 0.6,
        };
        assert!(!is_inside_triangle(&bc));
    }

    #[test]
    fn test_interpolate() {
        let bc = BaryCoord {
            u: 0.5,
            v: 0.3,
            w: 0.2,
        };
        let val = bary_interpolate(&bc, 1.0, 2.0, 3.0);
        assert!((val - 1.7).abs() < 1e-6);
    }

    #[test]
    fn test_interpolate3() {
        let bc = BaryCoord {
            u: 1.0,
            v: 0.0,
            w: 0.0,
        };
        let p = bary_interpolate3(&bc, [1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]);
        assert!((p[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_bary() {
        let bc = BaryCoord {
            u: -0.1,
            v: 0.5,
            w: 0.6,
        };
        let c = clamp_bary(&bc);
        assert!(c.u >= 0.0);
        assert!(((c.u + c.v + c.w) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_distance_to_center() {
        let bc = BaryCoord {
            u: 1.0 / 3.0,
            v: 1.0 / 3.0,
            w: 1.0 / 3.0,
        };
        assert!(bary_distance_to_center(&bc) < 1e-6);
    }

    #[test]
    fn test_bary_to_json() {
        let bc = BaryCoord {
            u: 0.5,
            v: 0.3,
            w: 0.2,
        };
        let j = bary_to_json(&bc);
        assert!(j.contains("\"u\":0.5"));
    }

    #[test]
    fn test_dot3() {
        assert!((dot3([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])).abs() < 1e-9);
    }
}
