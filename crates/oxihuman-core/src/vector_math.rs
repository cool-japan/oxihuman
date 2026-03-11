// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Add two 3D vectors.
pub fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Subtract two 3D vectors.
pub fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Scale a 3D vector by a scalar.
pub fn vec3_scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// Dot product of two 3D vectors.
pub fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Cross product of two 3D vectors.
pub fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Euclidean length of a 3D vector.
pub fn vec3_len(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// Normalize a 3D vector to unit length. Returns zero vector if length is zero.
pub fn vec3_norm(a: [f32; 3]) -> [f32; 3] {
    let len = vec3_len(a);
    if len < 1e-12 {
        [0.0, 0.0, 0.0]
    } else {
        vec3_scale(a, 1.0 / len)
    }
}

/// Linearly interpolate between two 3D vectors.
pub fn vec3_lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Angle between two 3D vectors in radians.
pub fn vec3_angle(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dot = vec3_dot(a, b);
    let lens = vec3_len(a) * vec3_len(b);
    if lens < 1e-12 {
        return 0.0;
    }
    (dot / lens).clamp(-1.0, 1.0).acos()
}

/// Reflect vector a around normal n (both assumed normalized).
pub fn vec3_reflect(a: [f32; 3], n: [f32; 3]) -> [f32; 3] {
    let d = 2.0 * vec3_dot(a, n);
    vec3_sub(a, vec3_scale(n, d))
}

/// Returns the component-wise maximum of two vectors.
pub fn vec3_max(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])]
}

/// Returns a vector perpendicular to the given one (not normalized).
pub fn vec3_perp(a: [f32; 3]) -> [f32; 3] {
    if a[0].abs() <= a[1].abs() && a[0].abs() <= a[2].abs() {
        vec3_cross(a, [1.0, 0.0, 0.0])
    } else if a[1].abs() <= a[2].abs() {
        vec3_cross(a, [0.0, 1.0, 0.0])
    } else {
        vec3_cross(a, [0.0, 0.0, 1.0])
    }
}

/// Negate a 3D vector (used internally, exposed for completeness).
pub fn vec3_neg(a: [f32; 3]) -> [f32; 3] {
    [-a[0], -a[1], -a[2]]
}

/// Utility: pi constant re-export (keeps `use std::f32::consts::PI` active).
pub fn vec3_pi() -> f32 {
    PI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_add() {
        /* basic addition */
        let r = vec3_add([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        assert!((r[0] - 5.0).abs() < 1e-6);
        assert!((r[1] - 7.0).abs() < 1e-6);
        assert!((r[2] - 9.0).abs() < 1e-6);
    }

    #[test]
    fn test_vec3_sub() {
        /* subtraction */
        let r = vec3_sub([10.0, 5.0, 3.0], [1.0, 2.0, 3.0]);
        assert!((r[0] - 9.0).abs() < 1e-6);
        assert!((r[1] - 3.0).abs() < 1e-6);
        assert!((r[2]).abs() < 1e-6);
    }

    #[test]
    fn test_vec3_scale() {
        /* scalar multiplication */
        let r = vec3_scale([1.0, 2.0, 3.0], 2.0);
        assert!((r[0] - 2.0).abs() < 1e-6);
        assert!((r[1] - 4.0).abs() < 1e-6);
        assert!((r[2] - 6.0).abs() < 1e-6);
    }

    #[test]
    fn test_vec3_dot() {
        /* dot product */
        let d = vec3_dot([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(d.abs() < 1e-6);
        let d2 = vec3_dot([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        assert!((d2 - 32.0).abs() < 1e-6);
    }

    #[test]
    fn test_vec3_cross() {
        /* cross product of unit axes */
        let r = vec3_cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((r[0]).abs() < 1e-6);
        assert!((r[1]).abs() < 1e-6);
        assert!((r[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vec3_len() {
        /* vector length */
        let l = vec3_len([3.0, 4.0, 0.0]);
        assert!((l - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_vec3_norm() {
        /* normalization */
        let n = vec3_norm([3.0, 0.0, 0.0]);
        assert!((n[0] - 1.0).abs() < 1e-6);
        /* zero vector stays zero */
        let z = vec3_norm([0.0, 0.0, 0.0]);
        assert!(z[0].abs() < 1e-6);
    }

    #[test]
    fn test_vec3_lerp() {
        /* interpolation at midpoint */
        let r = vec3_lerp([0.0, 0.0, 0.0], [2.0, 4.0, 6.0], 0.5);
        assert!((r[0] - 1.0).abs() < 1e-6);
        assert!((r[1] - 2.0).abs() < 1e-6);
        assert!((r[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_vec3_angle() {
        /* perpendicular vectors -> pi/2 */
        use std::f32::consts::FRAC_PI_2;
        let a = vec3_angle([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((a - FRAC_PI_2).abs() < 1e-5);
    }

    #[test]
    fn test_vec3_reflect() {
        /* reflection of downward vector off flat normal */
        let r = vec3_reflect([0.0, -1.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((r[1] - 1.0).abs() < 1e-6);
    }
}
