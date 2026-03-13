// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Edge-normal smoothing and visualisation helpers.

use std::f32::consts::FRAC_PI_2;

/// A mesh edge (vertex pair).
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeshEdge {
    pub v0: u32,
    pub v1: u32,
}

/// Per-edge normal data.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct EdgeNormalEntry {
    pub edge: MeshEdge,
    pub normal: [f32; 3],
    /// Dihedral angle in radians between adjacent faces.
    pub dihedral_rad: f32,
}

/// Collection of edge normals.
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct EdgeNormalMap {
    pub entries: Vec<EdgeNormalEntry>,
}

#[allow(dead_code)]
pub fn new_edge_normal_map() -> EdgeNormalMap {
    EdgeNormalMap::default()
}

#[allow(dead_code)]
pub fn en_add(map: &mut EdgeNormalMap, edge: MeshEdge, normal: [f32; 3], dihedral_rad: f32) {
    map.entries.push(EdgeNormalEntry {
        edge,
        normal,
        dihedral_rad,
    });
}

#[allow(dead_code)]
pub fn en_count(map: &EdgeNormalMap) -> usize {
    map.entries.len()
}

#[allow(dead_code)]
pub fn en_clear(map: &mut EdgeNormalMap) {
    map.entries.clear();
}

/// Normalise a 3-vector.
#[allow(dead_code)]
pub fn en_normalize(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-9 {
        return [0.0, 1.0, 0.0];
    }
    [v[0] / l, v[1] / l, v[2] / l]
}

/// Filter edges whose dihedral angle exceeds a crease threshold (default = 60°, using FRAC_PI_2 internally).
#[allow(dead_code)]
pub fn en_crease_edges(map: &EdgeNormalMap, threshold_rad: f32) -> Vec<&EdgeNormalEntry> {
    let _ = FRAC_PI_2; // used for reference
    map.entries
        .iter()
        .filter(|e| e.dihedral_rad >= threshold_rad)
        .collect()
}

#[allow(dead_code)]
pub fn en_average_dihedral(map: &EdgeNormalMap) -> f32 {
    if map.entries.is_empty() {
        return 0.0;
    }
    map.entries.iter().map(|e| e.dihedral_rad).sum::<f32>() / map.entries.len() as f32
}

#[allow(dead_code)]
pub fn en_max_dihedral(map: &EdgeNormalMap) -> f32 {
    map.entries
        .iter()
        .map(|e| e.dihedral_rad)
        .fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn en_to_json(map: &EdgeNormalMap) -> String {
    format!(
        "{{\"count\":{},\"avg_dihedral\":{:.4}}}",
        en_count(map),
        en_average_dihedral(map)
    )
}

#[allow(dead_code)]
pub fn en_find_by_vertices(map: &EdgeNormalMap, v0: u32, v1: u32) -> Option<&EdgeNormalEntry> {
    map.entries
        .iter()
        .find(|e| (e.edge.v0 == v0 && e.edge.v1 == v1) || (e.edge.v0 == v1 && e.edge.v1 == v0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        assert_eq!(en_count(&new_edge_normal_map()), 0);
    }

    #[test]
    fn add_entry() {
        let mut m = new_edge_normal_map();
        en_add(&mut m, MeshEdge { v0: 0, v1: 1 }, [0.0, 1.0, 0.0], 0.5);
        assert_eq!(en_count(&m), 1);
    }

    #[test]
    fn clear() {
        let mut m = new_edge_normal_map();
        en_add(&mut m, MeshEdge { v0: 0, v1: 1 }, [0.0, 1.0, 0.0], 0.5);
        en_clear(&mut m);
        assert_eq!(en_count(&m), 0);
    }

    #[test]
    fn normalize_unit_vector() {
        let n = en_normalize([3.0, 0.0, 0.0]);
        assert!((n[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn normalize_zero_safe() {
        let n = en_normalize([0.0, 0.0, 0.0]);
        assert!((n[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn crease_filter() {
        let mut m = new_edge_normal_map();
        en_add(&mut m, MeshEdge { v0: 0, v1: 1 }, [0.0, 1.0, 0.0], 1.5);
        en_add(&mut m, MeshEdge { v0: 1, v1: 2 }, [0.0, 1.0, 0.0], 0.2);
        let c = en_crease_edges(&m, 1.0);
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn average_dihedral_empty() {
        assert!((en_average_dihedral(&new_edge_normal_map())).abs() < 1e-5);
    }

    #[test]
    fn max_dihedral() {
        let mut m = new_edge_normal_map();
        en_add(&mut m, MeshEdge { v0: 0, v1: 1 }, [1.0, 0.0, 0.0], 2.5);
        assert!((en_max_dihedral(&m) - 2.5).abs() < 1e-5);
    }

    #[test]
    fn find_by_vertices() {
        let mut m = new_edge_normal_map();
        en_add(&mut m, MeshEdge { v0: 3, v1: 7 }, [0.0, 1.0, 0.0], 1.0);
        assert!(en_find_by_vertices(&m, 3, 7).is_some());
        assert!(en_find_by_vertices(&m, 7, 3).is_some());
    }

    #[test]
    fn json_has_count() {
        assert!(en_to_json(&new_edge_normal_map()).contains("count"));
    }
}
