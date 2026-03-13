// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cap hole: fill boundary loops with a fan of triangles.

/// Result of capping a hole.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CapHoleResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
    pub holes_capped: usize,
    pub new_faces: usize,
}

/// Default config.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CapHoleConfig {
    pub max_hole_size: usize,
}

#[allow(dead_code)]
pub fn default_cap_hole_config() -> CapHoleConfig {
    CapHoleConfig { max_hole_size: 64 }
}

/// Find boundary loops (ordered vertex rings).
#[allow(dead_code)]
pub fn find_boundary_loops(indices: &[[u32; 3]]) -> Vec<Vec<u32>> {
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
    let boundary: Vec<(u32, u32)> = edge_count
        .into_iter()
        .filter(|&(_, c)| c == 1)
        .map(|(k, _)| k)
        .collect();
    if boundary.is_empty() {
        return Vec::new();
    }
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    for &(a, b) in &boundary {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }
    let mut visited = std::collections::HashSet::new();
    let mut loops = Vec::new();
    for &(start, _) in &boundary {
        if visited.contains(&start) { continue; }
        let mut loop_verts = vec![start];
        visited.insert(start);
        let mut current = start;
        loop {
            let neighbors = adj.get(&current).cloned().unwrap_or_default();
            let next = neighbors.iter().find(|n| !visited.contains(n));
            match next {
                Some(&n) => {
                    loop_verts.push(n);
                    visited.insert(n);
                    current = n;
                }
                None => break,
            }
        }
        if loop_verts.len() >= 3 {
            loops.push(loop_verts);
        }
    }
    loops
}

/// Compute centroid of a vertex loop.
#[allow(dead_code)]
pub fn loop_centroid(positions: &[[f32; 3]], loop_verts: &[u32]) -> [f32; 3] {
    let mut c = [0.0f32; 3];
    for &v in loop_verts {
        let p = positions[v as usize];
        c[0] += p[0]; c[1] += p[1]; c[2] += p[2];
    }
    let inv = 1.0 / loop_verts.len() as f32;
    [c[0] * inv, c[1] * inv, c[2] * inv]
}

/// Cap a single boundary loop with a fan.
#[allow(dead_code)]
pub fn cap_single_loop(
    positions: &mut Vec<[f32; 3]>,
    indices: &mut Vec<[u32; 3]>,
    loop_verts: &[u32],
) -> usize {
    let center = loop_centroid(positions, loop_verts);
    let ci = positions.len() as u32;
    positions.push(center);
    let n = loop_verts.len();
    for i in 0..n {
        let a = loop_verts[i];
        let b = loop_verts[(i + 1) % n];
        indices.push([ci, a, b]);
    }
    n
}

/// Cap all boundary holes.
#[allow(dead_code)]
pub fn cap_holes(
    positions: &[[f32; 3]],
    indices: &[[u32; 3]],
    config: &CapHoleConfig,
) -> CapHoleResult {
    let mut new_pos = positions.to_vec();
    let mut new_idx = indices.to_vec();
    let loops = find_boundary_loops(indices);
    let mut holes_capped = 0;
    let mut new_faces = 0;
    for lp in &loops {
        if lp.len() <= config.max_hole_size {
            let f = cap_single_loop(&mut new_pos, &mut new_idx, lp);
            holes_capped += 1;
            new_faces += f;
        }
    }
    CapHoleResult {
        positions: new_pos,
        indices: new_idx,
        holes_capped,
        new_faces,
    }
}

/// Check if mesh has no boundary (is closed/watertight).
#[allow(dead_code)]
pub fn is_watertight(indices: &[[u32; 3]]) -> bool {
    find_boundary_loops(indices).is_empty()
}

/// Serialize cap hole result to JSON.
#[allow(dead_code)]
pub fn cap_hole_to_json(result: &CapHoleResult) -> String {
    format!(
        "{{\"holes_capped\":{},\"new_faces\":{},\"total_verts\":{},\"total_faces\":{}}}",
        result.holes_capped,
        result.new_faces,
        result.positions.len(),
        result.indices.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_tri() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0]];
        let i = vec![[0,1,2]];
        (p, i)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_cap_hole_config();
        assert_eq!(cfg.max_hole_size, 64);
    }

    #[test]
    fn test_find_boundary_loops_single_tri() {
        let (_, i) = open_tri();
        let loops = find_boundary_loops(&i);
        assert!(!loops.is_empty());
    }

    #[test]
    fn test_loop_centroid() {
        let p = vec![[0.0,0.0,0.0],[2.0,0.0,0.0],[1.0,2.0,0.0]];
        let c = loop_centroid(&p, &[0, 1, 2]);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cap_holes() {
        let (p, i) = open_tri();
        let cfg = default_cap_hole_config();
        let result = cap_holes(&p, &i, &cfg);
        assert!(result.positions.len() >= p.len());
    }

    #[test]
    fn test_is_watertight_open() {
        let (_, i) = open_tri();
        assert!(!is_watertight(&i));
    }

    #[test]
    fn test_cap_hole_to_json() {
        let (p, i) = open_tri();
        let cfg = default_cap_hole_config();
        let result = cap_holes(&p, &i, &cfg);
        let json = cap_hole_to_json(&result);
        assert!(json.contains("holes_capped"));
    }

    #[test]
    fn test_empty_mesh() {
        let result = cap_holes(&[], &[], &default_cap_hole_config());
        assert_eq!(result.holes_capped, 0);
    }

    #[test]
    fn test_closed_mesh_no_caps() {
        // Tetrahedron is closed
        let p = vec![
            [0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0],[0.5,0.5,1.0],
        ];
        let i = vec![[0,1,2],[0,1,3],[1,2,3],[0,2,3]];
        let result = cap_holes(&p, &i, &default_cap_hole_config());
        assert_eq!(result.holes_capped, 0);
    }

    #[test]
    fn test_cap_single_loop() {
        let mut p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0]];
        let mut i: Vec<[u32; 3]> = Vec::new();
        let n = cap_single_loop(&mut p, &mut i, &[0, 1, 2]);
        assert_eq!(n, 3);
        assert_eq!(i.len(), 3);
    }

    #[test]
    fn test_find_boundary_vertices_count() {
        let (_, i) = open_tri();
        let loops = find_boundary_loops(&i);
        let total_verts: usize = loops.iter().map(|l| l.len()).sum();
        assert!(total_verts > 0);
    }
}
