// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Feature line extraction: silhouettes, creases, boundaries, ridges, and valleys.
//!
//! This module provides utilities to detect and classify geometric feature lines
//! on triangle meshes based on dihedral angles, curvature signs, and view direction.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Classification of a feature line.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureLineType {
    Silhouette,
    Crease,
    Boundary,
    Ridge,
    Valley,
}

/// Configuration for feature line extraction.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FeatureLineConfig {
    /// Minimum dihedral angle (radians) to mark as a crease.
    pub crease_angle_threshold: f32,
    /// Minimum curvature magnitude to detect ridge/valley.
    pub curvature_threshold: f32,
    /// Whether to include boundary edges in results.
    pub include_boundary: bool,
    /// Whether to include silhouette edges.
    pub include_silhouette: bool,
    /// Whether to include crease edges.
    pub include_crease: bool,
}

/// A single extracted feature line (an edge between two vertex indices).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FeatureLine {
    pub v0: usize,
    pub v1: usize,
    pub line_type: FeatureLineType,
    /// Geometric length of the edge.
    pub length: f32,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// Result of merging multiple feature line sets.
pub type MergedLines = Vec<FeatureLine>;

// ── Helper math ───────────────────────────────────────────────────────────────

#[allow(dead_code)]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[allow(dead_code)]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[allow(dead_code)]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-12 {
        [0.0, 0.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

/// Compute the triangle face normal from three positions.
#[allow(dead_code)]
fn face_normal(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    normalize3(cross3(sub3(p1, p0), sub3(p2, p0)))
}

/// Euclidean distance between two 3-D points.
#[allow(dead_code)]
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    len3(sub3(a, b))
}

// ── Public functions ──────────────────────────────────────────────────────────

/// Return a sensible default [`FeatureLineConfig`].
#[allow(dead_code)]
pub fn default_feature_line_config() -> FeatureLineConfig {
    FeatureLineConfig {
        crease_angle_threshold: std::f32::consts::PI / 6.0, // 30°
        curvature_threshold: 0.05,
        include_boundary: true,
        include_silhouette: true,
        include_crease: true,
    }
}

/// Extract open-boundary edges (edges belonging to exactly one triangle).
///
/// Returns each boundary edge as a [`FeatureLine`] of type `Boundary`.
#[allow(dead_code)]
pub fn extract_boundary_edges(positions: &[[f32; 3]], indices: &[u32]) -> Vec<FeatureLine> {
    use std::collections::HashMap;

    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    let tris = indices.len() / 3;
    for t in 0..tris {
        let base = t * 3;
        let verts = [
            indices[base] as usize,
            indices[base + 1] as usize,
            indices[base + 2] as usize,
        ];
        for k in 0..3 {
            let a = verts[k].min(verts[(k + 1) % 3]);
            let b = verts[k].max(verts[(k + 1) % 3]);
            *edge_count.entry((a, b)).or_insert(0) += 1;
        }
    }

    edge_count
        .into_iter()
        .filter(|(_, count)| *count == 1)
        .map(|((a, b), _)| {
            let length = if a < positions.len() && b < positions.len() {
                dist3(positions[a], positions[b])
            } else {
                0.0
            };
            FeatureLine {
                v0: a,
                v1: b,
                line_type: FeatureLineType::Boundary,
                length,
            }
        })
        .collect()
}

/// Extract crease edges whose dihedral angle exceeds `threshold` radians.
#[allow(dead_code)]
pub fn extract_crease_edges(
    positions: &[[f32; 3]],
    indices: &[u32],
    threshold: f32,
) -> Vec<FeatureLine> {
    use std::collections::HashMap;

    // Map edge → list of adjacent face normals
    let mut edge_faces: HashMap<(usize, usize), Vec<[f32; 3]>> = HashMap::new();
    let tris = indices.len() / 3;
    for t in 0..tris {
        let base = t * 3;
        let v = [
            indices[base] as usize,
            indices[base + 1] as usize,
            indices[base + 2] as usize,
        ];
        let n = if v[0] < positions.len() && v[1] < positions.len() && v[2] < positions.len() {
            face_normal(positions[v[0]], positions[v[1]], positions[v[2]])
        } else {
            [0.0, 0.0, 1.0]
        };
        for k in 0..3 {
            let a = v[k].min(v[(k + 1) % 3]);
            let b = v[k].max(v[(k + 1) % 3]);
            edge_faces.entry((a, b)).or_default().push(n);
        }
    }

    let mut result = Vec::new();
    for ((a, b), normals) in &edge_faces {
        if normals.len() < 2 {
            continue;
        }
        let cos_angle = dot3(normals[0], normals[1]).clamp(-1.0, 1.0);
        let angle = cos_angle.acos();
        if angle > threshold {
            let length = if *a < positions.len() && *b < positions.len() {
                dist3(positions[*a], positions[*b])
            } else {
                0.0
            };
            result.push(FeatureLine {
                v0: *a,
                v1: *b,
                line_type: FeatureLineType::Crease,
                length,
            });
        }
    }
    result
}

/// Extract silhouette edges given a camera/view direction.
///
/// An edge is a silhouette if its two adjacent faces have normals whose
/// dot product with `view_dir` has opposite signs.
#[allow(dead_code)]
pub fn extract_silhouette_edges(
    positions: &[[f32; 3]],
    indices: &[u32],
    view_dir: [f32; 3],
) -> Vec<FeatureLine> {
    use std::collections::HashMap;

    let view = normalize3(view_dir);
    let mut edge_faces: HashMap<(usize, usize), Vec<[f32; 3]>> = HashMap::new();
    let tris = indices.len() / 3;
    for t in 0..tris {
        let base = t * 3;
        let v = [
            indices[base] as usize,
            indices[base + 1] as usize,
            indices[base + 2] as usize,
        ];
        let n = if v[0] < positions.len() && v[1] < positions.len() && v[2] < positions.len() {
            face_normal(positions[v[0]], positions[v[1]], positions[v[2]])
        } else {
            [0.0, 0.0, 1.0]
        };
        for k in 0..3 {
            let a = v[k].min(v[(k + 1) % 3]);
            let b = v[k].max(v[(k + 1) % 3]);
            edge_faces.entry((a, b)).or_default().push(n);
        }
    }

    let mut result = Vec::new();
    for ((a, b), normals) in &edge_faces {
        if normals.len() < 2 {
            continue;
        }
        let d0 = dot3(normals[0], view);
        let d1 = dot3(normals[1], view);
        if d0 * d1 < 0.0 {
            let length = if *a < positions.len() && *b < positions.len() {
                dist3(positions[*a], positions[*b])
            } else {
                0.0
            };
            result.push(FeatureLine {
                v0: *a,
                v1: *b,
                line_type: FeatureLineType::Silhouette,
                length,
            });
        }
    }
    result
}

/// Extract ridge and valley edges based on per-vertex curvature sign.
///
/// `curvatures` is a per-vertex mean curvature array; positive = ridge, negative = valley.
/// An edge is a ridge when both endpoint curvatures exceed `+threshold`,
/// a valley when both are below `-threshold`.
#[allow(dead_code)]
pub fn extract_ridge_valley(
    positions: &[[f32; 3]],
    indices: &[u32],
    curvatures: &[f32],
    threshold: f32,
) -> Vec<FeatureLine> {
    use std::collections::HashSet;

    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    let mut result = Vec::new();
    let tris = indices.len() / 3;
    for t in 0..tris {
        let base = t * 3;
        let v = [
            indices[base] as usize,
            indices[base + 1] as usize,
            indices[base + 2] as usize,
        ];
        for k in 0..3 {
            let a = v[k];
            let b = v[(k + 1) % 3];
            let key = (a.min(b), a.max(b));
            if seen.contains(&key) {
                continue;
            }
            seen.insert(key);
            if a >= curvatures.len() || b >= curvatures.len() {
                continue;
            }
            let ca = curvatures[a];
            let cb = curvatures[b];
            let line_type = if ca > threshold && cb > threshold {
                FeatureLineType::Ridge
            } else if ca < -threshold && cb < -threshold {
                FeatureLineType::Valley
            } else {
                continue;
            };
            let length = if a < positions.len() && b < positions.len() {
                dist3(positions[a], positions[b])
            } else {
                0.0
            };
            result.push(FeatureLine {
                v0: key.0,
                v1: key.1,
                line_type,
                length,
            });
        }
    }
    result
}

/// Count the total number of feature lines in a slice.
#[allow(dead_code)]
pub fn feature_line_count(lines: &[FeatureLine]) -> usize {
    lines.len()
}

/// Merge multiple feature line slices into one `Vec`.
#[allow(dead_code)]
pub fn merge_feature_lines(groups: &[&[FeatureLine]]) -> MergedLines {
    let total: usize = groups.iter().map(|g| g.len()).sum();
    let mut out = Vec::with_capacity(total);
    for g in groups {
        out.extend_from_slice(g);
    }
    out
}

/// Serialize feature lines to a compact JSON string.
#[allow(dead_code)]
pub fn feature_lines_to_json(lines: &[FeatureLine]) -> String {
    let mut out = String::from("[");
    for (i, l) in lines.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let t = match l.line_type {
            FeatureLineType::Silhouette => "silhouette",
            FeatureLineType::Crease => "crease",
            FeatureLineType::Boundary => "boundary",
            FeatureLineType::Ridge => "ridge",
            FeatureLineType::Valley => "valley",
        };
        out.push_str(&format!(
            "{{\"v0\":{},\"v1\":{},\"type\":\"{}\",\"length\":{}}}",
            l.v0, l.v1, t, l.length
        ));
    }
    out.push(']');
    out
}

/// Classify a single edge (v0, v1) from two adjacent face normals and a view direction.
///
/// Returns the most specific type that applies, in priority order:
/// Boundary → Silhouette → Crease → None (returns `None`).
#[allow(dead_code)]
pub fn classify_edge(
    n0: Option<[f32; 3]>,
    n1: Option<[f32; 3]>,
    view_dir: [f32; 3],
    crease_threshold: f32,
) -> Option<FeatureLineType> {
    match (n0, n1) {
        (Some(_), None) | (None, Some(_)) => Some(FeatureLineType::Boundary),
        (Some(na), Some(nb)) => {
            let view = normalize3(view_dir);
            let da = dot3(na, view);
            let db = dot3(nb, view);
            if da * db < 0.0 {
                return Some(FeatureLineType::Silhouette);
            }
            let cos_angle = dot3(na, nb).clamp(-1.0, 1.0);
            let angle = cos_angle.acos();
            if angle > crease_threshold {
                return Some(FeatureLineType::Crease);
            }
            None
        }
        (None, None) => None,
    }
}

/// Filter feature lines, keeping only those of the specified type.
#[allow(dead_code)]
pub fn filter_by_type(lines: &[FeatureLine], line_type: FeatureLineType) -> Vec<FeatureLine> {
    lines
        .iter()
        .filter(|l| l.line_type == line_type)
        .cloned()
        .collect()
}

/// Compute the total length of a feature line (same as `line.length`).
#[allow(dead_code)]
pub fn feature_line_length(line: &FeatureLine) -> f32 {
    line.length
}

/// Sort feature lines by descending length.
#[allow(dead_code)]
pub fn sort_feature_lines(lines: &mut [FeatureLine]) {
    lines.sort_by(|a, b| b.length.partial_cmp(&a.length).unwrap_or(std::cmp::Ordering::Equal));
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn quad_indices() -> Vec<u32> {
        vec![0, 1, 2, 0, 2, 3]
    }

    // 1. default_feature_line_config returns positive threshold
    #[test]
    fn test_default_config_threshold_positive() {
        let cfg = default_feature_line_config();
        assert!(cfg.crease_angle_threshold > 0.0);
        assert!(cfg.curvature_threshold > 0.0);
    }

    // 2. default_feature_line_config enables all features by default
    #[test]
    fn test_default_config_all_enabled() {
        let cfg = default_feature_line_config();
        assert!(cfg.include_boundary);
        assert!(cfg.include_silhouette);
        assert!(cfg.include_crease);
    }

    // 3. extract_boundary_edges on a single triangle → 3 boundaries
    #[test]
    fn test_boundary_single_triangle() {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0u32, 1, 2];
        let lines = extract_boundary_edges(&pos, &idx);
        assert_eq!(lines.len(), 3);
        for l in &lines {
            assert_eq!(l.line_type, FeatureLineType::Boundary);
        }
    }

    // 4. extract_boundary_edges on two triangles → 4 boundaries
    #[test]
    fn test_boundary_quad_has_four_boundary_edges() {
        let pos = quad_positions();
        let idx = quad_indices();
        let lines = extract_boundary_edges(&pos, &idx);
        assert_eq!(lines.len(), 4);
    }

    // 5. extract_crease_edges: flat quad has no creases
    #[test]
    fn test_crease_flat_quad_no_creases() {
        let pos = quad_positions();
        let idx = quad_indices();
        let lines = extract_crease_edges(&pos, &idx, std::f32::consts::PI / 6.0);
        // shared edge is flat (0°), should not be a crease
        assert_eq!(lines.len(), 0);
    }

    // 6. extract_crease_edges: folded mesh has creases
    #[test]
    fn test_crease_folded_mesh_detected() {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0], // first face is XY-plane
            [1.0, 0.0, 1.0], // second face bends into Z
        ];
        // Two triangles sharing edge (1,2)
        let idx = vec![0u32, 1, 2, 1, 3, 2];
        let lines = extract_crease_edges(&pos, &idx, 0.1);
        assert!(!lines.is_empty(), "Should detect a crease on folded mesh");
    }

    // 7. extract_silhouette_edges: looking from above flat quad → no silhouette
    #[test]
    fn test_silhouette_flat_from_top_no_silhouette() {
        let pos = quad_positions();
        let idx = quad_indices();
        // View from +Z; both faces face +Z → same sign → no silhouette
        let lines = extract_silhouette_edges(&pos, &idx, [0.0, 0.0, 1.0]);
        assert_eq!(lines.len(), 0);
    }

    // 8. extract_silhouette_edges: view from -Z → still no silhouette on flat quad
    #[test]
    fn test_silhouette_flat_from_bottom_no_silhouette() {
        let pos = quad_positions();
        let idx = quad_indices();
        let lines = extract_silhouette_edges(&pos, &idx, [0.0, 0.0, -1.0]);
        assert_eq!(lines.len(), 0);
    }

    // 9. extract_ridge_valley: curvature above threshold → ridge
    #[test]
    fn test_ridge_detected_above_threshold() {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0u32, 1, 2];
        let curv = vec![0.5f32, 0.5, 0.5];
        let lines = extract_ridge_valley(&pos, &idx, &curv, 0.1);
        assert!(lines.iter().any(|l| l.line_type == FeatureLineType::Ridge));
    }

    // 10. extract_ridge_valley: curvature below negative threshold → valley
    #[test]
    fn test_valley_detected_below_threshold() {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0u32, 1, 2];
        let curv = vec![-0.5f32, -0.5, -0.5];
        let lines = extract_ridge_valley(&pos, &idx, &curv, 0.1);
        assert!(lines.iter().any(|l| l.line_type == FeatureLineType::Valley));
    }

    // 11. feature_line_count matches vec length
    #[test]
    fn test_feature_line_count() {
        let pos = quad_positions();
        let idx = quad_indices();
        let lines = extract_boundary_edges(&pos, &idx);
        assert_eq!(feature_line_count(&lines), lines.len());
    }

    // 12. merge_feature_lines combines slices
    #[test]
    fn test_merge_feature_lines() {
        let pos = quad_positions();
        let idx = quad_indices();
        let a = extract_boundary_edges(&pos, &idx);
        let b = extract_boundary_edges(&pos, &idx);
        let merged = merge_feature_lines(&[&a, &b]);
        assert_eq!(merged.len(), a.len() + b.len());
    }

    // 13. feature_lines_to_json is non-empty for non-empty input
    #[test]
    fn test_feature_lines_to_json_non_empty() {
        let pos = quad_positions();
        let idx = quad_indices();
        let lines = extract_boundary_edges(&pos, &idx);
        let json = feature_lines_to_json(&lines);
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
        assert!(json.contains("boundary"));
    }

    // 14. feature_lines_to_json empty input → "[]"
    #[test]
    fn test_feature_lines_to_json_empty() {
        let json = feature_lines_to_json(&[]);
        assert_eq!(json, "[]");
    }

    // 15. classify_edge: one normal missing → Boundary
    #[test]
    fn test_classify_edge_boundary() {
        let n = [0.0f32, 0.0, 1.0];
        let result = classify_edge(Some(n), None, [0.0, 0.0, 1.0], 0.5);
        assert_eq!(result, Some(FeatureLineType::Boundary));
    }

    // 16. classify_edge: both normals same direction → no feature at low threshold
    #[test]
    fn test_classify_edge_no_feature_flat() {
        let n = [0.0f32, 0.0, 1.0];
        let result = classify_edge(Some(n), Some(n), [0.0, 1.0, 0.0], 2.0);
        // Dihedral angle is 0, well below threshold of 2.0 → None
        assert_eq!(result, None);
    }

    // 17. filter_by_type keeps only matching type
    #[test]
    fn test_filter_by_type() {
        let pos = quad_positions();
        let idx = quad_indices();
        let lines = extract_boundary_edges(&pos, &idx);
        let filtered = filter_by_type(&lines, FeatureLineType::Boundary);
        assert_eq!(filtered.len(), lines.len());
        let none = filter_by_type(&lines, FeatureLineType::Crease);
        assert!(none.is_empty());
    }

    // 18. feature_line_length matches stored length field
    #[test]
    fn test_feature_line_length_matches_field() {
        let line = FeatureLine {
            v0: 0,
            v1: 1,
            line_type: FeatureLineType::Boundary,
            length: std::f32::consts::PI,
        };
        assert!((feature_line_length(&line) - std::f32::consts::PI).abs() < 1e-6);
    }

    // 19. sort_feature_lines orders by descending length
    #[test]
    fn test_sort_feature_lines_descending() {
        let mut lines = vec![
            FeatureLine { v0: 0, v1: 1, line_type: FeatureLineType::Boundary, length: 1.0 },
            FeatureLine { v0: 1, v1: 2, line_type: FeatureLineType::Boundary, length: 3.0 },
            FeatureLine { v0: 2, v1: 3, line_type: FeatureLineType::Boundary, length: 2.0 },
        ];
        sort_feature_lines(&mut lines);
        assert!(lines[0].length >= lines[1].length);
        assert!(lines[1].length >= lines[2].length);
    }

    // 20. boundary edges all have positive length for valid positions
    #[test]
    fn test_boundary_edges_positive_length() {
        let pos = quad_positions();
        let idx = quad_indices();
        let lines = extract_boundary_edges(&pos, &idx);
        for l in &lines {
            assert!(l.length > 0.0, "boundary edge should have positive length");
        }
    }
}
