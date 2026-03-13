// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Body friction models: Coulomb friction, viscous friction, anisotropic friction.

/// Friction model type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrictionModelType {
    Coulomb,
    Viscous,
    Anisotropic,
}

/// Parameters for body friction.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BodyFrictionParams {
    pub model: FrictionModelType,
    pub static_coeff: f32,
    pub dynamic_coeff: f32,
    pub viscous_coeff: f32,
    pub primary_dir: [f32; 3],
    pub primary_coeff: f32,
    pub secondary_coeff: f32,
}

#[allow(dead_code)]
impl BodyFrictionParams {
    pub fn coulomb(static_c: f32, dynamic_c: f32) -> Self {
        Self {
            model: FrictionModelType::Coulomb,
            static_coeff: static_c,
            dynamic_coeff: dynamic_c,
            viscous_coeff: 0.0,
            primary_dir: [1.0, 0.0, 0.0],
            primary_coeff: dynamic_c,
            secondary_coeff: dynamic_c,
        }
    }

    pub fn viscous(coeff: f32) -> Self {
        Self {
            model: FrictionModelType::Viscous,
            static_coeff: 0.0,
            dynamic_coeff: 0.0,
            viscous_coeff: coeff,
            primary_dir: [1.0, 0.0, 0.0],
            primary_coeff: 0.0,
            secondary_coeff: 0.0,
        }
    }

    pub fn anisotropic(primary_dir: [f32; 3], primary: f32, secondary: f32) -> Self {
        Self {
            model: FrictionModelType::Anisotropic,
            static_coeff: primary,
            dynamic_coeff: secondary,
            viscous_coeff: 0.0,
            primary_dir: normalize(primary_dir),
            primary_coeff: primary,
            secondary_coeff: secondary,
        }
    }
}

#[allow(dead_code)]
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-10 {
        [1.0, 0.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

#[allow(dead_code)]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn v3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Compute Coulomb friction force magnitude.
#[allow(dead_code)]
pub fn coulomb_friction(normal_force: f32, tangent_speed: f32, params: &BodyFrictionParams) -> f32 {
    let coeff = if tangent_speed < 1e-4 {
        params.static_coeff
    } else {
        params.dynamic_coeff
    };
    coeff * normal_force
}

/// Compute viscous friction force.
#[allow(dead_code)]
pub fn viscous_friction(velocity: [f32; 3], params: &BodyFrictionParams) -> [f32; 3] {
    let c = -params.viscous_coeff;
    [velocity[0] * c, velocity[1] * c, velocity[2] * c]
}

/// Compute anisotropic friction: different coefficients along primary vs secondary direction.
#[allow(dead_code)]
pub fn anisotropic_friction(
    tangent_vel: [f32; 3],
    normal_force: f32,
    params: &BodyFrictionParams,
) -> [f32; 3] {
    let primary = dot3(tangent_vel, params.primary_dir);
    let secondary_vec = [
        tangent_vel[0] - primary * params.primary_dir[0],
        tangent_vel[1] - primary * params.primary_dir[1],
        tangent_vel[2] - primary * params.primary_dir[2],
    ];
    let secondary_speed = v3_len(secondary_vec);
    let primary_force = params.primary_coeff * normal_force;
    let secondary_force = params.secondary_coeff * normal_force;
    let tangent_speed = v3_len(tangent_vel);
    if tangent_speed < 1e-10 {
        return [0.0; 3];
    }
    let pf = if primary.abs() > 1e-10 {
        -primary_force * primary.signum()
    } else {
        0.0
    };
    let mut result = [
        pf * params.primary_dir[0],
        pf * params.primary_dir[1],
        pf * params.primary_dir[2],
    ];
    if secondary_speed > 1e-10 {
        let sn = [
            secondary_vec[0] / secondary_speed,
            secondary_vec[1] / secondary_speed,
            secondary_vec[2] / secondary_speed,
        ];
        result[0] -= secondary_force * sn[0];
        result[1] -= secondary_force * sn[1];
        result[2] -= secondary_force * sn[2];
    }
    result
}

/// Maximum static friction before sliding.
#[allow(dead_code)]
pub fn max_static_friction(normal_force: f32, static_coeff: f32) -> f32 {
    static_coeff * normal_force
}

/// Kinetic friction force magnitude.
#[allow(dead_code)]
pub fn kinetic_friction(normal_force: f32, dynamic_coeff: f32) -> f32 {
    dynamic_coeff * normal_force
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_1_SQRT_2;

    #[test]
    fn test_coulomb_static() {
        let p = BodyFrictionParams::coulomb(0.5, 0.3);
        let f = coulomb_friction(10.0, 0.0, &p);
        assert!((f - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_coulomb_dynamic() {
        let p = BodyFrictionParams::coulomb(0.5, 0.3);
        let f = coulomb_friction(10.0, 1.0, &p);
        assert!((f - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_viscous() {
        let p = BodyFrictionParams::viscous(2.0);
        let f = viscous_friction([3.0, 0.0, 0.0], &p);
        assert!((f[0] - (-6.0)).abs() < 0.01);
    }

    #[test]
    fn test_max_static() {
        assert!((max_static_friction(20.0, 0.5) - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_kinetic() {
        assert!((kinetic_friction(20.0, 0.3) - 6.0).abs() < 0.01);
    }

    #[test]
    fn test_anisotropic_primary() {
        let p = BodyFrictionParams::anisotropic([1.0, 0.0, 0.0], 0.5, 0.1);
        let f = anisotropic_friction([1.0, 0.0, 0.0], 10.0, &p);
        assert!(f[0] < 0.0); // opposes motion
        assert!((f[0].abs() - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_anisotropic_zero_vel() {
        let p = BodyFrictionParams::anisotropic([1.0, 0.0, 0.0], 0.5, 0.1);
        let f = anisotropic_friction([0.0; 3], 10.0, &p);
        assert!(v3_len(f) < 0.001);
    }

    #[test]
    fn test_friction_model_type() {
        let p = BodyFrictionParams::coulomb(0.5, 0.3);
        assert_eq!(p.model, FrictionModelType::Coulomb);
    }

    #[test]
    fn test_viscous_model_type() {
        let p = BodyFrictionParams::viscous(1.0);
        assert_eq!(p.model, FrictionModelType::Viscous);
    }

    #[test]
    fn test_frac_1_sqrt_2() {
        // Verify const usage
        let v = FRAC_1_SQRT_2;
        assert!((v * v - 0.5).abs() < 1e-6);
    }
}
