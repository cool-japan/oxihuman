// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Low-resolution cage proxy mesh for deformation.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CageProxyConfig {
    pub resolution: usize,
    pub padding: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CageProxy {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub bind_coords: Vec<[f32; 4]>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CageBindResult {
    pub bound_vertices: usize,
    pub max_bind_error: f32,
}

#[allow(dead_code)]
pub fn default_cage_proxy_config() -> CageProxyConfig {
    CageProxyConfig {
        resolution: 4,
        padding: 0.05,
    }
}

/// Compute axis-aligned bounding box of source positions.
fn compute_aabb(positions: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for p in positions {
        for k in 0..3 {
            if p[k] < mn[k] { mn[k] = p[k]; }
            if p[k] > mx[k] { mx[k] = p[k]; }
        }
    }
    (mn, mx)
}

/// Build a simple box cage from AABB.
#[allow(dead_code)]
pub fn build_cage_proxy(source_positions: &[[f32; 3]], config: &CageProxyConfig) -> CageProxy {
    let (mn, mx) = compute_aabb(source_positions);
    let pad = config.padding;
    let lo = [mn[0] - pad, mn[1] - pad, mn[2] - pad];
    let hi = [mx[0] + pad, mx[1] + pad, mx[2] + pad];

    // Box cage: 8 corners
    let positions = vec![
        [lo[0], lo[1], lo[2]],
        [hi[0], lo[1], lo[2]],
        [hi[0], hi[1], lo[2]],
        [lo[0], hi[1], lo[2]],
        [lo[0], lo[1], hi[2]],
        [hi[0], lo[1], hi[2]],
        [hi[0], hi[1], hi[2]],
        [lo[0], hi[1], hi[2]],
    ];

    // 12 triangles (6 quad faces split into 2 triangles each)
    #[allow(clippy::unreadable_literal)]
    let indices: Vec<u32> = vec![
        0,1,2, 0,2,3, // -Z
        4,6,5, 4,7,6, // +Z
        0,4,5, 0,5,1, // -Y
        2,6,7, 2,7,3, // +Y
        0,3,7, 0,7,4, // -X
        1,5,6, 1,6,2, // +X
    ];

    let bind_coords = Vec::new();
    CageProxy { positions, indices, bind_coords }
}

/// Compute barycentric-like bind coordinates for a vertex relative to the cage AABB.
#[allow(dead_code)]
pub fn cage_bind_vertex(cage: &CageProxy, vertex: [f32; 3]) -> [f32; 4] {
    if cage.positions.is_empty() {
        return [0.0; 4];
    }
    let (mn, mx) = compute_aabb(&cage.positions);
    let sx = mx[0] - mn[0];
    let sy = mx[1] - mn[1];
    let sz = mx[2] - mn[2];
    let u = if sx.abs() < 1e-9 { 0.5 } else { (vertex[0] - mn[0]) / sx };
    let v = if sy.abs() < 1e-9 { 0.5 } else { (vertex[1] - mn[1]) / sy };
    let w = if sz.abs() < 1e-9 { 0.5 } else { (vertex[2] - mn[2]) / sz };
    [u, v, w, 1.0]
}

#[allow(dead_code)]
pub fn cage_proxy_vertex_count(cage: &CageProxy) -> usize {
    cage.positions.len()
}

#[allow(dead_code)]
pub fn cage_proxy_face_count(cage: &CageProxy) -> usize {
    cage.indices.len() / 3
}

#[allow(dead_code)]
pub fn cage_bind_all(cage: &CageProxy, vertices: &[[f32; 3]]) -> CageBindResult {
    let mut max_err = 0.0f32;
    for v in vertices {
        let bc = cage_bind_vertex(cage, *v);
        // Error: distance of u,v,w from [0,1] clamp (how far outside cage)
        let err = bc[0..3]
            .iter()
            .map(|&x| (x - x.clamp(0.0, 1.0)).abs())
            .fold(0.0f32, f32::max);
        if err > max_err {
            max_err = err;
        }
    }
    CageBindResult {
        bound_vertices: vertices.len(),
        max_bind_error: max_err,
    }
}

#[allow(dead_code)]
pub fn cage_to_json(cage: &CageProxy) -> String {
    format!(
        "{{\"vertex_count\":{},\"face_count\":{},\"bind_count\":{}}}",
        cage.positions.len(),
        cage.indices.len() / 3,
        cage.bind_coords.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cloud() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
        ]
    }

    #[test]
    fn test_default_config() {
        let cfg = default_cage_proxy_config();
        assert_eq!(cfg.resolution, 4);
    }

    #[test]
    fn test_build_cage_vertex_count() {
        let cloud = sample_cloud();
        let cfg = default_cage_proxy_config();
        let cage = build_cage_proxy(&cloud, &cfg);
        assert_eq!(cage_proxy_vertex_count(&cage), 8); // box cage has 8 corners
    }

    #[test]
    fn test_build_cage_face_count() {
        let cloud = sample_cloud();
        let cfg = default_cage_proxy_config();
        let cage = build_cage_proxy(&cloud, &cfg);
        assert_eq!(cage_proxy_face_count(&cage), 12);
    }

    #[test]
    fn test_bind_vertex_inside() {
        let cloud = sample_cloud();
        let cfg = default_cage_proxy_config();
        let cage = build_cage_proxy(&cloud, &cfg);
        let bc = cage_bind_vertex(&cage, [0.5, 0.5, 0.5]);
        for &v in &bc[0..3] {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn test_bind_all_inside() {
        let cloud = sample_cloud();
        let cfg = default_cage_proxy_config();
        let cage = build_cage_proxy(&cloud, &cfg);
        // All source points should be inside the cage (with padding)
        let result = cage_bind_all(&cage, &cloud);
        assert_eq!(result.bound_vertices, cloud.len());
        assert!(result.max_bind_error < 1e-5);
    }

    #[test]
    fn test_bind_all_outside() {
        let cloud = sample_cloud();
        let cfg = default_cage_proxy_config();
        let cage = build_cage_proxy(&cloud, &cfg);
        let outside = vec![[100.0, 100.0, 100.0]];
        let result = cage_bind_all(&cage, &outside);
        assert!(result.max_bind_error > 0.0);
    }

    #[test]
    fn test_to_json() {
        let cloud = sample_cloud();
        let cfg = default_cage_proxy_config();
        let cage = build_cage_proxy(&cloud, &cfg);
        let j = cage_to_json(&cage);
        assert!(j.contains("vertex_count"));
    }

    #[test]
    fn test_padding_expands_cage() {
        let cloud = vec![[0.0f32, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let cfg = CageProxyConfig { resolution: 2, padding: 0.5 };
        let cage = build_cage_proxy(&cloud, &cfg);
        // With 0.5 padding, the cage should extend to -0.5..1.5
        assert!(cage.positions[0][0] < -0.4);
    }
}
