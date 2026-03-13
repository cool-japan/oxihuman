// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Physics simulation state snapshot export.

#[allow(dead_code)]
pub struct PhysicsStateExport {
    pub positions: Vec<[f32; 3]>,
    pub velocities: Vec<[f32; 3]>,
    pub masses: Vec<f32>,
    pub timestamp: f32,
}

#[allow(dead_code)]
pub fn new_physics_state_export(timestamp: f32) -> PhysicsStateExport {
    PhysicsStateExport {
        positions: Vec::new(),
        velocities: Vec::new(),
        masses: Vec::new(),
        timestamp,
    }
}

#[allow(dead_code)]
pub fn pse_add_particle(e: &mut PhysicsStateExport, pos: [f32; 3], vel: [f32; 3], mass: f32) {
    e.positions.push(pos);
    e.velocities.push(vel);
    e.masses.push(mass);
}

#[allow(dead_code)]
pub fn pse_particle_count(e: &PhysicsStateExport) -> usize {
    e.positions.len()
}

#[allow(dead_code)]
pub fn pse_total_kinetic_energy(e: &PhysicsStateExport) -> f32 {
    e.velocities.iter().zip(e.masses.iter()).map(|(v, &m)| {
        let v2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
        0.5 * m * v2
    }).sum()
}

#[allow(dead_code)]
pub fn pse_avg_speed(e: &PhysicsStateExport) -> f32 {
    if e.velocities.is_empty() { return 0.0; }
    let sum: f32 = e.velocities.iter().map(|v| {
        (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
    }).sum();
    sum / e.velocities.len() as f32
}

#[allow(dead_code)]
pub fn pse_to_json(e: &PhysicsStateExport) -> String {
    format!(
        r#"{{"timestamp":{},"particle_count":{},"total_kinetic_energy":{}}}"#,
        e.timestamp,
        pse_particle_count(e),
        pse_total_kinetic_energy(e)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let e = new_physics_state_export(1.5);
        assert!((e.timestamp - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_add_particle() {
        let mut e = new_physics_state_export(0.0);
        pse_add_particle(&mut e, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 2.0);
        assert_eq!(pse_particle_count(&e), 1);
    }

    #[test]
    fn test_particle_count() {
        let mut e = new_physics_state_export(0.0);
        for _ in 0..4 {
            pse_add_particle(&mut e, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);
        }
        assert_eq!(pse_particle_count(&e), 4);
    }

    #[test]
    fn test_kinetic_energy_zero_velocity() {
        let mut e = new_physics_state_export(0.0);
        pse_add_particle(&mut e, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 5.0);
        assert_eq!(pse_total_kinetic_energy(&e), 0.0);
    }

    #[test]
    fn test_kinetic_energy_known() {
        let mut e = new_physics_state_export(0.0);
        pse_add_particle(&mut e, [0.0, 0.0, 0.0], [2.0, 0.0, 0.0], 1.0);
        assert!((pse_total_kinetic_energy(&e) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let e = new_physics_state_export(0.0);
        let json = pse_to_json(&e);
        assert!(json.contains("timestamp"));
    }

    #[test]
    fn test_avg_speed_zero() {
        let e = new_physics_state_export(0.0);
        assert_eq!(pse_avg_speed(&e), 0.0);
    }

    #[test]
    fn test_avg_speed_known() {
        let mut e = new_physics_state_export(0.0);
        pse_add_particle(&mut e, [0.0, 0.0, 0.0], [3.0, 4.0, 0.0], 1.0);
        assert!((pse_avg_speed(&e) - 5.0).abs() < 1e-5);
    }
}
