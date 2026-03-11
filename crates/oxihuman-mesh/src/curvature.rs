// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use crate::mesh::MeshBuffers;

// ─── vector helpers ───────────────────────────────────────────────────────────

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Compute the angle at vertex `a` in the triangle (a, b, c).
#[inline]
fn angle_at(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ba = sub3(b, a);
    let ca = sub3(c, a);
    let len_ba = len3(ba);
    let len_ca = len3(ca);
    if len_ba < 1e-12 || len_ca < 1e-12 {
        return 0.0;
    }
    let cos_a = (dot3(ba, ca) / (len_ba * len_ca)).clamp(-1.0, 1.0);
    cos_a.acos()
}

// ─── adjacency helpers ────────────────────────────────────────────────────────

/// Build per-vertex list of incident triangle indices (each entry = face index).
fn build_vertex_faces(n_verts: usize, indices: &[u32]) -> Vec<Vec<usize>> {
    let mut vf: Vec<Vec<usize>> = vec![vec![]; n_verts];
    for (fi, tri) in indices.chunks_exact(3).enumerate() {
        for &vi in tri {
            let v = vi as usize;
            if v < n_verts {
                vf[v].push(fi);
            }
        }
    }
    vf
}

/// Build per-vertex neighbour list (unique neighbour vertex indices).
fn build_adjacency(n_verts: usize, indices: &[u32]) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> =
        vec![std::collections::HashSet::new(); n_verts];
    for tri in indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= n_verts || i1 >= n_verts || i2 >= n_verts {
            continue;
        }
        adj[i0].insert(i1);
        adj[i0].insert(i2);
        adj[i1].insert(i0);
        adj[i1].insert(i2);
        adj[i2].insert(i0);
        adj[i2].insert(i1);
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

// ─── public types ─────────────────────────────────────────────────────────────

/// Per-vertex curvature values.
#[derive(Debug, Clone)]
pub struct VertexCurvature {
    /// Mean curvature H = (k1 + k2) / 2.
    pub mean: f32,
    /// Gaussian curvature K = k1 * k2.
    pub gaussian: f32,
    /// Principal curvature k1 (larger).
    pub k1: f32,
    /// Principal curvature k2 (smaller).
    pub k2: f32,
    /// Shape index [-1, 1]: characterizes local shape type.
    pub shape_index: f32,
    /// Curvedness (magnitude): sqrt((k1²+k2²)/2).
    pub curvedness: f32,
}

/// Curvature statistics for a mesh.
#[derive(Debug, Clone)]
pub struct CurvatureStats {
    pub mean_curvature_mean: f32,
    pub mean_curvature_max: f32,
    pub gaussian_curvature_mean: f32,
    /// Integral of Gaussian curvature over the surface.
    /// Should ≈ 4π for a closed genus-0 surface (Gauss–Bonnet theorem).
    pub gaussian_curvature_integral: f32,
}

// ─── mean curvature ───────────────────────────────────────────────────────────

/// Compute per-vertex mean curvature using the normal-deviation / Laplacian
/// approximation.
///
/// For each interior vertex i we compute the discrete Laplacian
/// `L_i = vertex_i - mean(neighbours)` and use H ≈ |L_i| / 2.
/// Boundary vertices (no neighbours) get H = 0.
pub fn compute_mean_curvature(mesh: &MeshBuffers) -> Vec<f32> {
    let n = mesh.positions.len();
    if n == 0 {
        return Vec::new();
    }
    let adj = build_adjacency(n, &mesh.indices);
    let mut result = vec![0.0f32; n];

    for (i, neighbours) in adj.iter().enumerate() {
        if neighbours.is_empty() {
            continue;
        }
        let pi = mesh.positions[i];
        // Centroid of neighbours
        let mut centroid = [0.0f32; 3];
        for &j in neighbours {
            centroid = add3(centroid, mesh.positions[j]);
        }
        centroid = scale3(centroid, 1.0 / neighbours.len() as f32);
        // Discrete Laplacian vector
        let laplacian = sub3(pi, centroid);
        result[i] = len3(laplacian) * 0.5;
    }
    result
}

// ─── Gaussian curvature ───────────────────────────────────────────────────────

/// Compute per-vertex Gaussian curvature using the angle-defect method.
///
/// K_i = (2π − Σ θ_j) / A_i
///
/// where θ_j is the interior angle at vertex i in adjacent face j, and A_i is
/// the sum of one-third of each adjacent triangle's area (mixed area).
pub fn compute_gaussian_curvature(mesh: &MeshBuffers) -> Vec<f32> {
    let n = mesh.positions.len();
    if n == 0 {
        return Vec::new();
    }
    let vf = build_vertex_faces(n, &mesh.indices);
    let mut result = vec![0.0f32; n];

    for (vi, faces) in vf.iter().enumerate() {
        if faces.is_empty() {
            continue;
        }
        let pi = mesh.positions[vi];
        let mut angle_sum = 0.0f32;
        let mut area_sum = 0.0f32;

        for &fi in faces {
            let base = fi * 3;
            if base + 2 >= mesh.indices.len() {
                continue;
            }
            let i0 = mesh.indices[base] as usize;
            let i1 = mesh.indices[base + 1] as usize;
            let i2 = mesh.indices[base + 2] as usize;
            if i0 >= n || i1 >= n || i2 >= n {
                continue;
            }
            let p0 = mesh.positions[i0];
            let p1 = mesh.positions[i1];
            let p2 = mesh.positions[i2];

            // Identify which vertex of the triangle is vi and compute angle there
            let angle = if vi == i0 {
                angle_at(p0, p1, p2)
            } else if vi == i1 {
                angle_at(p1, p0, p2)
            } else {
                angle_at(p2, p0, p1)
            };
            angle_sum += angle;

            // Triangle area = 0.5 * |cross(e1, e2)|
            let e1 = sub3(p1, p0);
            let e2 = sub3(p2, p0);
            let tri_area = len3(cross3(e1, e2)) * 0.5;
            // Each vertex "owns" 1/3 of the triangle's area
            area_sum += tri_area / 3.0;
        }

        if area_sum > 1e-12 {
            result[vi] = (std::f32::consts::TAU - angle_sum) / area_sum;
        }

        // Sanity guard: clamp to a reasonable range to avoid exploding values on
        // degenerate triangles near the boundary.
        result[vi] = result[vi].clamp(-1e6, 1e6);
        // Replace NaN with 0
        if result[vi].is_nan() {
            result[vi] = 0.0;
        }

        let _ = pi; // suppress unused warning
    }
    result
}

// ─── full curvature ───────────────────────────────────────────────────────────

/// Compute full per-vertex curvature: mean and Gaussian → principal curvatures,
/// shape index, and curvedness.
pub fn compute_curvature(mesh: &MeshBuffers) -> Vec<VertexCurvature> {
    let h_vec = compute_mean_curvature(mesh);
    let k_vec = compute_gaussian_curvature(mesh);

    h_vec
        .iter()
        .zip(k_vec.iter())
        .map(|(&h, &k)| {
            // discriminant = sqrt(max(H² - K, 0))
            let disc = (h * h - k).max(0.0).sqrt();
            let k1 = h + disc; // larger
            let k2 = h - disc; // smaller

            // Shape index: (2/π) * atan((k1+k2)/(k1-k2)) when k1 ≠ k2
            let shape_index = if (k1 - k2).abs() < 1e-10 {
                0.0f32
            } else {
                (2.0 / std::f32::consts::PI) * f32::atan2(k1 + k2, k1 - k2)
            };
            // Clamp to [-1, 1] just in case
            let shape_index = shape_index.clamp(-1.0, 1.0);

            let curvedness = ((k1 * k1 + k2 * k2) * 0.5).sqrt();

            VertexCurvature {
                mean: h,
                gaussian: k,
                k1,
                k2,
                shape_index,
                curvedness,
            }
        })
        .collect()
}

// ─── feature / topology queries ───────────────────────────────────────────────

/// Find vertex indices where curvedness (magnitude) exceeds `threshold`.
///
/// These are potential feature points such as ridges or sharp edges.
pub fn find_feature_vertices(mesh: &MeshBuffers, threshold: f32) -> Vec<usize> {
    compute_curvature(mesh)
        .iter()
        .enumerate()
        .filter_map(|(i, vc)| {
            if vc.curvedness > threshold {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

/// Find vertices that are saddle points (k1 > 0, k2 < 0, K < 0).
pub fn find_saddle_points(mesh: &MeshBuffers) -> Vec<usize> {
    compute_curvature(mesh)
        .iter()
        .enumerate()
        .filter_map(|(i, vc)| {
            if vc.k1 > 0.0 && vc.k2 < 0.0 {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

/// Find vertices that are local curvature peaks (local maxima of curvedness).
///
/// A vertex is a peak if its curvedness exceeds the curvedness of all its
/// neighbours.
pub fn find_curvature_peaks(mesh: &MeshBuffers) -> Vec<usize> {
    let n = mesh.positions.len();
    if n == 0 {
        return Vec::new();
    }
    let curvatures = compute_curvature(mesh);
    let adj = build_adjacency(n, &mesh.indices);

    curvatures
        .iter()
        .enumerate()
        .filter_map(|(i, vc)| {
            let neighbours = &adj[i];
            if neighbours.is_empty() {
                return None;
            }
            let is_peak = neighbours
                .iter()
                .all(|&j| vc.curvedness > curvatures[j].curvedness);
            if is_peak {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

// ─── statistics ───────────────────────────────────────────────────────────────

/// Compute curvature statistics for a mesh.
pub fn curvature_stats(mesh: &MeshBuffers) -> CurvatureStats {
    let curvatures = compute_curvature(mesh);
    if curvatures.is_empty() {
        return CurvatureStats {
            mean_curvature_mean: 0.0,
            mean_curvature_max: 0.0,
            gaussian_curvature_mean: 0.0,
            gaussian_curvature_integral: 0.0,
        };
    }

    // Per-vertex areas for integration (1/3 of each adjacent triangle)
    let n = mesh.positions.len();
    let vf = build_vertex_faces(n, &mesh.indices);
    let mut areas = vec![0.0f32; n];
    for (vi, faces) in vf.iter().enumerate() {
        for &fi in faces {
            let base = fi * 3;
            if base + 2 >= mesh.indices.len() {
                continue;
            }
            let i0 = mesh.indices[base] as usize;
            let i1 = mesh.indices[base + 1] as usize;
            let i2 = mesh.indices[base + 2] as usize;
            if i0 >= n || i1 >= n || i2 >= n {
                continue;
            }
            let p0 = mesh.positions[i0];
            let p1 = mesh.positions[i1];
            let p2 = mesh.positions[i2];
            let e1 = sub3(p1, p0);
            let e2 = sub3(p2, p0);
            let tri_area = len3(cross3(e1, e2)) * 0.5;
            areas[vi] += tri_area / 3.0;
        }
    }

    let count = curvatures.len() as f32;
    let mean_curvature_mean = curvatures.iter().map(|vc| vc.mean).sum::<f32>() / count;
    let mean_curvature_max = curvatures
        .iter()
        .map(|vc| vc.mean)
        .fold(f32::NEG_INFINITY, f32::max);
    let gaussian_curvature_mean = curvatures.iter().map(|vc| vc.gaussian).sum::<f32>() / count;

    // Gauss–Bonnet integral: Σ K_i * A_i
    let gaussian_curvature_integral = curvatures
        .iter()
        .zip(areas.iter())
        .map(|(vc, &a)| vc.gaussian * a)
        .sum::<f32>();

    CurvatureStats {
        mean_curvature_mean,
        mean_curvature_max,
        gaussian_curvature_mean,
        gaussian_curvature_integral,
    }
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── mesh builders ──────────────────────────────────────────────────────────

    /// Flat n×n grid in the XZ plane (y = 0).
    fn flat_grid(n: usize) -> MeshBuffers {
        let mut positions = Vec::new();
        for i in 0..n {
            for j in 0..n {
                positions.push([i as f32, 0.0, j as f32]);
            }
        }
        let mut indices = Vec::new();
        for i in 0..n - 1 {
            for j in 0..n - 1 {
                let base = (i * n + j) as u32;
                indices.push(base);
                indices.push(base + 1);
                indices.push(base + n as u32);
                indices.push(base + 1);
                indices.push(base + n as u32 + 1);
                indices.push(base + n as u32);
            }
        }
        let nv = positions.len();
        MeshBuffers {
            positions,
            normals: vec![[0.0, 1.0, 0.0]; nv],
            uvs: vec![[0.0, 0.0]; nv],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; nv],
            colors: None,
            indices,
            has_suit: false,
        }
    }

    /// Single triangle.
    fn single_triangle() -> MeshBuffers {
        MeshBuffers {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
            colors: None,
            indices: vec![0, 1, 2],
            has_suit: false,
        }
    }

    /// Rough icosphere-like closed surface approximation for Gauss–Bonnet test.
    /// We use the 6-face octahedron (8 triangles) for simplicity.
    fn octahedron() -> MeshBuffers {
        // Vertices at ±1 on each axis (unit octahedron)
        let positions: Vec<[f32; 3]> = vec![
            [1.0, 0.0, 0.0],  // 0 +X
            [-1.0, 0.0, 0.0], // 1 -X
            [0.0, 1.0, 0.0],  // 2 +Y
            [0.0, -1.0, 0.0], // 3 -Y
            [0.0, 0.0, 1.0],  // 4 +Z
            [0.0, 0.0, -1.0], // 5 -Z
        ];
        // 8 triangular faces
        #[rustfmt::skip]
        let indices: Vec<u32> = vec![
            0, 2, 4,  // +X +Y +Z
            0, 4, 3,  // +X +Z -Y
            0, 3, 5,  // +X -Y -Z
            0, 5, 2,  // +X -Z +Y
            1, 4, 2,  // -X +Z +Y
            1, 3, 4,  // -X -Y +Z
            1, 5, 3,  // -X -Z -Y
            1, 2, 5,  // -X +Y -Z
        ];
        let nv = positions.len();
        MeshBuffers {
            positions,
            normals: vec![[0.0, 1.0, 0.0]; nv],
            uvs: vec![[0.0, 0.0]; nv],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; nv],
            colors: None,
            indices,
            has_suit: false,
        }
    }

    // ── mean curvature tests ───────────────────────────────────────────────────

    #[test]
    fn mean_curvature_length_matches_vertex_count() {
        let mesh = flat_grid(5);
        let mc = compute_mean_curvature(&mesh);
        assert_eq!(mc.len(), mesh.positions.len());
    }

    #[test]
    fn mean_curvature_flat_mesh_near_zero() {
        // Interior vertices of a flat grid should have near-zero mean curvature.
        let mesh = flat_grid(7);
        let mc = compute_mean_curvature(&mesh);
        // Check a few interior vertices (e.g. row 3 col 3 → index 3*7+3 = 24)
        for &idx in &[24usize, 17, 18, 25] {
            assert!(
                mc[idx] < 0.05,
                "expected near-zero mean curvature at interior vertex {}, got {}",
                idx,
                mc[idx]
            );
        }
    }

    // ── Gaussian curvature tests ───────────────────────────────────────────────

    #[test]
    fn gaussian_curvature_length_matches() {
        let mesh = flat_grid(5);
        let gc = compute_gaussian_curvature(&mesh);
        assert_eq!(gc.len(), mesh.positions.len());
    }

    #[test]
    fn gaussian_curvature_flat_mesh_near_zero() {
        // Interior vertices of a flat grid: angle defect ≈ 0 → K ≈ 0.
        let mesh = flat_grid(7);
        let gc = compute_gaussian_curvature(&mesh);
        // Interior vertex: row=3, col=3 → index 3*7+3 = 24
        let k = gc[24];
        assert!(
            k.abs() < 0.5,
            "expected near-zero gaussian curvature at interior vertex 24, got {}",
            k
        );
    }

    // ── full curvature tests ───────────────────────────────────────────────────

    #[test]
    fn compute_curvature_k1_gte_k2() {
        let mesh = flat_grid(5);
        let cv = compute_curvature(&mesh);
        for (i, vc) in cv.iter().enumerate() {
            assert!(
                vc.k1 >= vc.k2 - 1e-6,
                "k1 ({}) must be >= k2 ({}) at vertex {}",
                vc.k1,
                vc.k2,
                i
            );
        }
    }

    #[test]
    fn compute_curvature_curvedness_nonneg() {
        let mesh = flat_grid(5);
        let cv = compute_curvature(&mesh);
        for (i, vc) in cv.iter().enumerate() {
            assert!(
                vc.curvedness >= 0.0,
                "curvedness must be non-negative at vertex {}, got {}",
                i,
                vc.curvedness
            );
        }
    }

    #[test]
    fn compute_curvature_no_nan() {
        let mesh = flat_grid(5);
        let cv = compute_curvature(&mesh);
        for (i, vc) in cv.iter().enumerate() {
            assert!(!vc.mean.is_nan(), "mean NaN at vertex {}", i);
            assert!(!vc.gaussian.is_nan(), "gaussian NaN at vertex {}", i);
            assert!(!vc.k1.is_nan(), "k1 NaN at vertex {}", i);
            assert!(!vc.k2.is_nan(), "k2 NaN at vertex {}", i);
            assert!(!vc.shape_index.is_nan(), "shape_index NaN at vertex {}", i);
            assert!(!vc.curvedness.is_nan(), "curvedness NaN at vertex {}", i);
        }
    }

    #[test]
    fn vertex_curvature_shape_index_in_range() {
        // Shape index must lie in [-1, 1].
        let mesh = flat_grid(5);
        let cv = compute_curvature(&mesh);
        for (i, vc) in cv.iter().enumerate() {
            assert!(
                vc.shape_index >= -1.0 - 1e-6 && vc.shape_index <= 1.0 + 1e-6,
                "shape_index out of [-1,1] at vertex {}: {}",
                i,
                vc.shape_index
            );
        }
    }

    // ── feature / topology query tests ────────────────────────────────────────

    #[test]
    fn find_feature_vertices_empty_for_flat() {
        // A flat mesh with a high threshold should yield no feature vertices.
        let mesh = flat_grid(5);
        let fv = find_feature_vertices(&mesh, 10.0);
        assert!(
            fv.is_empty(),
            "expected no feature vertices for flat mesh with high threshold, got {:?}",
            fv
        );
    }

    #[test]
    fn find_saddle_points_valid_indices() {
        let mesh = flat_grid(5);
        let sp = find_saddle_points(&mesh);
        let n = mesh.positions.len();
        for &idx in &sp {
            assert!(
                idx < n,
                "saddle point index {} out of bounds (n={})",
                idx,
                n
            );
        }
    }

    #[test]
    fn find_curvature_peaks_valid_indices() {
        let mesh = flat_grid(5);
        let peaks = find_curvature_peaks(&mesh);
        let n = mesh.positions.len();
        for &idx in &peaks {
            assert!(idx < n, "peak index {} out of bounds (n={})", idx, n);
        }
    }

    // ── statistics tests ───────────────────────────────────────────────────────

    #[test]
    fn curvature_stats_on_simple_mesh() {
        let mesh = single_triangle();
        let stats = curvature_stats(&mesh);
        // Just verify fields are finite (not NaN/Inf) for a simple mesh.
        assert!(stats.mean_curvature_mean.is_finite());
        assert!(
            stats.mean_curvature_max.is_finite() || stats.mean_curvature_max == f32::NEG_INFINITY
        );
        assert!(stats.gaussian_curvature_mean.is_finite());
        assert!(stats.gaussian_curvature_integral.is_finite());
    }

    #[test]
    fn gaussian_integral_rough_check() {
        // For a closed genus-0 surface, the Gauss–Bonnet theorem gives:
        //   ∫ K dA = 4π ≈ 12.566
        // We use an octahedron (genus-0 polyhedron). The discrete angle-defect
        // integral won't be exact but should be in the ballpark (e.g. within a
        // factor of 2 of 4π).
        let mesh = octahedron();
        let stats = curvature_stats(&mesh);
        let four_pi = 4.0 * std::f32::consts::PI; // ≈ 12.566
                                                  // Loose check: integral should be positive and < 8π
        assert!(
            stats.gaussian_curvature_integral > 0.0,
            "Gauss–Bonnet integral should be positive, got {}",
            stats.gaussian_curvature_integral
        );
        assert!(
            stats.gaussian_curvature_integral < four_pi * 2.0,
            "Gauss–Bonnet integral should be < 8π, got {}",
            stats.gaussian_curvature_integral
        );
    }
}
