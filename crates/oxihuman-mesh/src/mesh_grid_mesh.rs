// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Generate a regular grid mesh primitive.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GridMeshConfig { pub rows: usize, pub cols: usize, pub spacing: f32 }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GridMeshResult { pub positions: Vec<[f32;3]>, pub indices: Vec<[u32;3]>, pub uvs: Vec<[f32;2]> }

#[allow(dead_code)]
pub fn default_grid_mesh_config() -> GridMeshConfig { GridMeshConfig { rows: 4, cols: 4, spacing: 1.0 } }

#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn generate_grid_mesh(config: &GridMeshConfig) -> GridMeshResult {
    let r = config.rows.max(1); let c = config.cols.max(1);
    let mut positions = Vec::new(); let mut uvs = Vec::new();
    for i in 0..=r {
        for j in 0..=c {
            positions.push([j as f32 * config.spacing, 0.0, i as f32 * config.spacing]);
            uvs.push([j as f32 / c as f32, i as f32 / r as f32]);
        }
    }
    let mut indices = Vec::new();
    let w = c + 1;
    for i in 0..r {
        for j in 0..c {
            let tl = (i*w+j) as u32; let tr = tl+1; let bl = ((i+1)*w+j) as u32; let br = bl+1;
            indices.push([tl, bl, tr]); indices.push([tr, bl, br]);
        }
    }
    GridMeshResult { positions, indices, uvs }
}

#[allow(dead_code)]
pub fn grid_mesh_vertex_count(config: &GridMeshConfig) -> usize { (config.rows+1)*(config.cols+1) }
#[allow(dead_code)]
pub fn grid_mesh_face_count(config: &GridMeshConfig) -> usize { config.rows*config.cols*2 }
#[allow(dead_code)]
pub fn grid_mesh_area(config: &GridMeshConfig) -> f32 { config.rows as f32 * config.cols as f32 * config.spacing * config.spacing }
#[allow(dead_code)]
pub fn grid_mesh_to_json(result: &GridMeshResult) -> String {
    format!("{{\"vertices\":{},\"faces\":{}}}", result.positions.len(), result.indices.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_default() { let c=default_grid_mesh_config(); assert_eq!(c.rows,4); assert_eq!(c.cols,4); }
    #[test] fn test_generate() { let r=generate_grid_mesh(&default_grid_mesh_config()); assert_eq!(r.positions.len(),25); }
    #[test] fn test_faces() { let r=generate_grid_mesh(&default_grid_mesh_config()); assert_eq!(r.indices.len(),32); }
    #[test] fn test_uvs() { let r=generate_grid_mesh(&default_grid_mesh_config()); assert_eq!(r.uvs.len(),r.positions.len()); }
    #[test] fn test_vertex_count() { assert_eq!(grid_mesh_vertex_count(&default_grid_mesh_config()),25); }
    #[test] fn test_face_count() { assert_eq!(grid_mesh_face_count(&default_grid_mesh_config()),32); }
    #[test] fn test_area() { let a=grid_mesh_area(&GridMeshConfig{rows:2,cols:3,spacing:1.0}); assert!((a-6.0).abs()<1e-6); }
    #[test] fn test_to_json() { let r=generate_grid_mesh(&default_grid_mesh_config()); assert!(grid_mesh_to_json(&r).contains("vertices")); }
    #[test] fn test_1x1() { let r=generate_grid_mesh(&GridMeshConfig{rows:1,cols:1,spacing:1.0}); assert_eq!(r.positions.len(),4); assert_eq!(r.indices.len(),2); }
    #[test] fn test_uv_range() { let r=generate_grid_mesh(&default_grid_mesh_config()); for uv in &r.uvs { assert!((0.0..=1.0).contains(&uv[0])); assert!((0.0..=1.0).contains(&uv[1])); } }
}
