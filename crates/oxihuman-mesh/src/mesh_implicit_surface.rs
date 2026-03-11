//! Implicit surface evaluation via signed potential fields (blobs / metaballs).
//!
//! Each blob contributes a radially decaying potential; the implicit surface is
//! the iso-contour where the total potential equals a user-defined level.
//! Useful for metaball sculpting and CSG soft-union operations.

#![allow(dead_code)]

// ── Math helpers ──────────────────────────────────────────────────────────────

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn len3sq(v: [f32; 3]) -> f32 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

// ── Public types ──────────────────────────────────────────────────────────────

/// A single implicit blob (metaball).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImplicitBlob {
    /// World-space centre of the blob.
    pub centre: [f32; 3],
    /// Radius of influence.
    pub radius: f32,
    /// Strength of the blob (scales the potential).
    pub strength: f32,
    /// Whether the blob is active.
    pub active: bool,
}

/// Configuration for the implicit surface evaluator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImplicitSurfaceConfig {
    /// The potential value that defines the surface.
    pub iso_level: f32,
    /// Step size used for gradient estimation.
    pub gradient_eps: f32,
}

/// Implicit surface with a collection of blobs.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImplicitSurface {
    /// All blobs contributing to the field.
    pub blobs: Vec<ImplicitBlob>,
    /// Configuration.
    pub config: ImplicitSurfaceConfig,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return default `ImplicitSurfaceConfig`.
#[allow(dead_code)]
pub fn default_implicit_surface_config() -> ImplicitSurfaceConfig {
    ImplicitSurfaceConfig { iso_level: 1.0, gradient_eps: 1e-3 }
}

/// Create a new empty `ImplicitSurface`.
#[allow(dead_code)]
pub fn new_implicit_surface(config: ImplicitSurfaceConfig) -> ImplicitSurface {
    ImplicitSurface { blobs: Vec::new(), config }
}

/// Add a blob to the implicit surface and return its index.
#[allow(dead_code)]
pub fn implicit_add_blob(
    surface: &mut ImplicitSurface,
    centre: [f32; 3],
    radius: f32,
    strength: f32,
) -> usize {
    let idx = surface.blobs.len();
    surface.blobs.push(ImplicitBlob {
        centre,
        radius: radius.max(1e-6),
        strength,
        active: true,
    });
    idx
}

/// Evaluate the total potential at world-space `point`.
///
/// Uses Blinn's classic radial basis: `f(r) = strength * (1 - (r/R)^2)^2`
/// clamped to zero outside the radius.
#[allow(dead_code)]
pub fn implicit_evaluate(surface: &ImplicitSurface, point: [f32; 3]) -> f32 {
    let mut total = 0.0f32;
    for blob in &surface.blobs {
        if !blob.active {
            continue;
        }
        let r2 = len3sq(sub3(point, blob.centre));
        let r2_max = blob.radius * blob.radius;
        if r2 >= r2_max {
            continue;
        }
        let t = 1.0 - r2 / r2_max;
        total += blob.strength * t * t;
    }
    total
}

/// Return the number of blobs in the surface.
#[allow(dead_code)]
pub fn implicit_blob_count(surface: &ImplicitSurface) -> usize {
    surface.blobs.len()
}

/// Return the configured iso level.
#[allow(dead_code)]
pub fn implicit_iso_level(surface: &ImplicitSurface) -> f32 {
    surface.config.iso_level
}

/// Remove all blobs from the surface.
#[allow(dead_code)]
pub fn implicit_surface_clear(surface: &mut ImplicitSurface) {
    surface.blobs.clear();
}

/// Serialise the surface to a JSON string.
#[allow(dead_code)]
pub fn implicit_surface_to_json(surface: &ImplicitSurface) -> String {
    format!(
        "{{\"blob_count\":{},\"iso_level\":{}}}",
        surface.blobs.len(),
        surface.config.iso_level,
    )
}

/// Estimate the field gradient at `point` via central differences.
#[allow(dead_code)]
pub fn implicit_gradient(surface: &ImplicitSurface, point: [f32; 3]) -> [f32; 3] {
    let h = surface.config.gradient_eps;
    let dx = implicit_evaluate(surface, [point[0] + h, point[1], point[2]])
           - implicit_evaluate(surface, [point[0] - h, point[1], point[2]]);
    let dy = implicit_evaluate(surface, [point[0], point[1] + h, point[2]])
           - implicit_evaluate(surface, [point[0], point[1] - h, point[2]]);
    let dz = implicit_evaluate(surface, [point[0], point[1], point[2] + h])
           - implicit_evaluate(surface, [point[0], point[1], point[2] - h]);
    let inv = 1.0 / (2.0 * h);
    [dx * inv, dy * inv, dz * inv]
}

/// Deactivate the blob at `index` (removing it from evaluation without
/// shifting subsequent indices).
#[allow(dead_code)]
pub fn implicit_remove_blob(surface: &mut ImplicitSurface, index: usize) {
    if let Some(b) = surface.blobs.get_mut(index) {
        b.active = false;
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn one_blob_surface() -> ImplicitSurface {
        let mut s = new_implicit_surface(default_implicit_surface_config());
        implicit_add_blob(&mut s, [0.0, 0.0, 0.0], 1.0, 1.0);
        s
    }

    #[test]
    fn test_default_config_iso_level_positive() {
        let cfg = default_implicit_surface_config();
        assert!(cfg.iso_level > 0.0);
    }

    #[test]
    fn test_new_surface_has_no_blobs() {
        let s = new_implicit_surface(default_implicit_surface_config());
        assert_eq!(implicit_blob_count(&s), 0);
    }

    #[test]
    fn test_add_blob_increases_count() {
        let mut s = new_implicit_surface(default_implicit_surface_config());
        implicit_add_blob(&mut s, [0.0, 0.0, 0.0], 1.0, 1.0);
        assert_eq!(implicit_blob_count(&s), 1);
    }

    #[test]
    fn test_evaluate_at_centre_is_strength() {
        let s = one_blob_surface();
        let v = implicit_evaluate(&s, [0.0, 0.0, 0.0]);
        // At r=0: (1 - 0)^2 * strength = strength
        assert!((v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_outside_radius_is_zero() {
        let s = one_blob_surface();
        let v = implicit_evaluate(&s, [10.0, 0.0, 0.0]);
        assert!(v.abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_decreases_with_distance() {
        let s = one_blob_surface();
        let v0 = implicit_evaluate(&s, [0.0, 0.0, 0.0]);
        let v1 = implicit_evaluate(&s, [0.5, 0.0, 0.0]);
        assert!(v0 > v1);
    }

    #[test]
    fn test_implicit_iso_level() {
        let cfg = ImplicitSurfaceConfig { iso_level: 2.5, ..default_implicit_surface_config() };
        let s = new_implicit_surface(cfg);
        assert!((implicit_iso_level(&s) - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_implicit_surface_clear() {
        let mut s = one_blob_surface();
        implicit_surface_clear(&mut s);
        assert_eq!(implicit_blob_count(&s), 0);
    }

    #[test]
    fn test_implicit_surface_to_json() {
        let s = one_blob_surface();
        let json = implicit_surface_to_json(&s);
        assert!(json.contains("blob_count"));
        assert!(json.contains("iso_level"));
    }

    #[test]
    fn test_implicit_gradient_non_zero_inside() {
        let s = one_blob_surface();
        let g = implicit_gradient(&s, [0.3, 0.0, 0.0]);
        // The gradient should point roughly in the -x direction (away from centre toward outside)
        let mag_sq = g[0] * g[0] + g[1] * g[1] + g[2] * g[2];
        assert!(mag_sq > 0.0);
    }

    #[test]
    fn test_implicit_remove_blob() {
        let mut s = one_blob_surface();
        implicit_remove_blob(&mut s, 0);
        let v = implicit_evaluate(&s, [0.0, 0.0, 0.0]);
        assert!(v.abs() < 1e-6, "removed blob should not contribute, got {}", v);
    }

    #[test]
    fn test_two_blobs_sum_potentials() {
        let mut s = new_implicit_surface(default_implicit_surface_config());
        implicit_add_blob(&mut s, [0.0, 0.0, 0.0], 1.0, 1.0);
        implicit_add_blob(&mut s, [0.0, 0.0, 0.0], 1.0, 1.0);
        let v = implicit_evaluate(&s, [0.0, 0.0, 0.0]);
        assert!((v - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_remove_blob_out_of_range_no_panic() {
        let mut s = one_blob_surface();
        implicit_remove_blob(&mut s, 999);
    }
}
