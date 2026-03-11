// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

/// A friction cone defined by a contact normal and coefficient of friction.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FrictionCone {
    pub normal: [f32; 3],
    pub mu: f32,
}

/// Create a new friction cone.
#[allow(dead_code)]
pub fn new_friction_cone(normal: [f32; 3], mu: f32) -> FrictionCone {
    FrictionCone { normal, mu }
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len3(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a);
    if l > 1e-8 {
        [a[0] / l, a[1] / l, a[2] / l]
    } else {
        [0.0, 1.0, 0.0]
    }
}

fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Project a force vector onto the friction cone.
/// If the force is already inside the cone, return it unchanged.
/// Otherwise, project it to the cone boundary.
#[allow(dead_code)]
pub fn project_to_cone(cone: &FrictionCone, force: [f32; 3]) -> [f32; 3] {
    let n = normalize3(cone.normal);
    let fn_ = dot3(force, n); // Normal component magnitude

    if fn_ <= 0.0 {
        return [0.0; 3];
    }

    let ft = sub3(force, scale3(n, fn_)); // Tangential component
    let ft_len = len3(ft);
    let max_ft = cone.mu * fn_;

    if ft_len <= max_ft {
        // Already inside cone
        force
    } else if ft_len < 1e-8 {
        scale3(n, fn_)
    } else {
        // Clamp tangential component
        let ft_clamped = scale3(normalize3(ft), max_ft);
        add3(scale3(n, fn_), ft_clamped)
    }
}

/// Check whether a force vector is inside the friction cone.
#[allow(dead_code)]
pub fn in_friction_cone(cone: &FrictionCone, force: [f32; 3]) -> bool {
    let n = normalize3(cone.normal);
    let fn_ = dot3(force, n);
    if fn_ <= 0.0 {
        return false;
    }
    let ft = sub3(force, scale3(n, fn_));
    let ft_len = len3(ft);
    ft_len <= cone.mu * fn_
}

/// Get the half-angle of the friction cone in radians.
/// θ = atan(mu)
#[allow(dead_code)]
pub fn cone_half_angle(mu: f32) -> f32 {
    mu.atan()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_normal_force_in_cone() {
        let cone = new_friction_cone([0.0, 1.0, 0.0], 0.5);
        assert!(in_friction_cone(&cone, [0.0, 1.0, 0.0]));
    }

    #[test]
    fn excessive_tangential_not_in_cone() {
        let cone = new_friction_cone([0.0, 1.0, 0.0], 0.3);
        // Tangential force much larger than mu * normal
        assert!(!in_friction_cone(&cone, [10.0, 1.0, 0.0]));
    }

    #[test]
    fn negative_normal_not_in_cone() {
        let cone = new_friction_cone([0.0, 1.0, 0.0], 0.5);
        assert!(!in_friction_cone(&cone, [0.0, -1.0, 0.0]));
    }

    #[test]
    fn project_to_cone_inside_unchanged() {
        let cone = new_friction_cone([0.0, 1.0, 0.0], 1.0);
        let force = [0.3, 1.0, 0.0];
        let projected = project_to_cone(&cone, force);
        // Should be very close to original
        assert!((projected[0] - force[0]).abs() < 1e-5);
        assert!((projected[1] - force[1]).abs() < 1e-5);
    }

    #[test]
    fn project_to_cone_clamps_tangential() {
        let cone = new_friction_cone([0.0, 1.0, 0.0], 0.5);
        let force = [10.0, 1.0, 0.0]; // Very large tangential
        let projected = project_to_cone(&cone, force);
        // Tangential component should be mu * normal = 0.5
        let ft = projected[0];
        assert!((ft - 0.5).abs() < 1e-4);
    }

    #[test]
    fn cone_half_angle_zero_mu() {
        assert!(cone_half_angle(0.0).abs() < 1e-6);
    }

    #[test]
    fn cone_half_angle_unit_mu() {
        // atan(1) = pi/4
        let angle = cone_half_angle(1.0);
        assert!((angle - std::f32::consts::FRAC_PI_4).abs() < 1e-5);
    }

    #[test]
    fn project_negative_normal_zeroed() {
        let cone = new_friction_cone([0.0, 1.0, 0.0], 0.5);
        let projected = project_to_cone(&cone, [0.0, -2.0, 0.0]);
        assert_eq!(projected, [0.0; 3]);
    }

    #[test]
    fn in_cone_boundary() {
        // Exactly on boundary: ft_len == mu * fn
        let cone = new_friction_cone([0.0, 1.0, 0.0], 1.0);
        // Normal=1, tangential=1 → ft_len = 1 = mu * fn = 1
        assert!(in_friction_cone(&cone, [1.0, 1.0, 0.0]));
    }

    #[test]
    fn high_mu_allows_large_tangential() {
        let cone = new_friction_cone([0.0, 1.0, 0.0], 10.0);
        assert!(in_friction_cone(&cone, [5.0, 1.0, 0.0]));
    }
}
