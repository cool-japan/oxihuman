// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Computes post-collision velocities for a 1D elastic collision.
#[allow(dead_code)]
pub fn elastic_1d(m1: f32, v1: f32, m2: f32, v2: f32) -> (f32, f32) {
    let total = m1 + m2;
    if total < 1e-10 {
        return (v1, v2);
    }
    let v1_new = ((m1 - m2) * v1 + 2.0 * m2 * v2) / total;
    let v2_new = ((m2 - m1) * v2 + 2.0 * m1 * v1) / total;
    (v1_new, v2_new)
}

/// Computes post-collision velocities for a 1D partially inelastic collision.
#[allow(dead_code)]
pub fn inelastic_1d(m1: f32, v1: f32, m2: f32, v2: f32, restitution: f32) -> (f32, f32) {
    let total = m1 + m2;
    if total < 1e-10 {
        return (v1, v2);
    }
    let e = restitution.clamp(0.0, 1.0);
    let v_cm = (m1 * v1 + m2 * v2) / total;
    let v1_new = v_cm + e * m2 * (v2 - v1) / total;
    let v2_new = v_cm + e * m1 * (v1 - v2) / total;
    (v1_new, v2_new)
}

/// Kinetic energy: 0.5 * m * v^2
#[allow(dead_code)]
pub fn kinetic_energy(mass: f32, velocity: f32) -> f32 {
    0.5 * mass * velocity * velocity
}

/// Momentum: m * v
#[allow(dead_code)]
pub fn momentum(mass: f32, velocity: f32) -> f32 {
    mass * velocity
}

/// Coefficient of restitution from drop height and bounce height.
#[allow(dead_code)]
pub fn restitution_from_heights(drop_height: f32, bounce_height: f32) -> f32 {
    if drop_height <= 0.0 {
        return 0.0;
    }
    (bounce_height / drop_height).sqrt().clamp(0.0, 1.0)
}

/// 3D elastic collision along a contact normal.
#[allow(dead_code)]
pub fn elastic_3d(
    m1: f32, v1: [f32; 3],
    m2: f32, v2: [f32; 3],
    normal: [f32; 3],
) -> ([f32; 3], [f32; 3]) {
    let rel_v = [v1[0] - v2[0], v1[1] - v2[1], v1[2] - v2[2]];
    let vn = rel_v[0] * normal[0] + rel_v[1] * normal[1] + rel_v[2] * normal[2];
    if vn > 0.0 {
        return (v1, v2);
    }
    let total = m1 + m2;
    if total < 1e-10 {
        return (v1, v2);
    }
    let j = -2.0 * vn / total;
    let impulse = [j * normal[0], j * normal[1], j * normal[2]];
    let v1_new = [
        v1[0] + impulse[0] * m2,
        v1[1] + impulse[1] * m2,
        v1[2] + impulse[2] * m2,
    ];
    let v2_new = [
        v2[0] - impulse[0] * m1,
        v2[1] - impulse[1] * m1,
        v2[2] - impulse[2] * m1,
    ];
    (v1_new, v2_new)
}

/// Impulse magnitude for a collision with given restitution.
#[allow(dead_code)]
pub fn impulse_magnitude(m1: f32, m2: f32, rel_velocity_normal: f32, restitution: f32) -> f32 {
    let inv_mass = if m1 > 0.0 { 1.0 / m1 } else { 0.0 } + if m2 > 0.0 { 1.0 / m2 } else { 0.0 };
    if inv_mass < 1e-10 {
        return 0.0;
    }
    -(1.0 + restitution) * rel_velocity_normal / inv_mass
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elastic_1d_equal_mass() {
        let (v1, v2) = elastic_1d(1.0, 1.0, 1.0, 0.0);
        assert!(v1.abs() < 1e-6);
        assert!((v2 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_elastic_1d_momentum_conservation() {
        let (v1, v2) = elastic_1d(2.0, 3.0, 1.0, -1.0);
        let p_before = 2.0 * 3.0 + (-1.0_f32);
        let p_after = 2.0 * v1 + 1.0 * v2;
        assert!((p_before - p_after).abs() < 1e-5);
    }

    #[test]
    fn test_inelastic_1d_perfectly_inelastic() {
        let (v1, v2) = inelastic_1d(1.0, 2.0, 1.0, 0.0, 0.0);
        assert!((v1 - v2).abs() < 1e-5);
    }

    #[test]
    fn test_inelastic_matches_elastic() {
        let (v1e, v2e) = elastic_1d(1.0, 1.0, 1.0, 0.0);
        let (v1i, v2i) = inelastic_1d(1.0, 1.0, 1.0, 0.0, 1.0);
        assert!((v1e - v1i).abs() < 1e-5);
        assert!((v2e - v2i).abs() < 1e-5);
    }

    #[test]
    fn test_kinetic_energy() {
        assert!((kinetic_energy(2.0, 3.0) - 9.0).abs() < 1e-6);
    }

    #[test]
    fn test_momentum_fn() {
        assert!((momentum(3.0, 4.0) - 12.0).abs() < 1e-6);
    }

    #[test]
    fn test_restitution_from_heights() {
        let e = restitution_from_heights(1.0, 0.25);
        assert!((e - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_restitution_from_zero_drop() {
        assert!((restitution_from_heights(0.0, 1.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_elastic_3d_separating() {
        let (v1, v2) = elastic_3d(1.0, [1.0, 0.0, 0.0], 1.0, [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        // Already separating (rel_v dot normal > 0), no change
        assert!((v1[0] - 1.0).abs() < 1e-6);
        assert!((v2[0] - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_impulse_magnitude() {
        let j = impulse_magnitude(1.0, 1.0, -2.0, 0.5);
        assert!(j > 0.0);
    }
}
