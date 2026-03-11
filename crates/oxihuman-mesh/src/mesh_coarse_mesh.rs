// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Coarse mesh generation by vertex clustering.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CoarseMesh {
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

#[allow(dead_code)]
pub fn coarsen_by_grid(positions: &[[f32; 3]], triangles: &[[u32; 3]], cell_size: f32) -> CoarseMesh {
    use std::collections::HashMap;
    if positions.is_empty() || cell_size <= 0.0 {
        return CoarseMesh { positions: Vec::new(), triangles: Vec::new() };
    }
    type ClusterMap = HashMap<(i32, i32, i32), (u32, [f64; 3], u32)>;
    let mut cluster_map: ClusterMap = HashMap::new();
    let mut vertex_remap = vec![0u32; positions.len()];
    let inv = 1.0 / cell_size;
    let mut next_id = 0u32;
    for (i, p) in positions.iter().enumerate() {
        let key = ((p[0] * inv).floor() as i32, (p[1] * inv).floor() as i32, (p[2] * inv).floor() as i32);
        let entry = cluster_map.entry(key).or_insert_with(|| { let id = next_id; next_id += 1; (id, [0.0, 0.0, 0.0], 0) });
        entry.1[0] += p[0] as f64;
        entry.1[1] += p[1] as f64;
        entry.1[2] += p[2] as f64;
        entry.2 += 1;
        vertex_remap[i] = entry.0;
    }
    let mut new_pos = vec![[0.0f32; 3]; cluster_map.len()];
    for (idx, sum, cnt) in cluster_map.values() {
        let n = *cnt as f64;
        new_pos[*idx as usize] = [(sum[0] / n) as f32, (sum[1] / n) as f32, (sum[2] / n) as f32];
    }
    let mut new_tris = Vec::new();
    for tri in triangles {
        let a = vertex_remap[tri[0] as usize];
        let b = vertex_remap[tri[1] as usize];
        let c = vertex_remap[tri[2] as usize];
        if a != b && b != c && a != c {
            new_tris.push([a, b, c]);
        }
    }
    CoarseMesh { positions: new_pos, triangles: new_tris }
}

#[allow(dead_code)]
pub fn coarse_vertex_count(cm: &CoarseMesh) -> usize { cm.positions.len() }

#[allow(dead_code)]
pub fn coarse_tri_count(cm: &CoarseMesh) -> usize { cm.triangles.len() }

#[allow(dead_code)]
pub fn coarse_reduction_ratio(original: usize, coarse: &CoarseMesh) -> f32 {
    if original == 0 { return 0.0; }
    1.0 - (coarse.positions.len() as f32 / original as f32)
}

#[allow(dead_code)]
pub fn coarse_validate(cm: &CoarseMesh) -> bool {
    cm.triangles.iter().all(|t| t.iter().all(|&v| (v as usize) < cm.positions.len()))
}

#[allow(dead_code)]
pub fn coarse_to_json(cm: &CoarseMesh) -> String {
    format!("{{\"vertices\":{},\"triangles\":{}}}", cm.positions.len(), cm.triangles.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[1.0,1.0,0.0],[0.0,1.0,0.0]], vec![[0,1,2],[0,2,3]])
    }

    #[test] fn test_coarsen_basic() { let(p,t)=quad(); let c=coarsen_by_grid(&p,&t,0.5); assert!(!c.positions.is_empty()); }
    #[test] fn test_coarsen_reduce() { let(p,t)=quad(); let c=coarsen_by_grid(&p,&t,2.0); assert!(c.positions.len()<=p.len()); }
    #[test] fn test_vertex_count() { let(p,t)=quad(); let c=coarsen_by_grid(&p,&t,0.5); assert!(coarse_vertex_count(&c)>0); }
    #[test] fn test_tri_count() { let(p,t)=quad(); let c=coarsen_by_grid(&p,&t,0.5); assert!(coarse_tri_count(&c)<=t.len()); }
    #[test] fn test_validate() { let(p,t)=quad(); let c=coarsen_by_grid(&p,&t,0.5); assert!(coarse_validate(&c)); }
    #[test] fn test_reduction() { let(p,t)=quad(); let c=coarsen_by_grid(&p,&t,2.0); assert!(coarse_reduction_ratio(p.len(),&c)>=0.0); }
    #[test] fn test_to_json() { let(p,t)=quad(); let c=coarsen_by_grid(&p,&t,0.5); assert!(coarse_to_json(&c).contains("vertices")); }
    #[test] fn test_empty() { let c=coarsen_by_grid(&[],&[],1.0); assert!(c.positions.is_empty()); }
    #[test] fn test_zero_cell() { let(p,t)=quad(); let c=coarsen_by_grid(&p,&t,0.0); assert!(c.positions.is_empty()); }
    #[test] fn test_large_cell() { let(p,t)=quad(); let c=coarsen_by_grid(&p,&t,100.0); assert_eq!(c.positions.len(),1); }
}
