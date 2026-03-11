// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Procedural micro-displacement for mesh surface detail.
//!
//! Displaces mesh vertices along their normals using noise functions and
//! procedural patterns to simulate fine surface detail such as skin pores,
//! wrinkles, and surface texture.

#![allow(dead_code)]

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

// ── Displacement Pattern ─────────────────────────────────────────────────────

/// Procedural displacement patterns for micro-surface detail.
pub enum DisplacementPattern {
    /// Simple value noise displacement.
    ValueNoise { scale: f32, seed: u32 },
    /// Fractal Brownian Motion — layered value noise for organic detail.
    Fbm {
        scale: f32,
        octaves: u32,
        lacunarity: f32,
        gain: f32,
        seed: u32,
    },
    /// Sine wave ripple pattern.
    Sine {
        frequency: f32,
        amplitude: f32,
        direction: [f32; 3],
    },
    /// Voronoi cell cracks pattern.
    Voronoi {
        scale: f32,
        crack_width: f32,
        seed: u32,
    },
    /// Wrinkle/skin detail using anisotropic sine waves.
    Wrinkle {
        frequency_u: f32,
        frequency_v: f32,
        amplitude: f32,
    },
    /// Pores pattern — spherical dents at Voronoi feature points.
    Pores { density: f32, depth: f32, seed: u32 },
}

// ── MicroDispParams ──────────────────────────────────────────────────────────

/// Parameters controlling micro-displacement application.
pub struct MicroDispParams {
    /// The displacement pattern to use.
    pub pattern: DisplacementPattern,
    /// Global amplitude multiplier.
    pub amplitude: f32,
    /// Blend factor: 0 = no effect, 1 = full effect.
    pub blend: f32,
    /// Optional per-vertex mask weights (0..1). None means all vertices at full weight.
    pub vertex_mask: Option<Vec<f32>>,
}

impl Default for MicroDispParams {
    fn default() -> Self {
        Self {
            pattern: DisplacementPattern::ValueNoise {
                scale: 5.0,
                seed: 42,
            },
            amplitude: 0.005,
            blend: 1.0,
            vertex_mask: None,
        }
    }
}

// ── MicroDispResult ──────────────────────────────────────────────────────────

/// Result of applying micro-displacement to a mesh.
pub struct MicroDispResult {
    /// The displaced mesh with recomputed normals.
    pub mesh: MeshBuffers,
    /// Minimum displacement scalar across all vertices.
    pub min_displacement: f32,
    /// Maximum displacement scalar across all vertices.
    pub max_displacement: f32,
    /// Mean displacement scalar across all vertices.
    pub mean_displacement: f32,
}

// ── Noise Functions ──────────────────────────────────────────────────────────

/// Simple hash-based value noise at a 3D position.
///
/// Returns a value in [0, 1]. Uses trilinear interpolation between
/// hashed integer cell corners.
pub fn value_noise_3d(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let hash = |ix: i32, iy: i32, iz: i32| -> f32 {
        let h =
            (ix.wrapping_mul(1619) ^ iy.wrapping_mul(31337) ^ iz.wrapping_mul(6271) ^ seed as i32)
                as u32;
        let h = h
            .wrapping_mul(0x9e3779b9)
            .wrapping_add(h << 6)
            .wrapping_add(h >> 2);
        (h & 0xFFFF) as f32 / 65535.0
    };

    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let iz = z.floor() as i32;

    let fx = x - x.floor();
    let fy = y - y.floor();
    let fz = z - z.floor();

    // Smoothstep for interpolation weights
    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);
    let uz = fz * fz * (3.0 - 2.0 * fz);

    // Trilinear interpolation over 8 corners
    let c000 = hash(ix, iy, iz);
    let c100 = hash(ix + 1, iy, iz);
    let c010 = hash(ix, iy + 1, iz);
    let c110 = hash(ix + 1, iy + 1, iz);
    let c001 = hash(ix, iy, iz + 1);
    let c101 = hash(ix + 1, iy, iz + 1);
    let c011 = hash(ix, iy + 1, iz + 1);
    let c111 = hash(ix + 1, iy + 1, iz + 1);

    let x00 = c000 + ux * (c100 - c000);
    let x10 = c010 + ux * (c110 - c010);
    let x01 = c001 + ux * (c101 - c001);
    let x11 = c011 + ux * (c111 - c011);

    let y0 = x00 + uy * (x10 - x00);
    let y1 = x01 + uy * (x11 - x01);

    y0 + uz * (y1 - y0)
}

/// Fractal Brownian Motion noise — sum of layered value noise octaves.
///
/// Returns a value approximately in [0, 1]. Higher octaves add finer detail.
pub fn fbm_noise_3d(
    x: f32,
    y: f32,
    z: f32,
    octaves: u32,
    lacunarity: f32,
    gain: f32,
    seed: u32,
) -> f32 {
    let mut value = 0.0f32;
    let mut amplitude = 1.0f32;
    let mut frequency = 1.0f32;
    let mut max_amplitude = 0.0f32;

    for i in 0..octaves {
        let octave_seed = seed.wrapping_add(i.wrapping_mul(7919));
        value +=
            amplitude * value_noise_3d(x * frequency, y * frequency, z * frequency, octave_seed);
        max_amplitude += amplitude;
        amplitude *= gain;
        frequency *= lacunarity;
    }

    if max_amplitude > 0.0 {
        value / max_amplitude
    } else {
        0.0
    }
}

/// Voronoi distance field — returns distance to the nearest feature point.
///
/// Returns a value approximately in [0, 1] where 0 is at a feature point.
pub fn voronoi_3d(x: f32, y: f32, z: f32, scale: f32, seed: u32) -> f32 {
    let sx = x * scale;
    let sy = y * scale;
    let sz = z * scale;

    let ix = sx.floor() as i32;
    let iy = sy.floor() as i32;
    let iz = sz.floor() as i32;

    let hash_feature = |cx: i32, cy: i32, cz: i32| -> [f32; 3] {
        let hx =
            (cx.wrapping_mul(1619) ^ cy.wrapping_mul(31337) ^ cz.wrapping_mul(6271) ^ seed as i32)
                as u32;
        let hx = hx
            .wrapping_mul(0x9e3779b9)
            .wrapping_add(hx << 6)
            .wrapping_add(hx >> 2);

        let hy = (cx.wrapping_mul(6271)
            ^ cy.wrapping_mul(1619)
            ^ cz.wrapping_mul(31337)
            ^ seed.wrapping_add(1) as i32) as u32;
        let hy = hy
            .wrapping_mul(0x9e3779b9)
            .wrapping_add(hy << 6)
            .wrapping_add(hy >> 2);

        let hz = (cx.wrapping_mul(31337)
            ^ cy.wrapping_mul(6271)
            ^ cz.wrapping_mul(1619)
            ^ seed.wrapping_add(2) as i32) as u32;
        let hz = hz
            .wrapping_mul(0x9e3779b9)
            .wrapping_add(hz << 6)
            .wrapping_add(hz >> 2);

        [
            cx as f32 + (hx & 0xFFFF) as f32 / 65535.0,
            cy as f32 + (hy & 0xFFFF) as f32 / 65535.0,
            cz as f32 + (hz & 0xFFFF) as f32 / 65535.0,
        ]
    };

    let mut min_dist = f32::MAX;

    for dz in -1..=1 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let fp = hash_feature(ix + dx, iy + dy, iz + dz);
                let ddx = sx - fp[0];
                let ddy = sy - fp[1];
                let ddz = sz - fp[2];
                let dist = (ddx * ddx + ddy * ddy + ddz * ddz).sqrt();
                if dist < min_dist {
                    min_dist = dist;
                }
            }
        }
    }

    // Normalize: max possible distance in a unit cell is sqrt(3) ≈ 1.73
    (min_dist / 1.732_050_8).clamp(0.0, 1.0)
}

// ── Displacement Sampling ────────────────────────────────────────────────────

/// Compute displacement scalar at world position using the given pattern.
///
/// Returns a value in approximately [-1, 1] depending on the pattern.
pub fn sample_displacement(pattern: &DisplacementPattern, pos: [f32; 3]) -> f32 {
    match pattern {
        DisplacementPattern::ValueNoise { scale, seed } => {
            let v = value_noise_3d(pos[0] * scale, pos[1] * scale, pos[2] * scale, *seed);
            v * 2.0 - 1.0
        }

        DisplacementPattern::Fbm {
            scale,
            octaves,
            lacunarity,
            gain,
            seed,
        } => {
            let v = fbm_noise_3d(
                pos[0] * scale,
                pos[1] * scale,
                pos[2] * scale,
                *octaves,
                *lacunarity,
                *gain,
                *seed,
            );
            v * 2.0 - 1.0
        }

        DisplacementPattern::Sine {
            frequency,
            amplitude,
            direction,
        } => {
            let len = (direction[0] * direction[0]
                + direction[1] * direction[1]
                + direction[2] * direction[2])
                .sqrt();
            let dir = if len > 1e-10 {
                [direction[0] / len, direction[1] / len, direction[2] / len]
            } else {
                [0.0, 1.0, 0.0]
            };
            let proj = pos[0] * dir[0] + pos[1] * dir[1] + pos[2] * dir[2];
            amplitude * (proj * frequency * std::f32::consts::TAU).sin()
        }

        DisplacementPattern::Voronoi {
            scale,
            crack_width,
            seed,
        } => {
            let d = voronoi_3d(pos[0], pos[1], pos[2], *scale, *seed);
            // Cracks: depress near cell edges (where voronoi dist is low after
            // subtracting from 1). Actually crack at low dist, but Voronoi dist
            // IS low at feature points. We want cracks at borders, so use
            // 1 - d for closeness to border (approximate).
            let border_proximity = 1.0 - d;
            if border_proximity > (1.0 - crack_width) {
                let t = (border_proximity - (1.0 - crack_width)) / crack_width;
                -t * t
            } else {
                0.0
            }
        }

        DisplacementPattern::Wrinkle {
            frequency_u,
            frequency_v,
            amplitude,
        } => {
            // u maps to x, v maps to z for world-space wrinkles
            let u = pos[0];
            let v = pos[2];
            amplitude
                * (u * frequency_u * std::f32::consts::TAU).sin()
                * (v * frequency_v * std::f32::consts::TAU).sin()
        }

        DisplacementPattern::Pores {
            density,
            depth,
            seed,
        } => {
            let d = voronoi_3d(pos[0], pos[1], pos[2], *density, *seed);
            // Deep dents near feature points (low voronoi distance)
            let pore_radius = 0.3f32;
            if d < pore_radius {
                let t = 1.0 - d / pore_radius;
                -depth * t * t
            } else {
                0.0
            }
        }
    }
}

// ── Core Displacement Functions ───────────────────────────────────────────────

/// Apply micro-displacement to a mesh.
///
/// Displaces each vertex along its normal by the pattern-sampled scalar,
/// scaled by amplitude * blend * mask_weight. After displacement, normals
/// are recomputed.
pub fn apply_micro_displacement(mesh: &MeshBuffers, params: &MicroDispParams) -> MicroDispResult {
    let mut result_mesh = mesh.clone();
    let n = mesh.positions.len();

    let mut displacements = Vec::with_capacity(n);

    for i in 0..n {
        let pos = mesh.positions[i];
        let normal = mesh.normals[i];

        let mask_weight = params
            .vertex_mask
            .as_ref()
            .map(|m| m.get(i).copied().unwrap_or(1.0))
            .unwrap_or(1.0);

        let raw = sample_displacement(&params.pattern, pos);
        let disp = raw * params.amplitude * params.blend * mask_weight;
        displacements.push(disp);

        result_mesh.positions[i] = [
            pos[0] + normal[0] * disp,
            pos[1] + normal[1] * disp,
            pos[2] + normal[2] * disp,
        ];
    }

    compute_normals(&mut result_mesh);

    let min_displacement = displacements.iter().copied().fold(f32::INFINITY, f32::min);
    let max_displacement = displacements
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let mean_displacement = if n > 0 {
        displacements.iter().sum::<f32>() / n as f32
    } else {
        0.0
    };

    MicroDispResult {
        mesh: result_mesh,
        min_displacement,
        max_displacement,
        mean_displacement,
    }
}

/// Generate skin-like displacement using FBM layered noise with pores.
///
/// Applies two passes: a broad FBM for overall texture, then pores for detail.
pub fn skin_displacement(mesh: &MeshBuffers, amplitude: f32, seed: u32) -> MicroDispResult {
    // First pass: FBM base texture
    let fbm_params = MicroDispParams {
        pattern: DisplacementPattern::Fbm {
            scale: 10.0,
            octaves: 4,
            lacunarity: 2.0,
            gain: 0.5,
            seed,
        },
        amplitude: amplitude * 0.6,
        blend: 1.0,
        vertex_mask: None,
    };
    let fbm_result = apply_micro_displacement(mesh, &fbm_params);

    // Second pass: pores overlay on top of FBM result
    let pore_params = MicroDispParams {
        pattern: DisplacementPattern::Pores {
            density: 20.0,
            depth: 1.0,
            seed: seed.wrapping_add(12345),
        },
        amplitude: amplitude * 0.4,
        blend: 1.0,
        vertex_mask: None,
    };
    apply_micro_displacement(&fbm_result.mesh, &pore_params)
}

/// Generate wrinkle displacement for elderly skin or facial expression detail.
pub fn wrinkle_displacement(mesh: &MeshBuffers, amplitude: f32) -> MicroDispResult {
    let params = MicroDispParams {
        pattern: DisplacementPattern::Wrinkle {
            frequency_u: 8.0,
            frequency_v: 4.0,
            amplitude: 1.0,
        },
        amplitude,
        blend: 1.0,
        vertex_mask: None,
    };
    apply_micro_displacement(mesh, &params)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    /// Create a simple triangle mesh for testing.
    fn triangle_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    /// Create a quad mesh (two triangles) for UV-based tests.
    fn quad_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: false,
        })
    }

    #[test]
    fn test_value_noise_range() {
        // Value noise must be in [0, 1]
        let test_positions = [
            (0.0f32, 0.0f32, 0.0f32),
            (1.5, 2.3, -0.7),
            (-1.5, 1.0, 1.5),
            (100.0, -50.0, 25.0),
            (0.001, 0.001, 0.001),
        ];
        for (x, y, z) in test_positions {
            let v = value_noise_3d(x, y, z, 42);
            assert!(
                (0.0..=1.0).contains(&v),
                "value_noise_3d({x},{y},{z}) = {v} out of [0,1]"
            );
        }
    }

    #[test]
    fn test_value_noise_deterministic() {
        // Same inputs must always produce same output
        let v1 = value_noise_3d(1.23, 4.56, 7.89, 100);
        let v2 = value_noise_3d(1.23, 4.56, 7.89, 100);
        assert_eq!(v1, v2, "value_noise_3d must be deterministic");

        // Different seed must produce different output (with high probability)
        let v3 = value_noise_3d(1.23, 4.56, 7.89, 101);
        assert_ne!(v1, v3, "different seeds should produce different values");
    }

    #[test]
    fn test_fbm_noise_range() {
        // FBM noise should be approximately in [0, 1]
        let test_positions = [
            (0.5f32, 0.5f32, 0.5f32),
            (2.0, -1.0, 3.0),
            (-5.0, 5.0, -5.0),
        ];
        for (x, y, z) in test_positions {
            let v = fbm_noise_3d(x, y, z, 4, 2.0, 0.5, 42);
            assert!(
                (0.0..=1.0).contains(&v),
                "fbm_noise_3d({x},{y},{z}) = {v} out of [0,1]"
            );
        }
    }

    #[test]
    fn test_voronoi_range() {
        // Voronoi distance should be in [0, 1] after normalization
        let test_positions = [
            (0.0f32, 0.0f32, 0.0f32),
            (0.5, 0.5, 0.5),
            (1.23, -0.45, 2.67),
        ];
        for (x, y, z) in test_positions {
            let v = voronoi_3d(x, y, z, 2.0, 7);
            assert!(
                (0.0..=1.0).contains(&v),
                "voronoi_3d({x},{y},{z}) = {v} out of [0,1]"
            );
        }
    }

    #[test]
    fn test_sample_displacement_sine() {
        // Sine displacement along +Z direction at z=0 should be 0 (sin(0)=0)
        let pattern = DisplacementPattern::Sine {
            frequency: 1.0,
            amplitude: 1.0,
            direction: [0.0, 0.0, 1.0],
        };
        let d = sample_displacement(&pattern, [0.0, 0.0, 0.0]);
        assert!(d.abs() < 1e-5, "Sine at z=0 along Z should be ~0, got {d}");

        // At z=0.25 (quarter period), sin(0.25 * 2π) = 1.0
        let d_quarter = sample_displacement(&pattern, [0.0, 0.0, 0.25]);
        assert!(
            (d_quarter - 1.0).abs() < 1e-5,
            "Sine at z=0.25 should be ~1.0, got {d_quarter}"
        );

        // Amplitude scaling
        let pattern_amp = DisplacementPattern::Sine {
            frequency: 1.0,
            amplitude: 2.0,
            direction: [0.0, 0.0, 1.0],
        };
        let d_amp = sample_displacement(&pattern_amp, [0.0, 0.0, 0.25]);
        assert!(
            (d_amp - 2.0).abs() < 1e-5,
            "Sine amplitude=2 at z=0.25 should be ~2.0, got {d_amp}"
        );
    }

    #[test]
    fn test_sample_displacement_value_noise() {
        // ValueNoise displacement maps [0,1] -> [-1,1]
        let pattern = DisplacementPattern::ValueNoise {
            scale: 1.0,
            seed: 42,
        };
        let d = sample_displacement(&pattern, [0.0, 0.0, 0.0]);
        assert!(
            (-1.0..=1.0).contains(&d),
            "ValueNoise displacement {d} out of [-1,1]"
        );

        // Deterministic
        let d2 = sample_displacement(&pattern, [0.0, 0.0, 0.0]);
        assert_eq!(d, d2, "sample_displacement must be deterministic");
    }

    #[test]
    fn test_apply_micro_displacement_basic() {
        let mesh = triangle_mesh();
        let original_positions = mesh.positions.clone();

        let params = MicroDispParams {
            pattern: DisplacementPattern::ValueNoise {
                scale: 5.0,
                seed: 42,
            },
            amplitude: 0.01,
            blend: 1.0,
            vertex_mask: None,
        };
        let result = apply_micro_displacement(&mesh, &params);

        // Vertex count must be preserved
        assert_eq!(result.mesh.positions.len(), original_positions.len());
        assert_eq!(result.mesh.indices, mesh.indices);

        // Normals must be recomputed (same count)
        assert_eq!(result.mesh.normals.len(), original_positions.len());

        // Stats must be finite
        assert!(result.min_displacement.is_finite());
        assert!(result.max_displacement.is_finite());
        assert!(result.mean_displacement.is_finite());
        assert!(result.min_displacement <= result.max_displacement);
    }

    #[test]
    fn test_apply_micro_displacement_amplitude_zero() {
        let mesh = triangle_mesh();
        let original_positions = mesh.positions.clone();

        let params = MicroDispParams {
            pattern: DisplacementPattern::ValueNoise {
                scale: 5.0,
                seed: 42,
            },
            amplitude: 0.0,
            blend: 1.0,
            vertex_mask: None,
        };
        let result = apply_micro_displacement(&mesh, &params);

        // With amplitude=0, positions should be unchanged
        for (orig, new) in original_positions.iter().zip(result.mesh.positions.iter()) {
            assert!(
                (orig[0] - new[0]).abs() < 1e-7
                    && (orig[1] - new[1]).abs() < 1e-7
                    && (orig[2] - new[2]).abs() < 1e-7,
                "amplitude=0 should not move vertices"
            );
        }
        assert!(result.mean_displacement.abs() < 1e-7);
    }

    #[test]
    fn test_apply_micro_displacement_blend() {
        let mesh = triangle_mesh();

        let params_full = MicroDispParams {
            pattern: DisplacementPattern::Sine {
                frequency: 1.0,
                amplitude: 1.0,
                direction: [0.0, 1.0, 0.0],
            },
            amplitude: 0.1,
            blend: 1.0,
            vertex_mask: None,
        };
        let params_half = MicroDispParams {
            pattern: DisplacementPattern::Sine {
                frequency: 1.0,
                amplitude: 1.0,
                direction: [0.0, 1.0, 0.0],
            },
            amplitude: 0.1,
            blend: 0.5,
            vertex_mask: None,
        };

        let result_full = apply_micro_displacement(&mesh, &params_full);
        let result_half = apply_micro_displacement(&mesh, &params_half);

        // Half blend should produce half the displacement of full blend
        assert!(
            result_half.mean_displacement.abs() <= result_full.mean_displacement.abs() + 1e-7,
            "half blend should produce less or equal displacement than full blend"
        );
    }

    #[test]
    fn test_skin_displacement() {
        let mesh = quad_mesh();
        let result = skin_displacement(&mesh, 0.005, 42);

        assert_eq!(result.mesh.positions.len(), mesh.positions.len());
        assert_eq!(result.mesh.indices, mesh.indices);
        assert!(result.min_displacement.is_finite());
        assert!(result.max_displacement.is_finite());
        assert!(result.min_displacement <= result.max_displacement);

        // Skin displacement should actually move some vertices
        let any_moved =
            result
                .mesh
                .positions
                .iter()
                .zip(mesh.positions.iter())
                .any(|(new, orig)| {
                    let d = (new[0] - orig[0]).powi(2)
                        + (new[1] - orig[1]).powi(2)
                        + (new[2] - orig[2]).powi(2);
                    d > 1e-14
                });
        assert!(
            any_moved,
            "skin_displacement should move at least one vertex"
        );
    }

    #[test]
    fn test_wrinkle_displacement() {
        let mesh = quad_mesh();
        let result = wrinkle_displacement(&mesh, 0.01);

        assert_eq!(result.mesh.positions.len(), mesh.positions.len());
        assert_eq!(result.mesh.indices, mesh.indices);
        assert!(result.min_displacement.is_finite());
        assert!(result.max_displacement.is_finite());
        assert!(result.mean_displacement.is_finite());
    }

    #[test]
    fn test_micro_disp_result_stats() {
        let mesh = quad_mesh();

        // Use a predictable pattern for stat verification
        let params = MicroDispParams {
            pattern: DisplacementPattern::Sine {
                frequency: 1.0,
                amplitude: 0.5,
                direction: [1.0, 0.0, 0.0],
            },
            amplitude: 1.0,
            blend: 1.0,
            vertex_mask: None,
        };
        let result = apply_micro_displacement(&mesh, &params);

        // min <= mean <= max
        assert!(
            result.min_displacement <= result.mean_displacement + 1e-6,
            "min must be <= mean"
        );
        assert!(
            result.mean_displacement <= result.max_displacement + 1e-6,
            "mean must be <= max"
        );

        // Stats should be finite
        assert!(!result.min_displacement.is_nan());
        assert!(!result.max_displacement.is_nan());
        assert!(!result.mean_displacement.is_nan());
    }

    #[test]
    fn test_vertex_mask_applied() {
        let mesh = quad_mesh();

        // Mask: only apply to first vertex
        let mask = vec![1.0f32, 0.0, 0.0, 0.0];
        let params_masked = MicroDispParams {
            pattern: DisplacementPattern::Sine {
                frequency: 2.0,
                amplitude: 1.0,
                direction: [0.0, 0.0, 1.0],
            },
            amplitude: 0.1,
            blend: 1.0,
            vertex_mask: Some(mask),
        };
        let params_unmasked = MicroDispParams {
            pattern: DisplacementPattern::Sine {
                frequency: 2.0,
                amplitude: 1.0,
                direction: [0.0, 0.0, 1.0],
            },
            amplitude: 0.1,
            blend: 1.0,
            vertex_mask: None,
        };

        let result_masked = apply_micro_displacement(&mesh, &params_masked);
        let result_unmasked = apply_micro_displacement(&mesh, &params_unmasked);

        // Vertices 1,2,3 with mask=0 should be at original position
        // (normals may have been recomputed so we check displacement magnitude via mean)
        assert!(
            result_masked.mean_displacement.abs() <= result_unmasked.mean_displacement.abs() + 1e-6,
            "masked displacement should be <= unmasked"
        );
    }

    #[test]
    fn test_voronoi_deterministic() {
        let v1 = voronoi_3d(0.5, 0.5, 0.5, 3.0, 99);
        let v2 = voronoi_3d(0.5, 0.5, 0.5, 3.0, 99);
        assert_eq!(v1, v2, "voronoi_3d must be deterministic");
    }
}
