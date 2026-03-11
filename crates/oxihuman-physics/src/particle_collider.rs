// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Particle-object collision detection and response.
//!
//! Supports sphere, box, capsule, and infinite-plane collider shapes.
//! Collisions are resolved via position correction (XPBD-compatible).

// ── Type aliases ─────────────────────────────────────────────────────────────

/// 3-D position / vector `[x, y, z]`.
#[allow(dead_code)]
pub type Vec3 = [f32; 3];

/// Axis-Aligned Bounding Box `([min_x, min_y, min_z], [max_x, max_y, max_z])`.
#[allow(dead_code)]
pub type Aabb = (Vec3, Vec3);

/// Penetration depth and contact normal returned by collision tests.
#[allow(dead_code)]
pub type ContactInfo = (f32, Vec3);

// ── Collider shape ────────────────────────────────────────────────────────────

/// Geometric shape used by a collider.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ColliderShape {
    /// Sphere at `center` with `radius`.
    Sphere { center: Vec3, radius: f32 },
    /// Axis-aligned box from `min` to `max`.
    Box { min: Vec3, max: Vec3 },
    /// Capsule defined by two end-point centres and a radius.
    Capsule { a: Vec3, b: Vec3, radius: f32 },
    /// Infinite plane with normal `n` passing through `point`.
    Plane { normal: Vec3, point: Vec3 },
}

// ── Config / Main struct ──────────────────────────────────────────────────────

/// Configuration for the particle-collider system.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParticleColliderConfig {
    /// Restitution coefficient (0 = inelastic, 1 = elastic).
    pub restitution: f32,
    /// Friction coefficient.
    pub friction: f32,
    /// Small positional slop to avoid jitter.
    pub slop: f32,
}

impl Default for ParticleColliderConfig {
    fn default() -> Self {
        Self {
            restitution: 0.0,
            friction: 0.3,
            slop: 1e-3,
        }
    }
}

/// A single solid collider object.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParticleCollider {
    /// Unique identifier.
    pub id: u32,
    /// Geometric shape.
    pub shape: ColliderShape,
    /// Whether this collider is active.
    pub enabled: bool,
}

/// The collection of colliders the particle system interacts with.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParticleColliderSystem {
    pub colliders: Vec<ParticleCollider>,
    pub config: ParticleColliderConfig,
    next_id: u32,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
fn scale(v: Vec3, s: f32) -> Vec3 {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
fn len(v: Vec3) -> f32 {
    dot(v, v).sqrt()
}

#[allow(dead_code)]
fn normalize(v: Vec3) -> Vec3 {
    let l = len(v);
    if l < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        scale(v, 1.0 / l)
    }
}

/// Closest point on segment `[a, b]` to point `p`.
#[allow(dead_code)]
fn closest_point_on_segment(p: Vec3, a: Vec3, b: Vec3) -> Vec3 {
    let ab = sub(b, a);
    let ap = sub(p, a);
    let t = dot(ap, ab) / (dot(ab, ab) + 1e-10);
    let t = t.clamp(0.0, 1.0);
    add(a, scale(ab, t))
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a `ParticleColliderConfig` with sensible defaults.
#[allow(dead_code)]
pub fn default_collider_config() -> ParticleColliderConfig {
    ParticleColliderConfig::default()
}

/// Create a new empty `ParticleColliderSystem`.
#[allow(dead_code)]
pub fn new_particle_collider(cfg: ParticleColliderConfig) -> ParticleColliderSystem {
    ParticleColliderSystem {
        colliders: Vec::new(),
        config: cfg,
        next_id: 0,
    }
}

/// Convenience: create a sphere collider (not yet added to a system).
#[allow(dead_code)]
pub fn sphere_collider(center: Vec3, radius: f32) -> ParticleCollider {
    ParticleCollider {
        id: 0,
        shape: ColliderShape::Sphere { center, radius },
        enabled: true,
    }
}

/// Convenience: create an axis-aligned box collider.
#[allow(dead_code)]
pub fn box_collider(min: Vec3, max: Vec3) -> ParticleCollider {
    ParticleCollider {
        id: 0,
        shape: ColliderShape::Box { min, max },
        enabled: true,
    }
}

/// Convenience: create a capsule collider.
#[allow(dead_code)]
pub fn capsule_collider(a: Vec3, b: Vec3, radius: f32) -> ParticleCollider {
    ParticleCollider {
        id: 0,
        shape: ColliderShape::Capsule { a, b, radius },
        enabled: true,
    }
}

/// Convenience: create an infinite-plane collider.
#[allow(dead_code)]
pub fn plane_collider(normal: Vec3, point: Vec3) -> ParticleCollider {
    ParticleCollider {
        id: 0,
        shape: ColliderShape::Plane { normal, point },
        enabled: true,
    }
}

/// Test whether a particle at `particle_pos` with `particle_radius` is
/// penetrating `collider`.
///
/// Returns `Some((depth, normal))` where `depth > 0` means penetration and
/// `normal` points from the collider surface towards the particle.
/// Returns `None` when there is no contact.
#[allow(dead_code)]
pub fn test_particle_vs_collider(
    particle_pos: Vec3,
    particle_radius: f32,
    collider: &ParticleCollider,
) -> Option<ContactInfo> {
    if !collider.enabled {
        return None;
    }
    match &collider.shape {
        ColliderShape::Sphere { center, radius } => {
            let d = sub(particle_pos, *center);
            let dist = len(d);
            let combined = radius + particle_radius;
            if dist < combined {
                let depth = combined - dist;
                let normal = if dist > 1e-10 {
                    scale(d, 1.0 / dist)
                } else {
                    [0.0, 1.0, 0.0]
                };
                Some((depth, normal))
            } else {
                None
            }
        }

        ColliderShape::Box { min, max } => {
            // Expand box by particle_radius.
            let exp_min = [min[0] - particle_radius, min[1] - particle_radius, min[2] - particle_radius];
            let exp_max = [max[0] + particle_radius, max[1] + particle_radius, max[2] + particle_radius];

            // Check inside expanded box.
            if particle_pos[0] < exp_min[0]
                || particle_pos[0] > exp_max[0]
                || particle_pos[1] < exp_min[1]
                || particle_pos[1] > exp_max[1]
                || particle_pos[2] < exp_min[2]
                || particle_pos[2] > exp_max[2]
            {
                return None;
            }
            // Find axis with minimum penetration.
            let dx0 = particle_pos[0] - exp_min[0];
            let dx1 = exp_max[0] - particle_pos[0];
            let dy0 = particle_pos[1] - exp_min[1];
            let dy1 = exp_max[1] - particle_pos[1];
            let dz0 = particle_pos[2] - exp_min[2];
            let dz1 = exp_max[2] - particle_pos[2];

            let candidates: [(f32, Vec3); 6] = [
                (dx0, [-1.0, 0.0, 0.0]),
                (dx1, [1.0, 0.0, 0.0]),
                (dy0, [0.0, -1.0, 0.0]),
                (dy1, [0.0, 1.0, 0.0]),
                (dz0, [0.0, 0.0, -1.0]),
                (dz1, [0.0, 0.0, 1.0]),
            ];
            let (depth, normal) = candidates
                .iter()
                .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
                .copied()
                .unwrap_or((0.0, [0.0, 1.0, 0.0]));
            Some((depth, normal))
        }

        ColliderShape::Capsule { a, b, radius } => {
            let closest = closest_point_on_segment(particle_pos, *a, *b);
            let d = sub(particle_pos, closest);
            let dist = len(d);
            let combined = radius + particle_radius;
            if dist < combined {
                let depth = combined - dist;
                let normal = if dist > 1e-10 {
                    scale(d, 1.0 / dist)
                } else {
                    [0.0, 1.0, 0.0]
                };
                Some((depth, normal))
            } else {
                None
            }
        }

        ColliderShape::Plane { normal, point } => {
            let n = normalize(*normal);
            let dist = dot(sub(particle_pos, *point), n);
            if dist < particle_radius {
                let depth = particle_radius - dist;
                Some((depth, n))
            } else {
                None
            }
        }
    }
}

/// Apply position correction to `particle_pos` so it no longer penetrates
/// `collider`.  Returns the corrected position.
#[allow(dead_code)]
pub fn resolve_particle_collision(
    particle_pos: Vec3,
    particle_radius: f32,
    collider: &ParticleCollider,
    config: &ParticleColliderConfig,
) -> Vec3 {
    if let Some((depth, normal)) = test_particle_vs_collider(particle_pos, particle_radius, collider) {
        let correction = (depth - config.slop).max(0.0);
        add(particle_pos, scale(normal, correction))
    } else {
        particle_pos
    }
}

/// Number of colliders in the system.
#[allow(dead_code)]
pub fn collider_count(system: &ParticleColliderSystem) -> usize {
    system.colliders.len()
}

/// Add a collider to the system, assigning a unique ID.  Returns the ID.
#[allow(dead_code)]
pub fn add_collider(system: &mut ParticleColliderSystem, mut collider: ParticleCollider) -> u32 {
    let id = system.next_id;
    system.next_id += 1;
    collider.id = id;
    system.colliders.push(collider);
    id
}

/// Remove a collider by its ID.  Returns `true` if found and removed.
#[allow(dead_code)]
pub fn remove_collider(system: &mut ParticleColliderSystem, id: u32) -> bool {
    let before = system.colliders.len();
    system.colliders.retain(|c| c.id != id);
    system.colliders.len() < before
}

/// Return the AABB that loosely bounds a collider.
#[allow(dead_code)]
pub fn collider_bounds(collider: &ParticleCollider) -> Aabb {
    match &collider.shape {
        ColliderShape::Sphere { center, radius } => (
            [center[0] - radius, center[1] - radius, center[2] - radius],
            [center[0] + radius, center[1] + radius, center[2] + radius],
        ),
        ColliderShape::Box { min, max } => (*min, *max),
        ColliderShape::Capsule { a, b, radius } => {
            let min_x = a[0].min(b[0]) - radius;
            let min_y = a[1].min(b[1]) - radius;
            let min_z = a[2].min(b[2]) - radius;
            let max_x = a[0].max(b[0]) + radius;
            let max_y = a[1].max(b[1]) + radius;
            let max_z = a[2].max(b[2]) + radius;
            ([min_x, min_y, min_z], [max_x, max_y, max_z])
        }
        ColliderShape::Plane { .. } => (
            [f32::NEG_INFINITY; 3],
            [f32::INFINITY; 3],
        ),
    }
}

/// Run one collision-response step: resolve all active colliders for a single
/// particle.  Returns the corrected position.
#[allow(dead_code)]
pub fn particle_collider_step(
    particle_pos: Vec3,
    particle_radius: f32,
    system: &ParticleColliderSystem,
) -> Vec3 {
    let mut pos = particle_pos;
    for collider in &system.colliders {
        pos = resolve_particle_collision(pos, particle_radius, collider, &system.config);
    }
    pos
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_collider_config() {
        let cfg = default_collider_config();
        assert!(cfg.restitution >= 0.0);
        assert!(cfg.friction >= 0.0);
        assert!(cfg.slop >= 0.0);
    }

    #[test]
    fn test_new_particle_collider_empty() {
        let system = new_particle_collider(default_collider_config());
        assert_eq!(collider_count(&system), 0);
    }

    #[test]
    fn test_add_collider_increments_count() {
        let mut sys = new_particle_collider(default_collider_config());
        add_collider(&mut sys, sphere_collider([0.0; 3], 1.0));
        assert_eq!(collider_count(&sys), 1);
    }

    #[test]
    fn test_add_collider_returns_unique_ids() {
        let mut sys = new_particle_collider(default_collider_config());
        let id0 = add_collider(&mut sys, sphere_collider([0.0; 3], 1.0));
        let id1 = add_collider(&mut sys, sphere_collider([5.0, 0.0, 0.0], 1.0));
        assert_ne!(id0, id1);
    }

    #[test]
    fn test_remove_collider() {
        let mut sys = new_particle_collider(default_collider_config());
        let id = add_collider(&mut sys, sphere_collider([0.0; 3], 1.0));
        assert!(remove_collider(&mut sys, id));
        assert_eq!(collider_count(&sys), 0);
    }

    #[test]
    fn test_remove_nonexistent_collider() {
        let mut sys = new_particle_collider(default_collider_config());
        assert!(!remove_collider(&mut sys, 99));
    }

    #[test]
    fn test_sphere_collider_penetration() {
        let collider = sphere_collider([0.0; 3], 1.0);
        // Particle at (0.5, 0, 0) with radius 0.1 → 0.5+0.1 = 0.6 < 1.0+0.1
        let result = test_particle_vs_collider([0.5, 0.0, 0.0], 0.1, &collider);
        assert!(result.is_some());
    }

    #[test]
    fn test_sphere_collider_no_contact() {
        let collider = sphere_collider([0.0; 3], 1.0);
        // Particle at (3, 0, 0) with radius 0.1 → clearly outside
        let result = test_particle_vs_collider([3.0, 0.0, 0.0], 0.1, &collider);
        assert!(result.is_none());
    }

    #[test]
    fn test_plane_collider_penetration() {
        let collider = plane_collider([0.0, 1.0, 0.0], [0.0; 3]);
        // Particle at y=-0.5 with radius 0.1 → penetrating floor
        let result = test_particle_vs_collider([0.0, -0.5, 0.0], 0.1, &collider);
        assert!(result.is_some());
        let (depth, _normal) = result.unwrap();
        assert!(depth > 0.0);
    }

    #[test]
    fn test_plane_collider_above() {
        let collider = plane_collider([0.0, 1.0, 0.0], [0.0; 3]);
        // Particle well above the floor
        let result = test_particle_vs_collider([0.0, 5.0, 0.0], 0.1, &collider);
        assert!(result.is_none());
    }

    #[test]
    fn test_capsule_collider_penetration() {
        let collider = capsule_collider([0.0, 0.0, 0.0], [0.0, 2.0, 0.0], 0.5);
        // Particle at (0.3, 1, 0) with radius 0.1 → inside capsule
        let result = test_particle_vs_collider([0.3, 1.0, 0.0], 0.1, &collider);
        assert!(result.is_some());
    }

    #[test]
    fn test_box_collider_penetration() {
        let collider = box_collider([-1.0; 3], [1.0; 3]);
        // Particle at origin with radius 0.1 → inside box
        let result = test_particle_vs_collider([0.0; 3], 0.1, &collider);
        assert!(result.is_some());
    }

    #[test]
    fn test_box_collider_outside() {
        let collider = box_collider([-1.0; 3], [1.0; 3]);
        let result = test_particle_vs_collider([5.0, 0.0, 0.0], 0.1, &collider);
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_moves_particle_out() {
        let collider = sphere_collider([0.0; 3], 1.0);
        let cfg = default_collider_config();
        let new_pos = resolve_particle_collision([0.5, 0.0, 0.0], 0.1, &collider, &cfg);
        // After resolution particle should be farther from origin
        let dist_new = len(new_pos);
        assert!(dist_new >= 0.5);
    }

    #[test]
    fn test_collider_bounds_sphere() {
        let collider = sphere_collider([1.0, 2.0, 3.0], 0.5);
        let (mn, mx) = collider_bounds(&collider);
        assert!((mn[0] - 0.5).abs() < 1e-5);
        assert!((mx[0] - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_particle_collider_step_no_colliders() {
        let sys = new_particle_collider(default_collider_config());
        let pos = [1.0, 2.0, 3.0];
        let result = particle_collider_step(pos, 0.1, &sys);
        assert_eq!(result, pos);
    }

    #[test]
    fn test_disabled_collider_skipped() {
        let mut collider = sphere_collider([0.0; 3], 1.0);
        collider.enabled = false;
        let result = test_particle_vs_collider([0.5, 0.0, 0.0], 0.1, &collider);
        assert!(result.is_none());
    }
}
