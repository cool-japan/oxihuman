// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export per-edge normal data.

/// A per-edge normal entry.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct EdgeNormalEntry {
    pub edge_v0: u32,
    pub edge_v1: u32,
    pub normal: [f32; 3],
}

/// Per-edge normal export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeNormalExport {
    pub entries: Vec<EdgeNormalEntry>,
}

/// Create a new edge normal export.
#[allow(dead_code)]
pub fn new_edge_normal_export() -> EdgeNormalExport {
    EdgeNormalExport {
        entries: Vec::new(),
    }
}

/// Add a per-edge normal.
#[allow(dead_code)]
pub fn add_edge_normal(export: &mut EdgeNormalExport, entry: EdgeNormalEntry) {
    export.entries.push(entry);
}

/// Count entries.
#[allow(dead_code)]
pub fn edge_normal_count(export: &EdgeNormalExport) -> usize {
    export.entries.len()
}

/// Normalize a 3-D vector.
#[allow(dead_code)]
pub fn normalize_en(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        return [0.0, 1.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Validate that all normals are unit length.
#[allow(dead_code)]
pub fn normals_unit_en(export: &EdgeNormalExport) -> bool {
    export.entries.iter().all(|e| {
        let len2 =
            e.normal[0] * e.normal[0] + e.normal[1] * e.normal[1] + e.normal[2] * e.normal[2];
        (len2 - 1.0).abs() < 1e-4
    })
}

/// Compute the average normal vector.
#[allow(dead_code)]
pub fn avg_edge_normal(export: &EdgeNormalExport) -> [f32; 3] {
    let n = export.entries.len();
    if n == 0 {
        return [0.0, 1.0, 0.0];
    }
    let mut sum = [0.0f32; 3];
    for e in &export.entries {
        sum[0] += e.normal[0];
        sum[1] += e.normal[1];
        sum[2] += e.normal[2];
    }
    normalize_en([sum[0] / n as f32, sum[1] / n as f32, sum[2] / n as f32])
}

/// Find entry for a given edge (order-insensitive).
#[allow(dead_code)]
pub fn find_edge_normal(export: &EdgeNormalExport, v0: u32, v1: u32) -> Option<[f32; 3]> {
    export
        .entries
        .iter()
        .find(|e| (e.edge_v0 == v0 && e.edge_v1 == v1) || (e.edge_v0 == v1 && e.edge_v1 == v0))
        .map(|e| e.normal)
}

/// Compute edge normals from mesh (average of adjacent face normals).
#[allow(dead_code)]
pub fn compute_from_mesh(positions: &[[f32; 3]], indices: &[u32]) -> EdgeNormalExport {
    let tri_count = indices.len() / 3;
    let mut edge_accum: std::collections::HashMap<(u32, u32), ([f32; 3], u32)> =
        std::collections::HashMap::new();
    for t in 0..tri_count {
        let i0 = indices[t * 3];
        let i1 = indices[t * 3 + 1];
        let i2 = indices[t * 3 + 2];
        if i0 as usize >= positions.len()
            || i1 as usize >= positions.len()
            || i2 as usize >= positions.len()
        {
            continue;
        }
        let p0 = positions[i0 as usize];
        let p1 = positions[i1 as usize];
        let p2 = positions[i2 as usize];
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let n = normalize_en([
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ]);
        for (a, b) in [(i0, i1), (i1, i2), (i2, i0)] {
            let key = if a < b { (a, b) } else { (b, a) };
            let acc = edge_accum.entry(key).or_insert(([0.0; 3], 0));
            acc.0[0] += n[0];
            acc.0[1] += n[1];
            acc.0[2] += n[2];
            acc.1 += 1;
        }
    }
    let mut entries = Vec::new();
    for ((v0, v1), (sum, cnt)) in edge_accum {
        let avg = normalize_en([
            sum[0] / cnt as f32,
            sum[1] / cnt as f32,
            sum[2] / cnt as f32,
        ]);
        entries.push(EdgeNormalEntry {
            edge_v0: v0,
            edge_v1: v1,
            normal: avg,
        });
    }
    EdgeNormalExport { entries }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn edge_normal_to_json(export: &EdgeNormalExport) -> String {
    format!("{{\"entry_count\":{}}}", export.entries.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri_export() -> EdgeNormalExport {
        compute_from_mesh(
            &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            &[0, 1, 2],
        )
    }

    #[test]
    fn test_compute_from_mesh_edge_count() {
        let e = tri_export();
        assert_eq!(edge_normal_count(&e), 3);
    }

    #[test]
    fn test_normals_unit() {
        let e = tri_export();
        assert!(normals_unit_en(&e));
    }

    #[test]
    fn test_add_and_count() {
        let mut e = new_edge_normal_export();
        add_edge_normal(
            &mut e,
            EdgeNormalEntry {
                edge_v0: 0,
                edge_v1: 1,
                normal: [0.0, 1.0, 0.0],
            },
        );
        assert_eq!(edge_normal_count(&e), 1);
    }

    #[test]
    fn test_find_edge_normal_found() {
        let mut e = new_edge_normal_export();
        add_edge_normal(
            &mut e,
            EdgeNormalEntry {
                edge_v0: 0,
                edge_v1: 1,
                normal: [0.0, 1.0, 0.0],
            },
        );
        assert!(find_edge_normal(&e, 0, 1).is_some());
    }

    #[test]
    fn test_find_edge_normal_reversed() {
        let mut e = new_edge_normal_export();
        add_edge_normal(
            &mut e,
            EdgeNormalEntry {
                edge_v0: 0,
                edge_v1: 1,
                normal: [0.0, 1.0, 0.0],
            },
        );
        assert!(find_edge_normal(&e, 1, 0).is_some());
    }

    #[test]
    fn test_avg_normal_single() {
        let mut e = new_edge_normal_export();
        add_edge_normal(
            &mut e,
            EdgeNormalEntry {
                edge_v0: 0,
                edge_v1: 1,
                normal: [0.0, 1.0, 0.0],
            },
        );
        let avg = avg_edge_normal(&e);
        assert!((avg[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_en_unit_vector() {
        let n = normalize_en([3.0, 0.0, 0.0]);
        assert!((n[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_edge_normal_to_json() {
        let e = tri_export();
        let j = edge_normal_to_json(&e);
        assert!(j.contains("entry_count"));
    }

    #[test]
    fn test_empty_mesh_zero_edges() {
        let e = compute_from_mesh(&[], &[]);
        assert_eq!(edge_normal_count(&e), 0);
    }
}
