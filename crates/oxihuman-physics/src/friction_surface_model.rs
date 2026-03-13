// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Friction surface model: combines static/kinetic friction with material properties.

/// Surface material for friction.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SurfaceMaterial {
    pub static_friction: f32,
    pub kinetic_friction: f32,
    pub rolling_friction: f32,
    pub restitution: f32,
}

/// Default rubber-on-concrete material.
#[allow(dead_code)]
pub fn rubber_concrete() -> SurfaceMaterial {
    SurfaceMaterial {
        static_friction: 0.9,
        kinetic_friction: 0.7,
        rolling_friction: 0.02,
        restitution: 0.4,
    }
}

/// Default ice-on-ice material.
#[allow(dead_code)]
pub fn ice_ice() -> SurfaceMaterial {
    SurfaceMaterial {
        static_friction: 0.1,
        kinetic_friction: 0.03,
        rolling_friction: 0.001,
        restitution: 0.1,
    }
}

/// Combined friction coefficient using geometric mean.
#[allow(dead_code)]
pub fn combine_friction(a: &SurfaceMaterial, b: &SurfaceMaterial) -> (f32, f32) {
    let stat = (a.static_friction * b.static_friction).sqrt();
    let kin = (a.kinetic_friction * b.kinetic_friction).sqrt();
    (stat, kin)
}

/// Whether the contact is sticking (tangential force within static cone).
#[allow(dead_code)]
pub fn is_sticking_contact(mu_static: f32, normal_force: f32, tangential_force: f32) -> bool {
    tangential_force.abs() <= mu_static * normal_force.abs()
}

/// Kinetic friction force magnitude.
#[allow(dead_code)]
pub fn kinetic_friction_force(mu_kinetic: f32, normal_force: f32) -> f32 {
    mu_kinetic * normal_force.abs()
}

/// Friction impulse given slip velocity and normal impulse.
#[allow(dead_code)]
pub fn friction_impulse(mu: f32, normal_impulse: f32, slip_speed: f32) -> f32 {
    if slip_speed < 1e-8 {
        0.0
    } else {
        (mu * normal_impulse.abs()).min(mu * normal_impulse.abs())
    }
}

/// Angle of repose for a material (angle at which a body starts sliding).
#[allow(dead_code)]
pub fn angle_of_repose(mu_static: f32) -> f32 {
    mu_static.atan()
}

/// Maximum static friction for a given slope angle.
#[allow(dead_code)]
pub fn static_friction_on_slope(mat: &SurfaceMaterial, mass: f32, g: f32, slope_rad: f32) -> f32 {
    let normal = mass * g * slope_rad.cos();
    mat.static_friction * normal
}

/// Rolling resistance force.
#[allow(dead_code)]
pub fn rolling_resistance(mat: &SurfaceMaterial, normal_force: f32) -> f32 {
    mat.rolling_friction * normal_force.abs()
}

/// Friction power dissipation.
#[allow(dead_code)]
pub fn friction_power(friction_force: f32, slip_speed: f32) -> f32 {
    friction_force.abs() * slip_speed.abs()
}

/// Combined restitution (geometric mean).
#[allow(dead_code)]
pub fn combine_restitution(a: &SurfaceMaterial, b: &SurfaceMaterial) -> f32 {
    (a.restitution * b.restitution).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sticking_within_cone() {
        assert!(is_sticking_contact(0.5, 10.0, 4.0));
        assert!(!is_sticking_contact(0.5, 10.0, 6.0));
    }

    #[test]
    fn test_kinetic_friction() {
        let f = kinetic_friction_force(0.3, 100.0);
        assert!((f - 30.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_combine_friction() {
        let a = rubber_concrete();
        let b = ice_ice();
        let (stat, kin) = combine_friction(&a, &b);
        assert!(stat < a.static_friction);
        assert!(kin < a.kinetic_friction);
    }

    #[test]
    fn test_angle_of_repose() {
        // For mu=1.0, repose angle = 45°
        let angle = angle_of_repose(1.0);
        assert!((angle - std::f32::consts::FRAC_PI_4).abs() < 1e-5);
    }

    #[test]
    fn test_rolling_resistance() {
        let mat = rubber_concrete();
        let r = rolling_resistance(&mat, 100.0);
        assert!((r - 2.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_friction_power() {
        let p = friction_power(10.0, 2.0);
        assert!((p - 20.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_combine_restitution() {
        let a = rubber_concrete();
        let b = rubber_concrete();
        let r = combine_restitution(&a, &b);
        assert!((r - 0.4_f32).abs() < 1e-5);
    }

    #[test]
    fn test_static_on_slope() {
        let mat = rubber_concrete();
        let f = static_friction_on_slope(&mat, 1.0, 9.81, 0.0);
        assert!((f - mat.static_friction * 9.81).abs() < 1e-4);
    }

    #[test]
    fn test_friction_impulse_zero_slip() {
        assert_eq!(friction_impulse(0.5, 10.0, 0.0), 0.0);
    }
}
