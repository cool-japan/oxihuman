// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Centroid-based mesh decomposition: split a mesh into regions around face centroids.

/// Configuration for centroid decomposition.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CentroidDecompConfig {
    pub num_regions: usize,
    pub max_iterations: usize,
}

/// A decomposition region.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecompRegion {
    pub centroid: [f32; 3],
    pub face_indices: Vec<usize>,
}

/// Result of centroid decomposition.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CentroidDecompResult {
    pub regions: Vec<DecompRegion>,
}

/// Default configuration.
#[allow(dead_code)]
pub fn default_centroid_decomp_config() -> CentroidDecompConfig {
    CentroidDecompConfig {
        num_regions: 4,
        max_iterations: 10,
    }
}

/// Compute the centroid of a triangle face.
#[allow(dead_code)]
pub fn face_centroid(positions: &[[f32; 3]], i0: usize, i1: usize, i2: usize) -> [f32; 3] {
    [
        (positions[i0][0] + positions[i1][0] + positions[i2][0]) / 3.0,
        (positions[i0][1] + positions[i1][1] + positions[i2][1]) / 3.0,
        (positions[i0][2] + positions[i1][2] + positions[i2][2]) / 3.0,
    ]
}

/// Compute all face centroids.
#[allow(dead_code)]
pub fn all_face_centroids(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let tri_count = indices.len() / 3;
    (0..tri_count)
        .map(|t| {
            face_centroid(
                positions,
                indices[t * 3] as usize,
                indices[t * 3 + 1] as usize,
                indices[t * 3 + 2] as usize,
            )
        })
        .collect()
}

/// Squared distance between two points.
#[allow(dead_code)]
fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

/// Decompose faces into regions using a k-means-like approach on centroids.
#[allow(dead_code)]
pub fn centroid_decompose(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &CentroidDecompConfig,
) -> CentroidDecompResult {
    let centroids = all_face_centroids(positions, indices);
    let n = centroids.len();
    if n == 0 || config.num_regions == 0 {
        return CentroidDecompResult {
            regions: Vec::new(),
        };
    }
    let k = config.num_regions.min(n);
    // Initialize seeds evenly spaced
    let mut seeds: Vec<[f32; 3]> = (0..k).map(|i| centroids[i * n / k]).collect();
    let mut assignments = vec![0usize; n];

    for _ in 0..config.max_iterations {
        // Assign each face to nearest seed
        for (fi, c) in centroids.iter().enumerate() {
            let mut best = 0;
            let mut best_d = f32::MAX;
            for (si, s) in seeds.iter().enumerate() {
                let d = dist_sq(*c, *s);
                if d < best_d {
                    best_d = d;
                    best = si;
                }
            }
            assignments[fi] = best;
        }
        // Recompute seeds
        for (si, seed) in seeds.iter_mut().enumerate() {
            let mut sum = [0.0f32; 3];
            let mut count = 0u32;
            for (fi, &a) in assignments.iter().enumerate() {
                if a == si {
                    sum[0] += centroids[fi][0];
                    sum[1] += centroids[fi][1];
                    sum[2] += centroids[fi][2];
                    count += 1;
                }
            }
            if count > 0 {
                *seed = [
                    sum[0] / count as f32,
                    sum[1] / count as f32,
                    sum[2] / count as f32,
                ];
            }
        }
    }

    let mut regions: Vec<DecompRegion> = (0..k)
        .map(|i| DecompRegion {
            centroid: seeds[i],
            face_indices: Vec::new(),
        })
        .collect();
    for (fi, &a) in assignments.iter().enumerate() {
        regions[a].face_indices.push(fi);
    }
    CentroidDecompResult { regions }
}

/// Number of regions in decomposition.
#[allow(dead_code)]
pub fn decomp_region_count(result: &CentroidDecompResult) -> usize {
    result.regions.len()
}

/// Total faces across all regions.
#[allow(dead_code)]
pub fn decomp_total_faces(result: &CentroidDecompResult) -> usize {
    result.regions.iter().map(|r| r.face_indices.len()).sum()
}

/// Convert decomposition result to JSON.
#[allow(dead_code)]
pub fn decomp_result_to_json(result: &CentroidDecompResult) -> String {
    format!(
        "{{\"region_count\":{},\"total_faces\":{}}}",
        decomp_region_count(result),
        decomp_total_faces(result)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_centroid_decomp_config();
        assert_eq!(cfg.num_regions, 4);
    }

    #[test]
    fn test_face_centroid() {
        let pos = vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]];
        let c = face_centroid(&pos, 0, 1, 2);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_all_face_centroids() {
        let (pos, idx) = quad_mesh();
        let centroids = all_face_centroids(&pos, &idx);
        assert_eq!(centroids.len(), 2);
    }

    #[test]
    fn test_decompose_single_region() {
        let (pos, idx) = quad_mesh();
        let cfg = CentroidDecompConfig {
            num_regions: 1,
            max_iterations: 5,
        };
        let result = centroid_decompose(&pos, &idx, &cfg);
        assert_eq!(decomp_region_count(&result), 1);
        assert_eq!(decomp_total_faces(&result), 2);
    }

    #[test]
    fn test_decompose_two_regions() {
        let (pos, idx) = quad_mesh();
        let cfg = CentroidDecompConfig {
            num_regions: 2,
            max_iterations: 5,
        };
        let result = centroid_decompose(&pos, &idx, &cfg);
        assert_eq!(decomp_region_count(&result), 2);
        assert_eq!(decomp_total_faces(&result), 2);
    }

    #[test]
    fn test_empty_mesh() {
        let cfg = default_centroid_decomp_config();
        let result = centroid_decompose(&[], &[], &cfg);
        assert_eq!(decomp_region_count(&result), 0);
    }

    #[test]
    fn test_decomp_result_to_json() {
        let (pos, idx) = quad_mesh();
        let cfg = CentroidDecompConfig {
            num_regions: 1,
            max_iterations: 3,
        };
        let result = centroid_decompose(&pos, &idx, &cfg);
        let json = decomp_result_to_json(&result);
        assert!(json.contains("\"region_count\":1"));
    }

    #[test]
    fn test_more_regions_than_faces() {
        let (pos, idx) = quad_mesh();
        let cfg = CentroidDecompConfig {
            num_regions: 10,
            max_iterations: 3,
        };
        let result = centroid_decompose(&pos, &idx, &cfg);
        // clamped to 2 (number of faces)
        assert_eq!(decomp_region_count(&result), 2);
    }

    #[test]
    fn test_decomp_total_faces_multi() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [1.5, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 1, 3, 4];
        let cfg = CentroidDecompConfig {
            num_regions: 2,
            max_iterations: 5,
        };
        let result = centroid_decompose(&pos, &idx, &cfg);
        assert_eq!(decomp_total_faces(&result), 2);
    }
}
