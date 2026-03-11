// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fluid velocity export: per-particle velocity field serialisation.

/// A fluid particle snapshot.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FluidParticle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub density: f32,
}

/// Fluid velocity export for a single frame.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FluidVelocityExport {
    pub particles: Vec<FluidParticle>,
    pub time: f32,
}

/// Create a new fluid velocity export.
#[allow(dead_code)]
pub fn new_fluid_velocity_export(time: f32) -> FluidVelocityExport {
    FluidVelocityExport {
        particles: Vec::new(),
        time,
    }
}

/// Add a particle.
#[allow(dead_code)]
pub fn add_fluid_particle(exp: &mut FluidVelocityExport, p: FluidParticle) {
    exp.particles.push(p);
}

/// Particle count.
#[allow(dead_code)]
pub fn fluid_particle_count(exp: &FluidVelocityExport) -> usize {
    exp.particles.len()
}

/// Average speed.
#[allow(dead_code)]
pub fn avg_speed(exp: &FluidVelocityExport) -> f32 {
    if exp.particles.is_empty() {
        return 0.0;
    }
    let sum: f32 = exp
        .particles
        .iter()
        .map(|p| {
            let v = p.velocity;
            (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
        })
        .sum();
    sum / exp.particles.len() as f32
}

/// Maximum speed.
#[allow(dead_code)]
pub fn max_speed(exp: &FluidVelocityExport) -> f32 {
    exp.particles
        .iter()
        .map(|p| {
            let v = p.velocity;
            (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
        })
        .fold(0.0_f32, f32::max)
}

/// Average density.
#[allow(dead_code)]
pub fn avg_density(exp: &FluidVelocityExport) -> f32 {
    if exp.particles.is_empty() {
        return 0.0;
    }
    exp.particles.iter().map(|p| p.density).sum::<f32>() / exp.particles.len() as f32
}

/// Export positions as flat array.
#[allow(dead_code)]
pub fn export_positions_flat(exp: &FluidVelocityExport) -> Vec<f32> {
    exp.particles.iter().flat_map(|p| p.position).collect()
}

/// Export velocities as flat array.
#[allow(dead_code)]
pub fn export_velocities_flat(exp: &FluidVelocityExport) -> Vec<f32> {
    exp.particles.iter().flat_map(|p| p.velocity).collect()
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn fluid_velocity_to_json(exp: &FluidVelocityExport) -> String {
    format!(
        "{{\"particle_count\":{},\"time\":{},\"avg_speed\":{}}}",
        fluid_particle_count(exp),
        exp.time,
        avg_speed(exp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn particle(vx: f32) -> FluidParticle {
        FluidParticle {
            position: [0.0; 3],
            velocity: [vx, 0.0, 0.0],
            density: 1000.0,
        }
    }

    #[test]
    fn new_export_empty() {
        let exp = new_fluid_velocity_export(0.0);
        assert_eq!(fluid_particle_count(&exp), 0);
    }

    #[test]
    fn add_particle_increments() {
        let mut exp = new_fluid_velocity_export(0.0);
        add_fluid_particle(&mut exp, particle(1.0));
        assert_eq!(fluid_particle_count(&exp), 1);
    }

    #[test]
    fn avg_speed_correct() {
        let mut exp = new_fluid_velocity_export(0.0);
        add_fluid_particle(&mut exp, particle(2.0));
        add_fluid_particle(&mut exp, particle(4.0));
        assert!((avg_speed(&exp) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn max_speed_correct() {
        let mut exp = new_fluid_velocity_export(0.0);
        add_fluid_particle(&mut exp, particle(1.0));
        add_fluid_particle(&mut exp, particle(5.0));
        assert!((max_speed(&exp) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn avg_density_correct() {
        let mut exp = new_fluid_velocity_export(0.0);
        add_fluid_particle(&mut exp, particle(0.0));
        assert!((avg_density(&exp) - 1000.0).abs() < 1e-5);
    }

    #[test]
    fn positions_flat_length() {
        let mut exp = new_fluid_velocity_export(0.0);
        add_fluid_particle(&mut exp, particle(1.0));
        add_fluid_particle(&mut exp, particle(2.0));
        assert_eq!(export_positions_flat(&exp).len(), 6);
    }

    #[test]
    fn velocities_flat_length() {
        let mut exp = new_fluid_velocity_export(0.0);
        add_fluid_particle(&mut exp, particle(1.0));
        assert_eq!(export_velocities_flat(&exp).len(), 3);
    }

    #[test]
    fn json_contains_particle_count() {
        let exp = new_fluid_velocity_export(1.5);
        let j = fluid_velocity_to_json(&exp);
        assert!(j.contains("particle_count"));
    }

    #[test]
    fn empty_avg_zero() {
        let exp = new_fluid_velocity_export(0.0);
        assert!((avg_speed(&exp)).abs() < 1e-6);
    }

    #[test]
    fn contains_range() {
        let v = 0.5_f32;
        assert!((0.0..=1.0).contains(&v));
    }
}
