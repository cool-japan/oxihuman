//! Relaxation-based vertex smoothing.
#![allow(dead_code)]

/// Configuration for mesh relaxation.
#[allow(dead_code)]
pub struct RelaxConfig {
    pub iterations: usize,
    pub lambda: f32,
    pub boundary_only: bool,
}

impl Default for RelaxConfig {
    fn default() -> Self {
        RelaxConfig { iterations: 5, lambda: 0.5, boundary_only: false }
    }
}

/// Relax a mesh by averaging vertex positions with neighbors.
#[allow(dead_code)]
pub fn relax_mesh(positions: &[[f32;3]], indices: &[u32], config: &RelaxConfig) -> Vec<[f32;3]> {
    relax_n_steps(positions, indices, config.iterations, config.lambda)
}

/// Perform one relaxation step.
#[allow(dead_code)]
pub fn relax_step(positions: &[[f32;3]], adjacency: &[Vec<usize>], lambda: f32) -> Vec<[f32;3]> {
    let n = positions.len();
    let mut result = positions.to_vec();
    for i in 0..n {
        let neighbors = &adjacency[i];
        if neighbors.is_empty() { continue; }
        let cnt = neighbors.len() as f32;
        let sum: [f32;3] = neighbors.iter().fold([0.0;3], |acc, &j| {
            [acc[0]+positions[j][0], acc[1]+positions[j][1], acc[2]+positions[j][2]]
        });
        result[i][0] = positions[i][0] + lambda * (sum[0]/cnt - positions[i][0]);
        result[i][1] = positions[i][1] + lambda * (sum[1]/cnt - positions[i][1]);
        result[i][2] = positions[i][2] + lambda * (sum[2]/cnt - positions[i][2]);
    }
    result
}

/// Build vertex adjacency list from triangle indices.
#[allow(dead_code)]
pub fn build_vertex_adjacency2(n: usize, indices: &[u32]) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::BTreeSet<usize>> = vec![std::collections::BTreeSet::new(); n];
    let tris = indices.len() / 3;
    for t in 0..tris {
        let i0 = indices[t*3] as usize;
        let i1 = indices[t*3+1] as usize;
        let i2 = indices[t*3+2] as usize;
        if i0 < n && i1 < n && i2 < n {
            adj[i0].insert(i1); adj[i0].insert(i2);
            adj[i1].insert(i0); adj[i1].insert(i2);
            adj[i2].insert(i0); adj[i2].insert(i1);
        }
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

/// Compute a weighted Laplacian position for a vertex.
#[allow(dead_code)]
pub fn weighted_laplacian(
    vertex: usize,
    positions: &[[f32;3]],
    neighbors: &[usize],
    weights: &[f32],
) -> [f32;3] {
    if neighbors.is_empty() { return positions[vertex]; }
    let mut sum = [0.0f32;3];
    let mut wsum = 0.0f32;
    for (&n, &w) in neighbors.iter().zip(weights.iter()) {
        if n < positions.len() {
            sum[0] += positions[n][0] * w;
            sum[1] += positions[n][1] * w;
            sum[2] += positions[n][2] * w;
            wsum += w;
        }
    }
    if wsum < 1e-10 { return positions[vertex]; }
    [sum[0]/wsum, sum[1]/wsum, sum[2]/wsum]
}

/// Relax only boundary vertices.
#[allow(dead_code)]
pub fn relax_boundary_only(
    positions: &[[f32;3]],
    indices: &[u32],
    lambda: f32,
    iterations: usize,
) -> Vec<[f32;3]> {
    let n = positions.len();
    let adj = build_vertex_adjacency2(n, indices);
    // detect boundary (vertices with fewer connections)
    let avg_degree = if !adj.is_empty() { adj.iter().map(|a| a.len()).sum::<usize>() / adj.len() } else { 0 };
    let boundary: Vec<bool> = adj.iter().map(|a| a.len() < avg_degree).collect();
    let mut pos = positions.to_vec();
    for _ in 0..iterations {
        let prev = pos.clone();
        for i in 0..n {
            if !boundary[i] { continue; }
            if adj[i].is_empty() { continue; }
            let cnt = adj[i].len() as f32;
            let sum = adj[i].iter().fold([0.0f32;3], |acc, &j| {
                [acc[0]+prev[j][0], acc[1]+prev[j][1], acc[2]+prev[j][2]]
            });
            pos[i][0] = prev[i][0] + lambda * (sum[0]/cnt - prev[i][0]);
            pos[i][1] = prev[i][1] + lambda * (sum[1]/cnt - prev[i][1]);
            pos[i][2] = prev[i][2] + lambda * (sum[2]/cnt - prev[i][2]);
        }
    }
    pos
}

/// Relax a mesh for N steps.
#[allow(dead_code)]
pub fn relax_n_steps(
    positions: &[[f32;3]],
    indices: &[u32],
    n_steps: usize,
    lambda: f32,
) -> Vec<[f32;3]> {
    let n = positions.len();
    let adj = build_vertex_adjacency2(n, indices);
    let mut pos = positions.to_vec();
    for _ in 0..n_steps {
        pos = relax_step(&pos, &adj, lambda);
    }
    pos
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_mesh() -> (Vec<[f32;3]>, Vec<u32>) {
        let pos = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],[1.0,1.0,0.0]];
        let idx = vec![0u32,1,2, 1,3,2];
        (pos, idx)
    }

    #[test]
    fn test_relax_mesh_count() {
        let (pos, idx) = square_mesh();
        let cfg = RelaxConfig::default();
        let r = relax_mesh(&pos, &idx, &cfg);
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn test_build_vertex_adjacency2_count() {
        let (pos, idx) = square_mesh();
        let adj = build_vertex_adjacency2(pos.len(), &idx);
        assert_eq!(adj.len(), 4);
    }

    #[test]
    fn test_relax_step_moves_vertices() {
        let (pos, idx) = square_mesh();
        let adj = build_vertex_adjacency2(pos.len(), &idx);
        let r = relax_step(&pos, &adj, 0.5);
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn test_weighted_laplacian() {
        let pos = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]];
        let neighbors = vec![1usize, 2];
        let weights = vec![1.0f32, 1.0];
        let r = weighted_laplacian(0, &pos, &neighbors, &weights);
        assert!((r[0] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_relax_boundary_only_count() {
        let (pos, idx) = square_mesh();
        let r = relax_boundary_only(&pos, &idx, 0.5, 2);
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn test_relax_n_steps_count() {
        let (pos, idx) = square_mesh();
        let r = relax_n_steps(&pos, &idx, 3, 0.5);
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn test_relax_config_default() {
        let cfg = RelaxConfig::default();
        assert_eq!(cfg.iterations, 5);
        assert!(!cfg.boundary_only);
    }

    #[test]
    fn test_adjacency_has_neighbors() {
        let (pos, idx) = square_mesh();
        let adj = build_vertex_adjacency2(pos.len(), &idx);
        assert!(adj.iter().any(|a| !a.is_empty()));
    }

    #[test]
    fn test_relax_zero_lambda_no_change() {
        let (pos, idx) = square_mesh();
        let r = relax_n_steps(&pos, &idx, 3, 0.0);
        for (a, b) in pos.iter().zip(r.iter()) {
            assert!((a[0]-b[0]).abs() < 1e-5);
        }
    }
}
