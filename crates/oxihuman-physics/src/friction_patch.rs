// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Friction patch: a contact region carrying friction state (stick/slip).

use std::f32::consts::FRAC_1_SQRT_2;

/// State of a friction patch contact point.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrictionState {
    Stick,
    Slip,
}

/// A friction patch contact point with accumulated tangential impulse.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FrictionPatch {
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
    pub accumulated_tangent: f32,
    pub accumulated_bitangent: f32,
    pub normal_impulse: f32,
    pub friction_coeff: f32,
    pub state: FrictionState,
}

#[allow(dead_code)]
impl FrictionPatch {
    pub fn new(normal: [f32; 3], friction_coeff: f32) -> Self {
        let (tangent, bitangent) = compute_tangent_basis(normal);
        Self {
            normal,
            tangent,
            bitangent,
            accumulated_tangent: 0.0,
            accumulated_bitangent: 0.0,
            normal_impulse: 0.0,
            friction_coeff,
            state: FrictionState::Stick,
        }
    }

    /// Maximum tangential impulse (Coulomb cone).
    pub fn max_tangential_impulse(&self) -> f32 {
        self.friction_coeff * self.normal_impulse.abs()
    }

    /// Apply tangential impulse delta; clamp to friction cone.
    pub fn apply_tangential(&mut self, dt: f32, db: f32) -> (f32, f32) {
        let new_t = self.accumulated_tangent + dt;
        let new_b = self.accumulated_bitangent + db;
        let mag = (new_t * new_t + new_b * new_b).sqrt();
        let max_t = self.max_tangential_impulse();

        if mag > max_t {
            // Slip: project onto friction cone
            let scale = max_t / mag.max(1e-9);
            self.accumulated_tangent = new_t * scale;
            self.accumulated_bitangent = new_b * scale;
            self.state = FrictionState::Slip;
        } else {
            self.accumulated_tangent = new_t;
            self.accumulated_bitangent = new_b;
            self.state = FrictionState::Stick;
        }
        (self.accumulated_tangent, self.accumulated_bitangent)
    }

    pub fn reset_accumulation(&mut self) {
        self.accumulated_tangent = 0.0;
        self.accumulated_bitangent = 0.0;
        self.state = FrictionState::Stick;
    }

    /// Tangential impulse vector in world space.
    pub fn tangential_impulse_world(&self) -> [f32; 3] {
        [
            self.tangent[0] * self.accumulated_tangent
                + self.bitangent[0] * self.accumulated_bitangent,
            self.tangent[1] * self.accumulated_tangent
                + self.bitangent[1] * self.accumulated_bitangent,
            self.tangent[2] * self.accumulated_tangent
                + self.bitangent[2] * self.accumulated_bitangent,
        ]
    }
}

/// Build an orthonormal tangent basis for a given normal.
#[allow(dead_code)]
pub fn compute_tangent_basis(n: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    // Choose least-aligned axis
    let t = if n[0].abs() < FRAC_1_SQRT_2 {
        let len = (n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
        [0.0, -n[2] / len, n[1] / len]
    } else {
        let len = (n[0] * n[0] + n[2] * n[2]).sqrt().max(1e-9);
        [-n[2] / len, 0.0, n[0] / len]
    };
    let b = [
        n[1] * t[2] - n[2] * t[1],
        n[2] * t[0] - n[0] * t[2],
        n[0] * t[1] - n[1] * t[0],
    ];
    (t, b)
}

/// Static friction check: true if tangential force is below static limit.
#[allow(dead_code)]
pub fn is_sticking(tangential_force: f32, normal_force: f32, static_coeff: f32) -> bool {
    tangential_force.abs() <= static_coeff * normal_force.abs()
}

/// Kinetic friction force magnitude.
#[allow(dead_code)]
pub fn kinetic_friction_magnitude(normal_force: f32, kinetic_coeff: f32) -> f32 {
    kinetic_coeff * normal_force.abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tangent_basis_orthogonal_to_normal() {
        let n = [0.0f32, 1.0, 0.0];
        let (t, b) = compute_tangent_basis(n);
        let nt = n[0] * t[0] + n[1] * t[1] + n[2] * t[2];
        let nb = n[0] * b[0] + n[1] * b[1] + n[2] * b[2];
        assert!(nt.abs() < 1e-5);
        assert!(nb.abs() < 1e-5);
    }

    #[test]
    fn tangent_basis_unit_vectors() {
        let n = [1.0f32, 0.0, 0.0];
        let (t, b) = compute_tangent_basis(n);
        let lt = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
        let lb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
        assert!((lt - 1.0).abs() < 1e-5);
        assert!((lb - 1.0).abs() < 1e-5);
    }

    #[test]
    fn friction_patch_new_stick_state() {
        let p = FrictionPatch::new([0.0, 1.0, 0.0], 0.5);
        assert_eq!(p.state, FrictionState::Stick);
    }

    #[test]
    fn apply_tangential_stays_stick_within_cone() {
        let mut p = FrictionPatch::new([0.0, 1.0, 0.0], 0.5);
        p.normal_impulse = 10.0;
        p.apply_tangential(1.0, 0.0); // 1.0 < 0.5*10=5
        assert_eq!(p.state, FrictionState::Stick);
    }

    #[test]
    fn apply_tangential_slips_outside_cone() {
        let mut p = FrictionPatch::new([0.0, 1.0, 0.0], 0.3);
        p.normal_impulse = 2.0;
        p.apply_tangential(5.0, 5.0); // mag > 0.3*2=0.6
        assert_eq!(p.state, FrictionState::Slip);
    }

    #[test]
    fn max_tangential_impulse_formula() {
        let mut p = FrictionPatch::new([0.0, 1.0, 0.0], 0.4);
        p.normal_impulse = 5.0;
        assert!((p.max_tangential_impulse() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn reset_accumulation() {
        let mut p = FrictionPatch::new([0.0, 1.0, 0.0], 0.5);
        p.normal_impulse = 10.0;
        p.apply_tangential(2.0, 2.0);
        p.reset_accumulation();
        assert_eq!(p.accumulated_tangent, 0.0);
        assert_eq!(p.state, FrictionState::Stick);
    }

    #[test]
    fn is_sticking_below_limit() {
        assert!(is_sticking(2.0, 10.0, 0.3));
    }

    #[test]
    fn is_sticking_above_limit() {
        assert!(!is_sticking(4.0, 10.0, 0.3));
    }

    #[test]
    fn kinetic_friction_magnitude_formula() {
        assert!((kinetic_friction_magnitude(10.0, 0.25) - 2.5).abs() < 1e-6);
    }
}
