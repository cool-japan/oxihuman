// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Particle system state export.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParticleExportConfig {
    pub max_particles: usize,
    pub include_velocities: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParticleSnapshot {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub age: f32,
    pub size: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParticlesExport {
    pub config: ParticleExportConfig,
    pub particles: Vec<ParticleSnapshot>,
}

#[allow(dead_code)]
pub fn default_particle_export_config() -> ParticleExportConfig {
    ParticleExportConfig {
        max_particles: 10_000,
        include_velocities: true,
    }
}

#[allow(dead_code)]
pub fn new_particles_export(config: ParticleExportConfig) -> ParticlesExport {
    ParticlesExport { config, particles: Vec::new() }
}

#[allow(dead_code)]
pub fn pe_add_particle(export: &mut ParticlesExport, particle: ParticleSnapshot) {
    if export.particles.len() < export.config.max_particles {
        export.particles.push(particle);
    }
}

#[allow(dead_code)]
pub fn pe_particle_count(export: &ParticlesExport) -> usize {
    export.particles.len()
}

#[allow(dead_code)]
pub fn pe_get_particle(export: &ParticlesExport, index: usize) -> Option<&ParticleSnapshot> {
    export.particles.get(index)
}

#[allow(dead_code)]
pub fn pe_clear(export: &mut ParticlesExport) {
    export.particles.clear();
}

#[allow(dead_code)]
pub fn pe_avg_age(export: &ParticlesExport) -> f32 {
    if export.particles.is_empty() {
        return 0.0;
    }
    export.particles.iter().map(|p| p.age).sum::<f32>() / export.particles.len() as f32
}

#[allow(dead_code)]
pub fn pe_validate(export: &ParticlesExport) -> bool {
    export.config.max_particles > 0
}

#[allow(dead_code)]
pub fn pe_to_json(export: &ParticlesExport) -> String {
    format!(
        "{{\"particle_count\":{},\"max_particles\":{},\"avg_age\":{}}}",
        export.particles.len(),
        export.config.max_particles,
        pe_avg_age(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_particle(age: f32) -> ParticleSnapshot {
        ParticleSnapshot {
            position: [0.0, age, 0.0],
            velocity: [0.0, 1.0, 0.0],
            age,
            size: 1.0,
        }
    }

    #[test]
    fn test_default_config() {
        let cfg = default_particle_export_config();
        assert_eq!(cfg.max_particles, 10_000);
    }

    #[test]
    fn test_add_particle() {
        let mut exp = new_particles_export(default_particle_export_config());
        pe_add_particle(&mut exp, make_particle(1.0));
        assert_eq!(pe_particle_count(&exp), 1);
    }

    #[test]
    fn test_get_particle() {
        let mut exp = new_particles_export(default_particle_export_config());
        pe_add_particle(&mut exp, make_particle(2.0));
        assert!((pe_get_particle(&exp, 0).expect("should succeed").age - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_clear() {
        let mut exp = new_particles_export(default_particle_export_config());
        pe_add_particle(&mut exp, make_particle(1.0));
        pe_clear(&mut exp);
        assert_eq!(pe_particle_count(&exp), 0);
    }

    #[test]
    fn test_avg_age_empty() {
        let exp = new_particles_export(default_particle_export_config());
        assert!((pe_avg_age(&exp)).abs() < 1e-6);
    }

    #[test]
    fn test_avg_age() {
        let mut exp = new_particles_export(default_particle_export_config());
        pe_add_particle(&mut exp, make_particle(2.0));
        pe_add_particle(&mut exp, make_particle(4.0));
        assert!((pe_avg_age(&exp) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_validate() {
        let exp = new_particles_export(default_particle_export_config());
        assert!(pe_validate(&exp));
    }

    #[test]
    fn test_to_json() {
        let exp = new_particles_export(default_particle_export_config());
        let j = pe_to_json(&exp);
        assert!(j.contains("particle_count"));
    }
}
