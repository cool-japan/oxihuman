// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Vertex relaxation (Laplacian relax).

/// Result from a relax operation.
#[derive(Debug, Clone)]
pub struct RelaxResult {
    pub positions: Vec<[f32; 3]>,
    pub iterations_run: usize,
    pub max_displacement: f32,
}

/// Build a simple adjacency list from a triangle index list.
pub fn build_adjacency(indices: &[u32], vertex_count: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<Vec<usize>> = vec![vec![]; vertex_count];
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if !adj[a].contains(&b) {
            adj[a].push(b);
        }
        if !adj[a].contains(&c) {
            adj[a].push(c);
        }
        if !adj[b].contains(&a) {
            adj[b].push(a);
        }
        if !adj[b].contains(&c) {
            adj[b].push(c);
        }
        if !adj[c].contains(&a) {
            adj[c].push(a);
        }
        if !adj[c].contains(&b) {
            adj[c].push(b);
        }
    }
    adj
}

/// Perform one Laplacian smoothing step.
pub fn laplacian_step(
    positions: &[[f32; 3]],
    adj: &[Vec<usize>],
    factor: f32,
    pin: &[bool],
) -> Vec<[f32; 3]> {
    let mut out = positions.to_vec();
    for (i, neighbours) in adj.iter().enumerate() {
        if pin[i] || neighbours.is_empty() {
            continue;
        }
        let count = neighbours.len() as f32;
        let avg = neighbours.iter().fold([0.0f32; 3], |acc, &j| {
            [
                acc[0] + positions[j][0],
                acc[1] + positions[j][1],
                acc[2] + positions[j][2],
            ]
        });
        let avg = [avg[0] / count, avg[1] / count, avg[2] / count];
        let p = positions[i];
        out[i] = [
            p[0] + factor * (avg[0] - p[0]),
            p[1] + factor * (avg[1] - p[1]),
            p[2] + factor * (avg[2] - p[2]),
        ];
    }
    out
}

/// Run Laplacian relaxation for `iterations` steps.
pub fn relax_mesh(
    positions: &[[f32; 3]],
    adj: &[Vec<usize>],
    factor: f32,
    iterations: usize,
    pin: &[bool],
) -> RelaxResult {
    let mut current = positions.to_vec();
    let mut max_disp = 0.0f32;
    for _ in 0..iterations {
        let next = laplacian_step(&current, adj, factor, pin);
        max_disp = next
            .iter()
            .zip(current.iter())
            .map(|(&a, &b)| {
                let dx = a[0] - b[0];
                let dy = a[1] - b[1];
                let dz = a[2] - b[2];
                (dx * dx + dy * dy + dz * dz).sqrt()
            })
            .fold(0.0f32, f32::max);
        current = next;
    }
    RelaxResult {
        positions: current,
        iterations_run: iterations,
        max_displacement: max_disp,
    }
}

/// Taubin lambda-mu smoothing: alternates positive and negative factors.
pub fn taubin_relax(
    positions: &[[f32; 3]],
    adj: &[Vec<usize>],
    lambda: f32,
    mu: f32,
    iterations: usize,
    pin: &[bool],
) -> RelaxResult {
    let mut current = positions.to_vec();
    for i in 0..iterations {
        let factor = if i % 2 == 0 { lambda } else { mu };
        current = laplacian_step(&current, adj, factor, pin);
    }
    RelaxResult {
        positions: current,
        iterations_run: iterations,
        max_displacement: 0.0,
    }
}

/// Relax vertices within a sphere of influence.
pub fn relax_in_sphere(
    positions: &[[f32; 3]],
    adj: &[Vec<usize>],
    centre: [f32; 3],
    radius: f32,
    factor: f32,
    iterations: usize,
) -> RelaxResult {
    let n = positions.len();
    let pin: Vec<bool> = positions
        .iter()
        .map(|&p| {
            let dx = p[0] - centre[0];
            let dy = p[1] - centre[1];
            let dz = p[2] - centre[2];
            (dx * dx + dy * dy + dz * dz).sqrt() >= radius
        })
        .collect();
    relax_mesh(positions, adj, factor, iterations, &pin[..n])
}

/// Build a uniform pin mask (none pinned).
pub fn no_pins(count: usize) -> Vec<bool> {
    vec![false; count]
}

/// Build a pin mask where boundary vertices are pinned.
pub fn pin_isolated(adj: &[Vec<usize>]) -> Vec<bool> {
    adj.iter().map(|nb| nb.is_empty()).collect()
}

/// Compute per-vertex displacement magnitude between two position sets.
pub fn displacement_magnitudes(a: &[[f32; 3]], b: &[[f32; 3]]) -> Vec<f32> {
    a.iter()
        .zip(b.iter())
        .map(|(&pa, &pb)| {
            let dx = pa[0] - pb[0];
            let dy = pa[1] - pb[1];
            let dz = pa[2] - pb[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .collect()
}

/// Return the mean displacement between two position sets.
pub fn mean_displacement(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    if a.is_empty() {
        return 0.0;
    }
    displacement_magnitudes(a, b).iter().sum::<f32>() / a.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        (pts, idx)
    }

    /* adjacency built correctly */
    #[test]
    fn test_build_adjacency() {
        let (_, idx) = triangle_mesh();
        let adj = build_adjacency(&idx, 3);
        assert_eq!(adj.len(), 3);
        assert!(adj[0].contains(&1));
        assert!(adj[0].contains(&2));
    }

    /* laplacian_step moves middle vertex */
    #[test]
    fn test_laplacian_step() {
        let pts = vec![[0.0f32; 3], [2.0, 0.0, 0.0], [4.0, 0.0, 0.0]];
        let adj = vec![vec![1usize], vec![0, 2], vec![1]];
        let pin = vec![false; 3];
        let out = laplacian_step(&pts, &adj, 1.0, &pin);
        /* middle should move to avg of 0 and 4 = 2, no change (already at 2) */
        assert!((out[1][0] - 2.0).abs() < 1e-5);
    }

    /* relax_mesh returns correct count */
    #[test]
    fn test_relax_mesh_count() {
        let (pts, idx) = triangle_mesh();
        let adj = build_adjacency(&idx, pts.len());
        let pin = no_pins(pts.len());
        let res = relax_mesh(&pts, &adj, 0.5, 3, &pin);
        assert_eq!(res.positions.len(), pts.len());
        assert_eq!(res.iterations_run, 3);
    }

    /* no_pins */
    #[test]
    fn test_no_pins() {
        let pins = no_pins(5);
        assert!(pins.iter().all(|&p| !p));
    }

    /* pin_isolated */
    #[test]
    fn test_pin_isolated() {
        let adj = vec![vec![1usize], vec![], vec![0]];
        let pins = pin_isolated(&adj);
        assert!(!pins[0]);
        assert!(pins[1]);
    }

    /* taubin relax returns same count */
    #[test]
    fn test_taubin_relax_count() {
        let (pts, idx) = triangle_mesh();
        let adj = build_adjacency(&idx, pts.len());
        let pin = no_pins(pts.len());
        let res = taubin_relax(&pts, &adj, 0.5, -0.53, 4, &pin);
        assert_eq!(res.positions.len(), pts.len());
    }

    /* displacement_magnitudes length */
    #[test]
    fn test_displacement_magnitudes_len() {
        let a = vec![[0.0f32; 3]; 4];
        let b = vec![[1.0f32; 3]; 4];
        assert_eq!(displacement_magnitudes(&a, &b).len(), 4);
    }

    /* mean_displacement */
    #[test]
    fn test_mean_displacement_zero() {
        let pts = vec![[1.0f32; 3]; 3];
        assert!(mean_displacement(&pts, &pts).abs() < 1e-6);
    }

    /* relax_in_sphere */
    #[test]
    fn test_relax_in_sphere_count() {
        let (pts, idx) = triangle_mesh();
        let adj = build_adjacency(&idx, pts.len());
        let res = relax_in_sphere(&pts, &adj, [0.5, 0.5, 0.0], 2.0, 0.5, 2);
        assert_eq!(res.positions.len(), pts.len());
    }
}
