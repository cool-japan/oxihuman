// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Surface reconstruction from point cloud using moving least squares (MLS).

/// Config for MLS reconstruction.
#[allow(dead_code)]
pub struct MlsReconstructConfig {
    pub search_radius: f32,
    pub polynomial_degree: u32,
    pub grid_resolution: u32,
}

#[allow(dead_code)]
impl Default for MlsReconstructConfig {
    fn default() -> Self {
        Self { search_radius: 0.1, polynomial_degree: 2, grid_resolution: 32 }
    }
}

/// Result of MLS reconstruction.
#[allow(dead_code)]
pub struct MlsReconstructResult {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub point_count: usize,
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0]-b[0], a[1]-b[1], a[2]-b[2]]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0]*b[0]+a[1]*b[1]+a[2]*b[2]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    dot3(v,v).sqrt()
}

/// Compute squared distance between two points.
#[allow(dead_code)]
pub fn sq_dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = sub3(a, b);
    dot3(d, d)
}

/// Gaussian weight for MLS.
#[allow(dead_code)]
pub fn gaussian_weight(dist_sq: f32, radius: f32) -> f32 {
    let h2 = radius * radius;
    (-dist_sq / h2).exp()
}

/// Find neighbours within radius.
#[allow(dead_code)]
pub fn find_neighbours(points: &[[f32; 3]], query: [f32; 3], radius: f32) -> Vec<usize> {
    let r2 = radius * radius;
    points.iter().enumerate()
        .filter(|(_, p)| sq_dist(query, **p) <= r2)
        .map(|(i, _)| i)
        .collect()
}

/// Compute the weighted centroid (MLS projection step).
#[allow(dead_code)]
pub fn weighted_centroid(points: &[[f32; 3]], weights: &[f32]) -> [f32; 3] {
    let total: f32 = weights.iter().sum();
    if total < 1e-10 {
        return [0.0; 3];
    }
    let mut sum = [0.0f32; 3];
    for (p, &w) in points.iter().zip(weights.iter()) {
        sum[0] += p[0] * w;
        sum[1] += p[1] * w;
        sum[2] += p[2] * w;
    }
    [sum[0]/total, sum[1]/total, sum[2]/total]
}

/// Compute a simple normal estimate via PCA (just a stub returning Z-up).
#[allow(dead_code)]
pub fn estimate_normal_stub(_points: &[[f32; 3]]) -> [f32; 3] {
    [0.0, 0.0, 1.0]
}

/// Project point onto local tangent plane.
#[allow(dead_code)]
pub fn project_to_plane(p: [f32; 3], centroid: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    let d = sub3(p, centroid);
    let dist = dot3(d, normal);
    [p[0]-dist*normal[0], p[1]-dist*normal[1], p[2]-dist*normal[2]]
}

/// Run MLS reconstruction (returns smoothed version of input).
#[allow(dead_code)]
pub fn mls_reconstruct(
    points: &[[f32; 3]],
    config: &MlsReconstructConfig,
) -> MlsReconstructResult {
    let mut smoothed = Vec::with_capacity(points.len());
    for &q in points {
        let nb = find_neighbours(points, q, config.search_radius);
        if nb.is_empty() {
            smoothed.push(q);
        } else {
            let nb_pts: Vec<[f32; 3]> = nb.iter().map(|&i| points[i]).collect();
            let weights: Vec<f32> = nb_pts.iter()
                .map(|&p| gaussian_weight(sq_dist(q, p), config.search_radius))
                .collect();
            smoothed.push(weighted_centroid(&nb_pts, &weights));
        }
    }
    let normals = vec![[0.0f32, 0.0, 1.0]; smoothed.len()];
    let n = smoothed.len();
    MlsReconstructResult { positions: smoothed, normals, point_count: n }
}

/// Coverage metric: fraction of query points that had neighbours.
#[allow(dead_code)]
pub fn mls_coverage(points: &[[f32; 3]], radius: f32) -> f32 {
    if points.is_empty() { return 0.0; }
    let covered = points.iter().filter(|&&q| {
        points.iter().any(|&p| sq_dist(q, p) < radius * radius * 1.001 && p != q)
    }).count();
    covered as f32 / points.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_points() -> Vec<[f32; 3]> {
        vec![
            [0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,0.5,0.0],
            [0.0,1.0,0.0],[1.0,1.0,0.0],
        ]
    }

    #[test]
    fn sq_dist_correct() {
        let d = sq_dist([0.0,0.0,0.0],[1.0,0.0,0.0]);
        assert!((d - 1.0).abs() < 1e-6);
    }

    #[test]
    fn gaussian_weight_at_zero() {
        let w = gaussian_weight(0.0, 1.0);
        assert!((w - 1.0).abs() < 1e-6);
    }

    #[test]
    fn find_neighbours_count() {
        let pts = sample_points();
        let nb = find_neighbours(&pts, [0.5, 0.5, 0.0], 0.8);
        assert!(!nb.is_empty());
    }

    #[test]
    fn weighted_centroid_uniform() {
        let pts = vec![[0.0,0.0,0.0],[2.0,0.0,0.0]];
        let weights = vec![1.0, 1.0];
        let c = weighted_centroid(&pts, &weights);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn project_to_plane_z_up() {
        let p = [1.0, 1.0, 5.0];
        let c = [0.0, 0.0, 0.0];
        let n = [0.0, 0.0, 1.0];
        let proj = project_to_plane(p, c, n);
        assert!(proj[2].abs() < 1e-5);
    }

    #[test]
    fn mls_reconstruct_preserves_count() {
        let pts = sample_points();
        let cfg = MlsReconstructConfig { search_radius: 2.0, ..Default::default() };
        let r = mls_reconstruct(&pts, &cfg);
        assert_eq!(r.point_count, pts.len());
    }

    #[test]
    fn estimate_normal_stub_unit() {
        let n = estimate_normal_stub(&[[0.0,0.0,0.0]]);
        let l = n[0]*n[0]+n[1]*n[1]+n[2]*n[2];
        assert!((l - 1.0).abs() < 1e-6);
    }

    #[test]
    fn coverage_all_close() {
        let pts = vec![[0.0,0.0,0.0],[0.1,0.0,0.0]];
        let cov = mls_coverage(&pts, 1.0);
        assert!((cov - 1.0).abs() < 1e-5);
    }

    #[test]
    fn default_config() {
        let c = MlsReconstructConfig::default();
        assert_eq!(c.grid_resolution, 32);
    }
}
