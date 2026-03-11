// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Target validation, inspection, merging, and mirroring utilities.
//!
//! This module provides quality-assurance and manipulation tools for morph
//! target delta arrays — checking symmetry, bounding boxes, displacement
//! magnitudes, weighted merging, and axis mirroring.

use serde::{Deserialize, Serialize};

use crate::delta_painter::MirrorAxis;

// ── Data types ───────────────────────────────────────────────────────────────

/// Summary information about a morph target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    /// Name of the target (empty if not set).
    pub name: String,
    /// Total number of vertices in the delta array.
    pub vertex_count: usize,
    /// Number of vertices with non-zero displacement.
    pub affected_count: usize,
    /// Maximum displacement magnitude among all vertices.
    pub max_displacement: f64,
    /// Mean displacement magnitude (over all vertices, including zero).
    pub average_displacement: f64,
    /// Fraction of zero-displacement vertices: `1 - affected / total`.
    pub sparsity: f64,
}

/// Report from a symmetry check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymmetryReport {
    /// Whether the target is considered symmetric within tolerance.
    pub symmetric: bool,
    /// Maximum asymmetry measured across any vertex pair.
    pub max_asymmetry: f64,
    /// Indices of vertices exceeding the symmetry tolerance.
    pub asymmetric_vertices: Vec<usize>,
}

/// A single validation warning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Classification.
    pub kind: WarningKind,
    /// Human-readable description.
    pub message: String,
}

/// Categories of validation warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningKind {
    /// One or more vertices have displacement exceeding a safe limit.
    ExcessiveDisplacement,
    /// Deltas risk causing mesh self-intersection (heuristic).
    SelfIntersectionRisk,
    /// The target is significantly asymmetric.
    Asymmetry,
    /// The target has no effective deltas at all.
    EmptyTarget,
}

// ── Helpers ──────────────────────────────────────────────────────────────────

#[inline]
fn magnitude(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn is_zero(v: &[f64; 3], threshold: f64) -> bool {
    magnitude(*v) < threshold
}

/// Default threshold for considering a delta as "zero".
const DEFAULT_ZERO_THRESHOLD: f64 = 1e-10;

// ── TargetValidator ──────────────────────────────────────────────────────────

/// Validates morph target data for common issues.
pub struct TargetValidator;

impl TargetValidator {
    /// Run a suite of validation checks on `deltas`.
    ///
    /// Returns a list of warnings (empty = no issues found).
    pub fn validate(
        deltas: &[[f64; 3]],
        vertex_count: usize,
    ) -> anyhow::Result<Vec<ValidationWarning>> {
        if deltas.len() != vertex_count {
            anyhow::bail!(
                "deltas length {} != vertex_count {}",
                deltas.len(),
                vertex_count
            );
        }

        let mut warnings = Vec::new();

        // Check for empty target
        let affected = deltas
            .iter()
            .filter(|d| !is_zero(d, DEFAULT_ZERO_THRESHOLD))
            .count();
        if affected == 0 {
            warnings.push(ValidationWarning {
                kind: WarningKind::EmptyTarget,
                message: "Target has no non-zero deltas".to_owned(),
            });
        }

        // Check for excessive displacement (heuristic: > 1.0 world unit)
        let max_disp = deltas.iter().map(|d| magnitude(*d)).fold(0.0_f64, f64::max);
        if max_disp > 1.0 {
            warnings.push(ValidationWarning {
                kind: WarningKind::ExcessiveDisplacement,
                message: format!(
                    "Maximum displacement {:.4} exceeds safe limit 1.0",
                    max_disp
                ),
            });
        }

        // Self-intersection risk heuristic: if any delta is > 0.5 and there
        // are neighbouring deltas pointing in opposite directions.
        // Simplified: check if the delta range in any axis exceeds 1.0
        let mut axis_min = [f64::MAX; 3];
        let mut axis_max = [f64::MIN; 3];
        for d in deltas {
            for i in 0..3 {
                if d[i] < axis_min[i] {
                    axis_min[i] = d[i];
                }
                if d[i] > axis_max[i] {
                    axis_max[i] = d[i];
                }
            }
        }
        for i in 0..3 {
            let range = axis_max[i] - axis_min[i];
            if range > 1.0 {
                let axis_name = match i {
                    0 => "X",
                    1 => "Y",
                    _ => "Z",
                };
                warnings.push(ValidationWarning {
                    kind: WarningKind::SelfIntersectionRisk,
                    message: format!(
                        "Delta range on {} axis is {:.4}, which may cause self-intersection",
                        axis_name, range
                    ),
                });
            }
        }

        Ok(warnings)
    }

    /// Check symmetry of deltas relative to vertex positions across the X axis.
    ///
    /// For each vertex on the positive X side, finds the closest vertex on the
    /// negative side and compares their deltas (with X component negated).
    pub fn check_symmetry(
        deltas: &[[f64; 3]],
        positions: &[[f64; 3]],
        tolerance: f64,
    ) -> SymmetryReport {
        let n = deltas.len().min(positions.len());
        let tol = tolerance.max(1e-12);
        let mut max_asym = 0.0_f64;
        let mut asym_verts = Vec::new();

        for i in 0..n {
            let pos = positions[i];
            // Only check vertices on the positive X side
            if pos[0] < 0.0 {
                continue;
            }

            // Find mirror vertex
            let mirror_pos = [-pos[0], pos[1], pos[2]];
            let mut best_j: Option<usize> = None;
            let mut best_dsq = f64::MAX;
            for (j, jpos) in positions[..n].iter().enumerate() {
                let dp0 = jpos[0] - mirror_pos[0];
                let dp1 = jpos[1] - mirror_pos[1];
                let dp2 = jpos[2] - mirror_pos[2];
                let dsq = dp0 * dp0 + dp1 * dp1 + dp2 * dp2;
                if dsq < best_dsq {
                    best_dsq = dsq;
                    best_j = Some(j);
                }
            }

            if let Some(j) = best_j {
                if best_dsq.sqrt() > tol * 10.0 {
                    // No mirror vertex found within reasonable range — skip
                    continue;
                }
                // Expected mirror delta: negate X component
                let expected = [-deltas[i][0], deltas[i][1], deltas[i][2]];
                let diff = [
                    deltas[j][0] - expected[0],
                    deltas[j][1] - expected[1],
                    deltas[j][2] - expected[2],
                ];
                let asym = magnitude(diff);
                if asym > max_asym {
                    max_asym = asym;
                }
                if asym > tol {
                    asym_verts.push(i);
                }
            }
        }

        SymmetryReport {
            symmetric: asym_verts.is_empty(),
            max_asymmetry: max_asym,
            asymmetric_vertices: asym_verts,
        }
    }

    /// Return indices of vertices whose displacement exceeds `max_displacement`.
    pub fn check_magnitude(deltas: &[[f64; 3]], max_displacement: f64) -> Vec<usize> {
        deltas
            .iter()
            .enumerate()
            .filter_map(|(i, d)| {
                if magnitude(*d) > max_displacement {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }
}

// ── TargetInspector ──────────────────────────────────────────────────────────

/// Read-only inspection utilities for morph target delta arrays.
pub struct TargetInspector;

impl TargetInspector {
    /// Produce a [`TargetInfo`] summary of the given deltas.
    pub fn inspect(deltas: &[[f64; 3]]) -> TargetInfo {
        Self::inspect_named(deltas, "")
    }

    /// Produce a [`TargetInfo`] summary with a name.
    pub fn inspect_named(deltas: &[[f64; 3]], name: &str) -> TargetInfo {
        let vertex_count = deltas.len();
        let mut affected_count = 0usize;
        let mut max_disp = 0.0_f64;
        let mut sum_disp = 0.0_f64;

        for d in deltas {
            let m = magnitude(*d);
            if m > DEFAULT_ZERO_THRESHOLD {
                affected_count += 1;
            }
            if m > max_disp {
                max_disp = m;
            }
            sum_disp += m;
        }

        let average_displacement = if vertex_count > 0 {
            sum_disp / vertex_count as f64
        } else {
            0.0
        };

        let sparsity = if vertex_count > 0 {
            1.0 - (affected_count as f64 / vertex_count as f64)
        } else {
            1.0
        };

        TargetInfo {
            name: name.to_owned(),
            vertex_count,
            affected_count,
            max_displacement: max_disp,
            average_displacement,
            sparsity,
        }
    }

    /// Return the indices of vertices displaced above `threshold`.
    pub fn affected_vertices(deltas: &[[f64; 3]], threshold: f64) -> Vec<usize> {
        deltas
            .iter()
            .enumerate()
            .filter_map(|(i, d)| {
                if magnitude(*d) > threshold {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Axis-aligned bounding box of the delta field: `(min, max)`.
    pub fn bounding_box(deltas: &[[f64; 3]]) -> ([f64; 3], [f64; 3]) {
        if deltas.is_empty() {
            return ([0.0; 3], [0.0; 3]);
        }
        let mut mn = [f64::MAX; 3];
        let mut mx = [f64::MIN; 3];
        for d in deltas {
            for i in 0..3 {
                if d[i] < mn[i] {
                    mn[i] = d[i];
                }
                if d[i] > mx[i] {
                    mx[i] = d[i];
                }
            }
        }
        (mn, mx)
    }

    /// Maximum displacement magnitude in the delta array.
    pub fn max_displacement(deltas: &[[f64; 3]]) -> f64 {
        deltas.iter().map(|d| magnitude(*d)).fold(0.0_f64, f64::max)
    }

    /// Root-mean-square displacement.
    pub fn rms_displacement(deltas: &[[f64; 3]]) -> f64 {
        if deltas.is_empty() {
            return 0.0;
        }
        let sum_sq: f64 = deltas
            .iter()
            .map(|d| d[0] * d[0] + d[1] * d[1] + d[2] * d[2])
            .sum();
        (sum_sq / deltas.len() as f64).sqrt()
    }
}

// ── Free functions: merge and mirror ─────────────────────────────────────────

/// Weighted merge of multiple targets into a single delta array.
///
/// Each entry is `(name, deltas, weight)`. All delta arrays must have the same
/// length. The result is the weighted sum of all inputs.
pub fn merge_targets(targets: &[(&str, &[[f64; 3]], f64)]) -> anyhow::Result<Vec<[f64; 3]>> {
    if targets.is_empty() {
        anyhow::bail!("no targets to merge");
    }

    let vertex_count = targets[0].1.len();
    for (name, deltas, _) in targets.iter().skip(1) {
        if deltas.len() != vertex_count {
            anyhow::bail!(
                "target '{}' has {} vertices, expected {}",
                name,
                deltas.len(),
                vertex_count
            );
        }
    }

    let mut result = vec![[0.0_f64; 3]; vertex_count];
    for (_name, deltas, weight) in targets {
        let w = *weight;
        for (i, d) in deltas.iter().enumerate() {
            result[i][0] += d[0] * w;
            result[i][1] += d[1] * w;
            result[i][2] += d[2] * w;
        }
    }
    Ok(result)
}

/// Mirror a target's deltas across the specified axis.
///
/// For each vertex at position P, finds the closest vertex at the
/// mirror-reflected position (within `tolerance`) and writes the delta
/// with the axis component negated.
pub fn mirror_target(
    deltas: &[[f64; 3]],
    positions: &[[f64; 3]],
    axis: MirrorAxis,
    tolerance: f64,
) -> anyhow::Result<Vec<[f64; 3]>> {
    let n = deltas.len();
    if positions.len() != n {
        anyhow::bail!(
            "deltas length {} != positions length {}",
            n,
            positions.len()
        );
    }
    if tolerance <= 0.0 {
        anyhow::bail!("tolerance must be positive, got {}", tolerance);
    }

    let ax = axis.idx();
    let tol_sq = tolerance * tolerance;
    let mut result = deltas.to_vec();

    for i in 0..n {
        let pos = positions[i];
        // Process vertices on positive side only
        if pos[ax] < 0.0 {
            continue;
        }

        let mut mirror_pos = pos;
        mirror_pos[ax] = -mirror_pos[ax];

        let mut best_j: Option<usize> = None;
        let mut best_dsq = f64::MAX;
        for (j, jpos) in positions[..n].iter().enumerate() {
            let dp0 = jpos[0] - mirror_pos[0];
            let dp1 = jpos[1] - mirror_pos[1];
            let dp2 = jpos[2] - mirror_pos[2];
            let dsq = dp0 * dp0 + dp1 * dp1 + dp2 * dp2;
            if dsq < best_dsq {
                best_dsq = dsq;
                best_j = Some(j);
            }
        }

        if let Some(j) = best_j {
            if best_dsq <= tol_sq {
                let mut d = deltas[i];
                d[ax] = -d[ax];
                result[j] = d;
            }
        }
    }
    Ok(result)
}

/// Subtract one target from another (element-wise).
pub fn subtract_targets(a: &[[f64; 3]], b: &[[f64; 3]]) -> anyhow::Result<Vec<[f64; 3]>> {
    if a.len() != b.len() {
        anyhow::bail!("target lengths differ: {} vs {}", a.len(), b.len());
    }
    Ok(a.iter()
        .zip(b.iter())
        .map(|(va, vb)| [va[0] - vb[0], va[1] - vb[1], va[2] - vb[2]])
        .collect())
}

/// Add two targets element-wise.
pub fn add_targets(a: &[[f64; 3]], b: &[[f64; 3]]) -> anyhow::Result<Vec<[f64; 3]>> {
    if a.len() != b.len() {
        anyhow::bail!("target lengths differ: {} vs {}", a.len(), b.len());
    }
    Ok(a.iter()
        .zip(b.iter())
        .map(|(va, vb)| [va[0] + vb[0], va[1] + vb[1], va[2] + vb[2]])
        .collect())
}

/// Scale a target uniformly.
pub fn scale_target(deltas: &[[f64; 3]], factor: f64) -> Vec<[f64; 3]> {
    deltas
        .iter()
        .map(|d| [d[0] * factor, d[1] * factor, d[2] * factor])
        .collect()
}

/// Clamp all deltas to a maximum displacement magnitude.
pub fn clamp_target(deltas: &[[f64; 3]], max_magnitude: f64) -> Vec<[f64; 3]> {
    deltas
        .iter()
        .map(|d| {
            let m = magnitude(*d);
            if m > max_magnitude && m > 1e-15 {
                let scale = max_magnitude / m;
                [d[0] * scale, d[1] * scale, d[2] * scale]
            } else {
                *d
            }
        })
        .collect()
}

/// Sparsify a delta array: set any delta below `threshold` to zero.
pub fn sparsify_target(deltas: &[[f64; 3]], threshold: f64) -> Vec<[f64; 3]> {
    deltas
        .iter()
        .map(|d| {
            if magnitude(*d) < threshold {
                [0.0; 3]
            } else {
                *d
            }
        })
        .collect()
}

// MirrorAxis::idx() is defined in delta_painter.rs and used here via the public method.

// ── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_target() {
        let deltas = vec![[0.0; 3]; 5];
        let warnings = TargetValidator::validate(&deltas, 5).expect("validate ok");
        assert!(warnings.iter().any(|w| w.kind == WarningKind::EmptyTarget));
    }

    #[test]
    fn test_validate_excessive_displacement() {
        let mut deltas = vec![[0.0; 3]; 5];
        deltas[0] = [2.0, 0.0, 0.0]; // > 1.0
        let warnings = TargetValidator::validate(&deltas, 5).expect("ok");
        assert!(warnings
            .iter()
            .any(|w| w.kind == WarningKind::ExcessiveDisplacement));
    }

    #[test]
    fn test_validate_length_mismatch() {
        let deltas = vec![[0.0; 3]; 5];
        assert!(TargetValidator::validate(&deltas, 10).is_err());
    }

    #[test]
    fn test_validate_clean_target() {
        let mut deltas = vec![[0.0; 3]; 5];
        deltas[0] = [0.1, 0.0, 0.0];
        let warnings = TargetValidator::validate(&deltas, 5).expect("ok");
        // Should have no warnings (displacement < 1.0, not empty, not self-intersecting)
        assert!(
            warnings.is_empty(),
            "expected no warnings, got {:?}",
            warnings
        );
    }

    #[test]
    fn test_check_symmetry_symmetric() {
        // Two vertices symmetric about X=0, with symmetric deltas
        let positions = vec![[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]];
        let deltas = vec![[0.1, 0.2, 0.3], [-0.1, 0.2, 0.3]]; // mirrored X
        let report = TargetValidator::check_symmetry(&deltas, &positions, 0.01);
        assert!(report.symmetric, "should be symmetric");
        assert!(report.max_asymmetry < 0.01);
    }

    #[test]
    fn test_check_symmetry_asymmetric() {
        let positions = vec![[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]];
        let deltas = vec![[0.1, 0.2, 0.3], [0.5, 0.0, 0.0]]; // NOT mirrored
        let report = TargetValidator::check_symmetry(&deltas, &positions, 0.01);
        assert!(!report.symmetric, "should be asymmetric");
    }

    #[test]
    fn test_check_magnitude() {
        let deltas = vec![[0.1, 0.0, 0.0], [2.0, 0.0, 0.0], [0.5, 0.0, 0.0]];
        let exceeding = TargetValidator::check_magnitude(&deltas, 1.0);
        assert_eq!(exceeding, vec![1]);
    }

    #[test]
    fn test_inspect_basic() {
        let deltas = vec![[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.5, 0.0]];
        let info = TargetInspector::inspect(&deltas);
        assert_eq!(info.vertex_count, 3);
        assert_eq!(info.affected_count, 2);
        assert!((info.max_displacement - 1.0).abs() < 1e-10);
        assert!(info.sparsity > 0.0 && info.sparsity < 1.0);
    }

    #[test]
    fn test_inspect_empty() {
        let deltas: Vec<[f64; 3]> = vec![];
        let info = TargetInspector::inspect(&deltas);
        assert_eq!(info.vertex_count, 0);
        assert_eq!(info.affected_count, 0);
        assert!((info.max_displacement).abs() < 1e-15);
    }

    #[test]
    fn test_affected_vertices() {
        let deltas = vec![[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.5, 0.0]];
        let affected = TargetInspector::affected_vertices(&deltas, 0.01);
        assert_eq!(affected, vec![0, 2]);
    }

    #[test]
    fn test_bounding_box() {
        let deltas = vec![[1.0, -2.0, 3.0], [-1.0, 4.0, 0.5]];
        let (mn, mx) = TargetInspector::bounding_box(&deltas);
        assert!((mn[0] - (-1.0)).abs() < 1e-15);
        assert!((mx[0] - 1.0).abs() < 1e-15);
        assert!((mn[1] - (-2.0)).abs() < 1e-15);
        assert!((mx[1] - 4.0).abs() < 1e-15);
        assert!((mn[2] - 0.5).abs() < 1e-15);
        assert!((mx[2] - 3.0).abs() < 1e-15);
    }

    #[test]
    fn test_bounding_box_empty() {
        let (mn, mx) = TargetInspector::bounding_box(&[]);
        assert_eq!(mn, [0.0; 3]);
        assert_eq!(mx, [0.0; 3]);
    }

    #[test]
    fn test_max_displacement() {
        let deltas = vec![[1.0, 0.0, 0.0], [0.0, 3.0, 4.0]]; // magnitudes: 1.0, 5.0
        let max_d = TargetInspector::max_displacement(&deltas);
        assert!((max_d - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_rms_displacement() {
        let deltas = vec![[1.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let rms = TargetInspector::rms_displacement(&deltas);
        // sqrt((1 + 0) / 2) = sqrt(0.5) ≈ 0.7071
        assert!((rms - (0.5_f64).sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_merge_targets_single() {
        let d = vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let result = merge_targets(&[("a", &d, 1.0)]).expect("merge ok");
        assert_eq!(result.len(), 2);
        assert!((result[0][0] - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_merge_targets_weighted() {
        let a = [[1.0, 0.0, 0.0]];
        let b = [[0.0, 2.0, 0.0]];
        let result = merge_targets(&[("a", &a[..], 0.5), ("b", &b[..], 0.5)]).expect("ok");
        assert!((result[0][0] - 0.5).abs() < 1e-15);
        assert!((result[0][1] - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_merge_targets_empty() {
        assert!(merge_targets(&[]).is_err());
    }

    #[test]
    fn test_merge_targets_length_mismatch() {
        let a = [[1.0, 0.0, 0.0]];
        let b = [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];
        assert!(merge_targets(&[("a", &a[..], 1.0), ("b", &b[..], 1.0)]).is_err());
    }

    #[test]
    fn test_mirror_target_x() {
        let positions = vec![[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]];
        let deltas = vec![[0.5, 0.3, 0.1], [0.0, 0.0, 0.0]];
        let mirrored = mirror_target(&deltas, &positions, MirrorAxis::X, 0.1).expect("mirror ok");
        // Vertex 1 (mirror of 0) should get negated X component
        assert!((mirrored[1][0] - (-0.5)).abs() < 1e-10);
        assert!((mirrored[1][1] - 0.3).abs() < 1e-10);
        assert!((mirrored[1][2] - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_mirror_target_length_mismatch() {
        let d = vec![[0.0; 3]; 3];
        let p = vec![[0.0; 3]; 2];
        assert!(mirror_target(&d, &p, MirrorAxis::X, 0.1).is_err());
    }

    #[test]
    fn test_subtract_targets() {
        let a = vec![[1.0, 2.0, 3.0]];
        let b = vec![[0.5, 0.5, 0.5]];
        let result = subtract_targets(&a, &b).expect("ok");
        assert!((result[0][0] - 0.5).abs() < 1e-15);
        assert!((result[0][1] - 1.5).abs() < 1e-15);
        assert!((result[0][2] - 2.5).abs() < 1e-15);
    }

    #[test]
    fn test_add_targets() {
        let a = vec![[1.0, 2.0, 3.0]];
        let b = vec![[0.5, 0.5, 0.5]];
        let result = add_targets(&a, &b).expect("ok");
        assert!((result[0][0] - 1.5).abs() < 1e-15);
    }

    #[test]
    fn test_scale_target() {
        let d = vec![[1.0, 2.0, 3.0]];
        let result = scale_target(&d, 2.0);
        assert!((result[0][0] - 2.0).abs() < 1e-15);
        assert!((result[0][1] - 4.0).abs() < 1e-15);
    }

    #[test]
    fn test_clamp_target() {
        let d = vec![[10.0, 0.0, 0.0], [0.1, 0.0, 0.0]];
        let clamped = clamp_target(&d, 1.0);
        assert!((magnitude(clamped[0]) - 1.0).abs() < 1e-10);
        assert!((clamped[1][0] - 0.1).abs() < 1e-15); // untouched
    }

    #[test]
    fn test_sparsify_target() {
        let d = vec![[1.0, 0.0, 0.0], [0.001, 0.0, 0.0], [0.0, 0.5, 0.0]];
        let sparse = sparsify_target(&d, 0.01);
        assert!((sparse[0][0] - 1.0).abs() < 1e-15);
        assert_eq!(sparse[1], [0.0; 3]); // below threshold
        assert!((sparse[2][1] - 0.5).abs() < 1e-15);
    }

    #[test]
    fn test_inspect_named() {
        let deltas = vec![[1.0, 0.0, 0.0]];
        let info = TargetInspector::inspect_named(&deltas, "my_target");
        assert_eq!(info.name, "my_target");
        assert_eq!(info.vertex_count, 1);
        assert_eq!(info.affected_count, 1);
    }

    #[test]
    fn test_self_intersection_warning() {
        let mut deltas = vec![[0.0; 3]; 5];
        deltas[0] = [0.8, 0.0, 0.0];
        deltas[1] = [-0.8, 0.0, 0.0]; // range = 1.6 on X > 1.0
        let warnings = TargetValidator::validate(&deltas, 5).expect("ok");
        assert!(warnings
            .iter()
            .any(|w| w.kind == WarningKind::SelfIntersectionRisk));
    }
}
