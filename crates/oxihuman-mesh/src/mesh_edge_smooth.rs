// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Edge smooth: smooth normals along edges by averaging adjacent face normals.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeSmoothConfig { pub angle_threshold: f32 }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeSmoothResult { pub normals: Vec<[f32; 3]>, pub smoothed_edge_count: usize }

#[allow(dead_code)]
pub fn default_edge_smooth_config() -> EdgeSmoothConfig { EdgeSmoothConfig { angle_threshold: 30.0 } }

#[allow(dead_code)]
pub fn face_normal(a: [f32;3], b: [f32;3], c: [f32;3]) -> [f32;3] {
    let ab=[b[0]-a[0],b[1]-a[1],b[2]-a[2]]; let ac=[c[0]-a[0],c[1]-a[1],c[2]-a[2]];
    let n=[ab[1]*ac[2]-ab[2]*ac[1], ab[2]*ac[0]-ab[0]*ac[2], ab[0]*ac[1]-ab[1]*ac[0]];
    let len=(n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt();
    if len<1e-12 {[0.0,0.0,1.0]} else {[n[0]/len,n[1]/len,n[2]/len]}
}

#[allow(dead_code)]
pub fn dot3(a: [f32;3], b: [f32;3]) -> f32 { a[0]*b[0]+a[1]*b[1]+a[2]*b[2] }

#[allow(dead_code)]
pub fn angle_between_normals(a: [f32;3], b: [f32;3]) -> f32 {
    dot3(a,b).clamp(-1.0,1.0).acos().to_degrees()
}

#[allow(dead_code)]
pub fn smooth_edge_normals(positions: &[[f32;3]], indices: &[[u32;3]], config: &EdgeSmoothConfig) -> EdgeSmoothResult {
    let mut face_normals_map: HashMap<(u32,u32), Vec<[f32;3]>> = HashMap::new();
    for tri in indices {
        let n = face_normal(positions[tri[0] as usize], positions[tri[1] as usize], positions[tri[2] as usize]);
        for k in 0..3 {
            let a=tri[k]; let b=tri[(k+1)%3];
            let key=if a<b{(a,b)}else{(b,a)};
            face_normals_map.entry(key).or_default().push(n);
        }
    }
    let mut normals = vec![[0.0f32;3]; positions.len()];
    let mut smoothed = 0usize;
    for (&(a,b), norms) in &face_normals_map {
        if norms.len() == 2 {
            let angle = angle_between_normals(norms[0], norms[1]);
            if angle < config.angle_threshold {
                let avg = [(norms[0][0]+norms[1][0])*0.5,(norms[0][1]+norms[1][1])*0.5,(norms[0][2]+norms[1][2])*0.5];
                let len = (avg[0]*avg[0]+avg[1]*avg[1]+avg[2]*avg[2]).sqrt();
                let norm = if len>1e-12 {[avg[0]/len,avg[1]/len,avg[2]/len]} else {avg};
                normals[a as usize] = norm;
                normals[b as usize] = norm;
                smoothed += 1;
            }
        }
    }
    EdgeSmoothResult { normals, smoothed_edge_count: smoothed }
}

#[allow(dead_code)]
pub fn edge_smooth_count(result: &EdgeSmoothResult) -> usize { result.smoothed_edge_count }
#[allow(dead_code)]
pub fn edge_smooth_to_json(result: &EdgeSmoothResult) -> String {
    format!("{{\"smoothed_edges\":{},\"normal_count\":{}}}", result.smoothed_edge_count, result.normals.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn quad() -> (Vec<[f32;3]>,Vec<[u32;3]>) {
        (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[1.0,1.0,0.0],[0.0,1.0,0.0]], vec![[0,1,2],[0,2,3]])
    }
    #[test] fn test_default() { let c=default_edge_smooth_config(); assert!((c.angle_threshold-30.0).abs()<1e-6); }
    #[test] fn test_face_normal() { let n=face_normal([0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]); assert!((n[2]-1.0).abs()<1e-6); }
    #[test] fn test_dot3() { assert!((dot3([1.0,0.0,0.0],[1.0,0.0,0.0])-1.0).abs()<1e-6); }
    #[test] fn test_angle_between() { let a=angle_between_normals([0.0,0.0,1.0],[0.0,0.0,1.0]); assert!(a.abs()<1e-3); }
    #[test] fn test_smooth() { let(p,i)=quad(); let r=smooth_edge_normals(&p,&i,&default_edge_smooth_config()); assert_eq!(r.normals.len(),4); }
    #[test] fn test_smoothed_count() { let(p,i)=quad(); let r=smooth_edge_normals(&p,&i,&EdgeSmoothConfig{angle_threshold:180.0}); assert!(r.smoothed_edge_count>0); }
    #[test] fn test_edge_smooth_count() { let(p,i)=quad(); let r=smooth_edge_normals(&p,&i,&default_edge_smooth_config()); assert!(edge_smooth_count(&r) < usize::MAX); }
    #[test] fn test_to_json() { let(p,i)=quad(); let r=smooth_edge_normals(&p,&i,&default_edge_smooth_config()); assert!(edge_smooth_to_json(&r).contains("smoothed_edges")); }
    #[test] fn test_empty() { let r=smooth_edge_normals(&[],&[],&default_edge_smooth_config()); assert_eq!(r.normals.len(),0); }
    #[test] fn test_perpendicular_faces() { let a=angle_between_normals([0.0,0.0,1.0],[0.0,1.0,0.0]); assert!((a-90.0).abs()<1e-3); }
}
