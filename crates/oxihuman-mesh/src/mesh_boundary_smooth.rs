// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Boundary smoothing: smooth boundary edges of a mesh while preserving
//! interior topology.

/// Configuration for boundary smoothing.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BoundarySmoothConfig {
    pub iterations: usize,
    pub factor: f32,
}

/// Result of boundary smoothing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoundarySmoothResult {
    pub positions: Vec<[f32; 3]>,
    pub smoothed_count: usize,
    pub max_displacement: f32,
}

/// Default boundary smooth config.
#[allow(dead_code)]
pub fn default_boundary_smooth_config() -> BoundarySmoothConfig {
    BoundarySmoothConfig {
        iterations: 3,
        factor: 0.5,
    }
}

/// Find boundary edges (edges shared by exactly one triangle).
#[allow(dead_code)]
pub fn find_boundary_edges(indices: &[[u32; 3]]) -> Vec<(u32, u32)> {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(u32, u32), usize> = HashMap::new();
    for tri in indices {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    edge_count
        .into_iter()
        .filter(|&(_, c)| c == 1)
        .map(|(k, _)| k)
        .collect()
}

/// Find boundary vertices.
#[allow(dead_code)]
pub fn find_boundary_vertices(indices: &[[u32; 3]]) -> Vec<u32> {
    use std::collections::HashSet;
    let edges = find_boundary_edges(indices);
    let mut verts: HashSet<u32> = HashSet::new();
    for (a, b) in &edges {
        verts.insert(*a);
        verts.insert(*b);
    }
    let mut result: Vec<u32> = verts.into_iter().collect();
    result.sort();
    result
}

/// Build adjacency for boundary vertices only.
#[allow(dead_code)]
pub fn boundary_adjacency(
    boundary_verts: &[u32],
    boundary_edges: &[(u32, u32)],
) -> Vec<Vec<u32>> {
    use std::collections::HashMap;
    let mut idx_map: HashMap<u32, usize> = HashMap::new();
    for (i, &v) in boundary_verts.iter().enumerate() {
        idx_map.insert(v, i);
    }
    let mut adj = vec![Vec::new(); boundary_verts.len()];
    for &(a, b) in boundary_edges {
        if let (Some(&ia), Some(&ib)) = (idx_map.get(&a), idx_map.get(&b)) {
            adj[ia].push(b);
            adj[ib].push(a);
        }
    }
    adj
}

/// Smooth boundary vertices for one iteration.
#[allow(dead_code)]
pub fn smooth_boundary_step(
    positions: &mut [[f32; 3]],
    boundary_verts: &[u32],
    boundary_edges: &[(u32, u32)],
    factor: f32,
) -> f32 {
    let adj = boundary_adjacency(boundary_verts, boundary_edges);
    let mut max_disp = 0.0f32;
    let old_positions: Vec<[f32; 3]> = positions.to_vec();
    for (i, &vi) in boundary_verts.iter().enumerate() {
        let neighbors = &adj[i];
        if neighbors.is_empty() {
            continue;
        }
        let mut avg = [0.0f32; 3];
        for &n in neighbors {
            avg[0] += old_positions[n as usize][0];
            avg[1] += old_positions[n as usize][1];
            avg[2] += old_positions[n as usize][2];
        }
        let inv = 1.0 / neighbors.len() as f32;
        avg[0] *= inv;
        avg[1] *= inv;
        avg[2] *= inv;

        let idx = vi as usize;
        let dx = (avg[0] - old_positions[idx][0]) * factor;
        let dy = (avg[1] - old_positions[idx][1]) * factor;
        let dz = (avg[2] - old_positions[idx][2]) * factor;
        positions[idx][0] += dx;
        positions[idx][1] += dy;
        positions[idx][2] += dz;
        let disp = (dx * dx + dy * dy + dz * dz).sqrt();
        if disp > max_disp {
            max_disp = disp;
        }
    }
    max_disp
}

/// Smooth boundary vertices of a mesh.
#[allow(dead_code)]
pub fn smooth_boundary(
    positions: &[[f32; 3]],
    indices: &[[u32; 3]],
    config: &BoundarySmoothConfig,
) -> BoundarySmoothResult {
    let mut new_positions = positions.to_vec();
    let boundary_edges = find_boundary_edges(indices);
    let boundary_verts = find_boundary_vertices(indices);
    let mut max_disp = 0.0f32;
    for _ in 0..config.iterations {
        let d = smooth_boundary_step(&mut new_positions, &boundary_verts, &boundary_edges, config.factor);
        if d > max_disp {
            max_disp = d;
        }
    }
    BoundarySmoothResult {
        positions: new_positions,
        smoothed_count: boundary_verts.len(),
        max_displacement: max_disp,
    }
}

/// Validate config.
#[allow(dead_code)]
pub fn validate_boundary_smooth_config(config: &BoundarySmoothConfig) -> bool {
    config.iterations >= 1 && (0.0..=1.0).contains(&config.factor)
}

/// Serialize result to JSON.
#[allow(dead_code)]
pub fn boundary_smooth_to_json(result: &BoundarySmoothResult) -> String {
    format!(
        "{{\"smoothed_count\":{},\"max_displacement\":{:.6}}}",
        result.smoothed_count, result.max_displacement
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_mesh() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let p = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ];
        let i = vec![[0, 1, 2]];
        (p, i)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_boundary_smooth_config();
        assert_eq!(cfg.iterations, 3);
    }

    #[test]
    fn test_find_boundary_edges() {
        let (_, i) = open_mesh();
        let edges = find_boundary_edges(&i);
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn test_find_boundary_vertices() {
        let (_, i) = open_mesh();
        let verts = find_boundary_vertices(&i);
        assert_eq!(verts.len(), 3);
    }

    #[test]
    fn test_boundary_adjacency() {
        let (_, i) = open_mesh();
        let edges = find_boundary_edges(&i);
        let verts = find_boundary_vertices(&i);
        let adj = boundary_adjacency(&verts, &edges);
        assert_eq!(adj.len(), 3);
    }

    #[test]
    fn test_smooth_boundary() {
        let (p, i) = open_mesh();
        let cfg = default_boundary_smooth_config();
        let result = smooth_boundary(&p, &i, &cfg);
        assert_eq!(result.positions.len(), p.len());
    }

    #[test]
    fn test_smoothed_count() {
        let (p, i) = open_mesh();
        let cfg = default_boundary_smooth_config();
        let result = smooth_boundary(&p, &i, &cfg);
        assert_eq!(result.smoothed_count, 3);
    }

    #[test]
    fn test_validate_config() {
        let cfg = default_boundary_smooth_config();
        assert!(validate_boundary_smooth_config(&cfg));
    }

    #[test]
    fn test_validate_bad_config() {
        let bad = BoundarySmoothConfig { iterations: 0, factor: 0.5 };
        assert!(!validate_boundary_smooth_config(&bad));
    }

    #[test]
    fn test_boundary_smooth_to_json() {
        let (p, i) = open_mesh();
        let cfg = default_boundary_smooth_config();
        let result = smooth_boundary(&p, &i, &cfg);
        let json = boundary_smooth_to_json(&result);
        assert!(json.contains("smoothed_count"));
    }

    #[test]
    fn test_empty_mesh() {
        let result = smooth_boundary(&[], &[], &default_boundary_smooth_config());
        assert_eq!(result.smoothed_count, 0);
    }
}
