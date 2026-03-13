// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Garment fitting to a body proxy using SDF proximity constraints and
//! iterative relaxation.

// ── Data structures ───────────────────────────────────────────────────────────

/// A vertex in a garment mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GarmentVertex {
    /// Current world-space position.
    pub position: [f32; 3],
    /// Rest (template) position for spring pull-back.
    pub rest_position: [f32; 3],
    /// Seam vertices have higher stiffness.
    pub is_seam: bool,
    /// Layer index: 0 = inner, 1 = middle, 2 = outer.
    pub layer: u8,
}

/// Configuration for garment fitting.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GarmentFitConfig {
    /// Number of relaxation iterations, default `20`.
    pub iterations: u32,
    /// Minimum clearance from body surface in metres, default `0.002`.
    pub body_offset: f32,
    /// Spring pull-back stiffness 0..1 for regular vertices, default `0.3`.
    pub stiffness: f32,
    /// Spring pull-back stiffness for seam vertices, default `0.8`.
    pub seam_stiffness: f32,
    /// Gravity vector, default `[0.0, -9.81, 0.0]`.
    pub gravity: [f32; 3],
}

impl Default for GarmentFitConfig {
    fn default() -> Self {
        Self {
            iterations: 20,
            body_offset: 0.002,
            stiffness: 0.3,
            seam_stiffness: 0.8,
            gravity: [0.0, -9.81, 0.0],
        }
    }
}

/// Result of a garment fitting pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GarmentFitResult {
    /// Fitted garment vertices.
    pub vertices: Vec<GarmentVertex>,
    /// Maximum penetration depth (negative means penetration; ≥ 0 means clear).
    pub max_penetration: f32,
    /// Average clearance from body surface.
    pub avg_clearance: f32,
    /// Number of relaxation iterations actually executed.
    pub iterations_run: u32,
}

// ── Vector helpers ────────────────────────────────────────────────────────────

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn len3(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

// ── SDF primitives ────────────────────────────────────────────────────────────

/// Signed distance from `point` to a capsule defined by endpoints `a`, `b`
/// and `radius`. Negative values indicate the point is inside the capsule.
#[allow(dead_code)]
pub fn point_to_capsule_sdf(point: [f32; 3], a: [f32; 3], b: [f32; 3], radius: f32) -> f32 {
    let ab = sub3(b, a);
    let ap = sub3(point, a);
    let len_ab_sq = dot3(ab, ab);
    let t = if len_ab_sq < 1e-12 {
        0.0
    } else {
        (dot3(ap, ab) / len_ab_sq).clamp(0.0, 1.0)
    };
    let closest = add3(a, scale3(ab, t));
    len3(sub3(point, closest)) - radius
}

/// Signed distance from `point` to a sphere. Negative values indicate inside.
#[allow(dead_code)]
pub fn point_to_sphere_sdf(point: [f32; 3], center: [f32; 3], radius: f32) -> f32 {
    len3(sub3(point, center)) - radius
}

/// Move `point` so that it is at least `clearance` away from the capsule
/// surface defined by endpoints `a`, `b` and `radius`.
///
/// If the point is on or inside the capsule, it is projected radially outward
/// from the nearest point on the capsule axis to distance `radius + clearance`.
#[allow(dead_code)]
pub fn push_out_of_capsule(
    point: [f32; 3],
    a: [f32; 3],
    b: [f32; 3],
    radius: f32,
    clearance: f32,
) -> [f32; 3] {
    let ab = sub3(b, a);
    let ap = sub3(point, a);
    let len_ab_sq = dot3(ab, ab);
    let t = if len_ab_sq < 1e-12 {
        0.0
    } else {
        (dot3(ap, ab) / len_ab_sq).clamp(0.0, 1.0)
    };
    let closest = add3(a, scale3(ab, t));
    let diff = sub3(point, closest);
    let dist = len3(diff);
    let target = radius + clearance;
    if dist < target {
        // Pick a push direction: use diff if large enough, otherwise pick
        // a direction perpendicular to the capsule axis.
        let dir = if dist > 1e-6 {
            scale3(diff, 1.0 / dist)
        } else {
            // point is on the capsule axis — pick a vector perpendicular to ab
            let ab_norm = if len_ab_sq > 1e-12 {
                scale3(ab, 1.0 / len_ab_sq.sqrt())
            } else {
                [0.0, 1.0, 0.0]
            };
            // cross with world-up to get a perpendicular; if parallel, use world-right
            let world_up = if ab_norm[1].abs() < 0.9 {
                [0.0_f32, 1.0, 0.0]
            } else {
                [1.0_f32, 0.0, 0.0]
            };
            let perp = [
                ab_norm[1] * world_up[2] - ab_norm[2] * world_up[1],
                ab_norm[2] * world_up[0] - ab_norm[0] * world_up[2],
                ab_norm[0] * world_up[1] - ab_norm[1] * world_up[0],
            ];
            let perp_len = len3(perp);
            if perp_len > 1e-9 {
                scale3(perp, 1.0 / perp_len)
            } else {
                [1.0, 0.0, 0.0]
            }
        };
        add3(closest, scale3(dir, target))
    } else {
        point
    }
}

/// Linear spring pull-back toward `rest`.
/// `result = current + stiffness * (rest - current)`
#[allow(dead_code)]
pub fn spring_pull(current: [f32; 3], rest: [f32; 3], stiffness: f32) -> [f32; 3] {
    let delta = sub3(rest, current);
    add3(current, scale3(delta, stiffness))
}

// ── Fitting ───────────────────────────────────────────────────────────────────

/// Iteratively fit a garment to body proxies.
///
/// Each iteration:
/// 1. For every vertex, check SDF against every capsule and sphere; push out
///    if penetrating or closer than `body_offset`.
/// 2. Apply spring pull-back toward `rest_position`.
/// 3. Apply a single gravity sub-step (`x += g * dt²`, `dt = 1/60`).
/// 4. Re-enforce push-out constraints after the spring/gravity step.
#[allow(dead_code)]
pub fn fit_garment_to_proxies(
    garment: &[GarmentVertex],
    proxies: &super::BodyProxies,
    cfg: &GarmentFitConfig,
) -> GarmentFitResult {
    let mut verts: Vec<GarmentVertex> = garment.to_vec();
    let dt = 1.0_f32 / 60.0;
    let grav_step = scale3(cfg.gravity, dt * dt);

    for _ in 0..cfg.iterations {
        for v in verts.iter_mut() {
            // --- push out of every proxy
            push_out_vertex(v, proxies, cfg.body_offset);

            // --- spring pull-back
            let k = if v.is_seam {
                cfg.seam_stiffness
            } else {
                cfg.stiffness
            };
            v.position = spring_pull(v.position, v.rest_position, k);

            // --- gravity
            v.position = add3(v.position, grav_step);

            // --- re-enforce push-out so gravity/spring doesn't drag back inside
            push_out_vertex(v, proxies, cfg.body_offset);
        }
    }

    // --- compute statistics
    let mut max_penetration = f32::MAX;
    let mut total_clearance = 0.0_f32;

    for v in &verts {
        let min_sdf = min_sdf_to_proxies(v.position, proxies);
        let sdf = if min_sdf == f32::MAX {
            cfg.body_offset // no proxies → treat as cleared
        } else {
            min_sdf
        };
        if sdf < max_penetration {
            max_penetration = sdf;
        }
        total_clearance += sdf;
    }

    let avg_clearance = if verts.is_empty() {
        0.0
    } else {
        total_clearance / verts.len() as f32
    };

    if max_penetration == f32::MAX {
        max_penetration = 0.0;
    }

    GarmentFitResult {
        vertices: verts,
        max_penetration,
        avg_clearance,
        iterations_run: cfg.iterations,
    }
}

/// Minimum SDF of `point` against all proxies (f32::MAX if no proxies).
fn min_sdf_to_proxies(point: [f32; 3], proxies: &super::BodyProxies) -> f32 {
    let mut min_sdf = f32::MAX;
    for cap in &proxies.capsules {
        let s = point_to_capsule_sdf(point, cap.center_a, cap.center_b, cap.radius);
        if s < min_sdf {
            min_sdf = s;
        }
    }
    for sph in &proxies.spheres {
        let s = point_to_sphere_sdf(point, sph.center, sph.radius);
        if s < min_sdf {
            min_sdf = s;
        }
    }
    min_sdf
}

/// Push a single vertex out of all proxies.
fn push_out_vertex(v: &mut GarmentVertex, proxies: &super::BodyProxies, body_offset: f32) {
    for cap in &proxies.capsules {
        if point_to_capsule_sdf(v.position, cap.center_a, cap.center_b, cap.radius) < body_offset {
            v.position = push_out_of_capsule(
                v.position,
                cap.center_a,
                cap.center_b,
                cap.radius,
                body_offset,
            );
        }
    }
    for sph in &proxies.spheres {
        let sdf = point_to_sphere_sdf(v.position, sph.center, sph.radius);
        if sdf < body_offset {
            let diff = sub3(v.position, sph.center);
            let dist = len3(diff);
            let target = sph.radius + body_offset;
            let dir = if dist > 1e-6 {
                scale3(diff, 1.0 / dist)
            } else {
                [0.0, 1.0, 0.0]
            };
            v.position = add3(sph.center, scale3(dir, target));
        }
    }
}

/// Human-readable summary of fitting statistics.
#[allow(dead_code)]
pub fn garment_clearance_stats(result: &GarmentFitResult) -> String {
    format!(
        "iterations={} max_penetration={:.4}m avg_clearance={:.4}m vertices={}",
        result.iterations_run,
        result.max_penetration,
        result.avg_clearance,
        result.vertices.len()
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BodyProxies, CapsuleProxy, SphereProxy};

    fn make_capsule_proxy(a: [f32; 3], b: [f32; 3], r: f32) -> CapsuleProxy {
        CapsuleProxy::new(a, b, r, "test")
    }

    fn make_sphere_proxy(c: [f32; 3], r: f32) -> SphereProxy {
        SphereProxy::new(c, r, "test")
    }

    // 1. point_to_capsule_sdf: inside (negative)
    #[test]
    fn capsule_sdf_inside() {
        let sdf = point_to_capsule_sdf([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        assert!(sdf < 0.0, "expected negative inside capsule, got {sdf}");
    }

    // 2. point_to_capsule_sdf: outside (positive)
    #[test]
    fn capsule_sdf_outside() {
        let sdf = point_to_capsule_sdf([2.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        assert!(sdf > 0.0, "expected positive outside capsule, got {sdf}");
    }

    // 3. point_to_capsule_sdf: on surface (≈0)
    #[test]
    fn capsule_sdf_on_surface() {
        let sdf = point_to_capsule_sdf([0.5, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        assert!(sdf.abs() < 1e-5, "expected ~0 on surface, got {sdf}");
    }

    // 4. point_to_sphere_sdf inside
    #[test]
    fn sphere_sdf_inside() {
        let sdf = point_to_sphere_sdf([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);
        assert!((sdf - (-1.0)).abs() < 1e-5, "expected -1.0, got {sdf}");
    }

    // 5. point_to_sphere_sdf outside
    #[test]
    fn sphere_sdf_outside() {
        let sdf = point_to_sphere_sdf([2.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);
        assert!((sdf - 1.0).abs() < 1e-5, "expected 1.0, got {sdf}");
    }

    // 6. point_to_sphere_sdf on surface
    #[test]
    fn sphere_sdf_on_surface() {
        let sdf = point_to_sphere_sdf([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);
        assert!(sdf.abs() < 1e-5, "expected ~0, got {sdf}");
    }

    // 7. push_out_of_capsule: result is outside (point starts at capsule axis)
    #[test]
    fn push_out_of_capsule_moves_outside() {
        let pushed = push_out_of_capsule(
            [0.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 1.0, 0.0],
            0.5,
            0.01,
        );
        let sdf = point_to_capsule_sdf(pushed, [0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        assert!(sdf >= -1e-5, "still inside after push, sdf={sdf}");
    }

    // 8. push_out_of_capsule: already outside → unchanged
    #[test]
    fn push_out_already_outside() {
        let p = [2.0_f32, 0.0, 0.0];
        let pushed = push_out_of_capsule(p, [0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 0.5, 0.01);
        assert!((pushed[0] - p[0]).abs() < 1e-5);
    }

    // 9. spring_pull at rest = no change
    #[test]
    fn spring_pull_at_rest() {
        let p = [1.0_f32, 2.0, 3.0];
        let out = spring_pull(p, p, 0.5);
        assert!((out[0] - p[0]).abs() < 1e-6);
        assert!((out[1] - p[1]).abs() < 1e-6);
        assert!((out[2] - p[2]).abs() < 1e-6);
    }

    // 10. spring_pull with stiffness=1.0 snaps to rest
    #[test]
    fn spring_pull_full_stiffness() {
        let current = [0.0_f32; 3];
        let rest = [1.0_f32, 2.0, 3.0];
        let out = spring_pull(current, rest, 1.0);
        assert!((out[0] - rest[0]).abs() < 1e-6);
        assert!((out[1] - rest[1]).abs() < 1e-6);
        assert!((out[2] - rest[2]).abs() < 1e-6);
    }

    fn single_sphere_proxies() -> BodyProxies {
        let mut p = BodyProxies::new();
        p.spheres.push(make_sphere_proxy([0.0; 3], 0.3));
        p
    }

    fn single_capsule_proxies() -> BodyProxies {
        let mut p = BodyProxies::new();
        p.capsules
            .push(make_capsule_proxy([0.0, -0.5, 0.0], [0.0, 0.5, 0.0], 0.3));
        p
    }

    fn make_vertex(pos: [f32; 3], is_seam: bool) -> GarmentVertex {
        GarmentVertex {
            position: pos,
            rest_position: pos,
            is_seam,
            layer: 0,
        }
    }

    // 11. fit_garment_to_proxies: no penetration after fitting (sphere)
    #[test]
    fn fit_no_penetration_sphere() {
        let proxies = single_sphere_proxies();
        let cfg = GarmentFitConfig {
            iterations: 30,
            body_offset: 0.005,
            ..Default::default()
        };
        // Vertex starts outside sphere, with rest position also outside
        let pos = [0.0_f32, 0.5, 0.0]; // just above sphere surface (r=0.3)
        let verts = vec![make_vertex(pos, false)];
        let result = fit_garment_to_proxies(&verts, &proxies, &cfg);
        assert!(
            result.max_penetration >= -1e-4,
            "penetration after fit: {}",
            result.max_penetration
        );
    }

    // 12. fit_garment_to_proxies: no penetration after fitting (capsule)
    #[test]
    fn fit_no_penetration_capsule() {
        let proxies = single_capsule_proxies();
        let cfg = GarmentFitConfig {
            iterations: 30,
            body_offset: 0.005,
            ..Default::default()
        };
        // Vertex starts off-axis near the capsule
        let pos = [0.4_f32, 0.0, 0.0]; // outside capsule (r=0.3)
        let verts = vec![make_vertex(pos, false)];
        let result = fit_garment_to_proxies(&verts, &proxies, &cfg);
        assert!(
            result.max_penetration >= -1e-4,
            "penetration after fit: {}",
            result.max_penetration
        );
    }

    // 13. max_penetration after fit with sphere and external start is non-negative
    #[test]
    fn fit_max_penetration_non_negative() {
        let proxies = single_sphere_proxies();
        let cfg = GarmentFitConfig {
            iterations: 30,
            body_offset: 0.005,
            ..Default::default()
        };
        // Rest position and start both outside
        let pos = [0.5_f32, 0.0, 0.0];
        let verts = vec![make_vertex(pos, false)];
        let result = fit_garment_to_proxies(&verts, &proxies, &cfg);
        assert!(result.max_penetration >= -1e-4);
    }

    // 14. seam vertex moves less than non-seam (higher stiffness snaps back to rest)
    #[test]
    fn seam_vertex_moves_less() {
        // No proxies → only spring forces and gravity act
        let proxies = BodyProxies::new();
        let cfg = GarmentFitConfig {
            iterations: 5,
            body_offset: 0.002,
            stiffness: 0.1,
            seam_stiffness: 0.9,
            gravity: [0.0, -9.81, 0.0],
        };
        // Place both at rest=5.0 but displaced to 4.0
        let rest = [0.0_f32, 5.0, 0.0];
        let displaced = [0.0_f32, 4.0, 0.0];

        let regular = GarmentVertex {
            position: displaced,
            rest_position: rest,
            is_seam: false,
            layer: 0,
        };
        let seam = GarmentVertex {
            position: displaced,
            rest_position: rest,
            is_seam: true,
            layer: 0,
        };

        let result = fit_garment_to_proxies(&[regular, seam], &proxies, &cfg);
        let pos_reg = result.vertices[0].position;
        let pos_seam = result.vertices[1].position;
        // Seam (stiffness=0.9) snaps closer to rest (y=5.0) than regular (stiffness=0.1)
        assert!(
            pos_seam[1] > pos_reg[1],
            "seam y={} should be > regular y={} (closer to rest=5.0)",
            pos_seam[1],
            pos_reg[1]
        );
    }

    // 15. garment_clearance_stats returns a non-empty string
    #[test]
    fn clearance_stats_non_empty() {
        let result = GarmentFitResult {
            vertices: vec![],
            max_penetration: 0.0,
            avg_clearance: 0.01,
            iterations_run: 10,
        };
        let s = garment_clearance_stats(&result);
        assert!(!s.is_empty());
        assert!(s.contains("iterations=10"));
    }
}
