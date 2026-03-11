// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Angle conversion and wrapping utilities.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Converts degrees to radians.
#[allow(dead_code)]
pub fn deg_to_rad(deg: f32) -> f32 {
    deg * (PI / 180.0)
}

/// Converts radians to degrees.
#[allow(dead_code)]
pub fn rad_to_deg(rad: f32) -> f32 {
    rad * (180.0 / PI)
}

/// Wraps an angle in radians to the range (-PI, PI].
#[allow(dead_code)]
pub fn wrap_angle(angle: f32) -> f32 {
    let mut a = angle % (2.0 * PI);
    if a > PI {
        a -= 2.0 * PI;
    } else if a <= -PI {
        a += 2.0 * PI;
    }
    a
}

/// Returns the shortest signed difference between two angles (radians), in (-PI, PI].
#[allow(dead_code)]
pub fn angle_diff(a: f32, b: f32) -> f32 {
    wrap_angle(b - a)
}

/// Linearly interpolates between two angles (radians), taking the shortest path.
#[allow(dead_code)]
pub fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let d = angle_diff(a, b);
    wrap_angle(a + d * t)
}

/// Returns the unit direction vector `[cos(angle), sin(angle)]` for a given angle in radians.
#[allow(dead_code)]
pub fn angle_to_dir(angle: f32) -> [f32; 2] {
    [angle.cos(), angle.sin()]
}

/// Returns the angle (radians) of a direction vector. Equivalent to `atan2(y, x)`.
#[allow(dead_code)]
pub fn dir_to_angle(d: [f32; 2]) -> f32 {
    d[1].atan2(d[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;

    #[test]
    fn test_deg_to_rad() {
        assert!((deg_to_rad(180.0) - PI).abs() < EPS);
        assert!((deg_to_rad(0.0)).abs() < EPS);
    }

    #[test]
    fn test_rad_to_deg() {
        assert!((rad_to_deg(PI) - 180.0).abs() < EPS);
        assert!((rad_to_deg(0.0)).abs() < EPS);
    }

    #[test]
    fn test_roundtrip() {
        let orig = 123.456_f32;
        let r = rad_to_deg(deg_to_rad(orig));
        assert!((r - orig).abs() < EPS);
    }

    #[test]
    fn test_wrap_angle_pi() {
        let a = wrap_angle(PI);
        assert!((a - PI).abs() < EPS || (a + PI).abs() < EPS);
    }

    #[test]
    fn test_wrap_angle_overflow() {
        let a = wrap_angle(3.0 * PI);
        assert!(a.abs() < EPS + PI);
    }

    #[test]
    fn test_angle_diff_zero() {
        assert!((angle_diff(1.0, 1.0)).abs() < EPS);
    }

    #[test]
    fn test_lerp_angle_t0() {
        assert!((lerp_angle(0.0, PI / 2.0, 0.0)).abs() < EPS);
    }

    #[test]
    fn test_lerp_angle_t1() {
        assert!((lerp_angle(0.0, PI / 2.0, 1.0) - PI / 2.0).abs() < EPS);
    }

    #[test]
    fn test_angle_to_dir_zero() {
        let d = angle_to_dir(0.0);
        assert!((d[0] - 1.0).abs() < EPS);
        assert!((d[1]).abs() < EPS);
    }

    #[test]
    fn test_dir_to_angle() {
        let a = dir_to_angle([1.0, 0.0]);
        assert!(a.abs() < EPS);
        let b = dir_to_angle([0.0, 1.0]);
        assert!((b - PI / 2.0).abs() < EPS);
    }
}
