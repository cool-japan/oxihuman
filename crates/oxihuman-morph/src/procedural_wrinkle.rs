// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Procedural wrinkle generator stub.

/// Wrinkle pattern type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WrinklePattern {
    Linear,
    Radial,
    Noise,
}

/// A single procedural wrinkle region.
#[derive(Debug, Clone)]
pub struct WrinkleRegion {
    pub pattern: WrinklePattern,
    pub center: [f32; 3],
    pub radius: f32,
    pub amplitude: f32,
    pub frequency: f32,
    pub driver_weight: f32,
}

/// Procedural wrinkle generator.
#[derive(Debug, Clone)]
pub struct ProceduralWrinkle {
    pub regions: Vec<WrinkleRegion>,
    pub vertex_count: usize,
    pub enabled: bool,
    pub global_scale: f32,
}

impl ProceduralWrinkle {
    pub fn new(vertex_count: usize) -> Self {
        ProceduralWrinkle {
            regions: Vec::new(),
            vertex_count,
            enabled: true,
            global_scale: 1.0,
        }
    }
}

/// Create a new procedural wrinkle generator.
pub fn new_procedural_wrinkle(vertex_count: usize) -> ProceduralWrinkle {
    ProceduralWrinkle::new(vertex_count)
}

/// Add a wrinkle region.
pub fn pw_add_region(pw: &mut ProceduralWrinkle, region: WrinkleRegion) {
    pw.regions.push(region);
}

/// Evaluate wrinkle offsets with no vertex positions (returns all-zero offsets).
///
/// When vertex positions are not available use [`pw_evaluate_with_positions`]
/// to obtain real wrinkle displacements.  This overload exists so that
/// callers that have not yet been updated continue to compile and run.
pub fn pw_evaluate(pw: &ProceduralWrinkle) -> Vec<[f32; 3]> {
    vec![[0.0_f32; 3]; pw.vertex_count]
}

/// Evaluate procedural wrinkle offsets for a given set of vertex positions.
///
/// For each vertex the function accumulates displacement contributions from
/// every [`WrinkleRegion`] whose sphere of influence covers that vertex:
///
/// 1. **Falloff** `t = 1 − dist/radius` (linear, 0 at the edge, 1 at the
///    centre) weights each region's contribution.
/// 2. **Pattern** determines the direction and shape of the displacement:
///    - [`WrinklePattern::Linear`] — displaces along the region's local +X
///      axis (world X) using a cosine ripple.
///    - [`WrinklePattern::Radial`] — displaces outward along the
///      centre → vertex direction using a cosine ripple.
///    - [`WrinklePattern::Noise`] — displaces along the world +Y axis using
///      a sine-based deterministic pseudo-noise pattern.
/// 3. Each displacement is additionally scaled by `driver_weight` and the
///    system-level `global_scale`.
///
/// `positions` must have the same length as `pw.vertex_count`.  If it is
/// shorter the function processes only the available vertices; if it is
/// longer only the first `vertex_count` entries are used.
pub fn pw_evaluate_with_positions(pw: &ProceduralWrinkle, positions: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let count = pw.vertex_count.min(positions.len());
    let mut offsets = vec![[0.0_f32; 3]; pw.vertex_count];

    if !pw.enabled || pw.global_scale <= 0.0 {
        return offsets;
    }

    for (v_idx, pos) in positions.iter().enumerate().take(count) {
        let mut acc = [0.0_f32; 3];

        for region in &pw.regions {
            // Vector from region centre to vertex.
            let dx = pos[0] - region.center[0];
            let dy = pos[1] - region.center[1];
            let dz = pos[2] - region.center[2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();

            if dist > region.radius {
                continue; // Outside influence sphere.
            }

            // Linear falloff: 1 at center, 0 at edge.
            let t = 1.0_f32 - dist / region.radius;

            // Cosine ripple magnitude (same formula for Linear and Radial).
            let ripple = t * region.amplitude * (dist * region.frequency).cos();

            let displacement: [f32; 3] = match region.pattern {
                WrinklePattern::Linear => {
                    // Displace along world +X axis.
                    [ripple, 0.0_f32, 0.0_f32]
                }
                WrinklePattern::Radial => {
                    // Displace along the centre → vertex direction.
                    if dist < 1e-8 {
                        [ripple, 0.0_f32, 0.0_f32]
                    } else {
                        let inv = 1.0_f32 / dist;
                        [ripple * dx * inv, ripple * dy * inv, ripple * dz * inv]
                    }
                }
                WrinklePattern::Noise => {
                    // Deterministic sine-based pseudo-noise along world +Y axis.
                    let noise = t * region.amplitude * (dist * region.frequency).sin();
                    [0.0_f32, noise, 0.0_f32]
                }
            };

            // Apply driver weight and global scale.
            let scale = region.driver_weight * pw.global_scale;
            acc[0] += displacement[0] * scale;
            acc[1] += displacement[1] * scale;
            acc[2] += displacement[2] * scale;
        }

        offsets[v_idx] = acc;
    }

    offsets
}

/// Set global scale.
pub fn pw_set_global_scale(pw: &mut ProceduralWrinkle, scale: f32) {
    pw.global_scale = scale.max(0.0);
}

/// Enable or disable.
pub fn pw_set_enabled(pw: &mut ProceduralWrinkle, enabled: bool) {
    pw.enabled = enabled;
}

/// Return region count.
pub fn pw_region_count(pw: &ProceduralWrinkle) -> usize {
    pw.regions.len()
}

/// Serialize to JSON-like string.
pub fn pw_to_json(pw: &ProceduralWrinkle) -> String {
    format!(
        r#"{{"vertex_count":{},"region_count":{},"global_scale":{},"enabled":{}}}"#,
        pw.vertex_count,
        pw.regions.len(),
        pw.global_scale,
        pw.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vertex_count() {
        let pw = new_procedural_wrinkle(200);
        assert_eq!(pw.vertex_count, 200 /* vertex count must match */,);
    }

    #[test]
    fn test_no_regions_initially() {
        let pw = new_procedural_wrinkle(100);
        assert_eq!(pw_region_count(&pw), 0 /* no regions initially */,);
    }

    #[test]
    fn test_add_region() {
        let mut pw = new_procedural_wrinkle(100);
        pw_add_region(
            &mut pw,
            WrinkleRegion {
                pattern: WrinklePattern::Linear,
                center: [0.0; 3],
                radius: 1.0,
                amplitude: 0.5,
                frequency: 2.0,
                driver_weight: 1.0,
            },
        );
        assert_eq!(pw_region_count(&pw), 1 /* one region after add */,);
    }

    #[test]
    fn test_evaluate_length() {
        let pw = new_procedural_wrinkle(50);
        let out = pw_evaluate(&pw);
        assert_eq!(
            out.len(),
            50, /* output length must match vertex count */
        );
    }

    #[test]
    fn test_evaluate_zeroed() {
        let pw = new_procedural_wrinkle(4);
        let out = pw_evaluate(&pw);
        assert!((out[0][0]).abs() < 1e-6 /* stub must return zeros */,);
    }

    #[test]
    fn test_set_global_scale() {
        let mut pw = new_procedural_wrinkle(10);
        pw_set_global_scale(&mut pw, 2.5);
        assert!((pw.global_scale - 2.5).abs() < 1e-5, /* scale must be set */);
    }

    #[test]
    fn test_global_scale_clamped_negative() {
        let mut pw = new_procedural_wrinkle(10);
        pw_set_global_scale(&mut pw, -1.0);
        assert!((pw.global_scale).abs() < 1e-6, /* negative scale clamped to 0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut pw = new_procedural_wrinkle(10);
        pw_set_enabled(&mut pw, false);
        assert!(!pw.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_vertex_count() {
        let pw = new_procedural_wrinkle(30);
        let j = pw_to_json(&pw);
        assert!(j.contains("\"vertex_count\""), /* json must contain vertex_count */);
    }

    #[test]
    fn test_enabled_default() {
        let pw = new_procedural_wrinkle(1);
        assert!(pw.enabled /* must be enabled by default */,);
    }

    #[test]
    fn pw_evaluate_region_affects_nearby_vertex() {
        // A radial region at the origin with radius 1.0 should produce a
        // non-zero offset for a vertex at [0.5, 0, 0] which lies inside it.
        let mut pw = new_procedural_wrinkle(1);
        pw_add_region(
            &mut pw,
            WrinkleRegion {
                pattern: WrinklePattern::Radial,
                center: [0.0, 0.0, 0.0],
                radius: 1.0,
                amplitude: 1.0,
                frequency: 1.0,
                driver_weight: 1.0,
            },
        );
        let positions: Vec<[f32; 3]> = vec![[0.5, 0.0, 0.0]];
        let offsets = pw_evaluate_with_positions(&pw, &positions);
        let mag = offsets[0].iter().map(|&v| v * v).sum::<f32>().sqrt();
        assert!(
            mag > 1e-6,
            "vertex inside region must receive a non-zero offset, got magnitude {mag}"
        );
    }

    #[test]
    fn pw_evaluate_vertex_outside_region_zero() {
        // A vertex at [2.0, 0, 0] is outside a region of radius 1.0 centred
        // at the origin and must receive exactly zero offset.
        let mut pw = new_procedural_wrinkle(1);
        pw_add_region(
            &mut pw,
            WrinkleRegion {
                pattern: WrinklePattern::Radial,
                center: [0.0, 0.0, 0.0],
                radius: 1.0,
                amplitude: 1.0,
                frequency: 1.0,
                driver_weight: 1.0,
            },
        );
        let positions: Vec<[f32; 3]> = vec![[2.0, 0.0, 0.0]];
        let offsets = pw_evaluate_with_positions(&pw, &positions);
        let mag = offsets[0].iter().map(|&v| v * v).sum::<f32>().sqrt();
        assert!(
            mag < 1e-6,
            "vertex outside region must have zero offset, got magnitude {mag}"
        );
    }
}
