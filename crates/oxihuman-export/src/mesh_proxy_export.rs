// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mesh proxy export: low-res stand-in mesh export.

/// Mesh proxy export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshProxyExport2 {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub source_vertex_count: usize,
}

#[allow(dead_code)]
pub fn new_mesh_proxy2(
    positions: &[[f32; 3]],
    indices: &[u32],
    source_count: usize,
) -> MeshProxyExport2 {
    MeshProxyExport2 {
        positions: positions.to_vec(),
        indices: indices.to_vec(),
        source_vertex_count: source_count,
    }
}

#[allow(dead_code)]
pub fn proxy2_vertex_count_fn(p: &MeshProxyExport2) -> usize {
    p.positions.len()
}

#[allow(dead_code)]
pub fn proxy2_triangle_count(p: &MeshProxyExport2) -> usize {
    p.indices.len() / 3
}

#[allow(dead_code)]
pub fn proxy2_reduction_ratio(p: &MeshProxyExport2) -> f32 {
    if p.source_vertex_count == 0 {
        return 0.0;
    }
    p.positions.len() as f32 / p.source_vertex_count as f32
}

#[allow(dead_code)]
pub fn proxy2_bounds_fn(p: &MeshProxyExport2) -> ([f32; 3], [f32; 3]) {
    if p.positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = p.positions[0];
    let mut mx = p.positions[0];
    for v in &p.positions {
        for k in 0..3 {
            mn[k] = mn[k].min(v[k]);
            mx[k] = mx[k].max(v[k]);
        }
    }
    (mn, mx)
}

#[allow(dead_code)]
pub fn proxy2_center_fn(p: &MeshProxyExport2) -> [f32; 3] {
    let (mn, mx) = proxy2_bounds_fn(p);
    [
        (mn[0] + mx[0]) * 0.5,
        (mn[1] + mx[1]) * 0.5,
        (mn[2] + mx[2]) * 0.5,
    ]
}

#[allow(dead_code)]
pub fn proxy2_to_obj(p: &MeshProxyExport2) -> String {
    let mut s = String::new();
    for v in &p.positions {
        s.push_str(&format!("v {} {} {}\n", v[0], v[1], v[2]));
    }
    let tri = p.indices.len() / 3;
    for t in 0..tri {
        s.push_str(&format!(
            "f {} {} {}\n",
            p.indices[t * 3] + 1,
            p.indices[t * 3 + 1] + 1,
            p.indices[t * 3 + 2] + 1
        ));
    }
    s
}

#[allow(dead_code)]
pub fn mesh_proxy2_to_json(p: &MeshProxyExport2) -> String {
    format!(
        "{{\"vertices\":{},\"triangles\":{},\"ratio\":{:.6}}}",
        p.positions.len(),
        p.indices.len() / 3,
        proxy2_reduction_ratio(p)
    )
}

#[allow(dead_code)]
pub fn proxy2_validate(p: &MeshProxyExport2) -> bool {
    p.indices.iter().all(|&i| (i as usize) < p.positions.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri() -> MeshProxyExport2 {
        new_mesh_proxy2(
            &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            &[0, 1, 2],
            100,
        )
    }

    #[test]
    fn test_new() {
        assert_eq!(proxy2_vertex_count_fn(&tri()), 3);
    }

    #[test]
    fn test_triangle_count() {
        assert_eq!(proxy2_triangle_count(&tri()), 1);
    }

    #[test]
    fn test_reduction_ratio() {
        assert!((proxy2_reduction_ratio(&tri()) - 0.03).abs() < 1e-6);
    }

    #[test]
    fn test_bounds() {
        let (mn, mx) = proxy2_bounds_fn(&tri());
        assert!((mn[0]).abs() < 1e-6);
        assert!((mx[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_center() {
        let c = proxy2_center_fn(&tri());
        assert!((c[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_obj() {
        let s = proxy2_to_obj(&tri());
        assert!(s.contains("v "));
        assert!(s.contains("f "));
    }

    #[test]
    fn test_to_json() {
        assert!(mesh_proxy2_to_json(&tri()).contains("\"vertices\":3"));
    }

    #[test]
    fn test_validate_ok() {
        assert!(proxy2_validate(&tri()));
    }

    #[test]
    fn test_validate_bad() {
        let p = MeshProxyExport2 {
            positions: vec![[0.0; 3]],
            indices: vec![5],
            source_vertex_count: 1,
        };
        assert!(!proxy2_validate(&p));
    }

    #[test]
    fn test_empty_ratio() {
        let p = new_mesh_proxy2(&[], &[], 0);
        assert!((proxy2_reduction_ratio(&p)).abs() < 1e-6);
    }
}
