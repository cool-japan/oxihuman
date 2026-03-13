// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Clip plane: clip a mesh by an arbitrary plane.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ClipPlaneConfig {
    pub normal: [f32; 3],
    pub distance: f32,
    pub keep_above: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClipPlaneResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
    pub clipped_vertex_count: usize,
}

#[allow(dead_code)]
pub fn default_clip_plane_config() -> ClipPlaneConfig {
    ClipPlaneConfig { normal: [0.0, 1.0, 0.0], distance: 0.0, keep_above: true }
}

#[allow(dead_code)]
pub fn signed_distance(point: [f32; 3], normal: [f32; 3], dist: f32) -> f32 {
    point[0] * normal[0] + point[1] * normal[1] + point[2] * normal[2] - dist
}

#[allow(dead_code)]
pub fn is_above(point: [f32; 3], normal: [f32; 3], dist: f32) -> bool {
    signed_distance(point, normal, dist) >= 0.0
}

#[allow(dead_code)]
pub fn clip_by_plane(
    positions: &[[f32; 3]],
    indices: &[[u32; 3]],
    config: &ClipPlaneConfig,
) -> ClipPlaneResult {
    let mut kept_faces = Vec::new();
    for tri in indices {
        let above: Vec<bool> = tri.iter()
            .map(|&v| is_above(positions[v as usize], config.normal, config.distance) == config.keep_above)
            .collect();
        if above.iter().all(|&a| a) {
            kept_faces.push(*tri);
        }
    }
    let mut used_verts = std::collections::HashSet::new();
    for tri in &kept_faces {
        used_verts.insert(tri[0]);
        used_verts.insert(tri[1]);
        used_verts.insert(tri[2]);
    }
    ClipPlaneResult {
        clipped_vertex_count: positions.len() - used_verts.len(),
        positions: positions.to_vec(),
        indices: kept_faces,
    }
}

#[allow(dead_code)]
pub fn clip_vertex_count(result: &ClipPlaneResult) -> usize {
    result.positions.len()
}

#[allow(dead_code)]
pub fn clip_face_count(result: &ClipPlaneResult) -> usize {
    result.indices.len()
}

#[allow(dead_code)]
pub fn validate_clip_config(config: &ClipPlaneConfig) -> bool {
    let len_sq = config.normal[0]*config.normal[0] + config.normal[1]*config.normal[1] + config.normal[2]*config.normal[2];
    (len_sq - 1.0).abs() < 0.01
}

#[allow(dead_code)]
pub fn clip_plane_to_json(result: &ClipPlaneResult) -> String {
    format!("{{\"faces\":{},\"clipped\":{}}}", result.indices.len(), result.clipped_vertex_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tri_above() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        (vec![[0.0,1.0,0.0],[1.0,1.0,0.0],[0.5,2.0,0.0]], vec![[0,1,2]])
    }
    fn tri_below() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        (vec![[0.0,-1.0,0.0],[1.0,-1.0,0.0],[0.5,-2.0,0.0]], vec![[0,1,2]])
    }
    #[test] fn test_default_config() { let c = default_clip_plane_config(); assert!(c.keep_above); }
    #[test] fn test_signed_distance() { let d = signed_distance([0.0,5.0,0.0],[0.0,1.0,0.0],0.0); assert!((d-5.0).abs()<1e-6); }
    #[test] fn test_is_above() { assert!(is_above([0.0,1.0,0.0],[0.0,1.0,0.0],0.0)); }
    #[test] fn test_clip_keeps_above() {
        let (p,i) = tri_above();
        let r = clip_by_plane(&p,&i,&default_clip_plane_config());
        assert_eq!(r.indices.len(), 1);
    }
    #[test] fn test_clip_removes_below() {
        let (p,i) = tri_below();
        let r = clip_by_plane(&p,&i,&default_clip_plane_config());
        assert!(r.indices.is_empty());
    }
    #[test] fn test_clip_vertex_count() {
        let (p,i) = tri_above();
        let r = clip_by_plane(&p,&i,&default_clip_plane_config());
        assert_eq!(clip_vertex_count(&r), 3);
    }
    #[test] fn test_validate_config() { assert!(validate_clip_config(&default_clip_plane_config())); }
    #[test] fn test_clip_plane_to_json() {
        let (p,i) = tri_above();
        let r = clip_by_plane(&p,&i,&default_clip_plane_config());
        let j = clip_plane_to_json(&r);
        assert!(j.contains("faces"));
    }
    #[test] fn test_empty() {
        let r = clip_by_plane(&[],&[],&default_clip_plane_config());
        assert!(r.indices.is_empty());
    }
    #[test] fn test_clip_face_count() {
        let (p,i) = tri_above();
        let r = clip_by_plane(&p,&i,&default_clip_plane_config());
        assert_eq!(clip_face_count(&r), 1);
    }
}
