// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 3D damped spring between two particles.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Spring3dConfig {
    pub stiffness: f32,
    pub damping: f32,
    pub rest_length: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Spring3d {
    pub pos_a: [f32; 3],
    pub pos_b: [f32; 3],
    pub vel_a: [f32; 3],
    pub vel_b: [f32; 3],
    pub config: Spring3dConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Spring3dResult {
    pub force_a: [f32; 3],
    pub force_b: [f32; 3],
    pub stretch: f32,
}

#[allow(dead_code)]
pub fn default_spring3d_config() -> Spring3dConfig {
    Spring3dConfig { stiffness: 100.0, damping: 5.0, rest_length: 1.0 }
}

#[allow(dead_code)]
pub fn new_spring3d(pos_a: [f32; 3], pos_b: [f32; 3], config: Spring3dConfig) -> Spring3d {
    Spring3d {
        pos_a,
        pos_b,
        vel_a: [0.0; 3],
        vel_b: [0.0; 3],
        config,
    }
}

fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
pub fn spring3d_length(s: &Spring3d) -> f32 {
    vec3_len(vec3_sub(s.pos_b, s.pos_a))
}

#[allow(dead_code)]
pub fn spring3d_stretch(s: &Spring3d) -> f32 {
    spring3d_length(s) - s.config.rest_length
}

#[allow(dead_code)]
pub fn spring3d_compute_force(s: &Spring3d) -> Spring3dResult {
    let delta = vec3_sub(s.pos_b, s.pos_a);
    let length = vec3_len(delta);
    let stretch = length - s.config.rest_length;

    if length < 1e-9 {
        return Spring3dResult {
            force_a: [0.0; 3],
            force_b: [0.0; 3],
            stretch: 0.0,
        };
    }

    let dir = [delta[0] / length, delta[1] / length, delta[2] / length];
    let rel_vel = vec3_sub(s.vel_b, s.vel_a);
    let rel_vel_along = vec3_dot(rel_vel, dir);

    let force_mag = s.config.stiffness * stretch + s.config.damping * rel_vel_along;

    let force_a = [dir[0] * force_mag, dir[1] * force_mag, dir[2] * force_mag];
    let force_b = [-force_a[0], -force_a[1], -force_a[2]];

    Spring3dResult { force_a, force_b, stretch }
}

#[allow(dead_code)]
pub fn spring3d_step(s: &mut Spring3d, dt: f32, inv_mass_a: f32, inv_mass_b: f32) {
    let res = spring3d_compute_force(s);
    s.vel_a[0] += res.force_a[0] * inv_mass_a * dt;
    s.vel_a[1] += res.force_a[1] * inv_mass_a * dt;
    s.vel_a[2] += res.force_a[2] * inv_mass_a * dt;
    s.vel_b[0] += res.force_b[0] * inv_mass_b * dt;
    s.vel_b[1] += res.force_b[1] * inv_mass_b * dt;
    s.vel_b[2] += res.force_b[2] * inv_mass_b * dt;
    s.pos_a[0] += s.vel_a[0] * dt;
    s.pos_a[1] += s.vel_a[1] * dt;
    s.pos_a[2] += s.vel_a[2] * dt;
    s.pos_b[0] += s.vel_b[0] * dt;
    s.pos_b[1] += s.vel_b[1] * dt;
    s.pos_b[2] += s.vel_b[2] * dt;
}

#[allow(dead_code)]
pub fn spring3d_energy(s: &Spring3d) -> f32 {
    let stretch = spring3d_stretch(s);
    0.5 * s.config.stiffness * stretch * stretch
}

#[allow(dead_code)]
pub fn spring3d_is_at_rest(s: &Spring3d, tol: f32) -> bool {
    let speed_a = vec3_len(s.vel_a);
    let speed_b = vec3_len(s.vel_b);
    speed_a < tol && speed_b < tol && spring3d_stretch(s).abs() < tol
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_spring() {
        let cfg = default_spring3d_config();
        let s = new_spring3d([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], cfg);
        assert!((spring3d_length(&s) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_stretch_at_rest_length() {
        let cfg = Spring3dConfig { stiffness: 100.0, damping: 5.0, rest_length: 1.0 };
        let s = new_spring3d([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], cfg);
        assert!(spring3d_stretch(&s).abs() < 1e-6);
    }

    #[test]
    fn test_stretch_positive_when_extended() {
        let cfg = Spring3dConfig { stiffness: 100.0, damping: 5.0, rest_length: 1.0 };
        let s = new_spring3d([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], cfg);
        assert!(spring3d_stretch(&s) > 0.0);
    }

    #[test]
    fn test_force_zero_at_rest() {
        let cfg = Spring3dConfig { stiffness: 100.0, damping: 5.0, rest_length: 1.0 };
        let s = new_spring3d([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], cfg);
        let res = spring3d_compute_force(&s);
        assert!(res.force_a[0].abs() < 1e-5);
    }

    #[test]
    fn test_force_nonzero_when_stretched() {
        let cfg = Spring3dConfig { stiffness: 100.0, damping: 0.0, rest_length: 1.0 };
        let s = new_spring3d([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], cfg);
        let res = spring3d_compute_force(&s);
        assert!(res.force_a[0].abs() > 0.0);
    }

    #[test]
    fn test_energy_at_rest_is_zero() {
        let cfg = Spring3dConfig { stiffness: 100.0, damping: 5.0, rest_length: 1.0 };
        let s = new_spring3d([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], cfg);
        assert!(spring3d_energy(&s).abs() < 1e-5);
    }

    #[test]
    fn test_energy_positive_when_stretched() {
        let cfg = Spring3dConfig { stiffness: 100.0, damping: 5.0, rest_length: 1.0 };
        let s = new_spring3d([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], cfg);
        assert!(spring3d_energy(&s) > 0.0);
    }

    #[test]
    fn test_is_at_rest() {
        let cfg = Spring3dConfig { stiffness: 100.0, damping: 5.0, rest_length: 1.0 };
        let s = new_spring3d([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], cfg);
        assert!(spring3d_is_at_rest(&s, 1e-4));
    }

    #[test]
    fn test_step_moves_particles() {
        let cfg = Spring3dConfig { stiffness: 100.0, damping: 0.0, rest_length: 0.5 };
        let mut s = new_spring3d([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], cfg);
        let pos_a_before = s.pos_a[0];
        spring3d_step(&mut s, 0.01, 1.0, 1.0);
        // After step, positions should change due to force
        assert!((s.pos_a[0] - pos_a_before).abs() > 0.0 || s.pos_b[0] != 1.0);
    }
}
