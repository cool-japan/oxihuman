// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hole filling from boundary edges: detect and fill mesh holes.

/// Configuration for hole filling.
#[derive(Debug, Clone)]
pub struct FillHolesConfig {
    /// Maximum number of boundary edges a hole can have.
    pub max_hole_edges: usize,
    /// Fill method: fan or ear-clip.
    pub use_fan_fill: bool,
}

impl Default for FillHolesConfig {
    fn default() -> Self {
        Self {
            max_hole_edges: 100,
            use_fan_fill: true,
        }
    }
}

/// A detected hole described by its boundary edge loop.
#[derive(Debug, Clone)]
pub struct MeshHoleDetected {
    pub boundary_verts: Vec<usize>,
}

/// Result of hole filling.
#[derive(Debug, Clone)]
pub struct FillHolesResult {
    pub filled_triangles: Vec<[usize; 3]>,
    pub hole_count: usize,
}

/// Detect boundary edge loops in a triangle mesh.
pub fn detect_holes(triangles: &[[usize; 3]]) -> Vec<MeshHoleDetected> {
    /* Count how many times each directed edge appears */
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    for &[a, b, c] in triangles {
        *edge_count.entry((a, b)).or_insert(0) += 1;
        *edge_count.entry((b, c)).or_insert(0) += 1;
        *edge_count.entry((c, a)).or_insert(0) += 1;
    }
    /* Boundary edges: directed edge (u,v) with no matching (v,u) in face edges */
    let mut boundary: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for &(u, v) in edge_count.keys() {
        if !edge_count.contains_key(&(v, u)) {
            boundary.insert(u, v);
        }
    }
    /* Walk loops */
    let mut visited: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut holes: Vec<MeshHoleDetected> = Vec::new();
    for &start in boundary.keys() {
        if visited.contains(&start) {
            continue;
        }
        let mut loop_verts = Vec::new();
        let mut cur = start;
        loop {
            if visited.contains(&cur) {
                break;
            }
            visited.insert(cur);
            loop_verts.push(cur);
            match boundary.get(&cur) {
                Some(&next) => cur = next,
                None => break,
            }
        }
        if loop_verts.len() >= 3 {
            holes.push(MeshHoleDetected {
                boundary_verts: loop_verts,
            });
        }
    }
    holes
}

/// Fan-fill a boundary loop.
fn fan_fill(loop_verts: &[usize]) -> Vec<[usize; 3]> {
    if loop_verts.len() < 3 {
        return vec![];
    }
    let pivot = loop_verts[0];
    (1..loop_verts.len().saturating_sub(1))
        .map(|i| [pivot, loop_verts[i], loop_verts[i + 1]])
        .collect()
}

/// Fill holes detected in the mesh.
pub fn fill_holes(triangles: &[[usize; 3]], config: &FillHolesConfig) -> FillHolesResult {
    let holes = detect_holes(triangles);
    let mut filled_triangles: Vec<[usize; 3]> = Vec::new();
    let mut hole_count = 0;

    for hole in &holes {
        if hole.boundary_verts.len() > config.max_hole_edges {
            continue;
        }
        let tris = fan_fill(&hole.boundary_verts);
        filled_triangles.extend(tris);
        hole_count += 1;
    }

    FillHolesResult {
        filled_triangles,
        hole_count,
    }
}

/// Count triangles generated to fill holes.
pub fn fill_triangle_count(result: &FillHolesResult) -> usize {
    result.filled_triangles.len()
}

/// Check if a mesh has any holes.
pub fn has_holes(triangles: &[[usize; 3]]) -> bool {
    !detect_holes(triangles).is_empty()
}

/// Count the boundary vertices in a detected hole.
pub fn boundary_vert_count(hole: &MeshHoleDetected) -> usize {
    hole.boundary_verts.len()
}

/// Merge the filled triangles with the original mesh.
pub fn merge_filled(original: &[[usize; 3]], result: &FillHolesResult) -> Vec<[usize; 3]> {
    let mut out = original.to_vec();
    out.extend_from_slice(&result.filled_triangles);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_box_tris() -> Vec<[usize; 3]> {
        /* A box missing one face → one boundary loop */
        vec![
            [0, 1, 2],
            [0, 2, 3], /* top face */
            [0, 1, 4],
            [1, 5, 4], /* right side */
            [1, 2, 5],
            [2, 6, 5], /* back */
            [3, 2, 6],
            [3, 6, 7], /* left */
        ]
    }

    #[test]
    fn test_detect_holes_open_box() {
        let tris = open_box_tris();
        let holes = detect_holes(&tris);
        /* At least one hole detected */
        assert!(!holes.is_empty());
    }

    #[test]
    fn test_has_holes_true() {
        assert!(has_holes(&open_box_tris()));
    }

    #[test]
    fn test_has_holes_closed() {
        /* A fully closed tetrahedron has no holes */
        let tris = vec![[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]];
        assert!(!has_holes(&tris));
    }

    #[test]
    fn test_fill_holes_produces_tris() {
        let tris = open_box_tris();
        let cfg = FillHolesConfig::default();
        let result = fill_holes(&tris, &cfg);
        assert!(fill_triangle_count(&result) > 0);
    }

    #[test]
    fn test_fill_holes_hole_count() {
        let tris = open_box_tris();
        let cfg = FillHolesConfig::default();
        let result = fill_holes(&tris, &cfg);
        assert!(result.hole_count > 0);
    }

    #[test]
    fn test_merge_filled() {
        let tris = open_box_tris();
        let cfg = FillHolesConfig::default();
        let result = fill_holes(&tris, &cfg);
        let merged = merge_filled(&tris, &result);
        assert!(merged.len() >= tris.len());
    }

    #[test]
    fn test_fill_triangle_count_zero_for_closed() {
        let tris = vec![[0usize, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]];
        let cfg = FillHolesConfig::default();
        let result = fill_holes(&tris, &cfg);
        assert_eq!(fill_triangle_count(&result), 0);
    }

    #[test]
    fn test_empty_mesh() {
        let cfg = FillHolesConfig::default();
        let result = fill_holes(&[], &cfg);
        assert_eq!(result.hole_count, 0);
    }

    #[test]
    fn test_max_hole_edges_filter() {
        let tris = open_box_tris();
        /* Very small max_hole_edges → holes filtered out */
        let cfg = FillHolesConfig {
            max_hole_edges: 1,
            use_fan_fill: true,
        };
        let result = fill_holes(&tris, &cfg);
        assert_eq!(result.hole_count, 0);
    }
}
