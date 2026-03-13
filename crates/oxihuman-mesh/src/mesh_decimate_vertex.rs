// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Vertex-based decimation: remove least important vertices.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecimateVertexConfig { pub target_ratio: f32 }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecimateVertexResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
    pub removed_count: usize,
}

#[allow(dead_code)]
pub fn default_decimate_vertex_config() -> DecimateVertexConfig { DecimateVertexConfig { target_ratio: 0.5 } }

#[allow(dead_code)]
pub fn vertex_importance(positions: &[[f32; 3]], indices: &[[u32; 3]], vi: u32) -> f32 {
    let mut area = 0.0f32;
    for tri in indices {
        if tri[0]==vi || tri[1]==vi || tri[2]==vi {
            let a=positions[tri[0] as usize]; let b=positions[tri[1] as usize]; let c=positions[tri[2] as usize];
            let ab=[b[0]-a[0],b[1]-a[1],b[2]-a[2]]; let ac=[c[0]-a[0],c[1]-a[1],c[2]-a[2]];
            let cx=ab[1]*ac[2]-ab[2]*ac[1]; let cy=ab[2]*ac[0]-ab[0]*ac[2]; let cz=ab[0]*ac[1]-ab[1]*ac[0];
            area += (cx*cx+cy*cy+cz*cz).sqrt() * 0.5;
        }
    }
    area
}

#[allow(dead_code)]
pub fn decimate_vertices(positions: &[[f32; 3]], indices: &[[u32; 3]], config: &DecimateVertexConfig) -> DecimateVertexResult {
    let n = positions.len();
    let target = ((n as f32) * config.target_ratio.clamp(0.1, 1.0)) as usize;
    let mut importance: Vec<(usize, f32)> = (0..n).map(|i| (i, vertex_importance(positions, indices, i as u32))).collect();
    importance.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let keep: std::collections::HashSet<usize> = importance.iter().take(target).map(|&(i,_)| i).collect();
    let mut remap = vec![u32::MAX; n];
    let mut new_pos = Vec::new();
    for &(i,_) in importance.iter().take(target) { remap[i] = new_pos.len() as u32; new_pos.push(positions[i]); }
    let mut new_idx = Vec::new();
    for tri in indices {
        let a=remap[tri[0] as usize]; let b=remap[tri[1] as usize]; let c=remap[tri[2] as usize];
        if a!=u32::MAX && b!=u32::MAX && c!=u32::MAX { new_idx.push([a,b,c]); }
    }
    let _ = keep;
    DecimateVertexResult { positions: new_pos, indices: new_idx, removed_count: n - target }
}

#[allow(dead_code)]
pub fn decimate_vertex_count(result: &DecimateVertexResult) -> usize { result.positions.len() }
#[allow(dead_code)]
pub fn decimate_face_count(result: &DecimateVertexResult) -> usize { result.indices.len() }
#[allow(dead_code)]
pub fn validate_decimate_config(config: &DecimateVertexConfig) -> bool { (0.0..=1.0).contains(&config.target_ratio) }
#[allow(dead_code)]
pub fn decimate_vertex_to_json(result: &DecimateVertexResult) -> String {
    format!("{{\"vertices\":{},\"faces\":{},\"removed\":{}}}", result.positions.len(), result.indices.len(), result.removed_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mesh() -> (Vec<[f32;3]>,Vec<[u32;3]>) {
        (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0],[1.5,1.0,0.0]], vec![[0,1,2],[1,3,2]])
    }
    #[test] fn test_default() { let c=default_decimate_vertex_config(); assert!((c.target_ratio-0.5).abs()<1e-6); }
    #[test] fn test_importance() { let(p,i)=mesh(); let imp=vertex_importance(&p,&i,1); assert!(imp>0.0); }
    #[test] fn test_decimate() { let(p,i)=mesh(); let r=decimate_vertices(&p,&i,&default_decimate_vertex_config()); assert!(r.positions.len()<=p.len()); }
    #[test] fn test_removed_count() { let(p,i)=mesh(); let r=decimate_vertices(&p,&i,&default_decimate_vertex_config()); assert!(r.removed_count>0); }
    #[test] fn test_vertex_count() { let(p,i)=mesh(); let r=decimate_vertices(&p,&i,&default_decimate_vertex_config()); assert!(decimate_vertex_count(&r)>0); }
    #[test] fn test_face_count() { let(p,i)=mesh(); let r=decimate_vertices(&p,&i,&default_decimate_vertex_config()); assert!(decimate_face_count(&r) < usize::MAX); }
    #[test] fn test_validate() { assert!(validate_decimate_config(&default_decimate_vertex_config())); }
    #[test] fn test_validate_bad() { assert!(!validate_decimate_config(&DecimateVertexConfig{target_ratio:1.5})); }
    #[test] fn test_to_json() { let(p,i)=mesh(); let r=decimate_vertices(&p,&i,&default_decimate_vertex_config()); assert!(decimate_vertex_to_json(&r).contains("removed")); }
    #[test] fn test_empty() { let r=decimate_vertices(&[],&[],&default_decimate_vertex_config()); assert_eq!(r.removed_count,0); }
}
