// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Translational kinetic energy: KE = 0.5 * mass * |vel|^2
#[allow(dead_code)]
pub fn kinetic_energy(mass: f32, vel: [f32; 3]) -> f32 {
    let v2 = vel[0] * vel[0] + vel[1] * vel[1] + vel[2] * vel[2];
    0.5 * mass * v2
}

/// Gravitational potential energy: PE = mass * g * height
#[allow(dead_code)]
pub fn potential_energy(mass: f32, height: f32, g: f32) -> f32 {
    mass * g * height
}

/// Rotational kinetic energy with diagonal inertia tensor: KE_rot = 0.5 * sum(I_i * omega_i^2)
#[allow(dead_code)]
pub fn rotational_ke(inertia_diag: [f32; 3], omega: [f32; 3]) -> f32 {
    0.5 * (inertia_diag[0] * omega[0] * omega[0]
        + inertia_diag[1] * omega[1] * omega[1]
        + inertia_diag[2] * omega[2] * omega[2])
}

/// Total mechanical energy: translational KE + PE.
#[allow(dead_code)]
pub fn total_mechanical_energy(mass: f32, vel: [f32; 3], h: f32, g: f32) -> f32 {
    kinetic_energy(mass, vel) + potential_energy(mass, h, g)
}

/// Spring potential energy: PE = 0.5 * k * displacement^2
#[allow(dead_code)]
pub fn spring_pe(k: f32, displacement: f32) -> f32 {
    0.5 * k * displacement * displacement
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinetic_energy_zero_velocity() {
        assert_eq!(kinetic_energy(1.0, [0.0; 3]), 0.0);
    }

    #[test]
    fn kinetic_energy_unit_mass_unit_speed() {
        let ke = kinetic_energy(1.0, [1.0, 0.0, 0.0]);
        assert!((ke - 0.5).abs() < 1e-6);
    }

    #[test]
    fn kinetic_energy_scales_with_mass() {
        let ke1 = kinetic_energy(1.0, [2.0, 0.0, 0.0]);
        let ke2 = kinetic_energy(2.0, [2.0, 0.0, 0.0]);
        assert!((ke2 - 2.0 * ke1).abs() < 1e-6);
    }

    #[test]
    fn potential_energy_basic() {
        let pe = potential_energy(1.0, 10.0, 9.81);
        assert!((pe - 98.1).abs() < 0.01);
    }

    #[test]
    fn potential_energy_zero_height() {
        assert_eq!(potential_energy(5.0, 0.0, 9.81), 0.0);
    }

    #[test]
    fn rotational_ke_zero_omega() {
        assert_eq!(rotational_ke([1.0, 2.0, 3.0], [0.0; 3]), 0.0);
    }

    #[test]
    fn rotational_ke_unit_inertia_unit_omega() {
        let ke = rotational_ke([1.0; 3], [1.0, 0.0, 0.0]);
        assert!((ke - 0.5).abs() < 1e-6);
    }

    #[test]
    fn total_mechanical_energy_sums_correctly() {
        let ke = kinetic_energy(2.0, [1.0, 0.0, 0.0]);
        let pe = potential_energy(2.0, 5.0, 9.81);
        let total = total_mechanical_energy(2.0, [1.0, 0.0, 0.0], 5.0, 9.81);
        assert!((total - (ke + pe)).abs() < 1e-5);
    }

    #[test]
    fn spring_pe_zero_displacement() {
        assert_eq!(spring_pe(100.0, 0.0), 0.0);
    }

    #[test]
    fn spring_pe_quadratic() {
        let pe1 = spring_pe(1.0, 1.0);
        let pe2 = spring_pe(1.0, 2.0);
        assert!((pe2 / pe1 - 4.0).abs() < 1e-6);
    }
}
