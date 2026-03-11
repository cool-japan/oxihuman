// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Configuration for mesh relaxation.
#[allow(dead_code)]
pub struct RelaxConfig {
    pub iterations: usize,
    pub factor: f32,
}

impl Default for RelaxConfig {
    fn default() -> Self {
        Self {
            iterations: 5,
            factor: 0.5,
        }
    }
}

/// Result of mesh relaxation.
#[allow(dead_code)]
pub struct RelaxResult {
    pub positions: Vec<[f32; 3]>,
    pub iterations_run: usize,
}

/// Build adjacency list from triangle indices.
#[allow(dead_code)]
pub fn build_adjacency_relax(n_verts: usize, indices: &[u32]) -> Vec<Vec<usize>> {
    let mut adj = vec![vec![]; n_verts];
    let n = indices.len() / 3;
    for fi in 0..n {
        let a = indices[fi * 3] as usize;
        let b = indices[fi * 3 + 1] as usize;
        let c = indices[fi * 3 + 2] as usize;
        for (u, v) in [(a, b), (b, c), (c, a)] {
            if !adj[u].contains(&v) {
                adj[u].push(v);
            }
            if !adj[v].contains(&u) {
                adj[v].push(u);
            }
        }
    }
    adj
}

/// Apply one step of Laplacian relaxation.
#[allow(dead_code)]
pub fn relax_step(positions: &[[f32; 3]], adj: &[Vec<usize>], factor: f32) -> Vec<[f32; 3]> {
    let mut out = positions.to_vec();
    for (i, neighbors) in adj.iter().enumerate() {
        if neighbors.is_empty() {
            continue;
        }
        let n = neighbors.len() as f32;
        let mut avg = [0.0_f32; 3];
        for &j in neighbors {
            avg[0] += positions[j][0];
            avg[1] += positions[j][1];
            avg[2] += positions[j][2];
        }
        avg[0] /= n;
        avg[1] /= n;
        avg[2] /= n;
        out[i][0] = positions[i][0] + factor * (avg[0] - positions[i][0]);
        out[i][1] = positions[i][1] + factor * (avg[1] - positions[i][1]);
        out[i][2] = positions[i][2] + factor * (avg[2] - positions[i][2]);
    }
    out
}

/// Run full mesh relaxation.
#[allow(dead_code)]
pub fn relax_mesh(positions: &[[f32; 3]], indices: &[u32], config: &RelaxConfig) -> RelaxResult {
    let adj = build_adjacency_relax(positions.len(), indices);
    let mut pos = positions.to_vec();
    for _ in 0..config.iterations {
        pos = relax_step(&pos, &adj, config.factor);
    }
    RelaxResult {
        positions: pos,
        iterations_run: config.iterations,
    }
}

/// Compute average displacement between original and relaxed positions.
#[allow(dead_code)]
pub fn relax_avg_displacement(original: &[[f32; 3]], relaxed: &[[f32; 3]]) -> f32 {
    if original.is_empty() {
        return 0.0;
    }
    let sum: f32 = original
        .iter()
        .zip(relaxed.iter())
        .map(|(a, b)| {
            let dx = a[0] - b[0];
            let dy = a[1] - b[1];
            let dz = a[2] - b[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum();
    sum / original.len() as f32
}

/// Serialize result to JSON.
#[allow(dead_code)]
pub fn relax_to_json(result: &RelaxResult) -> String {
    format!(
        r#"{{"vertices":{},"iterations":{}}}"#,
        result.positions.len(),
        result.iterations_run
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]];
        let idx = vec![0, 1, 2];
        (pos, idx)
    }

    #[test]
    fn relaxation_runs() {
        let (pos, idx) = triangle_mesh();
        let cfg = RelaxConfig::default();
        let r = relax_mesh(&pos, &idx, &cfg);
        assert_eq!(r.positions.len(), pos.len());
    }

    #[test]
    fn iterations_stored() {
        let (pos, idx) = triangle_mesh();
        let cfg = RelaxConfig {
            iterations: 3,
            factor: 0.5,
        };
        let r = relax_mesh(&pos, &idx, &cfg);
        assert_eq!(r.iterations_run, 3);
    }

    #[test]
    fn zero_iterations_no_change() {
        let (pos, idx) = triangle_mesh();
        let cfg = RelaxConfig {
            iterations: 0,
            factor: 0.5,
        };
        let r = relax_mesh(&pos, &idx, &cfg);
        for (a, b) in pos.iter().zip(r.positions.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-6);
        }
    }

    #[test]
    fn adjacency_non_empty() {
        let (pos, idx) = triangle_mesh();
        let adj = build_adjacency_relax(pos.len(), &idx);
        assert!(!adj[0].is_empty());
    }

    #[test]
    fn avg_displacement_nonnegative() {
        let (pos, idx) = triangle_mesh();
        let cfg = RelaxConfig::default();
        let r = relax_mesh(&pos, &idx, &cfg);
        assert!(relax_avg_displacement(&pos, &r.positions) >= 0.0);
    }

    #[test]
    fn json_has_vertices() {
        let (pos, idx) = triangle_mesh();
        let cfg = RelaxConfig::default();
        let r = relax_mesh(&pos, &idx, &cfg);
        let j = relax_to_json(&r);
        assert!(j.contains("\"vertices\":3"));
    }

    #[test]
    fn factor_zero_no_move() {
        let (pos, idx) = triangle_mesh();
        let cfg = RelaxConfig {
            iterations: 5,
            factor: 0.0,
        };
        let r = relax_mesh(&pos, &idx, &cfg);
        for (a, b) in pos.iter().zip(r.positions.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-6);
        }
    }

    #[test]
    fn step_produces_same_size() {
        let (pos, idx) = triangle_mesh();
        let adj = build_adjacency_relax(pos.len(), &idx);
        let s = relax_step(&pos, &adj, 0.5);
        assert_eq!(s.len(), pos.len());
    }

    #[test]
    fn empty_positions() {
        let r = relax_mesh(&[], &[], &RelaxConfig::default());
        assert_eq!(r.positions.len(), 0);
    }

    #[test]
    fn displacement_same_zero() {
        let pos = vec![[0.0_f32; 3]; 3];
        assert!((relax_avg_displacement(&pos, &pos) - 0.0).abs() < 1e-6);
    }
}
