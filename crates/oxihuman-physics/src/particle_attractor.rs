//! Point / plane attractor that pulls particles toward a target.
//!
//! Supports two attractor types:
//! - `Point` — particles are pulled toward a 3-D point with a strength that
//!   decays with distance.
//! - `Plane` — particles are pulled toward the nearest point on an infinite
//!   plane.

#![allow(dead_code)]

// ── Math helpers ──────────────────────────────────────────────────────────────

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn normalise3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l > 1e-10 { scale3(v, 1.0 / l) } else { [0.0; 3] }
}

// ── Public types ──────────────────────────────────────────────────────────────

/// Discriminates the attractor geometry.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttractorKind {
    /// Pulls particles toward a single 3-D point.
    Point,
    /// Pulls particles toward the closest point on an infinite plane.
    Plane,
}

/// Configuration for an attractor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AttractorConfig {
    /// Attractor type.
    pub kind: AttractorKind,
    /// Attractor strength (force magnitude scale).
    pub strength: f32,
    /// Position of the attractor (for `Point`) or a point on the plane (for `Plane`).
    pub position: [f32; 3],
    /// Outward normal of the plane (only used for `Plane` kind).
    pub plane_normal: [f32; 3],
    /// Whether the attractor is active.
    pub enabled: bool,
    /// Maximum influence radius (0 = unlimited).
    pub max_radius: f32,
}

/// An attractor instance.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParticleAttractor {
    /// Configuration.
    pub config: AttractorConfig,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a default `AttractorConfig` (point attractor at origin, strength 1).
#[allow(dead_code)]
pub fn default_attractor_config() -> AttractorConfig {
    AttractorConfig {
        kind: AttractorKind::Point,
        strength: 1.0,
        position: [0.0; 3],
        plane_normal: [0.0, 1.0, 0.0],
        enabled: true,
        max_radius: 0.0,
    }
}

/// Create a new point attractor at `position` with the given `strength`.
#[allow(dead_code)]
pub fn new_point_attractor(position: [f32; 3], strength: f32) -> ParticleAttractor {
    ParticleAttractor {
        config: AttractorConfig {
            kind: AttractorKind::Point,
            strength,
            position,
            plane_normal: [0.0, 1.0, 0.0],
            enabled: true,
            max_radius: 0.0,
        },
    }
}

/// Create a new plane attractor defined by `point_on_plane` and `normal`.
#[allow(dead_code)]
pub fn new_plane_attractor(
    point_on_plane: [f32; 3],
    normal: [f32; 3],
    strength: f32,
) -> ParticleAttractor {
    ParticleAttractor {
        config: AttractorConfig {
            kind: AttractorKind::Plane,
            strength,
            position: point_on_plane,
            plane_normal: normalise3(normal),
            enabled: true,
            max_radius: 0.0,
        },
    }
}

/// Compute the force vector the attractor exerts on a particle at `particle_pos`.
#[allow(dead_code)]
pub fn attractor_force_on(attractor: &ParticleAttractor, particle_pos: [f32; 3]) -> [f32; 3] {
    if !attractor.config.enabled {
        return [0.0; 3];
    }

    match attractor.config.kind {
        AttractorKind::Point => {
            let to_target = sub3(attractor.config.position, particle_pos);
            let dist = len3(to_target);
            if dist < 1e-10 {
                return [0.0; 3];
            }
            if attractor.config.max_radius > 1e-10 && dist > attractor.config.max_radius {
                return [0.0; 3];
            }
            // Inverse-square falloff
            let mag = attractor.config.strength / (dist * dist).max(1e-6);
            scale3(normalise3(to_target), mag)
        }
        AttractorKind::Plane => {
            let n = normalise3(attractor.config.plane_normal);
            // Signed distance from particle to plane
            let d = dot3(sub3(particle_pos, attractor.config.position), n);
            if attractor.config.max_radius > 1e-10 && d.abs() > attractor.config.max_radius {
                return [0.0; 3];
            }
            // Force toward the plane (opposite to signed normal direction)
            scale3(n, -d * attractor.config.strength)
        }
    }
}

/// Update the attractor strength.
#[allow(dead_code)]
pub fn attractor_set_strength(attractor: &mut ParticleAttractor, strength: f32) {
    attractor.config.strength = strength;
}

/// Update the attractor position (or point-on-plane for plane attractors).
#[allow(dead_code)]
pub fn attractor_set_position(attractor: &mut ParticleAttractor, position: [f32; 3]) {
    attractor.config.position = position;
}

/// Enable the attractor.
#[allow(dead_code)]
pub fn attractor_enable(attractor: &mut ParticleAttractor) {
    attractor.config.enabled = true;
}

/// Disable the attractor.
#[allow(dead_code)]
pub fn attractor_disable(attractor: &mut ParticleAttractor) {
    attractor.config.enabled = false;
}

/// Serialise the attractor to a JSON string.
#[allow(dead_code)]
pub fn attractor_to_json(attractor: &ParticleAttractor) -> String {
    format!(
        "{{\"kind\":\"{}\",\"strength\":{},\"enabled\":{}}}",
        attractor_kind_name(attractor),
        attractor.config.strength,
        attractor.config.enabled,
    )
}

/// Return a human-readable name for the attractor kind.
#[allow(dead_code)]
pub fn attractor_kind_name(attractor: &ParticleAttractor) -> &'static str {
    match attractor.config.kind {
        AttractorKind::Point => "Point",
        AttractorKind::Plane => "Plane",
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_point() {
        let cfg = default_attractor_config();
        assert_eq!(cfg.kind, AttractorKind::Point);
        assert!(cfg.enabled);
    }

    #[test]
    fn test_new_point_attractor_kind() {
        let a = new_point_attractor([1.0, 0.0, 0.0], 2.0);
        assert_eq!(a.config.kind, AttractorKind::Point);
        assert!((a.config.strength - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_plane_attractor_kind() {
        let a = new_plane_attractor([0.0; 3], [0.0, 1.0, 0.0], 1.0);
        assert_eq!(a.config.kind, AttractorKind::Plane);
    }

    #[test]
    fn test_point_attractor_force_direction() {
        // Attractor at origin, particle at [1,0,0] → force should point toward origin
        let a = new_point_attractor([0.0, 0.0, 0.0], 1.0);
        let f = attractor_force_on(&a, [1.0, 0.0, 0.0]);
        assert!(f[0] < 0.0, "force x should be negative (toward attractor)");
    }

    #[test]
    fn test_plane_attractor_force_direction() {
        // Plane at y=0 with normal [0,1,0]; particle at y=1 → force toward plane = -y
        let a = new_plane_attractor([0.0; 3], [0.0, 1.0, 0.0], 1.0);
        let f = attractor_force_on(&a, [0.0, 1.0, 0.0]);
        assert!(f[1] < 0.0, "force y should be negative (toward plane)");
    }

    #[test]
    fn test_disabled_attractor_zero_force() {
        let mut a = new_point_attractor([0.0; 3], 10.0);
        attractor_disable(&mut a);
        let f = attractor_force_on(&a, [1.0, 0.0, 0.0]);
        assert_eq!(f, [0.0; 3]);
    }

    #[test]
    fn test_enable_disable() {
        let mut a = new_point_attractor([0.0; 3], 1.0);
        attractor_disable(&mut a);
        assert!(!a.config.enabled);
        attractor_enable(&mut a);
        assert!(a.config.enabled);
    }

    #[test]
    fn test_set_strength() {
        let mut a = new_point_attractor([0.0; 3], 1.0);
        attractor_set_strength(&mut a, 5.0);
        assert!((a.config.strength - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_position() {
        let mut a = new_point_attractor([0.0; 3], 1.0);
        attractor_set_position(&mut a, [3.0, 4.0, 5.0]);
        assert!((a.config.position[0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_attractor_kind_name_point() {
        let a = new_point_attractor([0.0; 3], 1.0);
        assert_eq!(attractor_kind_name(&a), "Point");
    }

    #[test]
    fn test_attractor_kind_name_plane() {
        let a = new_plane_attractor([0.0; 3], [0.0, 1.0, 0.0], 1.0);
        assert_eq!(attractor_kind_name(&a), "Plane");
    }

    #[test]
    fn test_attractor_to_json() {
        let a = new_point_attractor([0.0; 3], 1.0);
        let json = attractor_to_json(&a);
        assert!(json.contains("kind"));
        assert!(json.contains("Point"));
        assert!(json.contains("strength"));
    }

    #[test]
    fn test_force_at_attractor_position_is_zero() {
        let a = new_point_attractor([5.0, 5.0, 5.0], 1.0);
        let f = attractor_force_on(&a, [5.0, 5.0, 5.0]);
        let mag = len3(f);
        assert!(mag < 1e-3, "force at attractor position should be near zero");
    }
}
