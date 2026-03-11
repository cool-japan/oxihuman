// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Capture Point (CP) stability metric stub.
//!
//! The Capture Point (also known as the Extrapolated Center of Mass) is a
//! stability measure for legged robots: `ξ = p + v / ω₀`, where `ω₀ = sqrt(g/h)`.

/// Capture point result.
#[derive(Debug, Clone, PartialEq)]
pub struct CapturePoint {
    pub x: f32,
    pub y: f32,
}

/// Compute the natural frequency of the linear inverted pendulum.
pub fn natural_frequency(height: f32, gravity: f32) -> f32 {
    (gravity.abs() / height.max(1e-6)).sqrt()
}

/// Compute the instantaneous capture point.
/// `ξ = p + v / ω₀`
pub fn compute_capture_point(
    com_x: f32,
    com_y: f32,
    vel_x: f32,
    vel_y: f32,
    height: f32,
    gravity: f32,
) -> CapturePoint {
    let omega = natural_frequency(height, gravity);
    CapturePoint {
        x: com_x + vel_x / omega.max(1e-6),
        y: com_y + vel_y / omega.max(1e-6),
    }
}

/// Return whether the capture point lies within the support polygon (AABB stub).
pub fn capture_point_stable(
    cp: &CapturePoint,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
) -> bool {
    (x_min..=x_max).contains(&cp.x) && (y_min..=y_max).contains(&cp.y)
}

/// Compute the distance from the capture point to the support polygon centre.
pub fn capture_point_distance_to_centre(cp: &CapturePoint, cx: f32, cy: f32) -> f32 {
    let dx = cp.x - cx;
    let dy = cp.y - cy;
    (dx * dx + dy * dy).sqrt()
}

/// Return the capture point velocity (time derivative, stub — returns scaled acceleration).
pub fn capture_point_velocity(
    vel_x: f32,
    vel_y: f32,
    acc_x: f32,
    acc_y: f32,
    omega: f32,
) -> [f32; 2] {
    let o = omega.max(1e-6);
    [vel_x + acc_x / o, vel_y + acc_y / o]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_frequency_positive() {
        /* natural frequency is positive */
        assert!(natural_frequency(1.0, 9.81) > 0.0);
    }

    #[test]
    fn test_natural_frequency_scaling() {
        /* higher com → lower frequency */
        let f1 = natural_frequency(1.0, 9.81);
        let f2 = natural_frequency(2.0, 9.81);
        assert!(f1 > f2);
    }

    #[test]
    fn test_cp_at_rest() {
        /* zero velocity → CP equals CoM */
        let cp = compute_capture_point(0.1, 0.0, 0.0, 0.0, 1.0, 9.81);
        assert!((cp.x - 0.1).abs() < 1e-5);
        assert!((cp.y).abs() < 1e-5);
    }

    #[test]
    fn test_cp_with_velocity() {
        /* forward velocity moves CP forward */
        let cp = compute_capture_point(0.0, 0.0, 1.0, 0.0, 1.0, 9.81);
        assert!(cp.x > 0.0);
    }

    #[test]
    fn test_cp_stable_in_support() {
        /* CP at origin is stable in centred support */
        let cp = CapturePoint { x: 0.0, y: 0.0 };
        assert!(capture_point_stable(&cp, -0.1, 0.1, -0.05, 0.05));
    }

    #[test]
    fn test_cp_unstable_outside() {
        /* CP outside support polygon */
        let cp = CapturePoint { x: 5.0, y: 5.0 };
        assert!(!capture_point_stable(&cp, -0.1, 0.1, -0.05, 0.05));
    }

    #[test]
    fn test_distance_to_centre_zero() {
        /* CP at centre has zero distance */
        let cp = CapturePoint { x: 0.0, y: 0.0 };
        assert!(capture_point_distance_to_centre(&cp, 0.0, 0.0) < 1e-6);
    }

    #[test]
    fn test_distance_to_centre_nonzero() {
        /* off-centre CP has positive distance */
        let cp = CapturePoint { x: 1.0, y: 0.0 };
        assert!((capture_point_distance_to_centre(&cp, 0.0, 0.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_cp_velocity_len() {
        /* velocity output has 2 components */
        let v = capture_point_velocity(0.1, 0.0, 0.5, 0.0, 3.13);
        assert_eq!(v.len(), 2);
    }
}
