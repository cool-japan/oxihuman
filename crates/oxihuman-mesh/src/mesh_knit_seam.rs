// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Knit mesh seams by welding nearby boundary vertices.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KnitSeamConfig { pub tolerance: f32 }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KnitSeamResult {
    pub positions: Vec<[f32;3]>,
    pub indices: Vec<[u32;3]>,
    pub welded_count: usize,
}

#[allow(dead_code)]
pub fn default_knit_seam_config() -> KnitSeamConfig { KnitSeamConfig { tolerance: 0.001 } }

#[allow(dead_code)]
pub fn dist3(a:[f32;3],b:[f32;3]) -> f32 {
    let d=[b[0]-a[0],b[1]-a[1],b[2]-a[2]];
    (d[0]*d[0]+d[1]*d[1]+d[2]*d[2]).sqrt()
}

#[allow(dead_code)]
pub fn find_boundary_verts(indices: &[[u32;3]]) -> Vec<u32> {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(u32,u32), usize> = HashMap::new();
    for tri in indices { for k in 0..3 { let a=tri[k]; let b=tri[(k+1)%3]; let key=if a<b{(a,b)}else{(b,a)}; *edge_count.entry(key).or_insert(0)+=1; } }
    let mut verts = std::collections::HashSet::new();
    for ((a,b),c) in &edge_count { if *c==1 { verts.insert(*a); verts.insert(*b); } }
    let mut v: Vec<u32> = verts.into_iter().collect(); v.sort(); v
}

#[allow(dead_code)]
pub fn knit_seams(positions:&[[f32;3]], indices:&[[u32;3]], config:&KnitSeamConfig) -> KnitSeamResult {
    let bv = find_boundary_verts(indices);
    let mut remap: Vec<u32> = (0..positions.len() as u32).collect();
    let mut welded = 0usize;
    for i in 0..bv.len() {
        for j in (i+1)..bv.len() {
            let a=bv[i] as usize; let b=bv[j] as usize;
            if dist3(positions[a], positions[b]) < config.tolerance && remap[b]==bv[j] {
                remap[b] = bv[i]; welded += 1;
            }
        }
    }
    let new_indices: Vec<[u32;3]> = indices.iter().map(|tri| [remap[tri[0] as usize], remap[tri[1] as usize], remap[tri[2] as usize]]).collect();
    KnitSeamResult { positions: positions.to_vec(), indices: new_indices, welded_count: welded }
}

#[allow(dead_code)]
pub fn knit_seam_welded(result:&KnitSeamResult) -> usize { result.welded_count }
#[allow(dead_code)]
pub fn validate_knit_config(config:&KnitSeamConfig) -> bool { config.tolerance > 0.0 }
#[allow(dead_code)]
pub fn knit_seam_to_json(result:&KnitSeamResult) -> String {
    format!("{{\"welded\":{},\"vertices\":{},\"faces\":{}}}", result.welded_count, result.positions.len(), result.indices.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn seam_mesh() -> (Vec<[f32;3]>,Vec<[u32;3]>) {
        (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0],[0.0,0.0001,0.0],[1.0,0.0001,0.0],[0.5,-1.0,0.0]],
         vec![[0,1,2],[3,4,5]])
    }
    #[test] fn test_default() { let c=default_knit_seam_config(); assert!(c.tolerance>0.0); }
    #[test] fn test_dist3() { assert!((dist3([0.0,0.0,0.0],[3.0,4.0,0.0])-5.0).abs()<1e-5); }
    #[test] fn test_find_boundary() { let(_,i)=seam_mesh(); let bv=find_boundary_verts(&i); assert!(!bv.is_empty()); }
    #[test] fn test_knit() { let(p,i)=seam_mesh(); let r=knit_seams(&p,&i,&KnitSeamConfig{tolerance:0.01}); assert!(r.welded_count>0); }
    #[test] fn test_no_weld_large_gap() { let(p,i)=seam_mesh(); let r=knit_seams(&p,&i,&KnitSeamConfig{tolerance:0.00001}); assert_eq!(r.welded_count,0); }
    #[test] fn test_knit_seam_welded() { let(p,i)=seam_mesh(); let r=knit_seams(&p,&i,&KnitSeamConfig{tolerance:0.01}); assert!(knit_seam_welded(&r)>0); }
    #[test] fn test_validate() { assert!(validate_knit_config(&default_knit_seam_config())); }
    #[test] fn test_validate_bad() { assert!(!validate_knit_config(&KnitSeamConfig{tolerance:0.0})); }
    #[test] fn test_to_json() { let(p,i)=seam_mesh(); let r=knit_seams(&p,&i,&default_knit_seam_config()); assert!(knit_seam_to_json(&r).contains("welded")); }
    #[test] fn test_empty() { let r=knit_seams(&[],&[],&default_knit_seam_config()); assert_eq!(r.welded_count,0); }
}
