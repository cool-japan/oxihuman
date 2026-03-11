// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Impulse-based collision resolution.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImpulseConfig {
    pub restitution: f32,
    pub friction: f32,
    pub slop: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImpulseBody {
    pub vel: [f32; 3],
    pub inv_mass: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImpulseResult {
    pub impulse: [f32; 3],
    pub delta_vel_a: [f32; 3],
    pub delta_vel_b: [f32; 3],
}

#[allow(dead_code)]
pub fn default_impulse_config() -> ImpulseConfig {
    ImpulseConfig { restitution: 0.5, friction: 0.3, slop: 0.01 }
}

#[allow(dead_code)]
pub fn new_impulse_body(vel: [f32; 3], inv_mass: f32) -> ImpulseBody {
    ImpulseBody { vel, inv_mass }
}

#[allow(dead_code)]
pub fn relative_velocity(a: &ImpulseBody, b: &ImpulseBody, normal: [f32; 3]) -> f32 {
    let rel = [
        a.vel[0] - b.vel[0],
        a.vel[1] - b.vel[1],
        a.vel[2] - b.vel[2],
    ];
    dot3(rel, normal)
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
pub fn is_separating(a: &ImpulseBody, b: &ImpulseBody, normal: [f32; 3]) -> bool {
    relative_velocity(a, b, normal) > 0.0
}

#[allow(dead_code)]
pub fn impulse_magnitude(
    a: &ImpulseBody,
    b: &ImpulseBody,
    normal: [f32; 3],
    depth: f32,
    config: &ImpulseConfig,
) -> f32 {
    let vrel = relative_velocity(a, b, normal);
    if vrel >= 0.0 {
        return 0.0;
    }
    let e = config.restitution;
    let total_inv_mass = a.inv_mass + b.inv_mass;
    if total_inv_mass <= 0.0 {
        return 0.0;
    }
    let correction = (depth - config.slop).max(0.0);
    let _ = correction;
    -(1.0 + e) * vrel / total_inv_mass
}

#[allow(dead_code)]
pub fn resolve_collision_impulse(
    a: &ImpulseBody,
    b: &ImpulseBody,
    normal: [f32; 3],
    depth: f32,
    config: &ImpulseConfig,
) -> ImpulseResult {
    let j = impulse_magnitude(a, b, normal, depth, config);
    let impulse = [normal[0] * j, normal[1] * j, normal[2] * j];
    let delta_vel_a = [impulse[0] * a.inv_mass, impulse[1] * a.inv_mass, impulse[2] * a.inv_mass];
    let delta_vel_b = [
        -impulse[0] * b.inv_mass,
        -impulse[1] * b.inv_mass,
        -impulse[2] * b.inv_mass,
    ];
    ImpulseResult { impulse, delta_vel_a, delta_vel_b }
}

#[allow(dead_code)]
pub fn apply_impulse_to_body(body: &mut ImpulseBody, impulse: [f32; 3], normal: [f32; 3]) {
    let proj = dot3(impulse, normal);
    body.vel[0] += normal[0] * proj * body.inv_mass;
    body.vel[1] += normal[1] * proj * body.inv_mass;
    body.vel[2] += normal[2] * proj * body.inv_mass;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_impulse_config();
        assert!(cfg.restitution >= 0.0);
    }

    #[test]
    fn test_relative_velocity_approaching() {
        let a = new_impulse_body([0.0, -1.0, 0.0], 1.0);
        let b = new_impulse_body([0.0, 1.0, 0.0], 1.0);
        let normal = [0.0, 1.0, 0.0];
        let rv = relative_velocity(&a, &b, normal);
        assert!(rv < 0.0);
    }

    #[test]
    fn test_is_separating_true() {
        let a = new_impulse_body([0.0, 2.0, 0.0], 1.0);
        let b = new_impulse_body([0.0, 0.0, 0.0], 1.0);
        let normal = [0.0, 1.0, 0.0];
        assert!(is_separating(&a, &b, normal));
    }

    #[test]
    fn test_is_separating_false() {
        let a = new_impulse_body([0.0, -2.0, 0.0], 1.0);
        let b = new_impulse_body([0.0, 0.0, 0.0], 1.0);
        let normal = [0.0, 1.0, 0.0];
        assert!(!is_separating(&a, &b, normal));
    }

    #[test]
    fn test_impulse_magnitude_zero_when_separating() {
        let a = new_impulse_body([0.0, 2.0, 0.0], 1.0);
        let b = new_impulse_body([0.0, 0.0, 0.0], 1.0);
        let normal = [0.0, 1.0, 0.0];
        let cfg = default_impulse_config();
        let j = impulse_magnitude(&a, &b, normal, 0.1, &cfg);
        assert!((j - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_resolve_collision_delta_vels_nonzero() {
        let a = new_impulse_body([0.0, -2.0, 0.0], 1.0);
        let b = new_impulse_body([0.0, 0.0, 0.0], 1.0);
        let normal = [0.0, 1.0, 0.0];
        let cfg = default_impulse_config();
        let res = resolve_collision_impulse(&a, &b, normal, 0.1, &cfg);
        assert!(res.delta_vel_a[1].abs() > 0.0);
    }

    #[test]
    fn test_apply_impulse_to_body() {
        let mut body = new_impulse_body([0.0, 0.0, 0.0], 1.0);
        let impulse = [0.0, 5.0, 0.0];
        let normal = [0.0, 1.0, 0.0];
        apply_impulse_to_body(&mut body, impulse, normal);
        assert!(body.vel[1].abs() > 0.0);
    }

    #[test]
    fn test_zero_inv_mass_no_resolution() {
        let a = new_impulse_body([0.0, -2.0, 0.0], 0.0);
        let b = new_impulse_body([0.0, 0.0, 0.0], 0.0);
        let normal = [0.0, 1.0, 0.0];
        let cfg = default_impulse_config();
        let j = impulse_magnitude(&a, &b, normal, 0.1, &cfg);
        assert!((j - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_impulse_body() {
        let b = new_impulse_body([1.0, 2.0, 3.0], 0.5);
        assert!((b.inv_mass - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_resolve_collision_impulse_direction() {
        let a = new_impulse_body([0.0, -3.0, 0.0], 1.0);
        let b = new_impulse_body([0.0, 0.0, 0.0], 1.0);
        let normal = [0.0, 1.0, 0.0];
        let cfg = default_impulse_config();
        let res = resolve_collision_impulse(&a, &b, normal, 0.05, &cfg);
        assert!(res.impulse[1] > 0.0);
        assert!(res.delta_vel_a[1] > 0.0);
        assert!(res.delta_vel_b[1] < 0.0);
    }
}
