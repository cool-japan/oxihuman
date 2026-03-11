// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Particle system v2: pool-based, with spawn rate and lifetime management.

/// A single particle.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct Particle2 {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub color: [f32; 4],
    pub lifetime: f32,
    pub age: f32,
    pub alive: bool,
    pub mass: f32,
}

impl Particle2 {
    #[allow(dead_code)]
    fn is_dead(&self) -> bool {
        self.age >= self.lifetime
    }
}

/// Particle system v2.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParticleSystemV2 {
    pub pool: Vec<Particle2>,
    pub gravity: [f32; 3],
    pub spawn_rate: f32,
    pub spawn_accum: f32,
    pub origin: [f32; 3],
    pub default_lifetime: f32,
    pub default_speed: f32,
}

/// Create a new `ParticleSystemV2` with `capacity` slots.
#[allow(dead_code)]
pub fn new_particle_system_v2(capacity: usize) -> ParticleSystemV2 {
    ParticleSystemV2 {
        pool: (0..capacity).map(|_| Particle2::default()).collect(),
        gravity: [0.0, -9.81, 0.0],
        spawn_rate: 10.0,
        spawn_accum: 0.0,
        origin: [0.0; 3],
        default_lifetime: 2.0,
        default_speed: 1.0,
    }
}

fn find_dead(ps: &ParticleSystemV2) -> Option<usize> {
    ps.pool.iter().position(|p| !p.alive)
}

fn spawn_one(ps: &mut ParticleSystemV2, idx: usize) {
    let p = &mut ps.pool[idx];
    p.pos = ps.origin;
    p.vel = [0.0, ps.default_speed, 0.0];
    p.color = [1.0, 1.0, 1.0, 1.0];
    p.lifetime = ps.default_lifetime;
    p.age = 0.0;
    p.alive = true;
    p.mass = 0.001;
}

/// Step the particle system.
#[allow(dead_code)]
pub fn ps2_step(ps: &mut ParticleSystemV2, dt: f32) {
    // Spawn
    ps.spawn_accum += ps.spawn_rate * dt;
    while ps.spawn_accum >= 1.0 {
        if let Some(idx) = find_dead(ps) {
            spawn_one(ps, idx);
        }
        ps.spawn_accum -= 1.0;
    }

    // Update
    for p in &mut ps.pool {
        if !p.alive {
            continue;
        }
        p.age += dt;
        if p.age >= p.lifetime {
            p.alive = false;
            continue;
        }
        for ax in 0..3 {
            p.vel[ax] += ps.gravity[ax] * dt;
            p.pos[ax] += p.vel[ax] * dt;
        }
    }
}

/// Number of alive particles.
#[allow(dead_code)]
pub fn ps2_alive_count(ps: &ParticleSystemV2) -> usize {
    ps.pool.iter().filter(|p| p.alive).count()
}

/// Pool capacity.
#[allow(dead_code)]
pub fn ps2_capacity(ps: &ParticleSystemV2) -> usize {
    ps.pool.len()
}

/// Kill all alive particles.
#[allow(dead_code)]
pub fn ps2_kill_all(ps: &mut ParticleSystemV2) {
    for p in &mut ps.pool {
        p.alive = false;
    }
}

/// Set spawn rate (particles per second).
#[allow(dead_code)]
pub fn ps2_set_spawn_rate(ps: &mut ParticleSystemV2, rate: f32) {
    ps.spawn_rate = rate.max(0.0);
}

/// Set origin.
#[allow(dead_code)]
pub fn ps2_set_origin(ps: &mut ParticleSystemV2, origin: [f32; 3]) {
    ps.origin = origin;
}

/// Average particle height (y).
#[allow(dead_code)]
pub fn ps2_avg_y(ps: &ParticleSystemV2) -> f32 {
    let alive: Vec<&Particle2> = ps.pool.iter().filter(|p| p.alive).collect();
    if alive.is_empty() {
        return 0.0;
    }
    let sum: f32 = alive.iter().map(|p| p.pos[1]).sum();
    sum / alive.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_system() {
        let ps = new_particle_system_v2(100);
        assert_eq!(ps2_capacity(&ps), 100);
        assert_eq!(ps2_alive_count(&ps), 0);
    }

    #[test]
    fn test_step_spawns_particles() {
        let mut ps = new_particle_system_v2(100);
        ps2_set_spawn_rate(&mut ps, 100.0);
        ps2_step(&mut ps, 0.1);
        assert!(ps2_alive_count(&ps) > 0);
    }

    #[test]
    fn test_particles_age_out() {
        let mut ps = new_particle_system_v2(10);
        ps.default_lifetime = 0.05;
        ps2_set_spawn_rate(&mut ps, 100.0);
        ps2_step(&mut ps, 0.1);
        ps2_step(&mut ps, 0.1);
        assert!(ps2_alive_count(&ps) <= 10);
    }

    #[test]
    fn test_kill_all() {
        let mut ps = new_particle_system_v2(50);
        ps2_set_spawn_rate(&mut ps, 50.0);
        ps2_step(&mut ps, 1.0);
        ps2_kill_all(&mut ps);
        assert_eq!(ps2_alive_count(&ps), 0);
    }

    #[test]
    fn test_capacity_respected() {
        let mut ps = new_particle_system_v2(5);
        ps.default_lifetime = 100.0;
        ps2_set_spawn_rate(&mut ps, 1000.0);
        ps2_step(&mut ps, 1.0);
        assert!(ps2_alive_count(&ps) <= 5);
    }

    #[test]
    fn test_set_origin() {
        let mut ps = new_particle_system_v2(10);
        ps2_set_origin(&mut ps, [1.0, 2.0, 3.0]);
        assert!((ps.origin[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_avg_y_increases_at_start() {
        let mut ps = new_particle_system_v2(10);
        ps.default_speed = 10.0;
        ps.default_lifetime = 1.0;
        ps2_set_spawn_rate(&mut ps, 10.0);
        ps2_step(&mut ps, 0.1);
        let y = ps2_avg_y(&ps);
        assert!(y >= 0.0);
    }

    #[test]
    fn test_no_alive_avg_y_zero() {
        let ps = new_particle_system_v2(10);
        assert!((ps2_avg_y(&ps)).abs() < 1e-6);
    }
}
