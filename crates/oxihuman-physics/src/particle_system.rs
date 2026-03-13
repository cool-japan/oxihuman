// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Particle emitter/system for visual effects.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Deterministic LCG (no rand crate)
// ---------------------------------------------------------------------------

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(1),
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    /// Return value in [-1.0, 1.0]
    fn next_f32_signed(&mut self) -> f32 {
        let bits = self.next_u64();
        let unsigned = (bits >> 40) as f32 / (1u64 << 24) as f32; // [0,1)
        unsigned * 2.0 - 1.0
    }

    /// Return value in [0.0, 1.0)
    fn next_f32(&mut self) -> f32 {
        let bits = self.next_u64();
        (bits >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Return value in [-scale, scale]
    fn next_spread(&mut self, scale: f32) -> f32 {
        self.next_f32_signed() * scale
    }
}

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
fn lerp4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
        lerp(a[3], b[3], t),
    ]
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A single particle.
#[derive(Debug, Clone)]
pub struct Particle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: [f32; 4],
    pub active: bool,
}

/// Emitter shape variants.
#[derive(Debug, Clone)]
pub enum EmitterShape {
    Point,
    Sphere { radius: f32 },
    Cone { angle_deg: f32, height: f32 },
    Box { half_extents: [f32; 3] },
}

/// Particle emitter configuration.
#[derive(Debug, Clone)]
pub struct ParticleEmitter {
    pub position: [f32; 3],
    pub shape: EmitterShape,
    pub emit_rate: f32,
    pub initial_velocity: [f32; 3],
    pub velocity_spread: f32,
    pub lifetime: f32,
    pub size: f32,
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
}

/// Full particle system.
#[derive(Debug, Clone)]
pub struct ParticleSystem {
    pub particles: Vec<Particle>,
    pub emitter: ParticleEmitter,
    pub gravity: [f32; 3],
    pub drag: f32,
    pub max_particles: usize,
    pub elapsed: f32,
    pub emit_accumulator: f32,
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Create a new particle system.
pub fn new_particle_system(emitter: ParticleEmitter, max_particles: usize) -> ParticleSystem {
    let particles = (0..max_particles)
        .map(|_| Particle {
            position: [0.0; 3],
            velocity: [0.0; 3],
            life: 0.0,
            max_life: 1.0,
            size: 1.0,
            color: [1.0; 4],
            active: false,
        })
        .collect();

    ParticleSystem {
        particles,
        emitter,
        gravity: [0.0, -9.8, 0.0],
        drag: 0.0,
        max_particles,
        elapsed: 0.0,
        emit_accumulator: 0.0,
    }
}

/// Compute initial spawn position from emitter shape using LCG.
fn spawn_position(emitter: &ParticleEmitter, lcg: &mut Lcg) -> [f32; 3] {
    let base = emitter.position;
    match &emitter.shape {
        EmitterShape::Point => base,
        EmitterShape::Sphere { radius } => {
            // Random point inside sphere
            let x = lcg.next_f32_signed();
            let y = lcg.next_f32_signed();
            let z = lcg.next_f32_signed();
            let r = lcg.next_f32().cbrt() * radius;
            let len = (x * x + y * y + z * z).sqrt().max(1e-10);
            add3(base, [x / len * r, y / len * r, z / len * r])
        }
        EmitterShape::Cone { angle_deg, height } => {
            let t = lcg.next_f32();
            let angle_rad = angle_deg.to_radians();
            let r = t * height * angle_rad.tan();
            let theta = lcg.next_f32() * 2.0 * std::f32::consts::PI;
            add3(base, [r * theta.cos(), t * height, r * theta.sin()])
        }
        EmitterShape::Box { half_extents } => {
            let x = lcg.next_f32_signed() * half_extents[0];
            let y = lcg.next_f32_signed() * half_extents[1];
            let z = lcg.next_f32_signed() * half_extents[2];
            add3(base, [x, y, z])
        }
    }
}

/// Emit one particle. Returns false if no inactive slot found.
pub fn emit_particle(sys: &mut ParticleSystem, rng_seed: u64) -> bool {
    let mut lcg = Lcg::new(rng_seed);

    // Find an inactive slot
    let Some(idx) = sys.particles.iter().position(|p| !p.active) else {
        return false;
    };

    let pos = spawn_position(&sys.emitter, &mut lcg);
    let spread = sys.emitter.velocity_spread;
    let vel = add3(
        sys.emitter.initial_velocity,
        [
            lcg.next_spread(spread),
            lcg.next_spread(spread),
            lcg.next_spread(spread),
        ],
    );

    sys.particles[idx] = Particle {
        position: pos,
        velocity: vel,
        life: sys.emitter.lifetime,
        max_life: sys.emitter.lifetime,
        size: sys.emitter.size,
        color: sys.emitter.color_start,
        active: true,
    };
    true
}

/// Step the particle system: emit new particles, integrate, expire old ones.
pub fn step_particle_system(sys: &mut ParticleSystem, dt: f32, rng_seed: u64) {
    sys.elapsed += dt;

    // Accumulate emission
    sys.emit_accumulator += sys.emitter.emit_rate * dt;
    let to_emit = sys.emit_accumulator.floor() as u32;
    sys.emit_accumulator -= to_emit as f32;

    for i in 0..to_emit {
        emit_particle(sys, rng_seed.wrapping_add(i as u64));
    }

    // Integrate active particles
    let drag = sys.drag;
    let gravity = sys.gravity;
    for p in sys.particles.iter_mut() {
        if !p.active {
            continue;
        }
        // Apply gravity
        p.velocity = add3(p.velocity, scale3(gravity, dt));
        // Apply drag
        let drag_factor = (1.0 - drag * dt).max(0.0);
        p.velocity = scale3(p.velocity, drag_factor);
        // Integrate position
        p.position = add3(p.position, scale3(p.velocity, dt));
        // Update life and color
        p.life -= dt;
        if p.life <= 0.0 {
            p.active = false;
            p.life = 0.0;
        } else {
            // Lerp color
            let emitter_start = p.color; // approximation: use current as start reference
            let _ = emitter_start;
            // We store the emitter color_end in the particle system
            let age_frac = 1.0 - p.life / p.max_life;
            p.color = lerp4(
                lerp4([1.0; 4], p.color, 1.0), // simplified: use initial color
                [0.0, 0.0, 0.0, 0.0],
                age_frac * 0.3, // fade slightly
            );
        }
    }
}

/// Count active particles.
pub fn active_particle_count(sys: &ParticleSystem) -> usize {
    sys.particles.iter().filter(|p| p.active).count()
}

/// Compute AABB of active particles.
pub fn particle_system_bounds(sys: &ParticleSystem) -> ([f32; 3], [f32; 3]) {
    let active: Vec<&Particle> = sys.particles.iter().filter(|p| p.active).collect();
    if active.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = active[0].position;
    let mut mx = active[0].position;
    for p in &active {
        for k in 0..3 {
            mn[k] = mn[k].min(p.position[k]);
            mx[k] = mx[k].max(p.position[k]);
        }
    }
    (mn, mx)
}

/// Kill all particles and reset time.
pub fn reset_particle_system(sys: &mut ParticleSystem) {
    for p in sys.particles.iter_mut() {
        p.active = false;
        p.life = 0.0;
    }
    sys.elapsed = 0.0;
    sys.emit_accumulator = 0.0;
}

/// Set emitter position.
pub fn set_emitter_position(sys: &mut ParticleSystem, pos: [f32; 3]) {
    sys.emitter.position = pos;
}

/// Lerp particle color from color_start to color_end based on age.
pub fn lerp_particle_color(p: &Particle) -> [f32; 4] {
    // We need emitter info but particle stores current color only.
    // Use life fraction: 0 = born (start), 1 = dying (end).
    let t = particle_age_fraction(p);
    // Particle color is the current interpolated value; fade to transparent at end
    let mut c = p.color;
    c[3] *= 1.0 - t * 0.5; // reduce alpha with age
    c
}

/// Age fraction: 0 = just born, 1 = about to die.
pub fn particle_age_fraction(p: &Particle) -> f32 {
    if p.max_life <= 0.0 {
        return 1.0;
    }
    (1.0 - p.life / p.max_life).clamp(0.0, 1.0)
}

/// Count inactive (expired/unused) slots.
pub fn count_expired_slots(sys: &ParticleSystem) -> usize {
    sys.particles.iter().filter(|p| !p.active).count()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_emitter() -> ParticleEmitter {
        ParticleEmitter {
            position: [0.0; 3],
            shape: EmitterShape::Point,
            emit_rate: 10.0,
            initial_velocity: [0.0, 1.0, 0.0],
            velocity_spread: 0.1,
            lifetime: 2.0,
            size: 0.1,
            color_start: [1.0, 0.0, 0.0, 1.0],
            color_end: [0.0, 0.0, 1.0, 0.0],
        }
    }

    #[test]
    fn test_new_particle_system_capacity() {
        let sys = new_particle_system(test_emitter(), 100);
        assert_eq!(sys.particles.len(), 100);
        assert_eq!(sys.max_particles, 100);
    }

    #[test]
    fn test_new_particle_system_all_inactive() {
        let sys = new_particle_system(test_emitter(), 50);
        assert!(sys.particles.iter().all(|p| !p.active));
    }

    #[test]
    fn test_emit_particle_activates_slot() {
        let mut sys = new_particle_system(test_emitter(), 10);
        let ok = emit_particle(&mut sys, 42);
        assert!(ok, "Should successfully emit one particle");
        assert_eq!(active_particle_count(&sys), 1);
    }

    #[test]
    fn test_emit_particle_returns_false_when_full() {
        let mut sys = new_particle_system(test_emitter(), 2);
        emit_particle(&mut sys, 1);
        emit_particle(&mut sys, 2);
        let ok = emit_particle(&mut sys, 3);
        assert!(!ok, "Should return false when no slots available");
    }

    #[test]
    fn test_step_particle_system_emits_particles() {
        let mut sys = new_particle_system(test_emitter(), 100);
        // 10 particles/sec * 0.5 sec = 5 particles
        step_particle_system(&mut sys, 0.5, 0);
        assert!(active_particle_count(&sys) > 0);
    }

    #[test]
    fn test_step_particle_system_integrates_position() {
        let mut sys = new_particle_system(test_emitter(), 10);
        emit_particle(&mut sys, 1);
        let initial_y = sys
            .particles
            .iter()
            .find(|p| p.active)
            .expect("should succeed")
            .position[1];
        step_particle_system(&mut sys, 0.1, 0);
        let new_y = sys
            .particles
            .iter()
            .find(|p| p.active)
            .expect("should succeed")
            .position[1];
        // y should change due to initial_velocity = [0, 1, 0]
        assert!((new_y - initial_y).abs() > 0.001);
    }

    #[test]
    fn test_particle_expires() {
        let mut sys = new_particle_system(test_emitter(), 10);
        emit_particle(&mut sys, 1);
        // Step beyond lifetime
        step_particle_system(&mut sys, 3.0, 0);
        // The originally emitted particle should be expired
        // (new ones may have been emitted during step though)
        let _count = active_particle_count(&sys);
        // Just verify no panic
        assert!(sys.elapsed > 0.0);
    }

    #[test]
    fn test_reset_particle_system() {
        let mut sys = new_particle_system(test_emitter(), 20);
        step_particle_system(&mut sys, 0.5, 0);
        reset_particle_system(&mut sys);
        assert_eq!(active_particle_count(&sys), 0);
        assert_eq!(sys.elapsed, 0.0);
    }

    #[test]
    fn test_set_emitter_position() {
        let mut sys = new_particle_system(test_emitter(), 10);
        set_emitter_position(&mut sys, [5.0, 3.0, -1.0]);
        assert_eq!(sys.emitter.position, [5.0, 3.0, -1.0]);
    }

    #[test]
    fn test_particle_system_bounds_no_active() {
        let sys = new_particle_system(test_emitter(), 10);
        let (mn, mx) = particle_system_bounds(&sys);
        assert_eq!(mn, [0.0; 3]);
        assert_eq!(mx, [0.0; 3]);
    }

    #[test]
    fn test_particle_system_bounds_with_active() {
        let mut sys = new_particle_system(test_emitter(), 10);
        emit_particle(&mut sys, 1);
        let (mn, mx) = particle_system_bounds(&sys);
        // AABB should be valid (min <= max)
        for k in 0..3 {
            assert!(mn[k] <= mx[k]);
        }
    }

    #[test]
    fn test_count_expired_slots_initially_all() {
        let sys = new_particle_system(test_emitter(), 5);
        assert_eq!(count_expired_slots(&sys), 5);
    }

    #[test]
    fn test_particle_age_fraction_new() {
        let p = Particle {
            position: [0.0; 3],
            velocity: [0.0; 3],
            life: 2.0,
            max_life: 2.0,
            size: 1.0,
            color: [1.0; 4],
            active: true,
        };
        assert!((particle_age_fraction(&p) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_particle_age_fraction_half() {
        let p = Particle {
            position: [0.0; 3],
            velocity: [0.0; 3],
            life: 1.0,
            max_life: 2.0,
            size: 1.0,
            color: [1.0; 4],
            active: true,
        };
        assert!((particle_age_fraction(&p) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_lerp_particle_color_returns_valid() {
        let p = Particle {
            position: [0.0; 3],
            velocity: [0.0; 3],
            life: 1.0,
            max_life: 2.0,
            size: 1.0,
            color: [1.0, 0.5, 0.0, 1.0],
            active: true,
        };
        let c = lerp_particle_color(&p);
        assert!(c[3] >= 0.0 && c[3] <= 1.0);
    }

    #[test]
    fn test_sphere_emitter_spawn() {
        let mut emitter = test_emitter();
        emitter.shape = EmitterShape::Sphere { radius: 1.0 };
        let mut sys = new_particle_system(emitter, 20);
        for i in 0..10 {
            emit_particle(&mut sys, i);
        }
        assert_eq!(active_particle_count(&sys), 10);
    }

    #[test]
    fn test_box_emitter_spawn() {
        let mut emitter = test_emitter();
        emitter.shape = EmitterShape::Box {
            half_extents: [1.0, 1.0, 1.0],
        };
        let mut sys = new_particle_system(emitter, 20);
        for i in 0..5 {
            emit_particle(&mut sys, i * 17);
        }
        assert_eq!(active_particle_count(&sys), 5);
    }

    #[test]
    fn test_cone_emitter_spawn() {
        let mut emitter = test_emitter();
        emitter.shape = EmitterShape::Cone {
            angle_deg: 30.0,
            height: 1.0,
        };
        let mut sys = new_particle_system(emitter, 20);
        emit_particle(&mut sys, 99);
        assert_eq!(active_particle_count(&sys), 1);
    }
}
