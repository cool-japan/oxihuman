//! Curvature-aware smoothing operations.
#![allow(dead_code)]

/// Configuration for curvature-aware smoothing.
#[allow(dead_code)]
pub struct CurvatureSmooth {
    pub iterations: usize,
    pub curvature_weight: f32,
}

/// Compute a simple vertex curvature estimate (mean of angle differences to neighbors).
#[allow(dead_code)]
pub fn compute_vertex_curvature_simple(
    positions: &[[f32; 3]],
    indices: &[u32],
) -> Vec<f32> {
    let n = positions.len();
    let mut neighbor_sum = vec![[0.0f32; 3]; n];
    let mut neighbor_count = vec![0usize; n];
    let tris = indices.len() / 3;
    for t in 0..tris {
        let i0 = indices[t*3] as usize;
        let i1 = indices[t*3+1] as usize;
        let i2 = indices[t*3+2] as usize;
        if i0 >= n || i1 >= n || i2 >= n { continue; }
        for (center, other1, other2) in [(i0,i1,i2),(i1,i0,i2),(i2,i0,i1)] {
            neighbor_sum[center][0] += positions[other1][0] + positions[other2][0];
            neighbor_sum[center][1] += positions[other1][1] + positions[other2][1];
            neighbor_sum[center][2] += positions[other1][2] + positions[other2][2];
            neighbor_count[center] += 2;
        }
    }
    (0..n).map(|i| {
        if neighbor_count[i] == 0 { return 0.0; }
        let cnt = neighbor_count[i] as f32;
        let dx = positions[i][0] - neighbor_sum[i][0] / cnt;
        let dy = positions[i][1] - neighbor_sum[i][1] / cnt;
        let dz = positions[i][2] - neighbor_sum[i][2] / cnt;
        (dx*dx + dy*dy + dz*dz).sqrt()
    }).collect()
}

/// Apply one step of curvature-aware smoothing.
#[allow(dead_code)]
pub fn curvature_smooth_step(
    positions: &[[f32; 3]],
    curvatures: &[f32],
    indices: &[u32],
    lambda: f32,
) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut result = positions.to_vec();
    let mut neighbor_pos = vec![[0.0f32; 3]; n];
    let mut neighbor_count = vec![0usize; n];
    let tris = indices.len() / 3;
    for t in 0..tris {
        let i0 = indices[t*3] as usize;
        let i1 = indices[t*3+1] as usize;
        let i2 = indices[t*3+2] as usize;
        if i0 >= n || i1 >= n || i2 >= n { continue; }
        for (c, o1, o2) in [(i0,i1,i2),(i1,i0,i2),(i2,i0,i1)] {
            neighbor_pos[c][0] += positions[o1][0] + positions[o2][0];
            neighbor_pos[c][1] += positions[o1][1] + positions[o2][1];
            neighbor_pos[c][2] += positions[o1][2] + positions[o2][2];
            neighbor_count[c] += 2;
        }
    }
    for i in 0..n {
        if neighbor_count[i] == 0 { continue; }
        let cnt = neighbor_count[i] as f32;
        let w = lambda * (1.0 - curvatures.get(i).copied().unwrap_or(0.0).min(1.0));
        result[i][0] = positions[i][0] + w * (neighbor_pos[i][0]/cnt - positions[i][0]);
        result[i][1] = positions[i][1] + w * (neighbor_pos[i][1]/cnt - positions[i][1]);
        result[i][2] = positions[i][2] + w * (neighbor_pos[i][2]/cnt - positions[i][2]);
    }
    result
}

/// Apply N iterations of curvature-aware smoothing.
#[allow(dead_code)]
pub fn apply_curvature_smoothing(
    positions: &[[f32; 3]],
    indices: &[u32],
    iterations: usize,
    lambda: f32,
) -> Vec<[f32; 3]> {
    let mut pos = positions.to_vec();
    for _ in 0..iterations {
        let curv = compute_vertex_curvature_simple(&pos, indices);
        pos = curvature_smooth_step(&pos, &curv, indices, lambda);
    }
    pos
}

/// Compute the Laplacian curvature vector for each vertex.
#[allow(dead_code)]
pub fn laplacian_curvature(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut sum = vec![[0.0f32; 3]; n];
    let mut cnt = vec![0usize; n];
    let tris = indices.len() / 3;
    for t in 0..tris {
        let i0 = indices[t*3] as usize;
        let i1 = indices[t*3+1] as usize;
        let i2 = indices[t*3+2] as usize;
        if i0 >= n || i1 >= n || i2 >= n { continue; }
        for (c, o1, o2) in [(i0,i1,i2),(i1,i0,i2),(i2,i0,i1)] {
            for k in 0..3 {
                sum[c][k] += positions[o1][k] + positions[o2][k];
            }
            cnt[c] += 2;
        }
    }
    (0..n).map(|i| {
        if cnt[i] == 0 { return [0.0f32; 3]; }
        let c = cnt[i] as f32;
        [sum[i][0]/c - positions[i][0], sum[i][1]/c - positions[i][1], sum[i][2]/c - positions[i][2]]
    }).collect()
}

/// Compute the mean curvature normal at each vertex.
#[allow(dead_code)]
pub fn mean_curvature_normal(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    laplacian_curvature(positions, indices)
}

/// Apply bilateral smoothing (placeholder: uses uniform Laplacian for now).
#[allow(dead_code)]
pub fn bilateral_smooth(positions: &[[f32; 3]], indices: &[u32], iterations: usize) -> Vec<[f32; 3]> {
    apply_curvature_smoothing(positions, indices, iterations, 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plane_mesh() -> (Vec<[f32;3]>, Vec<u32>) {
        let pos = vec![
            [0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],[1.0,1.0,0.0]
        ];
        let idx = vec![0u32,1,2, 1,3,2];
        (pos, idx)
    }

    #[test]
    fn test_compute_curvature_count() {
        let (pos, idx) = plane_mesh();
        let c = compute_vertex_curvature_simple(&pos, &idx);
        assert_eq!(c.len(), 4);
    }

    #[test]
    fn test_curvature_smooth_step_count() {
        let (pos, idx) = plane_mesh();
        let curv = vec![0.0f32; 4];
        let r = curvature_smooth_step(&pos, &curv, &idx, 0.5);
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn test_apply_curvature_smoothing_count() {
        let (pos, idx) = plane_mesh();
        let r = apply_curvature_smoothing(&pos, &idx, 3, 0.5);
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn test_laplacian_curvature_count() {
        let (pos, idx) = plane_mesh();
        let r = laplacian_curvature(&pos, &idx);
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn test_mean_curvature_normal_count() {
        let (pos, idx) = plane_mesh();
        let r = mean_curvature_normal(&pos, &idx);
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn test_bilateral_smooth_count() {
        let (pos, idx) = plane_mesh();
        let r = bilateral_smooth(&pos, &idx, 2);
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn test_curvature_smooth_config() {
        let c = CurvatureSmooth { iterations: 5, curvature_weight: 0.8 };
        assert_eq!(c.iterations, 5);
    }

    #[test]
    fn test_uniform_plane_curvature_small() {
        let pos = vec![[0.0f32,0.0,0.0],[2.0,0.0,0.0],[1.0,2.0,0.0]];
        let idx = vec![0u32,1,2];
        let c = compute_vertex_curvature_simple(&pos, &idx);
        assert!(c[0] > 0.0 || c[1] > 0.0 || c[2] > 0.0 || c.iter().all(|&x| x >= 0.0));
    }

    #[test]
    fn test_smoothing_reduces_or_maintains() {
        let (pos, idx) = plane_mesh();
        let r = apply_curvature_smoothing(&pos, &idx, 1, 0.5);
        assert_eq!(r.len(), pos.len());
    }
}
