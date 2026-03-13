// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Simple particle emitter/system.

#![allow(dead_code)]

/// Configuration for the particle system.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SimpleParticleConfig {
    pub emit_rate: f32,
    pub lifetime: f32,
    pub speed: f32,
}

/// A single particle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SimpleParticle {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub age: f32,
    pub alive: bool,
}

/// A simple particle system.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SimpleParticleSystem {
    config: SimpleParticleConfig,
    particles: Vec<SimpleParticle>,
    gravity: [f32; 3],
    total_emitted: u64,
}

/// Return default particle config.
#[allow(dead_code)]
pub fn default_simple_particle_config() -> SimpleParticleConfig {
    SimpleParticleConfig {
        emit_rate: 10.0,
        lifetime: 2.0,
        speed: 1.0,
    }
}

/// Create a new particle system.
#[allow(dead_code)]
pub fn new_simple_particle_system(config: SimpleParticleConfig) -> SimpleParticleSystem {
    SimpleParticleSystem {
        config,
        particles: Vec::new(),
        gravity: [0.0, -9.81, 0.0],
        total_emitted: 0,
    }
}

/// Emit a particle at position with velocity.
#[allow(dead_code)]
pub fn sps_emit(sys: &mut SimpleParticleSystem, pos: [f32; 3], vel: [f32; 3]) {
    // Reuse a dead slot if available
    if let Some(p) = sys.particles.iter_mut().find(|p| !p.alive) {
        p.pos = pos;
        p.vel = vel;
        p.age = 0.0;
        p.alive = true;
    } else {
        sys.particles.push(SimpleParticle { pos, vel, age: 0.0, alive: true });
    }
    sys.total_emitted += 1;
}

/// Step the simulation by dt seconds.
#[allow(dead_code)]
pub fn sps_step(sys: &mut SimpleParticleSystem, dt: f32) {
    for p in sys.particles.iter_mut() {
        if !p.alive {
            continue;
        }
        p.age += dt;
        if p.age >= sys.config.lifetime {
            p.alive = false;
            continue;
        }
        // Integrate velocity
        p.vel[0] += sys.gravity[0] * dt;
        p.vel[1] += sys.gravity[1] * dt;
        p.vel[2] += sys.gravity[2] * dt;
        // Integrate position
        p.pos[0] += p.vel[0] * dt;
        p.pos[1] += p.vel[1] * dt;
        p.pos[2] += p.vel[2] * dt;
    }
}

/// Return the number of alive particles.
#[allow(dead_code)]
pub fn sps_alive_count(sys: &SimpleParticleSystem) -> usize {
    sys.particles.iter().filter(|p| p.alive).count()
}

/// Clear all particles.
#[allow(dead_code)]
pub fn sps_clear(sys: &mut SimpleParticleSystem) {
    sys.particles.clear();
}

/// Get a reference to the particle at the given slot index.
#[allow(dead_code)]
pub fn sps_particle_at(sys: &SimpleParticleSystem, index: usize) -> Option<&SimpleParticle> {
    sys.particles.get(index)
}

/// Set gravity vector.
#[allow(dead_code)]
pub fn sps_set_gravity(sys: &mut SimpleParticleSystem, gravity: [f32; 3]) {
    sys.gravity = gravity;
}

/// Return total particles ever emitted.
#[allow(dead_code)]
pub fn sps_total_emitted(sys: &SimpleParticleSystem) -> u64 {
    sys.total_emitted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_simple_particle_config();
        assert!(cfg.emit_rate > 0.0);
        assert!(cfg.lifetime > 0.0);
    }

    #[test]
    fn test_new_system_empty() {
        let sys = new_simple_particle_system(default_simple_particle_config());
        assert_eq!(sps_alive_count(&sys), 0);
    }

    #[test]
    fn test_emit_creates_particle() {
        let mut sys = new_simple_particle_system(default_simple_particle_config());
        sps_emit(&mut sys, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(sps_alive_count(&sys), 1);
        assert_eq!(sps_total_emitted(&sys), 1);
    }

    #[test]
    fn test_step_moves_particle() {
        let mut sys = new_simple_particle_system(default_simple_particle_config());
        sps_emit(&mut sys, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        sps_step(&mut sys, 0.1);
        let p = sps_particle_at(&sys, 0).expect("should succeed");
        assert!(p.pos[0] > 0.0);
    }

    #[test]
    fn test_step_kills_expired() {
        let mut cfg = default_simple_particle_config();
        cfg.lifetime = 0.1;
        let mut sys = new_simple_particle_system(cfg);
        sps_emit(&mut sys, [0.0; 3], [0.0; 3]);
        sps_step(&mut sys, 0.5);
        assert_eq!(sps_alive_count(&sys), 0);
    }

    #[test]
    fn test_clear() {
        let mut sys = new_simple_particle_system(default_simple_particle_config());
        sps_emit(&mut sys, [0.0; 3], [0.0; 3]);
        sps_clear(&mut sys);
        assert_eq!(sps_alive_count(&sys), 0);
    }

    #[test]
    fn test_set_gravity() {
        let mut sys = new_simple_particle_system(default_simple_particle_config());
        sps_set_gravity(&mut sys, [0.0, 0.0, 0.0]);
        sps_emit(&mut sys, [0.0; 3], [1.0, 0.0, 0.0]);
        sps_step(&mut sys, 1.0);
        let p = sps_particle_at(&sys, 0).expect("should succeed");
        // With zero gravity, Y should not change
        assert!((p.pos[1]).abs() < 1e-6);
    }

    #[test]
    fn test_total_emitted_increments() {
        let mut sys = new_simple_particle_system(default_simple_particle_config());
        for _ in 0..5 {
            sps_emit(&mut sys, [0.0; 3], [0.0; 3]);
        }
        assert_eq!(sps_total_emitted(&sys), 5);
    }

    #[test]
    fn test_dead_slot_reuse() {
        let mut cfg = default_simple_particle_config();
        cfg.lifetime = 0.01;
        let mut sys = new_simple_particle_system(cfg);
        sps_emit(&mut sys, [0.0; 3], [0.0; 3]);
        sps_step(&mut sys, 1.0); // kills it
        sps_emit(&mut sys, [1.0; 3], [0.0; 3]); // reuse slot
        assert_eq!(sys.particles.len(), 1); // still only one slot
    }

    #[test]
    fn test_particle_at_none() {
        let sys = new_simple_particle_system(default_simple_particle_config());
        assert!(sps_particle_at(&sys, 99).is_none());
    }
}
