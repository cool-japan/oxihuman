// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

pub struct VortexRing {
    pub radius: f32,
    pub circulation: f32,
    pub core_radius: f32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
}

pub fn new_vortex_ring(radius: f32, circulation: f32) -> VortexRing {
    VortexRing {
        radius,
        circulation,
        core_radius: radius * 0.1,
        position: [0.0, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0],
    }
}

pub fn vortex_self_velocity(v: &VortexRing) -> f32 {
    /* Lamb formula stub: V ~ (Γ / 4πR) * (ln(8R/a) - 0.25) */
    let a = v.core_radius.max(1e-10);
    let r = v.radius.max(1e-10);
    let log_term = (8.0 * r / a).ln() - 0.25;
    v.circulation / (4.0 * PI * r) * log_term
}

pub fn vortex_step(v: &mut VortexRing, dt: f32) {
    let vs = vortex_self_velocity(v);
    /* ring translates along z axis by self-induced velocity */
    v.velocity[2] = vs;
    v.position[2] += vs * dt;
}

pub fn vortex_energy(v: &VortexRing) -> f32 {
    /* E ~ (1/2) * rho * Gamma^2 * R * (ln(8R/a) - 2) */
    let a = v.core_radius.max(1e-10);
    let r = v.radius.max(1e-10);
    let log_term = (8.0 * r / a).ln() - 2.0;
    0.5 * v.circulation * v.circulation * r * log_term
}

pub fn vortex_impulse(v: &VortexRing) -> [f32; 3] {
    /* Impulse P = rho * Gamma * pi * R^2 * axis (z) */
    let p = v.circulation * PI * v.radius * v.radius;
    [0.0, 0.0, p]
}

pub fn vortex_is_stable(v: &VortexRing) -> bool {
    v.radius > 0.0 && v.core_radius > 0.0 && v.core_radius < v.radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vortex_ring() {
        /* create vortex with radius and circulation */
        let v = new_vortex_ring(0.5, 2.0);
        assert_eq!(v.radius, 0.5);
        assert_eq!(v.circulation, 2.0);
    }

    #[test]
    fn test_vortex_self_velocity_positive() {
        /* self velocity should be positive for positive circulation */
        let v = new_vortex_ring(0.5, 1.0);
        assert!(vortex_self_velocity(&v) > 0.0);
    }

    #[test]
    fn test_vortex_step_moves_along_z() {
        /* ring translates in z direction */
        let mut v = new_vortex_ring(0.5, 1.0);
        let z0 = v.position[2];
        vortex_step(&mut v, 0.1);
        assert!(v.position[2] > z0);
    }

    #[test]
    fn test_vortex_energy_positive() {
        /* energy is positive */
        let v = new_vortex_ring(0.5, 1.0);
        assert!(vortex_energy(&v) > 0.0);
    }

    #[test]
    fn test_vortex_impulse_along_z() {
        /* impulse is along z axis */
        let v = new_vortex_ring(0.5, 1.0);
        let imp = vortex_impulse(&v);
        assert_eq!(imp[0], 0.0);
        assert_eq!(imp[1], 0.0);
        assert!(imp[2] > 0.0);
    }

    #[test]
    fn test_vortex_is_stable() {
        /* stable when core_radius < radius */
        let v = new_vortex_ring(1.0, 1.0);
        assert!(vortex_is_stable(&v));
    }
}
