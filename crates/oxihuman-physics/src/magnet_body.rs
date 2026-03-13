// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Magnet body: dipole magnetic field and force computations.

/// Magnetic dipole body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MagnetBody {
    pub position: [f32; 3],
    pub moment: [f32; 3],
    pub mass: f32,
    pub velocity: [f32; 3],
}

/// Create a new `MagnetBody`.
#[allow(dead_code)]
pub fn new_magnet_body(position: [f32; 3], moment: [f32; 3], mass: f32) -> MagnetBody {
    MagnetBody {
        position,
        moment,
        mass: mass.max(1e-9),
        velocity: [0.0; 3],
    }
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

const MU0_OVER_4PI: f32 = 1e-7; // μ₀/4π in SI

/// Magnetic field of a dipole at point `p` (SI units, T).
#[allow(dead_code)]
pub fn mb_field_at(body: &MagnetBody, p: [f32; 3]) -> [f32; 3] {
    let r_vec = sub3(p, body.position);
    let r = len3(r_vec);
    if r < 1e-9 {
        return [0.0; 3];
    }
    let r5 = r * r * r * r * r;
    let r3 = r * r * r;
    let m_dot_r = dot3(body.moment, r_vec);
    let term1 = scale3(r_vec, 3.0 * m_dot_r / r5);
    let term2 = scale3(body.moment, 1.0 / r3);
    let b = sub3(term1, term2);
    scale3(b, MU0_OVER_4PI)
}

/// Magnitude of the dipole moment.
#[allow(dead_code)]
pub fn mb_moment_mag(body: &MagnetBody) -> f32 {
    len3(body.moment)
}

/// Force on `body` due to an external field gradient (simplified).
/// Force = ∇(m · B) ≈ m_mag * |B| / r per dimension (crude approximation).
#[allow(dead_code)]
pub fn mb_force_from_field(body: &MagnetBody, b_field: [f32; 3]) -> [f32; 3] {
    let m_mag = mb_moment_mag(body);
    let b_mag = len3(b_field);
    let scale = m_mag * b_mag;
    if b_mag < 1e-12 {
        return [0.0; 3];
    }
    scale3(b_field, scale / b_mag)
}

/// Potential energy of dipole in external field: U = -m · B.
#[allow(dead_code)]
pub fn mb_potential_energy(body: &MagnetBody, b_field: [f32; 3]) -> f32 {
    -dot3(body.moment, b_field)
}

/// Step the body under magnetic force with Euler integration.
#[allow(dead_code)]
pub fn mb_step(body: &mut MagnetBody, force: [f32; 3], dt: f32) {
    let a = scale3(force, 1.0 / body.mass);
    body.velocity = add3(body.velocity, scale3(a, dt));
    body.position = add3(body.position, scale3(body.velocity, dt));
}

/// Distance between two magnet bodies.
#[allow(dead_code)]
pub fn mb_distance(a: &MagnetBody, b: &MagnetBody) -> f32 {
    len3(sub3(a.position, b.position))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_new_magnet_body() {
        let mb = new_magnet_body([0.0; 3], [0.0, 0.0, 1.0], 1.0);
        assert!((mb.mass - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_moment_mag() {
        let mb = new_magnet_body([0.0; 3], [3.0, 4.0, 0.0], 1.0);
        assert!((mb_moment_mag(&mb) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_field_at_origin_zero() {
        let mb = new_magnet_body([0.0; 3], [0.0, 0.0, 1.0], 1.0);
        let b = mb_field_at(&mb, [0.0; 3]);
        assert!((b[0].abs() + b[1].abs() + b[2].abs()) < 1e-9);
    }

    #[test]
    fn test_field_nonzero_at_distance() {
        let mb = new_magnet_body([0.0; 3], [0.0, 0.0, 1.0], 1.0);
        let b = mb_field_at(&mb, [0.0, 0.0, 1.0]);
        assert!(b[2].abs() > 0.0);
    }

    #[test]
    fn test_potential_energy_aligned() {
        let mb = new_magnet_body([0.0; 3], [0.0, 0.0, 1.0], 1.0);
        let b_field = [0.0, 0.0, 1.0];
        let u = mb_potential_energy(&mb, b_field);
        assert!(u < 0.0);
    }

    #[test]
    fn test_force_from_field() {
        let mb = new_magnet_body([0.0; 3], [0.0, 0.0, 1.0], 1.0);
        let f = mb_force_from_field(&mb, [0.0, 0.0, 1.0]);
        assert!(f[2].abs() > 0.0);
    }

    #[test]
    fn test_step_moves_body() {
        let mut mb = new_magnet_body([0.0; 3], [0.0, 0.0, 1.0], 1.0);
        mb_step(&mut mb, [1.0, 0.0, 0.0], 1.0);
        assert!(mb.position[0].abs() > 0.0);
    }

    #[test]
    fn test_distance() {
        let a = new_magnet_body([0.0; 3], [0.0, 0.0, 1.0], 1.0);
        let b = new_magnet_body([3.0, 4.0, 0.0], [0.0, 0.0, 1.0], 1.0);
        assert!((mb_distance(&a, &b) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_pi_used() {
        let circle_area = PI * 1.0 * 1.0;
        assert!(circle_area > 3.0);
    }
}
