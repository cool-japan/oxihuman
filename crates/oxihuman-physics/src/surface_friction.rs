// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Surface friction material properties.

#![allow(dead_code)]

/// Friction material properties for a surface.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FrictionMaterial {
    pub static_friction: f32,
    pub dynamic_friction: f32,
    pub restitution: f32,
    pub name: String,
}

/// Create a default friction material (rubber-like).
#[allow(dead_code)]
pub fn default_friction_material() -> FrictionMaterial {
    FrictionMaterial {
        static_friction: 0.6,
        dynamic_friction: 0.4,
        restitution: 0.3,
        name: "default".to_string(),
    }
}

/// Combine friction coefficients from two materials using geometric mean.
#[allow(dead_code)]
pub fn combined_friction(a: &FrictionMaterial, b: &FrictionMaterial) -> f32 {
    (a.dynamic_friction * b.dynamic_friction).sqrt()
}

/// Compute friction force magnitude given normal force and friction coefficient.
#[allow(dead_code)]
pub fn friction_force(normal_force: f32, coeff: f32) -> f32 {
    normal_force.abs() * coeff.abs()
}

/// Determine if an object is rolling (as opposed to sliding).
/// Rolling condition: |angular_vel * radius - linear_vel| < threshold
#[allow(dead_code)]
pub fn is_rolling(angular_vel: f32, linear_vel: f32, radius: f32) -> bool {
    let threshold = 1e-3;
    (angular_vel * radius - linear_vel).abs() < threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_material_values() {
        let m = default_friction_material();
        assert!(m.static_friction > 0.0);
        assert!(m.dynamic_friction > 0.0);
        assert!(m.restitution >= 0.0);
        assert_eq!(m.name, "default");
    }

    #[test]
    fn test_combined_friction_equal() {
        let m = default_friction_material();
        let c = combined_friction(&m, &m);
        assert!((c - m.dynamic_friction).abs() < 1e-5);
    }

    #[test]
    fn test_combined_friction_zero() {
        let mut m1 = default_friction_material();
        let m2 = default_friction_material();
        m1.dynamic_friction = 0.0;
        let c = combined_friction(&m1, &m2);
        assert!(c.abs() < 1e-9);
    }

    #[test]
    fn test_friction_force_positive() {
        let f = friction_force(10.0, 0.5);
        assert!((f - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_friction_force_zero_normal() {
        let f = friction_force(0.0, 0.8);
        assert!(f.abs() < 1e-9);
    }

    #[test]
    fn test_friction_force_uses_abs() {
        let f = friction_force(-10.0, 0.5);
        assert!((f - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_rolling_true() {
        // angular_vel=2, linear_vel=2, radius=1 → rolling
        assert!(is_rolling(2.0, 2.0, 1.0));
    }

    #[test]
    fn test_is_rolling_false() {
        // angular_vel=0, linear_vel=5 → sliding
        assert!(!is_rolling(0.0, 5.0, 1.0));
    }

    #[test]
    fn test_combined_friction_geometric_mean() {
        let mut a = default_friction_material();
        let mut b = default_friction_material();
        a.dynamic_friction = 0.4;
        b.dynamic_friction = 0.9;
        let c = combined_friction(&a, &b);
        let expected = (0.4f32 * 0.9).sqrt();
        assert!((c - expected).abs() < 1e-6);
    }
}
