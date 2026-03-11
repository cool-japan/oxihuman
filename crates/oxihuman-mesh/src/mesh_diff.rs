// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use crate::heat_map::{scalars_to_colors_range, ColorRamp};
use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

/// Per-vertex displacement between two meshes.
pub struct DisplacementField {
    /// Displacement vectors: new_pos - old_pos for each vertex.
    pub deltas: Vec<[f32; 3]>,
    /// Scalar magnitudes of each displacement.
    pub magnitudes: Vec<f32>,
    /// Max displacement magnitude.
    pub max_magnitude: f32,
    /// Mean displacement magnitude.
    pub mean_magnitude: f32,
    /// RMS displacement.
    pub rms: f32,
}

impl DisplacementField {
    /// Compute a displacement field between `base` and `deformed` meshes.
    ///
    /// Returns an error if the vertex counts differ.
    pub fn compute(base: &MeshBuffers, deformed: &MeshBuffers) -> anyhow::Result<Self> {
        compute_displacement(base, deformed)
    }

    /// Number of vertices in this displacement field.
    pub fn vertex_count(&self) -> usize {
        self.deltas.len()
    }

    /// True when every displacement magnitude is effectively zero.
    pub fn is_zero(&self) -> bool {
        self.max_magnitude < f32::EPSILON
    }

    /// Return indices of the top `n` vertices with the largest displacement,
    /// sorted descending by magnitude.
    pub fn top_displaced(&self, n: usize) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.magnitudes.len()).collect();
        indices.sort_unstable_by(|&a, &b| {
            self.magnitudes[b]
                .partial_cmp(&self.magnitudes[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        indices.truncate(n);
        indices
    }

    /// Return a new field with all deltas scaled by `factor`.
    pub fn scale(&self, factor: f32) -> Self {
        let deltas: Vec<[f32; 3]> = self
            .deltas
            .iter()
            .map(|d| [d[0] * factor, d[1] * factor, d[2] * factor])
            .collect();
        let magnitudes: Vec<f32> = deltas
            .iter()
            .map(|d| (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
            .collect();
        let max_magnitude = magnitudes.iter().cloned().fold(0.0f32, f32::max);
        let mean_magnitude = if magnitudes.is_empty() {
            0.0
        } else {
            magnitudes.iter().sum::<f32>() / magnitudes.len() as f32
        };
        let rms = if magnitudes.is_empty() {
            0.0
        } else {
            let mean_sq = magnitudes.iter().map(|m| m * m).sum::<f32>() / magnitudes.len() as f32;
            mean_sq.sqrt()
        };
        DisplacementField {
            deltas,
            magnitudes,
            max_magnitude,
            mean_magnitude,
            rms,
        }
    }

    /// Apply this displacement field to `mesh`, returning a new mesh with
    /// updated positions (and recomputed normals).
    pub fn apply_to(&self, mesh: &MeshBuffers) -> MeshBuffers {
        let count = mesh.positions.len().min(self.deltas.len());
        let mut new_positions = mesh.positions.clone();
        for (p, d) in new_positions.iter_mut().zip(self.deltas.iter()).take(count) {
            p[0] += d[0];
            p[1] += d[1];
            p[2] += d[2];
        }
        let mut result = MeshBuffers {
            positions: new_positions,
            normals: mesh.normals.clone(),
            tangents: mesh.tangents.clone(),
            uvs: mesh.uvs.clone(),
            indices: mesh.indices.clone(),
            colors: mesh.colors.clone(),
            has_suit: mesh.has_suit,
        };
        compute_normals(&mut result);
        result
    }

    /// Return a new field where deltas with magnitude below `min_magnitude`
    /// are zeroed out.
    pub fn threshold(&self, min_magnitude: f32) -> Self {
        let deltas: Vec<[f32; 3]> = self
            .deltas
            .iter()
            .zip(self.magnitudes.iter())
            .map(|(d, &mag)| {
                if mag < min_magnitude {
                    [0.0, 0.0, 0.0]
                } else {
                    *d
                }
            })
            .collect();
        let magnitudes: Vec<f32> = deltas
            .iter()
            .map(|d| (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
            .collect();
        let max_magnitude = magnitudes.iter().cloned().fold(0.0f32, f32::max);
        let mean_magnitude = if magnitudes.is_empty() {
            0.0
        } else {
            magnitudes.iter().sum::<f32>() / magnitudes.len() as f32
        };
        let rms = if magnitudes.is_empty() {
            0.0
        } else {
            let mean_sq = magnitudes.iter().map(|m| m * m).sum::<f32>() / magnitudes.len() as f32;
            mean_sq.sqrt()
        };
        DisplacementField {
            deltas,
            magnitudes,
            max_magnitude,
            mean_magnitude,
            rms,
        }
    }
}

/// Statistics comparing two meshes with the same topology.
pub struct MeshDiffStats {
    pub vertex_count: usize,
    pub max_displacement: f32,
    pub mean_displacement: f32,
    pub rms_displacement: f32,
    /// Approximated Hausdorff distance: max of symmetric mean displacements.
    pub hausdorff_approx: f32,
    /// Percentage of vertices with displacement greater than `epsilon`.
    pub percent_changed: f32,
    pub epsilon: f32,
}

/// Compute a per-vertex displacement field between two same-topology meshes.
///
/// Returns an error when vertex counts differ.
pub fn compute_displacement(
    base: &MeshBuffers,
    deformed: &MeshBuffers,
) -> anyhow::Result<DisplacementField> {
    if base.positions.len() != deformed.positions.len() {
        anyhow::bail!(
            "mesh_diff: vertex count mismatch ({} vs {})",
            base.positions.len(),
            deformed.positions.len()
        );
    }
    let deltas: Vec<[f32; 3]> = base
        .positions
        .iter()
        .zip(deformed.positions.iter())
        .map(|(b, d)| [d[0] - b[0], d[1] - b[1], d[2] - b[2]])
        .collect();
    let magnitudes: Vec<f32> = deltas
        .iter()
        .map(|d| (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
        .collect();
    let max_magnitude = magnitudes.iter().cloned().fold(0.0f32, f32::max);
    let mean_magnitude = if magnitudes.is_empty() {
        0.0
    } else {
        magnitudes.iter().sum::<f32>() / magnitudes.len() as f32
    };
    let rms = if magnitudes.is_empty() {
        0.0
    } else {
        let mean_sq = magnitudes.iter().map(|m| m * m).sum::<f32>() / magnitudes.len() as f32;
        mean_sq.sqrt()
    };
    Ok(DisplacementField {
        deltas,
        magnitudes,
        max_magnitude,
        mean_magnitude,
        rms,
    })
}

/// Compute statistics comparing two same-topology meshes.
///
/// `epsilon` is the threshold below which a vertex is considered unchanged.
pub fn mesh_diff_stats(
    base: &MeshBuffers,
    deformed: &MeshBuffers,
    epsilon: f32,
) -> anyhow::Result<MeshDiffStats> {
    let field_ab = compute_displacement(base, deformed)?;
    // For Hausdorff approximation we use the symmetric mean:
    // max(mean A→B, mean B→A).  Since we compare same-topology meshes the
    // B→A magnitudes are identical to A→B (same per-vertex distances), so
    // hausdorff_approx == mean_displacement.
    let hausdorff_approx = field_ab.mean_magnitude;

    let n = field_ab.magnitudes.len();
    let changed = field_ab.magnitudes.iter().filter(|&&m| m > epsilon).count();
    let percent_changed = if n == 0 {
        0.0
    } else {
        changed as f32 / n as f32 * 100.0
    };

    Ok(MeshDiffStats {
        vertex_count: n,
        max_displacement: field_ab.max_magnitude,
        mean_displacement: field_ab.mean_magnitude,
        rms_displacement: field_ab.rms,
        hausdorff_approx,
        percent_changed,
        epsilon,
    })
}

/// Return `true` when both meshes have the same vertex count and every
/// corresponding pair of positions is within `epsilon` of each other.
pub fn meshes_approx_equal(a: &MeshBuffers, b: &MeshBuffers, epsilon: f32) -> bool {
    if a.positions.len() != b.positions.len() {
        return false;
    }
    a.positions.iter().zip(b.positions.iter()).all(|(pa, pb)| {
        let dx = pa[0] - pb[0];
        let dy = pa[1] - pb[1];
        let dz = pa[2] - pb[2];
        (dx * dx + dy * dy + dz * dz).sqrt() <= epsilon
    })
}

/// Linearly blend all vertex positions between `base` (t = 0) and `target` (t = 1).
///
/// Returns an error when vertex counts differ.
pub fn blend_meshes(
    base: &MeshBuffers,
    target: &MeshBuffers,
    t: f32,
) -> anyhow::Result<MeshBuffers> {
    if base.positions.len() != target.positions.len() {
        anyhow::bail!(
            "blend_meshes: vertex count mismatch ({} vs {})",
            base.positions.len(),
            target.positions.len()
        );
    }
    let t = t.clamp(0.0, 1.0);
    let positions: Vec<[f32; 3]> = base
        .positions
        .iter()
        .zip(target.positions.iter())
        .map(|(b, tgt)| {
            [
                b[0] + (tgt[0] - b[0]) * t,
                b[1] + (tgt[1] - b[1]) * t,
                b[2] + (tgt[2] - b[2]) * t,
            ]
        })
        .collect();
    let mut result = MeshBuffers {
        positions,
        normals: base.normals.clone(),
        tangents: base.tangents.clone(),
        uvs: base.uvs.clone(),
        indices: base.indices.clone(),
        colors: base.colors.clone(),
        has_suit: base.has_suit,
    };
    compute_normals(&mut result);
    Ok(result)
}

/// Interpolate a sequence of mesh frames at normalised time `t` (0..1 across
/// the whole sequence).
///
/// Returns an error when `frames` is empty.
pub fn interpolate_mesh_sequence(frames: &[MeshBuffers], t: f32) -> anyhow::Result<MeshBuffers> {
    if frames.is_empty() {
        anyhow::bail!("interpolate_mesh_sequence: frames slice is empty");
    }
    if frames.len() == 1 {
        return Ok(frames[0].clone());
    }
    let t = t.clamp(0.0, 1.0);
    let last = (frames.len() - 1) as f32;
    let pos = t * last;
    let lo = (pos.floor() as usize).min(frames.len() - 2);
    let hi = lo + 1;
    let frac = pos - lo as f32;
    blend_meshes(&frames[lo], &frames[hi], frac)
}

/// Create a copy of `base` whose per-vertex colors encode the displacement
/// magnitude via the `Rainbow` heat-map ramp.
pub fn displacement_to_heat_mesh(base: &MeshBuffers, field: &DisplacementField) -> MeshBuffers {
    let ramp = ColorRamp::Rainbow;
    let max_mag = field.max_magnitude;
    let min_mag = 0.0f32;
    let rgb_colors = scalars_to_colors_range(&field.magnitudes, &ramp, min_mag, max_mag);
    let rgba_colors: Vec<[f32; 4]> = rgb_colors
        .into_iter()
        .map(|c| [c[0], c[1], c[2], 1.0])
        .collect();
    MeshBuffers {
        positions: base.positions.clone(),
        normals: base.normals.clone(),
        tangents: base.tangents.clone(),
        uvs: base.uvs.clone(),
        indices: base.indices.clone(),
        colors: Some(rgba_colors),
        has_suit: base.has_suit,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn make_mesh(positions: Vec<[f32; 3]>) -> MeshBuffers {
        let n = positions.len();
        let normals = vec![[0.0f32, 0.0, 1.0]; n];
        let uvs = vec![[0.0f32, 0.0]; n];
        // Simple triangle strip of fans; just need valid indices for normals.
        let indices: Vec<u32> = if n >= 3 {
            (0..((n as u32) - 2))
                .flat_map(|i| [0, i + 1, i + 2])
                .collect()
        } else {
            vec![]
        };
        MeshBuffers::from_morph(MB {
            positions,
            normals,
            uvs,
            indices,
            has_suit: false,
        })
    }

    fn base_triangle() -> MeshBuffers {
        make_mesh(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
    }

    fn deformed_triangle() -> MeshBuffers {
        make_mesh(vec![
            [0.0, 0.0, 1.0], // displaced by 1 in Z
            [1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
        ])
    }

    // ── DisplacementField ────────────────────────────────────────────────────

    #[test]
    fn test_displacement_field_zero() {
        let base = base_triangle();
        let field = DisplacementField::compute(&base, &base).unwrap();
        assert_eq!(field.vertex_count(), 3);
        assert!(field.is_zero());
        assert!(field.max_magnitude < 1e-6);
    }

    #[test]
    fn test_displacement_field_nonzero() {
        let base = base_triangle();
        let deformed = deformed_triangle();
        let field = DisplacementField::compute(&base, &deformed).unwrap();
        assert!(!field.is_zero());
        assert!((field.max_magnitude - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_displacement_magnitudes() {
        let base = base_triangle();
        let deformed = deformed_triangle();
        let field = DisplacementField::compute(&base, &deformed).unwrap();
        assert_eq!(field.magnitudes.len(), 3);
        for &m in &field.magnitudes {
            assert!(
                (m - 1.0).abs() < 1e-5,
                "each magnitude should be 1.0, got {m}"
            );
        }
        assert!((field.mean_magnitude - 1.0).abs() < 1e-5);
        assert!((field.rms - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_displacement_top_displaced() {
        let base = make_mesh(vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        ]);
        let deformed = make_mesh(vec![
            [0.0, 0.0, 3.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 4.0],
            [0.0, 0.0, 2.0],
        ]);
        let field = DisplacementField::compute(&base, &deformed).unwrap();
        let top2 = field.top_displaced(2);
        assert_eq!(top2.len(), 2);
        // Largest is index 2 (mag=4), second is index 0 (mag=3)
        assert_eq!(top2[0], 2);
        assert_eq!(top2[1], 0);
    }

    #[test]
    fn test_displacement_scale() {
        let base = base_triangle();
        let deformed = deformed_triangle();
        let field = DisplacementField::compute(&base, &deformed).unwrap();
        let scaled = field.scale(2.0);
        assert!((scaled.max_magnitude - 2.0).abs() < 1e-5);
        assert!((scaled.mean_magnitude - 2.0).abs() < 1e-5);
        for d in &scaled.deltas {
            assert!((d[2] - 2.0).abs() < 1e-5);
        }
    }

    #[test]
    fn test_displacement_threshold() {
        let base = make_mesh(vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]);
        let deformed = make_mesh(vec![
            [0.0, 0.0, 0.5], // mag 0.5 – below threshold 0.8
            [0.0, 0.0, 1.0], // mag 1.0 – above threshold
            [0.0, 0.0, 0.3], // mag 0.3 – below threshold
        ]);
        let field = DisplacementField::compute(&base, &deformed).unwrap();
        let thresholded = field.threshold(0.8);
        // Only index 1 should survive
        assert!(thresholded.magnitudes[0] < 1e-6);
        assert!(thresholded.magnitudes[2] < 1e-6);
        assert!((thresholded.magnitudes[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_displacement_apply_to() {
        let base = base_triangle();
        let deformed = deformed_triangle();
        let field = DisplacementField::compute(&base, &deformed).unwrap();
        let result = field.apply_to(&base);
        // Positions should match deformed
        for (r, d) in result.positions.iter().zip(deformed.positions.iter()) {
            assert!((r[0] - d[0]).abs() < 1e-5);
            assert!((r[1] - d[1]).abs() < 1e-5);
            assert!((r[2] - d[2]).abs() < 1e-5);
        }
    }

    // ── meshes_approx_equal ──────────────────────────────────────────────────

    #[test]
    fn test_meshes_approx_equal_true() {
        let a = base_triangle();
        let b = base_triangle();
        assert!(meshes_approx_equal(&a, &b, 1e-6));
    }

    #[test]
    fn test_meshes_approx_equal_false() {
        let a = base_triangle();
        let b = deformed_triangle();
        assert!(!meshes_approx_equal(&a, &b, 1e-6));
        // But with a large enough epsilon it should pass
        assert!(meshes_approx_equal(&a, &b, 2.0));
    }

    // ── blend_meshes ─────────────────────────────────────────────────────────

    #[test]
    fn test_blend_meshes_zero() {
        let base = base_triangle();
        let target = deformed_triangle();
        let blended = blend_meshes(&base, &target, 0.0).unwrap();
        assert!(meshes_approx_equal(&blended, &base, 1e-5));
    }

    #[test]
    fn test_blend_meshes_one() {
        let base = base_triangle();
        let target = deformed_triangle();
        let blended = blend_meshes(&base, &target, 1.0).unwrap();
        assert!(meshes_approx_equal(&blended, &target, 1e-5));
    }

    #[test]
    fn test_blend_meshes_half() {
        let base = base_triangle();
        let target = deformed_triangle();
        let blended = blend_meshes(&base, &target, 0.5).unwrap();
        // Every vertex should be at Z = 0.5
        for p in &blended.positions {
            assert!((p[2] - 0.5).abs() < 1e-5, "expected Z=0.5, got {}", p[2]);
        }
    }

    // ── mesh_diff_stats ──────────────────────────────────────────────────────

    #[test]
    fn test_mesh_diff_stats() {
        let base = base_triangle();
        let deformed = deformed_triangle();
        let stats = mesh_diff_stats(&base, &deformed, 0.5).unwrap();
        assert_eq!(stats.vertex_count, 3);
        assert!((stats.max_displacement - 1.0).abs() < 1e-5);
        assert!((stats.mean_displacement - 1.0).abs() < 1e-5);
        assert!((stats.rms_displacement - 1.0).abs() < 1e-5);
        // All 3 vertices displaced by 1.0 > epsilon 0.5
        assert!((stats.percent_changed - 100.0).abs() < 1e-3);
    }

    // ── interpolate_mesh_sequence ────────────────────────────────────────────

    #[test]
    fn test_interpolate_sequence() {
        let f0 = base_triangle();
        let f1 = deformed_triangle();
        let frames = vec![f0.clone(), f1.clone()];

        // t=0 → first frame
        let r0 = interpolate_mesh_sequence(&frames, 0.0).unwrap();
        assert!(meshes_approx_equal(&r0, &f0, 1e-5));

        // t=1 → last frame
        let r1 = interpolate_mesh_sequence(&frames, 1.0).unwrap();
        assert!(meshes_approx_equal(&r1, &f1, 1e-5));

        // t=0.5 → midpoint (Z = 0.5 for all verts)
        let r_half = interpolate_mesh_sequence(&frames, 0.5).unwrap();
        for p in &r_half.positions {
            assert!((p[2] - 0.5).abs() < 1e-5);
        }

        // Empty slice → error
        let empty: Vec<MeshBuffers> = vec![];
        assert!(interpolate_mesh_sequence(&empty, 0.5).is_err());
    }
}
