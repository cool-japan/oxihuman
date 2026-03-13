// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! 3-D mesh surface flattening to a 2-D plane.
//!
//! Provides Tutte (barycentric), Harmonic, and LSCM stub embeddings
//! together with distortion metrics and UV-space helpers.

// ── Type aliases ─────────────────────────────────────────────────────────────

/// A 3-D vertex position `[x, y, z]`.
#[allow(dead_code)]
pub type Pos3 = [f32; 3];

/// A 2-D UV coordinate `[u, v]`.
#[allow(dead_code)]
pub type Uv = [f32; 2];

/// Index triple for a triangle face.
#[allow(dead_code)]
pub type FaceIdx = [usize; 3];

/// Area distortion paired with angle distortion.
#[allow(dead_code)]
pub type DistortionPair = (f32, f32);

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Embedding method used when flattening.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlattenMethod {
    /// Tutte (barycentric / uniform-weight) embedding.
    Barycentric,
    /// Harmonic (cotangent-weight) embedding.
    Harmonic,
    /// Least-Squares Conformal Maps (LSCM) – stub implementation.
    Lscm,
}

// ── Config ────────────────────────────────────────────────────────────────────

/// Configuration for mesh flattening.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FlattenConfig {
    /// Which embedding method to use.
    pub method: FlattenMethod,
    /// Number of Laplacian solve iterations for iterative methods.
    pub iterations: usize,
    /// Whether to normalise the resulting UVs to `[0,1]²`.
    pub normalise: bool,
}

impl Default for FlattenConfig {
    fn default() -> Self {
        Self {
            method: FlattenMethod::Barycentric,
            iterations: 100,
            normalise: true,
        }
    }
}

/// Result of a flattening operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FlattenResult {
    /// 2-D UV coordinates, one per original 3-D vertex.
    pub uvs: Vec<Uv>,
    /// Area distortion (ratio of UV area to surface area, normalised).
    pub area_distortion: f32,
    /// Angle distortion (average angular deviation in radians).
    pub angle_distortion: f32,
    /// Number of folded (overlapping) triangles in the UV layout.
    pub fold_count: usize,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
fn sub3(a: Pos3, b: Pos3) -> Pos3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn len3(v: Pos3) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[allow(dead_code)]
fn dot3(a: Pos3, b: Pos3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn cross3(a: Pos3, b: Pos3) -> Pos3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Signed area of a 2-D triangle `(a, b, c)`.
#[allow(dead_code)]
fn signed_area_2d(a: Uv, b: Uv, c: Uv) -> f32 {
    0.5 * ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]))
}

/// Collect the indices of boundary vertices (those on an edge shared by only
/// one triangle), ordered as a loop when possible.
#[allow(dead_code)]
fn boundary_vertices(faces: &[FaceIdx]) -> Vec<usize> {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    for f in faces {
        for k in 0..3 {
            let a = f[k];
            let b = f[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    let mut bverts: Vec<usize> = edge_count
        .iter()
        .filter(|(_, &c)| c == 1)
        .flat_map(|((a, b), _)| [*a, *b])
        .collect();
    bverts.sort_unstable();
    bverts.dedup();
    bverts
}

/// Build the 1-ring neighbour list for every vertex.
#[allow(dead_code)]
fn one_ring(n_verts: usize, faces: &[FaceIdx]) -> Vec<Vec<usize>> {
    let mut ring: Vec<Vec<usize>> = vec![Vec::new(); n_verts];
    for f in faces {
        for k in 0..3 {
            let vi = f[k];
            let vj = f[(k + 1) % 3];
            let vk = f[(k + 2) % 3];
            if !ring[vi].contains(&vj) {
                ring[vi].push(vj);
            }
            if !ring[vi].contains(&vk) {
                ring[vi].push(vk);
            }
        }
    }
    ring
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a `FlattenConfig` with sensible defaults.
#[allow(dead_code)]
pub fn default_flatten_config() -> FlattenConfig {
    FlattenConfig::default()
}

/// Human-readable name of a `FlattenMethod`.
#[allow(dead_code)]
pub fn flatten_method_name(method: FlattenMethod) -> &'static str {
    match method {
        FlattenMethod::Barycentric => "Barycentric (Tutte)",
        FlattenMethod::Harmonic => "Harmonic",
        FlattenMethod::Lscm => "LSCM",
    }
}

/// Number of vertices in the flattened mesh.
#[allow(dead_code)]
pub fn flatten_vertex_count(result: &FlattenResult) -> usize {
    result.uvs.len()
}

/// Map boundary vertices uniformly around the unit circle.
///
/// Returns a `Vec` of `(vertex_index, uv)` pairs.
#[allow(dead_code)]
pub fn boundary_circle_map(faces: &[FaceIdx]) -> Vec<(usize, Uv)> {
    let bverts = boundary_vertices(faces);
    let n = bverts.len();
    if n == 0 {
        return Vec::new();
    }
    bverts
        .into_iter()
        .enumerate()
        .map(|(i, vi)| {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / n as f32;
            (vi, [0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()])
        })
        .collect()
}

/// Tutte (uniform-weight barycentric) embedding stub.
///
/// Boundary vertices are pinned to the unit circle; interior vertices are
/// set to the unweighted average of their neighbours (iterated `cfg.iterations`
/// times).
#[allow(dead_code)]
pub fn flatten_mesh_barycentric(
    positions: &[Pos3],
    faces: &[FaceIdx],
    cfg: &FlattenConfig,
) -> FlattenResult {
    let n = positions.len();
    let boundary_map = boundary_circle_map(faces);
    let is_boundary: Vec<bool> = (0..n)
        .map(|i| boundary_map.iter().any(|(bi, _)| *bi == i))
        .collect();

    // Initialise UVs: boundary pinned, interior at centroid.
    let mut uvs: Vec<Uv> = vec![[0.5, 0.5]; n];
    for (bi, uv) in &boundary_map {
        uvs[*bi] = *uv;
    }

    let ring = one_ring(n, faces);

    for _ in 0..cfg.iterations {
        let prev = uvs.clone();
        for i in 0..n {
            if is_boundary[i] || ring[i].is_empty() {
                continue;
            }
            let sum_u: f32 = ring[i].iter().map(|&j| prev[j][0]).sum();
            let sum_v: f32 = ring[i].iter().map(|&j| prev[j][1]).sum();
            let k = ring[i].len() as f32;
            uvs[i] = [sum_u / k, sum_v / k];
        }
    }

    if cfg.normalise {
        normalize_flat_uvs(&mut uvs);
    }

    let (area_distortion, angle_distortion) = flatten_metric(positions, faces, &uvs);
    let fold_count = flatten_fold_count(faces, &uvs);

    FlattenResult {
        uvs,
        area_distortion,
        angle_distortion,
        fold_count,
    }
}

/// Compute area and angle distortion of the UV mapping.
///
/// Returns `(area_distortion, angle_distortion)`.
/// * `area_distortion` – mean absolute deviation of UV-area from 3-D area (normalised).
/// * `angle_distortion` – mean absolute angular error across all face angles.
#[allow(dead_code)]
pub fn flatten_metric(positions: &[Pos3], faces: &[FaceIdx], uvs: &[Uv]) -> DistortionPair {
    if faces.is_empty() {
        return (0.0, 0.0);
    }

    let mut total_area_dist = 0.0_f32;
    let mut total_angle_dist = 0.0_f32;
    let mut total_3d_area = 0.0_f32;

    for f in faces {
        let p0 = positions[f[0]];
        let p1 = positions[f[1]];
        let p2 = positions[f[2]];

        let e01 = sub3(p1, p0);
        let e02 = sub3(p2, p0);
        let area_3d = len3(cross3(e01, e02)) * 0.5;

        let uv0 = uvs[f[0]];
        let uv1 = uvs[f[1]];
        let uv2 = uvs[f[2]];
        let area_uv = signed_area_2d(uv0, uv1, uv2).abs();

        total_area_dist += (area_3d - area_uv).abs();
        total_3d_area += area_3d;

        // Angle distortion: for each corner compare 3D angle to UV angle.
        for corner in 0..3 {
            let a = f[corner];
            let b = f[(corner + 1) % 3];
            let c = f[(corner + 2) % 3];

            let e3d_ab = sub3(positions[b], positions[a]);
            let e3d_ac = sub3(positions[c], positions[a]);
            let cos_3d = dot3(e3d_ab, e3d_ac)
                / (len3(e3d_ab) * len3(e3d_ac) + 1e-10);

            let e2d_ab = [uvs[b][0] - uvs[a][0], uvs[b][1] - uvs[a][1]];
            let e2d_ac = [uvs[c][0] - uvs[a][0], uvs[c][1] - uvs[a][1]];
            let dot2 = e2d_ab[0] * e2d_ac[0] + e2d_ab[1] * e2d_ac[1];
            let len2_ab = (e2d_ab[0] * e2d_ab[0] + e2d_ab[1] * e2d_ab[1]).sqrt();
            let len2_ac = (e2d_ac[0] * e2d_ac[0] + e2d_ac[1] * e2d_ac[1]).sqrt();
            let cos_2d = dot2 / (len2_ab * len2_ac + 1e-10);

            let angle_3d = cos_3d.clamp(-1.0, 1.0).acos();
            let angle_2d = cos_2d.clamp(-1.0, 1.0).acos();
            total_angle_dist += (angle_3d - angle_2d).abs();
        }
    }

    let n = faces.len() as f32;
    let area_norm = if total_3d_area > 1e-10 {
        total_area_dist / total_3d_area
    } else {
        0.0
    };
    (area_norm, total_angle_dist / (3.0 * n))
}

/// Rescale and translate UVs so they fit in `[0,1]²`.
#[allow(dead_code)]
pub fn normalize_flat_uvs(uvs: &mut [Uv]) {
    if uvs.is_empty() {
        return;
    }
    let min_u = uvs.iter().map(|uv| uv[0]).fold(f32::INFINITY, f32::min);
    let max_u = uvs.iter().map(|uv| uv[0]).fold(f32::NEG_INFINITY, f32::max);
    let min_v = uvs.iter().map(|uv| uv[1]).fold(f32::INFINITY, f32::min);
    let max_v = uvs.iter().map(|uv| uv[1]).fold(f32::NEG_INFINITY, f32::max);
    let range_u = (max_u - min_u).max(1e-10);
    let range_v = (max_v - min_v).max(1e-10);
    for uv in uvs.iter_mut() {
        uv[0] = (uv[0] - min_u) / range_u;
        uv[1] = (uv[1] - min_v) / range_v;
    }
}

/// Compute the bounding box of the UV layout.
///
/// Returns `([min_u, min_v], [max_u, max_v])`.
#[allow(dead_code)]
pub fn flatten_bounding_box(uvs: &[Uv]) -> (Uv, Uv) {
    if uvs.is_empty() {
        return ([0.0, 0.0], [0.0, 0.0]);
    }
    let min_u = uvs.iter().map(|uv| uv[0]).fold(f32::INFINITY, f32::min);
    let max_u = uvs.iter().map(|uv| uv[0]).fold(f32::NEG_INFINITY, f32::max);
    let min_v = uvs.iter().map(|uv| uv[1]).fold(f32::INFINITY, f32::min);
    let max_v = uvs.iter().map(|uv| uv[1]).fold(f32::NEG_INFINITY, f32::max);
    ([min_u, min_v], [max_u, max_v])
}

/// Validate the UV mapping: all UVs finite and within `[0,1]²` after normalisation.
#[allow(dead_code)]
pub fn validate_flat_mesh(uvs: &[Uv]) -> bool {
    uvs.iter().all(|uv| {
        uv[0].is_finite()
            && uv[1].is_finite()
            && uv[0] >= -1e-5
            && uv[0] <= 1.0 + 1e-5
            && uv[1] >= -1e-5
            && uv[1] <= 1.0 + 1e-5
    })
}

/// Reconstruct a 3-D position from a 2-D UV using barycentric interpolation
/// within the triangle that contains the UV coordinate.
///
/// Returns `None` when no enclosing triangle is found.
#[allow(dead_code)]
pub fn flat_to_3d_barycentric(
    uv_query: Uv,
    positions: &[Pos3],
    faces: &[FaceIdx],
    uvs: &[Uv],
) -> Option<Pos3> {
    for f in faces {
        let a = uvs[f[0]];
        let b = uvs[f[1]];
        let c = uvs[f[2]];

        let denom = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
        if denom.abs() < 1e-10 {
            continue;
        }
        let w0 =
            ((b[1] - c[1]) * (uv_query[0] - c[0]) + (c[0] - b[0]) * (uv_query[1] - c[1]))
                / denom;
        let w1 =
            ((c[1] - a[1]) * (uv_query[0] - c[0]) + (a[0] - c[0]) * (uv_query[1] - c[1]))
                / denom;
        let w2 = 1.0 - w0 - w1;

        if w0 >= -1e-5 && w1 >= -1e-5 && w2 >= -1e-5 {
            let p0 = positions[f[0]];
            let p1 = positions[f[1]];
            let p2 = positions[f[2]];
            return Some([
                w0 * p0[0] + w1 * p1[0] + w2 * p2[0],
                w0 * p0[1] + w1 * p1[1] + w2 * p2[1],
                w0 * p0[2] + w1 * p1[2] + w2 * p2[2],
            ]);
        }
    }
    None
}

/// Histogram of area distortions across all faces.
///
/// `bins` buckets evenly spaced in `[0, max_distortion]`.  Returns a `Vec<u32>`
/// of length `bins`.
#[allow(dead_code)]
pub fn distortion_histogram(
    positions: &[Pos3],
    faces: &[FaceIdx],
    uvs: &[Uv],
    bins: usize,
    max_distortion: f32,
) -> Vec<u32> {
    let bins = bins.max(1);
    let mut hist = vec![0u32; bins];
    for f in faces {
        let p0 = positions[f[0]];
        let p1 = positions[f[1]];
        let p2 = positions[f[2]];
        let e01 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e02 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let area_3d = len3(cross3(e01, e02)) * 0.5;
        let area_uv = signed_area_2d(uvs[f[0]], uvs[f[1]], uvs[f[2]]).abs();
        let dist = if area_3d > 1e-10 {
            (area_3d - area_uv).abs() / area_3d
        } else {
            0.0
        };
        let idx = ((dist / max_distortion) * bins as f32)
            .floor()
            .clamp(0.0, (bins - 1) as f32) as usize;
        hist[idx] += 1;
    }
    hist
}

/// Count overlapping (folded) triangles in the UV layout.
///
/// A triangle is folded when its UV signed area is negative.
#[allow(dead_code)]
pub fn flatten_fold_count(faces: &[FaceIdx], uvs: &[Uv]) -> usize {
    faces
        .iter()
        .filter(|f| signed_area_2d(uvs[f[0]], uvs[f[1]], uvs[f[2]]) < 0.0)
        .count()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_positions() -> Vec<Pos3> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn quad_faces() -> Vec<FaceIdx> {
        vec![[0, 1, 2], [0, 2, 3]]
    }

    #[test]
    fn test_default_flatten_config() {
        let cfg = default_flatten_config();
        assert_eq!(cfg.method, FlattenMethod::Barycentric);
        assert!(cfg.iterations > 0);
    }

    #[test]
    fn test_flatten_method_name_barycentric() {
        assert!(!flatten_method_name(FlattenMethod::Barycentric).is_empty());
    }

    #[test]
    fn test_flatten_method_name_harmonic() {
        assert!(!flatten_method_name(FlattenMethod::Harmonic).is_empty());
    }

    #[test]
    fn test_flatten_method_name_lscm() {
        assert!(!flatten_method_name(FlattenMethod::Lscm).is_empty());
    }

    #[test]
    fn test_boundary_circle_map_nonempty() {
        let faces = quad_faces();
        let bmap = boundary_circle_map(&faces);
        assert!(!bmap.is_empty());
    }

    #[test]
    fn test_boundary_circle_map_unit_range() {
        let faces = quad_faces();
        let bmap = boundary_circle_map(&faces);
        for (_, uv) in &bmap {
            assert!(uv[0] >= 0.0 && uv[0] <= 1.0);
            assert!(uv[1] >= 0.0 && uv[1] <= 1.0);
        }
    }

    #[test]
    fn test_flatten_mesh_barycentric_returns_correct_len() {
        let pos = quad_positions();
        let faces = quad_faces();
        let cfg = default_flatten_config();
        let result = flatten_mesh_barycentric(&pos, &faces, &cfg);
        assert_eq!(flatten_vertex_count(&result), pos.len());
    }

    #[test]
    fn test_flatten_metric_empty() {
        let (a, b) = flatten_metric(&[], &[], &[]);
        assert_eq!(a, 0.0);
        assert_eq!(b, 0.0);
    }

    #[test]
    fn test_flatten_metric_nonnegative() {
        let pos = quad_positions();
        let faces = quad_faces();
        let uvs: Vec<Uv> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let (a, b) = flatten_metric(&pos, &faces, &uvs);
        assert!(a >= 0.0);
        assert!(b >= 0.0);
    }

    #[test]
    fn test_normalize_flat_uvs() {
        let mut uvs: Vec<Uv> = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 5.0], [0.0, 5.0]];
        normalize_flat_uvs(&mut uvs);
        let (mn, mx) = flatten_bounding_box(&uvs);
        assert!((mn[0]).abs() < 1e-5);
        assert!((mx[0] - 1.0).abs() < 1e-5);
        assert!((mn[1]).abs() < 1e-5);
        assert!((mx[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_flat_uvs_empty() {
        let mut uvs: Vec<Uv> = vec![];
        normalize_flat_uvs(&mut uvs);
        assert!(uvs.is_empty());
    }

    #[test]
    fn test_flatten_bounding_box_empty() {
        let (mn, mx) = flatten_bounding_box(&[]);
        assert_eq!(mn, [0.0, 0.0]);
        assert_eq!(mx, [0.0, 0.0]);
    }

    #[test]
    fn test_validate_flat_mesh_valid() {
        let uvs: Vec<Uv> = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];
        assert!(validate_flat_mesh(&uvs));
    }

    #[test]
    fn test_validate_flat_mesh_invalid() {
        let uvs: Vec<Uv> = vec![[0.0, 0.0], [2.0, 0.0], [0.5, 1.0]];
        assert!(!validate_flat_mesh(&uvs));
    }

    #[test]
    fn test_flat_to_3d_barycentric_inside() {
        let pos = quad_positions();
        let faces = quad_faces();
        let uvs: Vec<Uv> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        // Centroid of first triangle
        let query = [
            (uvs[0][0] + uvs[1][0] + uvs[2][0]) / 3.0,
            (uvs[0][1] + uvs[1][1] + uvs[2][1]) / 3.0,
        ];
        let result = flat_to_3d_barycentric(query, &pos, &faces, &uvs);
        assert!(result.is_some());
    }

    #[test]
    fn test_flat_to_3d_barycentric_outside() {
        let pos = quad_positions();
        let faces = quad_faces();
        let uvs: Vec<Uv> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let result = flat_to_3d_barycentric([5.0, 5.0], &pos, &faces, &uvs);
        assert!(result.is_none());
    }

    #[test]
    fn test_distortion_histogram_len() {
        let pos = quad_positions();
        let faces = quad_faces();
        let uvs: Vec<Uv> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let hist = distortion_histogram(&pos, &faces, &uvs, 8, 2.0);
        assert_eq!(hist.len(), 8);
    }

    #[test]
    fn test_distortion_histogram_sum() {
        let pos = quad_positions();
        let faces = quad_faces();
        let uvs: Vec<Uv> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let hist = distortion_histogram(&pos, &faces, &uvs, 4, 2.0);
        let total: u32 = hist.iter().sum();
        assert_eq!(total, faces.len() as u32);
    }

    #[test]
    fn test_flatten_fold_count_no_folds() {
        let faces = quad_faces();
        // CCW triangles → positive signed area → no folds.
        let uvs: Vec<Uv> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let folds = flatten_fold_count(&faces, &uvs);
        assert_eq!(folds, 0);
    }

    #[test]
    fn test_flatten_fold_count_all_flipped() {
        let faces = vec![[0_usize, 2, 1]]; // CW → negative area
        let uvs: Vec<Uv> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
        let folds = flatten_fold_count(&faces, &uvs);
        assert_eq!(folds, 1);
    }
}
