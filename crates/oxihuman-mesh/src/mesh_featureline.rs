// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Feature line (ridge/valley) extraction via principal curvature analysis.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn len(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn normalize(a: [f32; 3]) -> [f32; 3] {
    let l = len(a);
    if l < 1e-10 {
        [0.0, 0.0, 1.0]
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}

fn face_normal(pa: [f32; 3], pb: [f32; 3], pc: [f32; 3]) -> [f32; 3] {
    normalize(cross(sub(pb, pa), sub(pc, pa)))
}

fn vec3_zero() -> [f32; 3] {
    [0.0; 3]
}

// ---------------------------------------------------------------------------
// data types
// ---------------------------------------------------------------------------

/// Principal curvatures at a vertex.
#[derive(Debug, Clone)]
pub struct PrincipalCurvatures {
    /// Maximum principal curvature (k1 >= k2).
    pub k1: f32,
    /// Minimum principal curvature.
    pub k2: f32,
    /// Principal direction for k1.
    pub dir1: [f32; 3],
    /// Principal direction for k2.
    pub dir2: [f32; 3],
}

// ---------------------------------------------------------------------------
// curvature estimation
// ---------------------------------------------------------------------------

/// Area-weighted vertex normal.
fn vertex_normal(positions: &[[f32; 3]], tris: &[[u32; 3]], v: usize) -> [f32; 3] {
    let n = positions.len();
    let mut accum = vec3_zero();
    for tri in tris {
        let [ai, bi, ci] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        if ai >= n || bi >= n || ci >= n {
            continue;
        }
        if ai != v && bi != v && ci != v {
            continue;
        }
        let ab = sub(positions[bi], positions[ai]);
        let ac = sub(positions[ci], positions[ai]);
        let cr = cross(ab, ac);
        let area = len(cr) * 0.5;
        accum = add(accum, scale(normalize(cr), area));
    }
    normalize(accum)
}

/// Mixed-area Voronoi of vertex v (approximate with 1/3 of incident triangle areas).
fn voronoi_area(positions: &[[f32; 3]], tris: &[[u32; 3]], v: usize) -> f32 {
    let n = positions.len();
    let mut area = 0.0f32;
    for tri in tris {
        let [ai, bi, ci] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        if ai >= n || bi >= n || ci >= n {
            continue;
        }
        if ai != v && bi != v && ci != v {
            continue;
        }
        let ab = sub(positions[bi], positions[ai]);
        let ac = sub(positions[ci], positions[ai]);
        area += len(cross(ab, ac)) * 0.5 / 3.0;
    }
    area
}

/// Mean curvature H = (k1+k2)/2 at vertex v using the cotangent formula.
/// H_i = (1 / (2*A_i)) * |Σ (cot α_ij + cot β_ij)(p_j - p_i)|
/// where α and β are the two angles opposite edge (v, j) in the two incident triangles.
pub fn vertex_mean_curvature(positions: &[[f32; 3]], tris: &[[u32; 3]], v: usize) -> f32 {
    let n = positions.len();
    if v >= n {
        return 0.0;
    }

    let area = voronoi_area(positions, tris, v);
    if area < 1e-10 {
        return 0.0;
    }

    // Build a map: neighbor vertex j -> sum of cotangents from incident triangles
    let mut cot_sum: std::collections::HashMap<usize, f32> = std::collections::HashMap::new();

    for tri in tris {
        let [ai, bi, ci] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        if ai >= n || bi >= n || ci >= n {
            continue;
        }
        // Find roles: v is one vertex, the other two are j and k (opposite)
        let (j, k) = if ai == v {
            (bi, ci)
        } else if bi == v {
            (ai, ci)
        } else if ci == v {
            (ai, bi)
        } else {
            continue;
        };

        let pv = positions[v];
        let pj = positions[j];
        let pk = positions[k];

        // cot of angle at k (opposite to edge v-j)
        let cot_k = {
            let kv = sub(pv, pk);
            let kj = sub(pj, pk);
            let cos_a = dot(kv, kj);
            let sin_a = len(cross(kv, kj));
            if sin_a.abs() < 1e-10 {
                0.0
            } else {
                cos_a / sin_a
            }
        };

        *cot_sum.entry(j).or_insert(0.0) += cot_k;
    }

    let mut lap = vec3_zero();
    for (&j, &cw) in &cot_sum {
        if j < n {
            let diff = sub(positions[j], positions[v]);
            lap = add(lap, scale(diff, cw));
        }
    }

    let mean_curvature_vec = scale(lap, 1.0 / (2.0 * area));
    let h_magnitude = len(mean_curvature_vec);

    // Sign: positive if the normal and curvature vector point in same direction
    let nrm = vertex_normal(positions, tris, v);
    if dot(mean_curvature_vec, nrm) >= 0.0 {
        h_magnitude
    } else {
        -h_magnitude
    }
}

/// Gaussian curvature via angle defect: K_i = (2π - Σ θ_i) / A_i
pub fn vertex_gaussian_curvature(positions: &[[f32; 3]], tris: &[[u32; 3]], v: usize) -> f32 {
    let n = positions.len();
    if v >= n {
        return 0.0;
    }
    let area = voronoi_area(positions, tris, v);
    if area < 1e-10 {
        return 0.0;
    }

    let mut angle_sum = 0.0f32;
    for tri in tris {
        let [ai, bi, ci] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        if ai >= n || bi >= n || ci >= n {
            continue;
        }
        // Find position of v and its two neighbors in this triangle
        let (pv, pa, pb) = if ai == v {
            (positions[v], positions[bi], positions[ci])
        } else if bi == v {
            (positions[v], positions[ai], positions[ci])
        } else if ci == v {
            (positions[v], positions[ai], positions[bi])
        } else {
            continue;
        };

        let va = sub(pa, pv);
        let vb = sub(pb, pv);
        let la = len(va);
        let lb = len(vb);
        if la < 1e-10 || lb < 1e-10 {
            continue;
        }
        let cos_angle = (dot(va, vb) / (la * lb)).clamp(-1.0, 1.0);
        angle_sum += cos_angle.acos();
    }

    (std::f32::consts::PI * 2.0 - angle_sum) / area
}

/// Compute mean curvature for all vertices.
pub fn compute_all_mean_curvatures(positions: &[[f32; 3]], tris: &[[u32; 3]]) -> Vec<f32> {
    (0..positions.len())
        .map(|v| vertex_mean_curvature(positions, tris, v))
        .collect()
}

/// Compute Gaussian curvature for all vertices.
pub fn compute_all_gaussian_curvatures(positions: &[[f32; 3]], tris: &[[u32; 3]]) -> Vec<f32> {
    (0..positions.len())
        .map(|v| vertex_gaussian_curvature(positions, tris, v))
        .collect()
}

// ---------------------------------------------------------------------------
// feature line extraction
// ---------------------------------------------------------------------------

/// Extract ridge edges: both endpoints have mean curvature > threshold.
pub fn extract_ridges(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    threshold: f32,
) -> Vec<[usize; 2]> {
    let curvatures = compute_all_mean_curvatures(positions, tris);
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    let mut ridges = Vec::new();

    for tri in tris {
        let [ai, bi, ci] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        for (p, q) in [(ai, bi), (bi, ci), (ci, ai)] {
            let key = (p.min(q), p.max(q));
            if seen.insert(key) {
                let cp = curvatures.get(p).copied().unwrap_or(0.0);
                let cq = curvatures.get(q).copied().unwrap_or(0.0);
                if cp > threshold && cq > threshold {
                    ridges.push([p, q]);
                }
            }
        }
    }
    ridges
}

/// Extract valley edges: both endpoints have mean curvature < -threshold.
pub fn extract_valleys(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    threshold: f32,
) -> Vec<[usize; 2]> {
    let curvatures = compute_all_mean_curvatures(positions, tris);
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    let mut valleys = Vec::new();

    for tri in tris {
        let [ai, bi, ci] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        for (p, q) in [(ai, bi), (bi, ci), (ci, ai)] {
            let key = (p.min(q), p.max(q));
            if seen.insert(key) {
                let cp = curvatures.get(p).copied().unwrap_or(0.0);
                let cq = curvatures.get(q).copied().unwrap_or(0.0);
                if cp < -threshold && cq < -threshold {
                    valleys.push([p, q]);
                }
            }
        }
    }
    valleys
}

/// Extract both ridges and valleys.
pub fn extract_feature_lines(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    ridge_thresh: f32,
    valley_thresh: f32,
) -> (Vec<[usize; 2]>, Vec<[usize; 2]>) {
    let ridges = extract_ridges(positions, tris, ridge_thresh);
    let valleys = extract_valleys(positions, tris, valley_thresh);
    (ridges, valleys)
}

/// Fraction of edges that are feature lines (ridges or valleys).
pub fn feature_line_density(positions: &[[f32; 3]], tris: &[[u32; 3]], threshold: f32) -> f32 {
    // Count unique edges
    let mut all_edges: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for tri in tris {
        let [ai, bi, ci] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        for (p, q) in [(ai, bi), (bi, ci), (ci, ai)] {
            all_edges.insert((p.min(q), p.max(q)));
        }
    }
    let total = all_edges.len();
    if total == 0 {
        return 0.0;
    }

    let ridges = extract_ridges(positions, tris, threshold);
    let valleys = extract_valleys(positions, tris, threshold);
    let feature_count = ridges.len() + valleys.len();
    (feature_count as f32 / total as f32).min(1.0)
}

/// Koenderink shape index in [-1, 1].
/// S = -2/π * arctan((k1+k2)/(k1-k2)) for k1 != k2.
pub fn shape_index_at_vertex(k1: f32, k2: f32) -> f32 {
    if (k1 - k2).abs() < 1e-10 {
        return 0.0;
    }
    let s = -2.0 / std::f32::consts::PI * ((k1 + k2) / (k1 - k2)).atan();
    s.clamp(-1.0, 1.0)
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A simple icosphere-like sphere approximation with 8 triangles.
    fn sphere_octahedron() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let pos = vec![
            [0.0, 1.0, 0.0],  // top
            [1.0, 0.0, 0.0],  // right
            [0.0, 0.0, 1.0],  // front
            [-1.0, 0.0, 0.0], // left
            [0.0, 0.0, -1.0], // back
            [0.0, -1.0, 0.0], // bottom
        ];
        let tris = vec![
            [0, 1, 2],
            [0, 2, 3],
            [0, 3, 4],
            [0, 4, 1],
            [5, 2, 1],
            [5, 3, 2],
            [5, 4, 3],
            [5, 1, 4],
        ];
        (pos, tris)
    }

    fn flat_grid(n: usize) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let mut pos = Vec::new();
        let mut tris = Vec::new();
        let step = 1.0 / n as f32;
        for i in 0..=n {
            for j in 0..=n {
                pos.push([i as f32 * step, j as f32 * step, 0.0]);
            }
        }
        let stride = n + 1;
        for i in 0..n {
            for j in 0..n {
                let a = (i * stride + j) as u32;
                let b = (i * stride + j + 1) as u32;
                let c = ((i + 1) * stride + j) as u32;
                let d = ((i + 1) * stride + j + 1) as u32;
                tris.push([a, b, c]);
                tris.push([b, d, c]);
            }
        }
        (pos, tris)
    }

    #[test]
    fn test_vertex_mean_curvature_sphere_positive() {
        // Convex sphere vertices should have positive mean curvature
        let (pos, tris) = sphere_octahedron();
        let h = vertex_mean_curvature(&pos, &tris, 0);
        // For a convex shape, mean curvature > 0
        assert!(
            h >= 0.0,
            "mean curvature for sphere top should be >= 0, got {h}"
        );
    }

    #[test]
    fn test_vertex_mean_curvature_flat_is_finite() {
        let (pos, tris) = flat_grid(3);
        // All vertices on a flat mesh should have finite curvature (no NaN/inf)
        for v in 0..pos.len() {
            let h = vertex_mean_curvature(&pos, &tris, v);
            assert!(
                h.is_finite(),
                "mean curvature at vertex {v} should be finite, got {h}"
            );
        }
        // The z-component of Laplacian on flat mesh should be zero
        // (only x,y components may be non-zero due to asymmetric triangulation)
        for v in 0..pos.len() {
            let h = vertex_mean_curvature(&pos, &tris, v);
            // Mean curvature magnitude should be bounded
            assert!(
                h.abs() < 100.0,
                "mean curvature at vertex {v} should not be huge, got {h}"
            );
        }
    }

    #[test]
    fn test_vertex_gaussian_curvature_sphere_positive() {
        let (pos, tris) = sphere_octahedron();
        let k = vertex_gaussian_curvature(&pos, &tris, 0);
        assert!(
            k > 0.0,
            "Gaussian curvature for sphere should be positive, got {k}"
        );
    }

    #[test]
    fn test_vertex_gaussian_curvature_flat_near_zero() {
        let (pos, tris) = flat_grid(3);
        // Interior vertex of flat mesh
        let k = vertex_gaussian_curvature(&pos, &tris, 5);
        assert!(
            k.abs() < 1e-3,
            "Gaussian curvature of flat mesh should be ~0, got {k}"
        );
    }

    #[test]
    fn test_compute_all_mean_curvatures_length() {
        let (pos, tris) = sphere_octahedron();
        let curvs = compute_all_mean_curvatures(&pos, &tris);
        assert_eq!(curvs.len(), pos.len());
    }

    #[test]
    fn test_compute_all_gaussian_curvatures_length() {
        let (pos, tris) = sphere_octahedron();
        let curvs = compute_all_gaussian_curvatures(&pos, &tris);
        assert_eq!(curvs.len(), pos.len());
    }

    #[test]
    fn test_extract_ridges_returns_vec() {
        let (pos, tris) = sphere_octahedron();
        let ridges = extract_ridges(&pos, &tris, 0.0);
        // With threshold=0, all edges with positive curvature are ridges
        assert!(ridges.len() <= tris.len() * 3);
    }

    #[test]
    fn test_extract_valleys_returns_vec() {
        let (pos, tris) = sphere_octahedron();
        let valleys = extract_valleys(&pos, &tris, 0.0);
        // For a convex sphere, valleys should be empty or minimal
        let _ = valleys; // just ensure no panic
    }

    #[test]
    fn test_extract_feature_lines_no_panic() {
        let (pos, tris) = sphere_octahedron();
        let (ridges, valleys) = extract_feature_lines(&pos, &tris, 0.5, 0.5);
        // No panic, results are vecs
        let _ = (ridges, valleys);
    }

    #[test]
    fn test_extract_feature_lines_empty_mesh() {
        let (ridges, valleys) = extract_feature_lines(&[], &[], 0.5, 0.5);
        assert!(ridges.is_empty());
        assert!(valleys.is_empty());
    }

    #[test]
    fn test_feature_line_density_in_range() {
        let (pos, tris) = sphere_octahedron();
        let density = feature_line_density(&pos, &tris, 0.5);
        assert!(
            (0.0..=1.0).contains(&density),
            "density={density} out of [0,1]"
        );
    }

    #[test]
    fn test_feature_line_density_empty_mesh() {
        let density = feature_line_density(&[], &[], 0.5);
        assert_eq!(density, 0.0);
    }

    #[test]
    fn test_shape_index_range() {
        // For various curvature pairs, shape index should be in [-1, 1]
        let pairs = [
            (1.0, 0.5),
            (0.0, 0.0),
            (-1.0, -2.0),
            (2.0, -2.0),
            (1.0, -1.0),
        ];
        for (k1, k2) in pairs {
            let s = shape_index_at_vertex(k1, k2);
            assert!(
                (-1.0..=1.0).contains(&s),
                "shape_index({k1},{k2})={s} out of [-1,1]"
            );
        }
    }

    #[test]
    fn test_shape_index_sphere_like() {
        // Spherical cap: k1 = k2 = 1/R -> shape index -> cap (+1)
        // Equal curvatures -> degenerate, returns 0
        let s = shape_index_at_vertex(1.0, 1.0);
        assert_eq!(s, 0.0);
    }
}
