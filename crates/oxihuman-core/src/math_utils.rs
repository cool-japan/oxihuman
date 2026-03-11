// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Math utilities: quaternions, matrix operations, and easing functions.
//!
//! Provides `Quat` and `Mat4` types together with common geometric
//! operations needed throughout the oxihuman runtime.

/// A unit quaternion stored as `[x, y, z, w]`.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat(pub [f32; 4]);

/// A column-major 4×4 matrix stored as `[[col0..], [col1..], ..]`.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4(pub [[f32; 4]; 4]);

// --------------------------------------------------------------------------
// Quaternion operations
// --------------------------------------------------------------------------

/// Returns the identity quaternion `[0, 0, 0, 1]`.
#[allow(dead_code)]
pub fn quat_identity() -> Quat {
    Quat([0.0, 0.0, 0.0, 1.0])
}

/// Constructs a quaternion from an axis and angle (radians).
/// The axis is normalized internally.
#[allow(dead_code)]
pub fn quat_from_axis_angle(axis: [f32; 3], angle_rad: f32) -> Quat {
    let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len < 1e-9 {
        return quat_identity();
    }
    let inv_len = 1.0 / len;
    let nx = axis[0] * inv_len;
    let ny = axis[1] * inv_len;
    let nz = axis[2] * inv_len;
    let half = angle_rad * 0.5;
    let s = half.sin();
    Quat([nx * s, ny * s, nz * s, half.cos()])
}

/// Multiplies two quaternions: `a * b`.
#[allow(dead_code)]
pub fn quat_mul(a: Quat, b: Quat) -> Quat {
    let [ax, ay, az, aw] = a.0;
    let [bx, by, bz, bw] = b.0;
    Quat([
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ])
}

/// Spherical linear interpolation between two quaternions.
#[allow(dead_code)]
pub fn quat_slerp_utils(a: Quat, b: Quat, t: f32) -> Quat {
    let [ax, ay, az, aw] = a.0;
    let mut dot = ax * b.0[0] + ay * b.0[1] + az * b.0[2] + aw * b.0[3];
    let bx;
    let by;
    let bz;
    let bw;
    if dot < 0.0 {
        dot = -dot;
        bx = -b.0[0];
        by = -b.0[1];
        bz = -b.0[2];
        bw = -b.0[3];
    } else {
        bx = b.0[0];
        by = b.0[1];
        bz = b.0[2];
        bw = b.0[3];
    }
    let (sa, sb) = if dot > 0.9995 {
        (1.0 - t, t)
    } else {
        let theta = dot.acos();
        let sin_theta = theta.sin();
        (
            ((1.0 - t) * theta).sin() / sin_theta,
            (t * theta).sin() / sin_theta,
        )
    };
    Quat([
        ax * sa + bx * sb,
        ay * sa + by * sb,
        az * sa + bz * sb,
        aw * sa + bw * sb,
    ])
}

/// Converts a quaternion to a 3×3 rotation matrix (row-major, as nested arrays).
#[allow(dead_code)]
pub fn quat_to_mat3(q: Quat) -> [[f32; 3]; 3] {
    let [x, y, z, w] = q.0;
    let x2 = x + x;
    let y2 = y + y;
    let z2 = z + z;
    let xx = x * x2;
    let xy = x * y2;
    let xz = x * z2;
    let yy = y * y2;
    let yz = y * z2;
    let zz = z * z2;
    let wx = w * x2;
    let wy = w * y2;
    let wz = w * z2;
    [
        [1.0 - (yy + zz), xy + wz, xz - wy],
        [xy - wz, 1.0 - (xx + zz), yz + wx],
        [xz + wy, yz - wx, 1.0 - (xx + yy)],
    ]
}

// --------------------------------------------------------------------------
// Matrix operations
// --------------------------------------------------------------------------

/// Returns the 4×4 identity matrix.
#[allow(dead_code)]
pub fn mat4_identity() -> Mat4 {
    Mat4([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

/// Multiplies two 4×4 matrices: `a * b`.
#[allow(dead_code)]
pub fn mat4_mul(a: Mat4, b: Mat4) -> Mat4 {
    let mut result = [[0.0f32; 4]; 4];
    #[allow(clippy::needless_range_loop)]
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                result[i][j] += a.0[i][k] * b.0[k][j];
            }
        }
    }
    Mat4(result)
}

/// Returns a translation matrix for the given `(x, y, z)` offset.
#[allow(dead_code)]
pub fn mat4_translate(x: f32, y: f32, z: f32) -> Mat4 {
    Mat4([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [x, y, z, 1.0],
    ])
}

/// Returns a uniform-scale matrix for the given `(sx, sy, sz)` factors.
#[allow(dead_code)]
pub fn mat4_scale(sx: f32, sy: f32, sz: f32) -> Mat4 {
    Mat4([
        [sx, 0.0, 0.0, 0.0],
        [0.0, sy, 0.0, 0.0],
        [0.0, 0.0, sz, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

// --------------------------------------------------------------------------
// Easing and scalar math
// --------------------------------------------------------------------------

/// Cubic ease-in-out: smooth acceleration and deceleration. `t` must be in [0, 1].
#[allow(dead_code)]
pub fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) * 0.5
    }
}

/// Linear interpolation between `a` and `b` by `t` in [0, 1].
#[allow(dead_code)]
pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Clamps `v` to the range `[min_val, max_val]`.
#[allow(dead_code)]
pub fn clamp_f32(v: f32, min_val: f32, max_val: f32) -> f32 {
    v.clamp(min_val, max_val)
}

/// Remaps `value` from range `[in_min, in_max]` to `[out_min, out_max]`.
#[allow(dead_code)]
pub fn remap_range(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let in_range = in_max - in_min;
    if in_range.abs() < 1e-9 {
        return out_min;
    }
    let t = (value - in_min) / in_range;
    lerp_f32(out_min, out_max, t)
}

/// Wraps an angle in degrees into the range `(-180, 180]`.
#[allow(dead_code)]
pub fn angle_wrap(deg: f32) -> f32 {
    let mut a = deg % 360.0;
    if a > 180.0 {
        a -= 360.0;
    } else if a <= -180.0 {
        a += 360.0;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-5;

    // --- Quat tests ---

    #[test]
    fn test_quat_identity() {
        let q = quat_identity();
        assert_eq!(q.0, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_quat_from_axis_angle_zero() {
        let q = quat_from_axis_angle([0.0, 0.0, 0.0], 0.0);
        assert!((q.0[3] - 1.0).abs() < EPS);
    }

    #[test]
    fn test_quat_from_axis_angle_90_y() {
        let q = quat_from_axis_angle([0.0, 1.0, 0.0], std::f32::consts::FRAC_PI_2);
        let expected_w = (std::f32::consts::FRAC_PI_4).cos();
        let expected_y = (std::f32::consts::FRAC_PI_4).sin();
        assert!((q.0[3] - expected_w).abs() < EPS);
        assert!((q.0[1] - expected_y).abs() < EPS);
    }

    #[test]
    fn test_quat_mul_identity() {
        let id = quat_identity();
        let q = quat_from_axis_angle([1.0, 0.0, 0.0], 0.5);
        let r = quat_mul(id, q);
        for i in 0..4 {
            assert!((r.0[i] - q.0[i]).abs() < EPS);
        }
    }

    #[test]
    fn test_quat_mul_self_gives_double_angle() {
        let q = quat_from_axis_angle([0.0, 0.0, 1.0], std::f32::consts::FRAC_PI_4);
        let q2 = quat_mul(q, q);
        let expected = quat_from_axis_angle([0.0, 0.0, 1.0], std::f32::consts::FRAC_PI_2);
        for i in 0..4 {
            assert!((q2.0[i] - expected.0[i]).abs() < 1e-4);
        }
    }

    #[test]
    fn test_quat_slerp_t0() {
        let a = quat_identity();
        let b = quat_from_axis_angle([0.0, 1.0, 0.0], 1.0);
        let r = quat_slerp_utils(a, b, 0.0);
        for i in 0..4 {
            assert!((r.0[i] - a.0[i]).abs() < EPS);
        }
    }

    #[test]
    fn test_quat_slerp_t1() {
        let a = quat_identity();
        let b = quat_from_axis_angle([0.0, 1.0, 0.0], 1.0);
        let r = quat_slerp_utils(a, b, 1.0);
        for i in 0..4 {
            assert!((r.0[i] - b.0[i]).abs() < EPS);
        }
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn test_quat_to_mat3_identity() {
        let m = quat_to_mat3(quat_identity());
        // Diagonal should be 1, off-diagonal 0
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (m[i][j] - expected).abs() < EPS,
                    "m[{i}][{j}] = {}",
                    m[i][j]
                );
            }
        }
    }

    #[test]
    fn test_quat_to_mat3_90_z() {
        let q = quat_from_axis_angle([0.0, 0.0, 1.0], std::f32::consts::FRAC_PI_2);
        let m = quat_to_mat3(q);
        // Should rotate X axis into Y axis
        assert!((m[0][0]).abs() < 1e-4); // col0 x ~ 0
        assert!((m[0][1] - 1.0).abs() < 1e-4); // col0 y ~ 1
    }

    // --- Mat4 tests ---

    #[test]
    fn test_mat4_identity() {
        let m = mat4_identity();
        for i in 0..4 {
            for j in 0..4 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((m.0[i][j] - expected).abs() < EPS);
            }
        }
    }

    #[test]
    fn test_mat4_mul_identity() {
        let id = mat4_identity();
        let t = mat4_translate(1.0, 2.0, 3.0);
        let r = mat4_mul(id, t);
        for i in 0..4 {
            for j in 0..4 {
                assert!((r.0[i][j] - t.0[i][j]).abs() < EPS);
            }
        }
    }

    #[test]
    fn test_mat4_translate() {
        let m = mat4_translate(5.0, 6.0, 7.0);
        assert!((m.0[3][0] - 5.0).abs() < EPS);
        assert!((m.0[3][1] - 6.0).abs() < EPS);
        assert!((m.0[3][2] - 7.0).abs() < EPS);
        assert!((m.0[3][3] - 1.0).abs() < EPS);
    }

    #[test]
    fn test_mat4_scale() {
        let m = mat4_scale(2.0, 3.0, 4.0);
        assert!((m.0[0][0] - 2.0).abs() < EPS);
        assert!((m.0[1][1] - 3.0).abs() < EPS);
        assert!((m.0[2][2] - 4.0).abs() < EPS);
    }

    // --- Easing / scalar tests ---

    #[test]
    fn test_ease_in_out_endpoints() {
        assert!((ease_in_out(0.0)).abs() < EPS);
        assert!((ease_in_out(1.0) - 1.0).abs() < EPS);
    }

    #[test]
    fn test_ease_in_out_midpoint() {
        // At t=0.5 a cubic ease-in-out should equal 0.5 (symmetry)
        assert!((ease_in_out(0.5) - 0.5).abs() < EPS);
    }

    #[test]
    fn test_ease_in_out_clamp() {
        assert!((ease_in_out(-1.0)).abs() < EPS);
        assert!((ease_in_out(2.0) - 1.0).abs() < EPS);
    }

    #[test]
    fn test_lerp_f32() {
        assert!((lerp_f32(0.0, 10.0, 0.5) - 5.0).abs() < EPS);
        assert!((lerp_f32(2.0, 4.0, 0.25) - 2.5).abs() < EPS);
    }

    #[test]
    fn test_clamp_f32() {
        assert!((clamp_f32(5.0, 0.0, 1.0) - 1.0).abs() < EPS);
        assert!((clamp_f32(-1.0, 0.0, 1.0)).abs() < EPS);
        assert!((clamp_f32(0.5, 0.0, 1.0) - 0.5).abs() < EPS);
    }

    #[test]
    fn test_remap_range() {
        let v = remap_range(5.0, 0.0, 10.0, 0.0, 1.0);
        assert!((v - 0.5).abs() < EPS);
    }

    #[test]
    fn test_remap_range_zero_input() {
        let v = remap_range(5.0, 5.0, 5.0, 0.0, 1.0);
        assert!((v).abs() < EPS); // returns out_min
    }

    #[test]
    fn test_angle_wrap_positive() {
        let a = angle_wrap(270.0);
        assert!((a - (-90.0)).abs() < EPS);
    }

    #[test]
    fn test_angle_wrap_negative() {
        let a = angle_wrap(-270.0);
        assert!((a - 90.0).abs() < EPS);
    }

    #[test]
    fn test_angle_wrap_within_range() {
        let a = angle_wrap(45.0);
        assert!((a - 45.0).abs() < EPS);
    }

    #[test]
    fn test_angle_wrap_zero() {
        assert!((angle_wrap(0.0)).abs() < EPS);
    }

    #[test]
    fn test_angle_wrap_360() {
        let a = angle_wrap(360.0);
        assert!(a.abs() < EPS);
    }
}
