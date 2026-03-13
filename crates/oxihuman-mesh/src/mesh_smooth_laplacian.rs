// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Laplacian smoothing with configurable iterations.

#![allow(dead_code)]

/// Configuration for Laplacian smoothing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LaplacianSmoothConfig {
    /// Number of smoothing iterations.
    pub iterations: usize,
    /// Smoothing factor in the range (0, 1].
    pub factor: f32,
    /// Whether to keep boundary vertices fixed.
    pub preserve_boundary: bool,
}

/// Result of Laplacian smoothing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LaplacianSmoothResult {
    /// Smoothed vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Number of iterations actually performed.
    pub iterations_done: usize,
    /// Maximum vertex displacement over all iterations.
    pub max_displacement: f32,
}

/// Returns the default [`LaplacianSmoothConfig`].
#[allow(dead_code)]
pub fn default_laplacian_smooth_config() -> LaplacianSmoothConfig {
    LaplacianSmoothConfig {
        iterations: 5,
        factor: 0.5,
        preserve_boundary: true,
    }
}

/// Applies Laplacian smoothing to `positions` given `indices`.
#[allow(dead_code)]
pub fn smooth_laplacian(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &LaplacianSmoothConfig,
) -> LaplacianSmoothResult {
    let n = positions.len();
    if n == 0 || config.iterations == 0 {
        return LaplacianSmoothResult {
            positions: positions.to_vec(),
            iterations_done: 0,
            max_displacement: 0.0,
        };
    }

    let factor = config.factor.clamp(0.0, 1.0);

    // Build adjacency list
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if !adj[a].contains(&b) { adj[a].push(b); }
        if !adj[a].contains(&c) { adj[a].push(c); }
        if !adj[b].contains(&a) { adj[b].push(a); }
        if !adj[b].contains(&c) { adj[b].push(c); }
        if !adj[c].contains(&a) { adj[c].push(a); }
        if !adj[c].contains(&b) { adj[c].push(b); }
    }

    // Identify boundary vertices if needed
    let boundary: Vec<bool> = if config.preserve_boundary {
        let mut edge_count: std::collections::HashMap<(u32, u32), usize> =
            std::collections::HashMap::new();
        for tri in indices.chunks(3) {
            if tri.len() < 3 { continue; }
            for k in 0..3 {
                let a = tri[k];
                let b = tri[(k + 1) % 3];
                let key = if a < b { (a, b) } else { (b, a) };
                *edge_count.entry(key).or_insert(0) += 1;
            }
        }
        let mut is_boundary = vec![false; n];
        for ((a, b), &count) in &edge_count {
            if count == 1 {
                is_boundary[*a as usize] = true;
                is_boundary[*b as usize] = true;
            }
        }
        is_boundary
    } else {
        vec![false; n]
    };

    let mut cur = positions.to_vec();
    let mut max_disp = 0.0f32;

    for _ in 0..config.iterations {
        let prev = cur.clone();
        for i in 0..n {
            if boundary[i] || adj[i].is_empty() {
                continue;
            }
            let k = adj[i].len() as f32;
            let cx = adj[i].iter().map(|&j| prev[j][0]).sum::<f32>() / k;
            let cy = adj[i].iter().map(|&j| prev[j][1]).sum::<f32>() / k;
            let cz = adj[i].iter().map(|&j| prev[j][2]).sum::<f32>() / k;
            let dx = factor * (cx - prev[i][0]);
            let dy = factor * (cy - prev[i][1]);
            let dz = factor * (cz - prev[i][2]);
            cur[i] = [prev[i][0] + dx, prev[i][1] + dy, prev[i][2] + dz];
            let d = (dx * dx + dy * dy + dz * dz).sqrt();
            if d > max_disp {
                max_disp = d;
            }
        }
    }

    LaplacianSmoothResult {
        positions: cur,
        iterations_done: config.iterations,
        max_displacement: max_disp,
    }
}

/// Returns the Euclidean norm of the displacement between two position arrays.
#[allow(dead_code)]
pub fn smooth_displacement_norm(original: &[[f32; 3]], smoothed: &[[f32; 3]]) -> f32 {
    original
        .iter()
        .zip(smoothed.iter())
        .map(|(a, b)| {
            let dx = a[0] - b[0];
            let dy = a[1] - b[1];
            let dz = a[2] - b[2];
            dx * dx + dy * dy + dz * dz
        })
        .fold(0.0f32, f32::max)
        .sqrt()
}

/// Validates that a [`LaplacianSmoothConfig`] has sensible parameters.
#[allow(dead_code)]
pub fn smooth_validate_config(config: &LaplacianSmoothConfig) -> bool {
    config.factor > 0.0 && config.factor <= 1.0
}

/// Serialises the result to a minimal JSON string.
#[allow(dead_code)]
pub fn smooth_to_json(result: &LaplacianSmoothResult) -> String {
    format!(
        "{{\"vertices\":{},\"iterations_done\":{},\"max_displacement\":{}}}",
        result.positions.len(),
        result.iterations_done,
        result.max_displacement
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
        let cfg = default_laplacian_smooth_config();
        assert_eq!(cfg.iterations, 5);
        assert_eq!(cfg.factor, 0.5);
    }

    #[test]
    fn test_smooth_produces_positions() {
        let (pos, idx) = quad_mesh();
        let cfg = default_laplacian_smooth_config();
        let result = smooth_laplacian(&pos, &idx, &cfg);
        assert_eq!(result.positions.len(), pos.len());
    }

    #[test]
    fn test_iterations_done() {
        let (pos, idx) = quad_mesh();
        let cfg = LaplacianSmoothConfig { iterations: 3, factor: 0.5, preserve_boundary: false };
        let result = smooth_laplacian(&pos, &idx, &cfg);
        assert_eq!(result.iterations_done, 3);
    }

    #[test]
    fn test_zero_iterations() {
        let (pos, idx) = quad_mesh();
        let cfg = LaplacianSmoothConfig { iterations: 0, factor: 0.5, preserve_boundary: false };
        let result = smooth_laplacian(&pos, &idx, &cfg);
        assert_eq!(result.iterations_done, 0);
        assert_eq!(result.positions, pos);
    }

    #[test]
    fn test_validate_config_valid() {
        let cfg = default_laplacian_smooth_config();
        assert!(smooth_validate_config(&cfg));
    }

    #[test]
    fn test_validate_config_invalid() {
        let cfg = LaplacianSmoothConfig { iterations: 1, factor: 0.0, preserve_boundary: false };
        assert!(!smooth_validate_config(&cfg));
    }

    #[test]
    fn test_displacement_norm() {
        let a = vec![[0.0, 0.0, 0.0]];
        let b = vec![[1.0, 0.0, 0.0]];
        let norm = smooth_displacement_norm(&a, &b);
        assert!((norm - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let (pos, idx) = quad_mesh();
        let cfg = default_laplacian_smooth_config();
        let result = smooth_laplacian(&pos, &idx, &cfg);
        let json = smooth_to_json(&result);
        assert!(json.contains("vertices"));
        assert!(json.contains("iterations_done"));
    }

    #[test]
    fn test_preserve_boundary_no_move() {
        // With preserve_boundary and a simple quad all verts are boundary → no change
        let (pos, idx) = quad_mesh();
        let cfg = LaplacianSmoothConfig { iterations: 5, factor: 0.5, preserve_boundary: true };
        let result = smooth_laplacian(&pos, &idx, &cfg);
        // Displacement should be very small since all 4 verts are boundary
        assert!(result.max_displacement < 1e-4);
    }
}
