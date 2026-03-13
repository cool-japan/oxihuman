// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Snow accumulation and deformation simulation using MPM-inspired approach.

// ── Structs ───────────────────────────────────────────────────────────────────

/// Physical parameters governing snow behaviour.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SnowConfig {
    pub density: f32,
    pub hardness: f32,
    pub cohesion: f32,
    pub melting_temp: f32,
    pub accumulation_rate: f32,
}

/// A single snow particle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SnowParticle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub mass: f32,
    pub temperature: f32,
    pub compressed: bool,
}

/// The full snow simulation state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SnowSystem {
    pub particles: Vec<SnowParticle>,
    pub config: SnowConfig,
    pub ambient_temp: f32,
    pub time: f32,
}

/// Per-step diagnostic result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SnowResult {
    pub particle_count: usize,
    pub melted_count: usize,
    pub avg_compression: f32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return sensible default `SnowConfig` values.
#[allow(dead_code)]
pub fn default_snow_config() -> SnowConfig {
    SnowConfig {
        density: 300.0,
        hardness: 0.5,
        cohesion: 0.2,
        melting_temp: 0.0,
        accumulation_rate: 0.01,
    }
}

/// Construct a `SnowParticle` at `pos` with the given mass.
/// Temperature is initialised to −5 °C (below freezing).
#[allow(dead_code)]
pub fn new_snow_particle(pos: [f32; 3], mass: f32) -> SnowParticle {
    SnowParticle {
        position: pos,
        velocity: [0.0; 3],
        mass,
        temperature: -5.0,
        compressed: false,
    }
}

/// Construct a new, empty `SnowSystem` with the given configuration.
#[allow(dead_code)]
pub fn new_snow_system(cfg: SnowConfig) -> SnowSystem {
    SnowSystem {
        particles: Vec::new(),
        config: cfg,
        ambient_temp: -2.0,
        time: 0.0,
    }
}

/// Append a particle to the system.
#[allow(dead_code)]
pub fn add_snow_particle(sys: &mut SnowSystem, p: SnowParticle) {
    sys.particles.push(p);
}

/// Advance the simulation by one timestep `dt`.
///
/// - Gravity is applied to falling particles.
/// - Temperature diffuses toward `ambient_temp`.
/// - Particles above `melting_temp` are marked for removal and counted.
#[allow(dead_code)]
pub fn step_snow(sys: &mut SnowSystem, dt: f32) -> SnowResult {
    let dt = dt.max(1e-6);
    let gravity = -9.81_f32;
    let t_ambient = sys.ambient_temp;
    let t_melt = sys.config.melting_temp;
    let thermal_alpha = 0.5_f32; // thermal diffusivity coefficient

    let mut melted_count = 0usize;
    let mut compressed_count = 0usize;

    for p in &mut sys.particles {
        // Apply gravity along Y.
        p.velocity[1] += gravity * dt;

        // Integrate position.
        p.position[0] += p.velocity[0] * dt;
        p.position[1] += p.velocity[1] * dt;
        p.position[2] += p.velocity[2] * dt;

        // Ground plane: bounce / stop at y = 0.
        if p.position[1] < 0.0 {
            p.position[1] = 0.0;
            p.velocity[1] = 0.0;
            // Mark compressed once it settles.
            if p.velocity[0].abs() < 0.01 && p.velocity[2].abs() < 0.01 {
                p.compressed = true;
            }
        }

        // Thermal diffusion toward ambient.
        p.temperature += thermal_alpha * (t_ambient - p.temperature) * dt;

        if p.temperature >= t_melt {
            melted_count += 1;
        }
        if p.compressed {
            compressed_count += 1;
        }
    }

    // Remove melted particles.
    sys.particles.retain(|p| p.temperature < t_melt);

    let remaining = sys.particles.len();
    let avg_compression = if remaining > 0 {
        compressed_count as f32 / remaining as f32
    } else {
        0.0
    };

    sys.time += dt;

    SnowResult {
        particle_count: remaining,
        melted_count,
        avg_compression,
    }
}

/// Return `true` if the particle's temperature is at or above the melting point.
#[allow(dead_code)]
pub fn is_melting(p: &SnowParticle, cfg: &SnowConfig) -> bool {
    p.temperature >= cfg.melting_temp
}

/// Return the number of particles currently in the system.
#[allow(dead_code)]
pub fn snow_particle_count(sys: &SnowSystem) -> usize {
    sys.particles.len()
}

/// Serialize the system to a compact JSON string.
#[allow(dead_code)]
pub fn snow_system_to_json(sys: &SnowSystem) -> String {
    format!(
        "{{\"particle_count\":{},\"ambient_temp\":{},\"time\":{}}}",
        sys.particles.len(),
        sys.ambient_temp,
        sys.time
    )
}

/// Serialize a `SnowResult` to a JSON string.
#[allow(dead_code)]
pub fn snow_result_to_json(r: &SnowResult) -> String {
    format!(
        "{{\"particle_count\":{},\"melted_count\":{},\"avg_compression\":{}}}",
        r.particle_count, r.melted_count, r.avg_compression
    )
}

/// Override the ambient temperature of the system.
#[allow(dead_code)]
pub fn set_ambient_temperature(sys: &mut SnowSystem, temp: f32) {
    sys.ambient_temp = temp;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_sane() {
        let cfg = default_snow_config();
        assert!(cfg.density > 0.0);
        assert!(cfg.hardness > 0.0);
        assert!(cfg.melting_temp >= -273.15);
    }

    #[test]
    fn new_particle_below_freezing() {
        let p = new_snow_particle([0.0, 5.0, 0.0], 1.0);
        assert!(p.temperature < 0.0);
        assert!(!p.compressed);
    }

    #[test]
    fn add_particle_increments_count() {
        let mut sys = new_snow_system(default_snow_config());
        add_snow_particle(&mut sys, new_snow_particle([0.0, 1.0, 0.0], 1.0));
        assert_eq!(snow_particle_count(&sys), 1);
    }

    #[test]
    fn gravity_pulls_particle_down() {
        let mut sys = new_snow_system(default_snow_config());
        add_snow_particle(&mut sys, new_snow_particle([0.0, 10.0, 0.0], 1.0));
        let before_y = sys.particles[0].position[1];
        step_snow(&mut sys, 0.016);
        assert!(sys.particles[0].position[1] < before_y);
    }

    #[test]
    fn hot_particles_melt() {
        let mut sys = new_snow_system(default_snow_config());
        let mut hot = new_snow_particle([0.0, 0.0, 0.0], 1.0);
        hot.temperature = 50.0; // well above melting_temp = 0.0
        add_snow_particle(&mut sys, hot);
        let r = step_snow(&mut sys, 0.016);
        assert_eq!(r.melted_count, 1);
        assert_eq!(snow_particle_count(&sys), 0);
    }

    #[test]
    fn is_melting_detects_warm_particle() {
        let cfg = default_snow_config();
        let mut p = new_snow_particle([0.0, 0.0, 0.0], 1.0);
        p.temperature = 1.0;
        assert!(is_melting(&p, &cfg));
        p.temperature = -1.0;
        assert!(!is_melting(&p, &cfg));
    }

    #[test]
    fn set_ambient_temperature_works() {
        let mut sys = new_snow_system(default_snow_config());
        set_ambient_temperature(&mut sys, 20.0);
        assert!((sys.ambient_temp - 20.0).abs() < 1e-6);
    }

    #[test]
    fn json_contains_particle_count() {
        let mut sys = new_snow_system(default_snow_config());
        add_snow_particle(&mut sys, new_snow_particle([0.0, 0.0, 0.0], 1.0));
        let j = snow_system_to_json(&sys);
        assert!(j.contains("\"particle_count\":1"));
    }

    #[test]
    fn result_json_fields_present() {
        let r = SnowResult {
            particle_count: 5,
            melted_count: 2,
            avg_compression: 0.4,
        };
        let j = snow_result_to_json(&r);
        assert!(j.contains("\"melted_count\":2"));
        assert!(j.contains("\"particle_count\":5"));
    }
}
