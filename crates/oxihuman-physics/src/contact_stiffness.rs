// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Contact stiffness model for collision response.
#[derive(Debug, Clone)]
pub struct ContactStiffnessModel {
    pub stiffness: f32,
    pub restitution: f32,
    pub friction: f32,
}

/// Create a new ContactStiffnessModel.
pub fn new_contact_stiffness(k: f32, e: f32, mu: f32) -> ContactStiffnessModel {
    ContactStiffnessModel {
        stiffness: k,
        restitution: e,
        friction: mu,
    }
}

/// Normal force from Hooke's law: F = k * penetration.
pub fn contact_normal_force(m: &ContactStiffnessModel, penetration: f32) -> f32 {
    m.stiffness * penetration.max(0.0)
}

/// Impulse needed to resolve relative velocity along contact normal.
pub fn contact_impulse(m: &ContactStiffnessModel, rel_vel: f32, mass: f32) -> f32 {
    if mass < 1e-12 {
        return 0.0;
    }
    -(1.0 + m.restitution) * rel_vel / mass
}

/// Friction force: mu * normal_force.
pub fn contact_friction_force(m: &ContactStiffnessModel, normal_f: f32) -> f32 {
    m.friction * normal_f.abs()
}

/// Energy lost in collision (kinetic energy difference).
pub fn contact_energy_loss(
    m: &ContactStiffnessModel,
    v_before: f32,
    v_after: f32,
    mass: f32,
) -> f32 {
    let _ = m;
    0.5 * mass * (v_before * v_before - v_after * v_after)
}

/// Returns true when the contact stiffness exceeds 1000 N/m.
pub fn contact_is_stiff(m: &ContactStiffnessModel) -> bool {
    m.stiffness > 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_contact_stiffness() {
        /* constructor fields */
        let m = new_contact_stiffness(500.0, 0.5, 0.3);
        assert!((m.stiffness - 500.0).abs() < 1e-9);
        assert!((m.restitution - 0.5).abs() < 1e-9);
        assert!((m.friction - 0.3).abs() < 1e-9);
    }

    #[test]
    fn test_contact_normal_force_positive() {
        /* positive penetration */
        let m = new_contact_stiffness(200.0, 0.5, 0.2);
        let f = contact_normal_force(&m, 0.1);
        assert!((f - 20.0).abs() < 1e-5);
    }

    #[test]
    fn test_contact_normal_force_no_penetration() {
        /* negative penetration -> zero force */
        let m = new_contact_stiffness(200.0, 0.5, 0.2);
        let f = contact_normal_force(&m, -0.05);
        assert!(f.abs() < 1e-9);
    }

    #[test]
    fn test_contact_impulse() {
        /* impulse for head-on collision */
        let m = new_contact_stiffness(1000.0, 1.0, 0.0);
        let j = contact_impulse(&m, -2.0, 1.0);
        assert!((j - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_contact_impulse_zero_mass() {
        /* zero mass returns 0 */
        let m = new_contact_stiffness(1000.0, 1.0, 0.0);
        assert!(contact_impulse(&m, -2.0, 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_contact_friction_force() {
        /* friction = mu * normal */
        let m = new_contact_stiffness(100.0, 0.0, 0.4);
        let ff = contact_friction_force(&m, 10.0);
        assert!((ff - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_contact_energy_loss() {
        /* loss from v=4 to v=2 for mass=1 -> 6 J */
        let m = new_contact_stiffness(100.0, 0.5, 0.3);
        let e = contact_energy_loss(&m, 4.0, 2.0, 1.0);
        assert!((e - 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_contact_is_stiff_true() {
        let m = new_contact_stiffness(2000.0, 0.0, 0.0);
        assert!(contact_is_stiff(&m));
    }

    #[test]
    fn test_contact_is_stiff_false() {
        let m = new_contact_stiffness(500.0, 0.0, 0.0);
        assert!(!contact_is_stiff(&m));
    }

    #[test]
    fn test_contact_is_stiff_boundary() {
        /* exactly 1000 is NOT stiff (> not >=) */
        let m = new_contact_stiffness(1000.0, 0.0, 0.0);
        assert!(!contact_is_stiff(&m));
    }
}
