// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Disk mapping: project a mesh patch to a disk (unit circle) parameterization.

use std::f32::consts::TAU;

/// Configuration for disk mapping.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiskMapConfig {
    pub smoothing_iterations: usize,
}

/// Result of disk mapping.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiskMapResult {
    pub uvs: Vec<[f32; 2]>,
}

/// Default config.
#[allow(dead_code)]
pub fn default_disk_map_config() -> DiskMapConfig {
    DiskMapConfig {
        smoothing_iterations: 10,
    }
}

/// Map boundary vertices to a circle of radius 1.
#[allow(dead_code)]
pub fn map_boundary_to_circle(boundary: &[usize], count: usize) -> Vec<[f32; 2]> {
    let mut uvs = vec![[0.0f32; 2]; count];
    let n = boundary.len();
    if n == 0 {
        return uvs;
    }
    for (i, &vi) in boundary.iter().enumerate() {
        let angle = TAU * i as f32 / n as f32;
        uvs[vi] = [angle.cos() * 0.5 + 0.5, angle.sin() * 0.5 + 0.5];
    }
    uvs
}

/// Check if a UV is inside the unit disk (centered at 0.5, 0.5, radius 0.5).
#[allow(dead_code)]
pub fn is_in_unit_disk(uv: [f32; 2]) -> bool {
    let dx = uv[0] - 0.5;
    let dy = uv[1] - 0.5;
    dx * dx + dy * dy <= 0.25 + 1e-6
}

/// Compute a simple disk parameterization for a mesh patch.
/// `boundary` lists boundary vertex indices in order.
/// Interior vertices are initialized at center and smoothed via Tutte embedding.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn disk_map(
    positions: &[[f32; 3]],
    indices: &[u32],
    boundary: &[usize],
    config: &DiskMapConfig,
) -> DiskMapResult {
    let n = positions.len();
    let mut uvs = map_boundary_to_circle(boundary, n);
    let boundary_set: std::collections::HashSet<usize> = boundary.iter().copied().collect();

    // Build adjacency
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let vs = [
            indices[t * 3] as usize,
            indices[t * 3 + 1] as usize,
            indices[t * 3 + 2] as usize,
        ];
        for k in 0..3 {
            let a = vs[k];
            let b = vs[(k + 1) % 3];
            if !adj[a].contains(&b) {
                adj[a].push(b);
            }
            if !adj[b].contains(&a) {
                adj[b].push(a);
            }
        }
    }

    // Initialize interior vertices at center
    for vi in 0..n {
        if !boundary_set.contains(&vi) {
            uvs[vi] = [0.5, 0.5];
        }
    }

    // Tutte smoothing
    for _ in 0..config.smoothing_iterations {
        for vi in 0..n {
            if boundary_set.contains(&vi) || adj[vi].is_empty() {
                continue;
            }
            let mut su = 0.0f32;
            let mut sv = 0.0f32;
            let cnt = adj[vi].len() as f32;
            for &nb in &adj[vi] {
                su += uvs[nb][0];
                sv += uvs[nb][1];
            }
            uvs[vi] = [su / cnt, sv / cnt];
        }
    }

    DiskMapResult { uvs }
}

/// UV count in the result.
#[allow(dead_code)]
pub fn disk_map_vertex_count(result: &DiskMapResult) -> usize {
    result.uvs.len()
}

/// Compute UV area of a triangle.
#[allow(dead_code)]
pub fn uv_triangle_area(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs() * 0.5
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn disk_map_to_json(result: &DiskMapResult) -> String {
    format!("{{\"uv_count\":{}}}", result.uvs.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_config() {
        let cfg = default_disk_map_config();
        assert_eq!(cfg.smoothing_iterations, 10);
    }

    #[test]
    fn test_map_boundary_to_circle() {
        let uvs = map_boundary_to_circle(&[0, 1, 2, 3], 4);
        for &vi in &[0, 1, 2, 3] {
            assert!(is_in_unit_disk(uvs[vi]));
        }
    }

    #[test]
    fn test_is_in_unit_disk() {
        assert!(is_in_unit_disk([0.5, 0.5]));
        assert!(is_in_unit_disk([0.0, 0.5]));
        assert!(!is_in_unit_disk([1.5, 0.5]));
    }

    #[test]
    fn test_uv_triangle_area() {
        let area = uv_triangle_area([0.0, 0.0], [1.0, 0.0], [0.0, 1.0]);
        assert!((area - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_disk_map_simple() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.5, 0.5, 0.0],
        ];
        let idx = vec![0, 1, 4, 1, 2, 4, 2, 3, 4, 3, 0, 4];
        let boundary = vec![0, 1, 2, 3];
        let cfg = DiskMapConfig {
            smoothing_iterations: 5,
        };
        let result = disk_map(&pos, &idx, &boundary, &cfg);
        assert_eq!(disk_map_vertex_count(&result), 5);
    }

    #[test]
    fn test_disk_map_boundary_on_circle() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let boundary = vec![0, 1, 2];
        let cfg = default_disk_map_config();
        let result = disk_map(&pos, &idx, &boundary, &cfg);
        for &bi in &[0, 1, 2] {
            assert!(is_in_unit_disk(result.uvs[bi]));
        }
    }

    #[test]
    fn test_empty_boundary() {
        let uvs = map_boundary_to_circle(&[], 3);
        assert_eq!(uvs.len(), 3);
    }

    #[test]
    fn test_disk_map_to_json() {
        let result = DiskMapResult {
            uvs: vec![[0.5, 0.5]; 3],
        };
        let json = disk_map_to_json(&result);
        assert!(json.contains("\"uv_count\":3"));
    }

    #[test]
    fn test_pi_tau() {
        // Ensure we use std consts
        assert!((TAU - 2.0 * PI).abs() < 1e-6);
    }

    #[test]
    fn test_uv_triangle_area_degenerate() {
        let area = uv_triangle_area([0.0, 0.0], [1.0, 0.0], [2.0, 0.0]);
        assert!(area.abs() < 1e-6);
    }
}
