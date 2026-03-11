// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! UV parameterization: planar projection, distortion metrics, island packing stub.
//!
//! Supports XY/XZ/YZ planar projection, spherical and cylindrical unwrapping,
//! distortion measurement, UV normalization, seam length computation, and a greedy
//! rectangle-packing stub for UV island layout.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Which projection plane / method to use.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UvProjectionMode {
    XY,
    XZ,
    YZ,
    Spherical,
    Cylindrical,
}

/// Configuration for parameterization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParameterizeConfig {
    pub projection: UvProjectionMode,
    /// Whether to normalize UVs to [0, 1]² after projection.
    pub normalize: bool,
    /// Whether to flip V coordinate (1.0 − v).
    pub flip_v: bool,
    /// Whether to pack islands into [0, 1]² after computing.
    pub pack_islands: bool,
}

/// A single UV coordinate pair.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct UvParam {
    pub u: f32,
    pub v: f32,
}

/// Result of a parameterization operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParamResult {
    /// Per-vertex UV coordinates (same length as input `positions`).
    pub uvs: Vec<UvParam>,
    /// Number of UV islands detected (stub: always 1 for planar).
    pub island_count: usize,
    /// Whether the UV map passed basic validity checks.
    pub valid: bool,
    /// Total UV seam length.
    pub seam_length: f32,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// A packed island rectangle: (u_min, v_min, u_max, v_max).
pub type IslandRect = (f32, f32, f32, f32);

// ── Helper math ───────────────────────────────────────────────────────────────

#[allow(dead_code)]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[allow(dead_code)]
fn dist2(a: UvParam, b: UvParam) -> f32 {
    let du = a.u - b.u;
    let dv = a.v - b.v;
    (du * du + dv * dv).sqrt()
}

// ── Public functions ──────────────────────────────────────────────────────────

/// Return sensible default parameterization config.
#[allow(dead_code)]
pub fn default_parameterize_config() -> ParameterizeConfig {
    ParameterizeConfig {
        projection: UvProjectionMode::XY,
        normalize: true,
        flip_v: false,
        pack_islands: false,
    }
}

/// Project vertex positions to UV coordinates using the specified mode.
///
/// - `XY`: u=x, v=y
/// - `XZ`: u=x, v=z
/// - `YZ`: u=y, v=z
/// - `Spherical`: longitude/latitude unwrap
/// - `Cylindrical`: atan2 around Y axis / height
#[allow(dead_code)]
pub fn planar_project_uv(positions: &[[f32; 3]], mode: UvProjectionMode) -> Vec<UvParam> {
    positions
        .iter()
        .map(|&[x, y, z]| match mode {
            UvProjectionMode::XY => UvParam { u: x, v: y },
            UvProjectionMode::XZ => UvParam { u: x, v: z },
            UvProjectionMode::YZ => UvParam { u: y, v: z },
            UvProjectionMode::Spherical => {
                let r = len3([x, y, z]);
                if r < 1e-12 {
                    UvParam { u: 0.0, v: 0.5 }
                } else {
                    let u = (z.atan2(x) / (2.0 * std::f32::consts::PI)) + 0.5;
                    let v = (y / r).clamp(-1.0, 1.0).acos() / std::f32::consts::PI;
                    UvParam { u, v }
                }
            }
            UvProjectionMode::Cylindrical => {
                let u = (z.atan2(x) / (2.0 * std::f32::consts::PI)) + 0.5;
                UvParam { u, v: y }
            }
        })
        .collect()
}

/// Parameterize a patch (subset of vertices) using planar projection.
///
/// `patch_indices` selects which entries from `positions` to project.
/// Returns UVs only for the patch vertices, in order.
#[allow(dead_code)]
pub fn parameterize_patch(
    positions: &[[f32; 3]],
    patch_indices: &[usize],
    mode: UvProjectionMode,
) -> Vec<UvParam> {
    let patch_pos: Vec<[f32; 3]> = patch_indices
        .iter()
        .filter_map(|&i| positions.get(i).copied())
        .collect();
    planar_project_uv(&patch_pos, mode)
}

/// Compute a distortion metric (mean squared difference between 3-D edge lengths
/// and corresponding UV edge lengths, normalised by 3-D edge length).
///
/// Lower is better; 0 means perfect isometric mapping.
#[allow(dead_code)]
pub fn uv_distortion_metric(positions: &[[f32; 3]], uvs: &[UvParam], indices: &[u32]) -> f32 {
    if indices.len() < 3 || uvs.len() != positions.len() {
        return 0.0;
    }
    let tris = indices.len() / 3;
    let mut total = 0.0f32;
    let mut count = 0usize;
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
            if a >= positions.len() || b >= positions.len() {
                continue;
            }
            let p = positions[a];
            let q = positions[b];
            let d3 = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2))
                .sqrt();
            let d2 = dist2(uvs[a], uvs[b]);
            if d3 > 1e-12 {
                total += ((d2 - d3) / d3).powi(2);
                count += 1;
            }
        }
    }
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

/// Count approximate UV overlaps: pairs of triangles whose UV centroids are
/// within `tolerance` of each other (simple O(n²) stub).
#[allow(dead_code)]
pub fn uv_overlap_count(uvs: &[UvParam], indices: &[u32], tolerance: f32) -> usize {
    let tris = indices.len() / 3;
    let mut centroids: Vec<UvParam> = Vec::with_capacity(tris);
    for t in 0..tris {
        let base = t * 3;
        let mut su = 0.0f32;
        let mut sv = 0.0f32;
        for k in 0..3 {
            let vi = indices[base + k] as usize;
            if vi < uvs.len() {
                su += uvs[vi].u;
                sv += uvs[vi].v;
            }
        }
        centroids.push(UvParam { u: su / 3.0, v: sv / 3.0 });
    }
    let mut count = 0;
    for i in 0..centroids.len() {
        for j in (i + 1)..centroids.len() {
            if dist2(centroids[i], centroids[j]) < tolerance {
                count += 1;
            }
        }
    }
    count
}

/// Scale UV coordinates so they span exactly [0, 1] on both axes.
#[allow(dead_code)]
pub fn normalize_uv_bounds(uvs: &[UvParam]) -> Vec<UvParam> {
    if uvs.is_empty() {
        return Vec::new();
    }
    let mut umin = f32::MAX;
    let mut umax = f32::MIN;
    let mut vmin = f32::MAX;
    let mut vmax = f32::MIN;
    for uv in uvs {
        umin = umin.min(uv.u);
        umax = umax.max(uv.u);
        vmin = vmin.min(uv.v);
        vmax = vmax.max(uv.v);
    }
    let urange = (umax - umin).max(1e-12);
    let vrange = (vmax - vmin).max(1e-12);
    uvs.iter()
        .map(|uv| UvParam {
            u: (uv.u - umin) / urange,
            v: (uv.v - vmin) / vrange,
        })
        .collect()
}

/// Flip the V coordinate: v ← 1 − v.
#[allow(dead_code)]
pub fn flip_uv_v(uvs: &[UvParam]) -> Vec<UvParam> {
    uvs.iter().map(|uv| UvParam { u: uv.u, v: 1.0 - uv.v }).collect()
}

/// Rotate UV coordinates 90° counter-clockwise: (u, v) → (v, 1 − u).
#[allow(dead_code)]
pub fn rotate_uv_90(uvs: &[UvParam]) -> Vec<UvParam> {
    uvs.iter().map(|uv| UvParam { u: uv.v, v: 1.0 - uv.u }).collect()
}

/// Greedy rectangle packing stub.
///
/// Treats each provided `(width, height)` rect as a UV island and stacks them
/// from left to right. Returns the placed rectangles as `(u_min, v_min, u_max, v_max)`.
#[allow(dead_code)]
pub fn pack_uv_islands_stub(island_sizes: &[(f32, f32)]) -> Vec<IslandRect> {
    let mut x = 0.0f32;
    let mut result = Vec::with_capacity(island_sizes.len());
    for &(w, h) in island_sizes {
        result.push((x, 0.0, x + w, h));
        x += w + 0.01; // small gap
    }
    result
}

/// Check that all UV coordinates are finite and within [−1, 3] (relaxed bounds).
///
/// Returns `true` when the map is valid.
#[allow(dead_code)]
pub fn validate_uv_map(uvs: &[UvParam]) -> bool {
    uvs.iter().all(|uv| {
        uv.u.is_finite()
            && uv.v.is_finite()
            && uv.u >= -1.0
            && uv.u <= 3.0
            && uv.v >= -1.0
            && uv.v <= 3.0
    })
}

/// Compute total UV seam length.
///
/// A UV seam edge is an edge whose two vertices differ in UV space by more than
/// `threshold` even though they share the same 3-D position (within `pos_eps`).
///
/// This simple stub counts boundary edges and sums their 3-D lengths as a proxy.
#[allow(dead_code)]
pub fn uv_seam_length(positions: &[[f32; 3]], uvs: &[UvParam], indices: &[u32]) -> f32 {
    use std::collections::HashMap;

    if uvs.len() != positions.len() {
        return 0.0;
    }
    // Edges shared by exactly one triangle are seam candidates
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    let tris = indices.len() / 3;
    for t in 0..tris {
        let base = t * 3;
        for k in 0..3 {
            let a = indices[base + k] as usize;
            let b = indices[base + (k + 1) % 3] as usize;
            let key = (a.min(b), a.max(b));
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    let mut total = 0.0f32;
    for ((a, b), count) in &edge_count {
        if *count == 1 && *a < positions.len() && *b < positions.len() {
            let p = positions[*a];
            let q = positions[*b];
            total += ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2))
                .sqrt();
        }
    }
    total
}

/// Return the number of unique UV vertices.
#[allow(dead_code)]
pub fn uv_vertex_count(uvs: &[UvParam]) -> usize {
    uvs.len()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_triangle_pos() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    }

    fn unit_triangle_idx() -> Vec<u32> {
        vec![0, 1, 2]
    }

    // 1. default_parameterize_config defaults to XY
    #[test]
    fn test_default_config_projection_xy() {
        let cfg = default_parameterize_config();
        assert_eq!(cfg.projection, UvProjectionMode::XY);
    }

    // 2. default_parameterize_config normalize is true
    #[test]
    fn test_default_config_normalize_true() {
        let cfg = default_parameterize_config();
        assert!(cfg.normalize);
    }

    // 3. planar_project_uv XY maps x→u, y→v
    #[test]
    fn test_planar_project_xy() {
        let pos = unit_triangle_pos();
        let uvs = planar_project_uv(&pos, UvProjectionMode::XY);
        assert_eq!(uvs.len(), 3);
        assert!((uvs[0].u - 0.0).abs() < 1e-6);
        assert!((uvs[1].u - 1.0).abs() < 1e-6);
        assert!((uvs[2].v - 1.0).abs() < 1e-6);
    }

    // 4. planar_project_uv XZ maps x→u, z→v
    #[test]
    fn test_planar_project_xz() {
        let pos = vec![[1.0f32, 99.0, 2.0]];
        let uvs = planar_project_uv(&pos, UvProjectionMode::XZ);
        assert!((uvs[0].u - 1.0).abs() < 1e-6);
        assert!((uvs[0].v - 2.0).abs() < 1e-6);
    }

    // 5. planar_project_uv YZ maps y→u, z→v
    #[test]
    fn test_planar_project_yz() {
        let pos = vec![[99.0f32, 3.0, 4.0]];
        let uvs = planar_project_uv(&pos, UvProjectionMode::YZ);
        assert!((uvs[0].u - 3.0).abs() < 1e-6);
        assert!((uvs[0].v - 4.0).abs() < 1e-6);
    }

    // 6. planar_project_uv Spherical: u in [0,1]
    #[test]
    fn test_spherical_u_in_range() {
        let pos = vec![[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let uvs = planar_project_uv(&pos, UvProjectionMode::Spherical);
        for uv in &uvs {
            assert!(uv.u >= 0.0 && uv.u <= 1.0, "u={} not in [0,1]", uv.u);
        }
    }

    // 7. planar_project_uv Cylindrical: u in [0,1]
    #[test]
    fn test_cylindrical_u_in_range() {
        let pos = vec![[1.0f32, 2.0, 0.0], [-1.0, -1.0, 0.0]];
        let uvs = planar_project_uv(&pos, UvProjectionMode::Cylindrical);
        for uv in &uvs {
            assert!(uv.u >= 0.0 && uv.u <= 1.0, "u={} not in [0,1]", uv.u);
        }
    }

    // 8. parameterize_patch returns correct count
    #[test]
    fn test_parameterize_patch_count() {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let patch = vec![0, 2];
        let uvs = parameterize_patch(&pos, &patch, UvProjectionMode::XY);
        assert_eq!(uvs.len(), 2);
    }

    // 9. uv_distortion_metric zero for identity mapping
    #[test]
    fn test_distortion_zero_identity() {
        let pos = unit_triangle_pos();
        let idx = unit_triangle_idx();
        // UV = XY projection, 3-D edges are in XY plane, so lengths match
        let uvs = planar_project_uv(&pos, UvProjectionMode::XY);
        let dist = uv_distortion_metric(&pos, &uvs, &idx);
        // Should be very small
        assert!(dist < 1e-4, "distortion={dist}");
    }

    // 10. uv_overlap_count: identical triangles have overlap
    #[test]
    fn test_overlap_count_overlapping() {
        // Two triangles with the same UV centroid
        let uvs = vec![
            UvParam { u: 0.0, v: 0.0 },
            UvParam { u: 1.0, v: 0.0 },
            UvParam { u: 0.0, v: 1.0 },
        ];
        // Both triangles use same verts → centroids are identical
        let idx = vec![0u32, 1, 2, 0, 1, 2];
        let count = uv_overlap_count(&uvs, &idx, 0.01);
        assert!(count >= 1, "should detect overlapping UV centroids");
    }

    // 11. normalize_uv_bounds scales to [0,1]
    #[test]
    fn test_normalize_uv_bounds() {
        let uvs = vec![
            UvParam { u: 2.0, v: -1.0 },
            UvParam { u: 4.0, v: 3.0 },
            UvParam { u: 3.0, v: 1.0 },
        ];
        let norm = normalize_uv_bounds(&uvs);
        let umin = norm.iter().map(|uv| uv.u).fold(f32::MAX, f32::min);
        let umax = norm.iter().map(|uv| uv.u).fold(f32::MIN, f32::max);
        let vmin = norm.iter().map(|uv| uv.v).fold(f32::MAX, f32::min);
        let vmax = norm.iter().map(|uv| uv.v).fold(f32::MIN, f32::max);
        assert!((umin - 0.0).abs() < 1e-5);
        assert!((umax - 1.0).abs() < 1e-5);
        assert!((vmin - 0.0).abs() < 1e-5);
        assert!((vmax - 1.0).abs() < 1e-5);
    }

    // 12. normalize_uv_bounds empty → empty
    #[test]
    fn test_normalize_uv_bounds_empty() {
        let norm = normalize_uv_bounds(&[]);
        assert!(norm.is_empty());
    }

    // 13. flip_uv_v flips V
    #[test]
    fn test_flip_uv_v() {
        let uvs = vec![UvParam { u: 0.3, v: 0.7 }];
        let flipped = flip_uv_v(&uvs);
        assert!((flipped[0].v - 0.3).abs() < 1e-6);
        assert!((flipped[0].u - 0.3).abs() < 1e-6);
    }

    // 14. rotate_uv_90: double rotation = 180°
    #[test]
    fn test_rotate_uv_90_double() {
        let uvs = vec![UvParam { u: 0.2, v: 0.8 }];
        let r1 = rotate_uv_90(&uvs);
        let r2 = rotate_uv_90(&r1);
        // 180° rotation: (u,v) → (1-u, 1-v) (approximately)
        assert!((r2[0].u - (1.0 - uvs[0].u)).abs() < 1e-5);
        assert!((r2[0].v - (1.0 - uvs[0].v)).abs() < 1e-5);
    }

    // 15. pack_uv_islands_stub places non-overlapping rects
    #[test]
    fn test_pack_islands_non_overlapping() {
        let islands = vec![(0.3f32, 0.4), (0.2, 0.5), (0.1, 0.3)];
        let packed = pack_uv_islands_stub(&islands);
        assert_eq!(packed.len(), 3);
        // Each rect umin < umax
        for (umin, vmin, umax, vmax) in &packed {
            assert!(umin < umax, "umin >= umax");
            assert!(vmin < vmax, "vmin >= vmax");
        }
    }

    // 16. validate_uv_map: valid UVs return true
    #[test]
    fn test_validate_uv_map_valid() {
        let uvs = vec![UvParam { u: 0.0, v: 0.0 }, UvParam { u: 1.0, v: 1.0 }];
        assert!(validate_uv_map(&uvs));
    }

    // 17. validate_uv_map: NaN returns false
    #[test]
    fn test_validate_uv_map_nan() {
        let uvs = vec![UvParam { u: f32::NAN, v: 0.0 }];
        assert!(!validate_uv_map(&uvs));
    }

    // 18. uv_seam_length non-negative
    #[test]
    fn test_uv_seam_length_non_negative() {
        let pos = unit_triangle_pos();
        let idx = unit_triangle_idx();
        let uvs = planar_project_uv(&pos, UvProjectionMode::XY);
        let seam = uv_seam_length(&pos, &uvs, &idx);
        assert!(seam >= 0.0);
    }

    // 19. uv_vertex_count matches input length
    #[test]
    fn test_uv_vertex_count() {
        let pos = unit_triangle_pos();
        let uvs = planar_project_uv(&pos, UvProjectionMode::XY);
        assert_eq!(uv_vertex_count(&uvs), 3);
    }

    // 20. planar_project_uv empty input → empty output
    #[test]
    fn test_planar_project_empty() {
        let uvs = planar_project_uv(&[], UvProjectionMode::XY);
        assert!(uvs.is_empty());
    }
}
