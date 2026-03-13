// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! GPU-ready particle system rendering data preparation.

// ── ParticleRenderMode ────────────────────────────────────────────────────────

/// How individual particles are rendered.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ParticleRenderMode {
    /// Camera-facing quad (default).
    Billboard,
    /// Full 3-D mesh per particle.
    Mesh,
    /// Continuous trail behind each particle.
    Trail,
    /// Sprite sheet frame per particle.
    Sprite,
}

// ── ParticleRenderConfig ──────────────────────────────────────────────────────

/// Configuration for a particle render batch.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParticleRenderConfig {
    /// Rendering mode.
    pub mode: ParticleRenderMode,
    /// Maximum number of particles the batch can hold.
    pub max_particles: u32,
    /// If true, particles are sorted back-to-front before rasterisation.
    pub sort_by_depth: bool,
    /// If true, additive blending is used (good for fire / sparks).
    pub additive_blend: bool,
}

// ── PsRenderParticle ──────────────────────────────────────────────────────────

/// One particle ready for upload to the GPU.
///
/// Named `PsRenderParticle` to avoid collision with `RenderParticle` already
/// exported from `particle_renderer`.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PsRenderParticle {
    /// World-space position.
    pub position: [f32; 3],
    /// RGBA colour (linear, 0.0–1.0).
    pub color: [f32; 4],
    /// Billboard / sprite size in world units.
    pub size: f32,
    /// Roll rotation in radians.
    pub rotation: f32,
    /// Particle age in seconds.
    pub age: f32,
}

// ── ParticleRenderBatch ───────────────────────────────────────────────────────

/// A batch of particles paired with their rendering configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParticleRenderBatch {
    /// Active particles in this batch.
    pub particles: Vec<PsRenderParticle>,
    /// Rendering configuration.
    pub config: ParticleRenderConfig,
    /// Number of particles that will be submitted for drawing.
    pub draw_count: usize,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a sensible default [`ParticleRenderConfig`].
#[allow(dead_code)]
pub fn default_particle_render_config() -> ParticleRenderConfig {
    ParticleRenderConfig {
        mode: ParticleRenderMode::Billboard,
        max_particles: 4096,
        sort_by_depth: false,
        additive_blend: false,
    }
}

/// Construct a new [`PsRenderParticle`] with default age and rotation.
#[allow(dead_code)]
pub fn new_render_particle(pos: [f32; 3], color: [f32; 4], size: f32) -> PsRenderParticle {
    PsRenderParticle {
        position: pos,
        color,
        size,
        rotation: 0.0,
        age: 0.0,
    }
}

/// Create an empty [`ParticleRenderBatch`] with the given config.
#[allow(dead_code)]
pub fn new_particle_batch(cfg: ParticleRenderConfig) -> ParticleRenderBatch {
    ParticleRenderBatch {
        particles: Vec::new(),
        config: cfg,
        draw_count: 0,
    }
}

/// Add a particle to `batch`, respecting `max_particles`.
#[allow(dead_code)]
pub fn add_render_particle(batch: &mut ParticleRenderBatch, p: PsRenderParticle) {
    if batch.particles.len() < batch.config.max_particles as usize {
        batch.particles.push(p);
        batch.draw_count = batch.particles.len();
    }
}

/// Sort particles in `batch` by descending distance from `camera_pos` (back-to-front).
#[allow(dead_code)]
pub fn sort_particles_by_depth(batch: &mut ParticleRenderBatch, camera_pos: [f32; 3]) {
    batch.particles.sort_by(|a, b| {
        let dist_a = dist_sq(a.position, camera_pos);
        let dist_b = dist_sq(b.position, camera_pos);
        dist_b
            .partial_cmp(&dist_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Remove all particles older than `max_age` seconds.
#[allow(dead_code)]
pub fn cull_dead_particles(batch: &mut ParticleRenderBatch, max_age: f32) {
    batch.particles.retain(|p| p.age < max_age);
    batch.draw_count = batch.particles.len();
}

/// Return the number of particles that will be drawn.
#[allow(dead_code)]
pub fn particle_draw_count(batch: &ParticleRenderBatch) -> usize {
    batch.draw_count
}

/// Return a human-readable name for the render mode.
#[allow(dead_code)]
pub fn render_mode_name(cfg: &ParticleRenderConfig) -> &'static str {
    match cfg.mode {
        ParticleRenderMode::Billboard => "Billboard",
        ParticleRenderMode::Mesh => "Mesh",
        ParticleRenderMode::Trail => "Trail",
        ParticleRenderMode::Sprite => "Sprite",
    }
}

/// Serialise a [`ParticleRenderBatch`] to a JSON string (summary form).
#[allow(dead_code)]
pub fn batch_to_json(batch: &ParticleRenderBatch) -> String {
    format!(
        r#"{{"mode":"{}","max_particles":{},"draw_count":{},"particle_count":{}}}"#,
        render_mode_name(&batch.config),
        batch.config.max_particles,
        batch.draw_count,
        batch.particles.len()
    )
}

/// Advance the age of all particles in `batch` by `dt` seconds.
#[allow(dead_code)]
pub fn update_particle_ages(batch: &mut ParticleRenderBatch, dt: f32) {
    for p in &mut batch.particles {
        p.age += dt;
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_billboard_mode() {
        let cfg = default_particle_render_config();
        assert_eq!(cfg.mode, ParticleRenderMode::Billboard);
        assert_eq!(cfg.max_particles, 4096);
        assert!(!cfg.sort_by_depth);
        assert!(!cfg.additive_blend);
    }

    #[test]
    fn add_render_particle_respects_max() {
        let cfg = ParticleRenderConfig {
            mode: ParticleRenderMode::Billboard,
            max_particles: 2,
            sort_by_depth: false,
            additive_blend: false,
        };
        let mut batch = new_particle_batch(cfg);
        for _ in 0..5 {
            let p = new_render_particle([0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 1.0);
            add_render_particle(&mut batch, p);
        }
        assert_eq!(batch.particles.len(), 2);
        assert_eq!(particle_draw_count(&batch), 2);
    }

    #[test]
    fn cull_dead_particles_removes_old() {
        let cfg = default_particle_render_config();
        let mut batch = new_particle_batch(cfg);
        let mut p1 = new_render_particle([0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 1.0);
        p1.age = 5.0;
        let p2 = new_render_particle([1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0], 1.0); // age 0
        add_render_particle(&mut batch, p1);
        add_render_particle(&mut batch, p2);
        cull_dead_particles(&mut batch, 2.0);
        assert_eq!(batch.particles.len(), 1);
        assert_eq!(batch.draw_count, 1);
    }

    #[test]
    fn sort_particles_by_depth_orders_back_to_front() {
        let cfg = default_particle_render_config();
        let mut batch = new_particle_batch(cfg);
        let near = new_render_particle([0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0], 1.0);
        let far = new_render_particle([0.0, 0.0, 10.0], [1.0, 1.0, 1.0, 1.0], 1.0);
        add_render_particle(&mut batch, near);
        add_render_particle(&mut batch, far);
        sort_particles_by_depth(&mut batch, [0.0, 0.0, 0.0]);
        // Far particle should now be first
        assert!((batch.particles[0].position[2] - 10.0).abs() < 1e-5);
    }

    #[test]
    fn update_particle_ages_increments() {
        let cfg = default_particle_render_config();
        let mut batch = new_particle_batch(cfg);
        let p = new_render_particle([0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 1.0);
        add_render_particle(&mut batch, p);
        update_particle_ages(&mut batch, 0.016);
        assert!((batch.particles[0].age - 0.016).abs() < 1e-5);
    }

    #[test]
    fn render_mode_name_returns_correct_strings() {
        let mut cfg = default_particle_render_config();
        cfg.mode = ParticleRenderMode::Billboard;
        assert_eq!(render_mode_name(&cfg), "Billboard");
        cfg.mode = ParticleRenderMode::Mesh;
        assert_eq!(render_mode_name(&cfg), "Mesh");
        cfg.mode = ParticleRenderMode::Trail;
        assert_eq!(render_mode_name(&cfg), "Trail");
        cfg.mode = ParticleRenderMode::Sprite;
        assert_eq!(render_mode_name(&cfg), "Sprite");
    }

    #[test]
    fn batch_to_json_contains_draw_count() {
        let cfg = default_particle_render_config();
        let mut batch = new_particle_batch(cfg);
        let p = new_render_particle([0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 1.0);
        add_render_particle(&mut batch, p);
        let json = batch_to_json(&batch);
        assert!(json.contains("\"draw_count\":1"));
    }

    #[test]
    fn particle_draw_count_reflects_additions() {
        let cfg = default_particle_render_config();
        let mut batch = new_particle_batch(cfg);
        assert_eq!(particle_draw_count(&batch), 0);
        let p = new_render_particle([1.0, 2.0, 3.0], [0.5, 0.5, 0.5, 1.0], 0.5);
        add_render_particle(&mut batch, p);
        assert_eq!(particle_draw_count(&batch), 1);
    }
}
