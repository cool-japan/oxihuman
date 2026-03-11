// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Particle emitter system with lifetime and spawn control.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// LCG random number generator (no rand crate dependency)
// ---------------------------------------------------------------------------

fn lcg_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

fn lcg_f32(state: &mut u64) -> f32 {
    let bits = lcg_next(state);
    (bits >> 40) as f32 / (1u64 << 24) as f32
}

fn lcg_f32_signed(state: &mut u64) -> f32 {
    lcg_f32(state) * 2.0 - 1.0
}

// ---------------------------------------------------------------------------
// Structures
// ---------------------------------------------------------------------------

/// Shape of the particle emitter volume.
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PeEmitterShape {
    Point,
    Sphere,
    Box,
    Cone,
    Disk,
}

/// Configuration for a particle emitter.
#[allow(dead_code)]
pub struct PeEmitterConfig {
    pub shape: PeEmitterShape,
    /// Particles emitted per second.
    pub rate: f32,
    /// Particle lifetime in seconds.
    pub lifetime: f32,
    pub speed: f32,
    pub speed_variance: f32,
    pub size: f32,
    pub size_variance: f32,
    /// RGBA colour.
    pub color: [f32; 4],
    pub gravity_scale: f32,
    /// Number of particles in a burst emission.
    pub burst_count: u32,
}

/// A single live particle.
#[allow(dead_code)]
pub struct PeEmittedParticle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub size: f32,
    pub color: [f32; 4],
    pub age: f32,
    pub lifetime: f32,
    pub id: u32,
}

/// The particle emitter state.
#[allow(dead_code)]
pub struct PeParticleEmitter {
    pub config: PeEmitterConfig,
    pub particles: Vec<PeEmittedParticle>,
    pub position: [f32; 3],
    pub enabled: bool,
    pub accumulated_time: f32,
    pub next_id: u32,
    pub rng: u64,
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Build a default emitter configuration.
#[allow(dead_code)]
pub fn default_emitter_config() -> PeEmitterConfig {
    PeEmitterConfig {
        shape: PeEmitterShape::Point,
        rate: 20.0,
        lifetime: 2.0,
        speed: 3.0,
        speed_variance: 0.5,
        size: 0.1,
        size_variance: 0.02,
        color: [1.0, 1.0, 1.0, 1.0],
        gravity_scale: 1.0,
        burst_count: 10,
    }
}

/// Create a new particle emitter at the given position.
#[allow(dead_code)]
pub fn new_particle_emitter(pos: [f32; 3], cfg: PeEmitterConfig) -> PeParticleEmitter {
    PeParticleEmitter {
        config: cfg,
        particles: Vec::new(),
        position: pos,
        enabled: true,
        accumulated_time: 0.0,
        next_id: 0,
        rng: 0xDEAD_BEEF_1234_5678,
    }
}

/// Spawn a single particle from the emitter (does not add to internal list, returns it).
#[allow(dead_code)]
pub fn spawn_particle(emitter: &mut PeParticleEmitter) -> PeEmittedParticle {
    let rng = &mut emitter.rng;
    let pos = emitter_point_position_from_rng(&emitter.config.shape, emitter.position, rng);

    let speed =
        (emitter.config.speed + lcg_f32_signed(rng) * emitter.config.speed_variance).max(0.0);
    let vx = lcg_f32_signed(rng);
    let vy = lcg_f32(rng).abs() + 0.1;
    let vz = lcg_f32_signed(rng);
    let vlen = (vx * vx + vy * vy + vz * vz).sqrt().max(1e-12);
    let velocity = [vx / vlen * speed, vy / vlen * speed, vz / vlen * speed];

    let size =
        (emitter.config.size + lcg_f32_signed(rng) * emitter.config.size_variance).max(0.001);

    let id = emitter.next_id;
    emitter.next_id += 1;

    PeEmittedParticle {
        position: pos,
        velocity,
        size,
        color: emitter.config.color,
        age: 0.0,
        lifetime: emitter.config.lifetime,
        id,
    }
}

/// Update the emitter: advance particle ages, remove dead ones, spawn new ones.
#[allow(dead_code)]
pub fn update_emitter(emitter: &mut PeParticleEmitter, dt: f32, gravity: [f32; 3]) {
    // Age and integrate existing particles
    let gs = emitter.config.gravity_scale;
    for p in emitter.particles.iter_mut() {
        p.age += dt;
        p.velocity[0] += gravity[0] * gs * dt;
        p.velocity[1] += gravity[1] * gs * dt;
        p.velocity[2] += gravity[2] * gs * dt;
        p.position[0] += p.velocity[0] * dt;
        p.position[1] += p.velocity[1] * dt;
        p.position[2] += p.velocity[2] * dt;
    }
    // Remove expired particles
    emitter.particles.retain(|p| p.age < p.lifetime);

    // Spawn new particles
    if emitter.enabled && emitter.config.rate > 0.0 {
        emitter.accumulated_time += dt;
        let interval = 1.0 / emitter.config.rate;
        while emitter.accumulated_time >= interval {
            emitter.accumulated_time -= interval;
            let p = spawn_particle(emitter);
            emitter.particles.push(p);
        }
    }
}

/// Emit a burst of `count` particles immediately.
#[allow(dead_code)]
pub fn emit_burst(emitter: &mut PeParticleEmitter, count: u32) {
    for _ in 0..count {
        let p = spawn_particle(emitter);
        emitter.particles.push(p);
    }
}

/// Total number of particles in the emitter (live and dead).
#[allow(dead_code)]
pub fn emitter_particle_count(emitter: &PeParticleEmitter) -> usize {
    emitter.particles.len()
}

/// Number of live (age < lifetime) particles.
#[allow(dead_code)]
pub fn alive_emitter_count(emitter: &PeParticleEmitter) -> usize {
    emitter
        .particles
        .iter()
        .filter(|p| p.age < p.lifetime)
        .count()
}

/// Enable the emitter.
#[allow(dead_code)]
pub fn enable_emitter(emitter: &mut PeParticleEmitter) {
    emitter.enabled = true;
}

/// Disable the emitter (stops spawning but does not kill particles).
#[allow(dead_code)]
pub fn disable_emitter(emitter: &mut PeParticleEmitter) {
    emitter.enabled = false;
}

/// Remove all particles from the emitter.
#[allow(dead_code)]
pub fn clear_emitter_particles(emitter: &mut PeParticleEmitter) {
    emitter.particles.clear();
}

/// Sample a position from the emitter shape.
#[allow(dead_code)]
pub fn emitter_point_position(emitter: &PeParticleEmitter, rng: &mut u64) -> [f32; 3] {
    emitter_point_position_from_rng(&emitter.config.shape, emitter.position, rng)
}

/// Returns age / lifetime for a particle (clamped to [0, 1]).
#[allow(dead_code)]
pub fn particle_age_fraction_pe(p: &PeEmittedParticle) -> f32 {
    (p.age / p.lifetime.max(1e-12)).min(1.0)
}

/// Total particles ever spawned by this emitter (next_id tracks this).
#[allow(dead_code)]
pub fn emitter_total_spawned(emitter: &PeParticleEmitter) -> u32 {
    emitter.next_id
}

// ---------------------------------------------------------------------------
// Internal helper
// ---------------------------------------------------------------------------

fn emitter_point_position_from_rng(
    shape: &PeEmitterShape,
    origin: [f32; 3],
    rng: &mut u64,
) -> [f32; 3] {
    match shape {
        PeEmitterShape::Point => origin,
        PeEmitterShape::Sphere => {
            let x = lcg_f32_signed(rng);
            let y = lcg_f32_signed(rng);
            let z = lcg_f32_signed(rng);
            let len = (x * x + y * y + z * z).sqrt().max(1e-12);
            let r = lcg_f32(rng).cbrt(); // uniform in sphere volume
            [
                origin[0] + x / len * r,
                origin[1] + y / len * r,
                origin[2] + z / len * r,
            ]
        }
        PeEmitterShape::Box => [
            origin[0] + lcg_f32_signed(rng) * 0.5,
            origin[1] + lcg_f32_signed(rng) * 0.5,
            origin[2] + lcg_f32_signed(rng) * 0.5,
        ],
        PeEmitterShape::Cone => {
            let angle = lcg_f32(rng) * std::f32::consts::PI * 0.25; // half-angle 45°
            let phi = lcg_f32(rng) * 2.0 * std::f32::consts::PI;
            let h = lcg_f32(rng);
            let r = h * angle.tan();
            [
                origin[0] + r * phi.cos(),
                origin[1] + h,
                origin[2] + r * phi.sin(),
            ]
        }
        PeEmitterShape::Disk => {
            let angle = lcg_f32(rng) * 2.0 * std::f32::consts::PI;
            let r = lcg_f32(rng).sqrt() * 0.5;
            [
                origin[0] + r * angle.cos(),
                origin[1],
                origin[2] + r * angle.sin(),
            ]
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_emitter_config() {
        let cfg = default_emitter_config();
        assert!(cfg.rate > 0.0);
        assert!(cfg.lifetime > 0.0);
        assert_eq!(cfg.shape, PeEmitterShape::Point);
    }

    #[test]
    fn test_new_emitter() {
        let cfg = default_emitter_config();
        let emitter = new_particle_emitter([0.0, 0.0, 0.0], cfg);
        assert!(emitter.enabled);
        assert_eq!(emitter.particles.len(), 0);
        assert_eq!(emitter.next_id, 0);
    }

    #[test]
    fn test_update_spawns_particles() {
        let cfg = default_emitter_config(); // rate=20 => one particle every 0.05s
        let mut emitter = new_particle_emitter([0.0, 0.0, 0.0], cfg);
        update_emitter(&mut emitter, 0.5, [0.0, -9.81, 0.0]);
        assert!(
            !emitter.particles.is_empty(),
            "should have spawned particles"
        );
    }

    #[test]
    fn test_emit_burst() {
        let cfg = default_emitter_config();
        let mut emitter = new_particle_emitter([0.0, 0.0, 0.0], cfg);
        emit_burst(&mut emitter, 5);
        assert_eq!(emitter.particles.len(), 5);
    }

    #[test]
    fn test_emitter_particle_count() {
        let cfg = default_emitter_config();
        let mut emitter = new_particle_emitter([0.0, 0.0, 0.0], cfg);
        emit_burst(&mut emitter, 3);
        assert_eq!(emitter_particle_count(&emitter), 3);
    }

    #[test]
    fn test_alive_emitter_count() {
        let cfg = default_emitter_config();
        let mut emitter = new_particle_emitter([0.0, 0.0, 0.0], cfg);
        emit_burst(&mut emitter, 4);
        assert_eq!(alive_emitter_count(&emitter), 4);
    }

    #[test]
    fn test_enable_disable_emitter() {
        let cfg = default_emitter_config();
        let mut emitter = new_particle_emitter([0.0, 0.0, 0.0], cfg);
        disable_emitter(&mut emitter);
        assert!(!emitter.enabled);
        enable_emitter(&mut emitter);
        assert!(emitter.enabled);
    }

    #[test]
    fn test_disabled_emitter_no_spawn() {
        let cfg = default_emitter_config();
        let mut emitter = new_particle_emitter([0.0, 0.0, 0.0], cfg);
        disable_emitter(&mut emitter);
        update_emitter(&mut emitter, 1.0, [0.0, -9.81, 0.0]);
        assert!(
            emitter.particles.is_empty(),
            "disabled emitter should not spawn"
        );
    }

    #[test]
    fn test_clear_emitter_particles() {
        let cfg = default_emitter_config();
        let mut emitter = new_particle_emitter([0.0, 0.0, 0.0], cfg);
        emit_burst(&mut emitter, 10);
        clear_emitter_particles(&mut emitter);
        assert!(emitter.particles.is_empty());
    }

    #[test]
    fn test_spawn_particle_fields() {
        let cfg = default_emitter_config();
        let mut emitter = new_particle_emitter([1.0, 2.0, 3.0], cfg);
        let p = spawn_particle(&mut emitter);
        assert_eq!(p.id, 0);
        assert_eq!(p.age, 0.0);
        assert!(p.lifetime > 0.0);
        assert!(p.size > 0.0);
    }

    #[test]
    fn test_particle_age_fraction_zero() {
        let cfg = default_emitter_config();
        let mut emitter = new_particle_emitter([0.0, 0.0, 0.0], cfg);
        let p = spawn_particle(&mut emitter);
        let frac = particle_age_fraction_pe(&p);
        assert_eq!(frac, 0.0);
    }

    #[test]
    fn test_particle_age_fraction_clamped() {
        let p = PeEmittedParticle {
            position: [0.0; 3],
            velocity: [0.0; 3],
            size: 0.1,
            color: [1.0; 4],
            age: 10.0,
            lifetime: 1.0,
            id: 0,
        };
        let frac = particle_age_fraction_pe(&p);
        assert_eq!(frac, 1.0);
    }

    #[test]
    fn test_emitter_point_position_sphere() {
        let cfg = PeEmitterConfig {
            shape: PeEmitterShape::Sphere,
            ..default_emitter_config()
        };
        let emitter = new_particle_emitter([0.0, 0.0, 0.0], cfg);
        let mut rng = 0x1234_5678u64;
        let pos = emitter_point_position(&emitter, &mut rng);
        // Position should be within unit sphere (radius <= 1)
        let r = (pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2]).sqrt();
        assert!(r <= 1.1, "sphere spawn position radius {r} should be <= 1");
    }

    #[test]
    fn test_emitter_total_spawned() {
        let cfg = default_emitter_config();
        let mut emitter = new_particle_emitter([0.0, 0.0, 0.0], cfg);
        emit_burst(&mut emitter, 7);
        assert_eq!(emitter_total_spawned(&emitter), 7);
    }

    #[test]
    fn test_particles_expire_over_time() {
        let mut cfg = default_emitter_config();
        cfg.lifetime = 0.1; // very short lifetime
        let mut emitter = new_particle_emitter([0.0, 0.0, 0.0], cfg);
        emit_burst(&mut emitter, 5);
        // After 0.2s all should be expired
        update_emitter(&mut emitter, 0.2, [0.0, 0.0, 0.0]);
        // With rate=20 and dt=0.2, 4 new ones may have spawned. But the burst ones are dead.
        // Check the total alive count is bounded by newly spawned
        let alive = alive_emitter_count(&emitter);
        assert!(alive <= 5, "no more than newly-spawned should be alive");
    }
}
