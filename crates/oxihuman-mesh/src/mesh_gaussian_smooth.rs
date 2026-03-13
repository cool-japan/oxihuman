// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Gaussian smoothing of vertex positions.

use std::f32::consts::E;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GaussianSmoothConfig { pub sigma: f32, pub iterations: usize }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GaussianSmoothResult { pub positions: Vec<[f32;3]>, pub max_displacement: f32 }

#[allow(dead_code)]
pub fn default_gaussian_smooth_config() -> GaussianSmoothConfig { GaussianSmoothConfig { sigma: 1.0, iterations: 3 } }

#[allow(dead_code)]
pub fn gaussian_weight(dist: f32, sigma: f32) -> f32 {
    E.powf(-(dist * dist) / (2.0 * sigma * sigma))
}

#[allow(dead_code)]
pub fn build_adjacency(indices: &[[u32;3]], vertex_count: usize) -> Vec<Vec<u32>> {
    let mut adj = vec![Vec::new(); vertex_count];
    for tri in indices {
        for k in 0..3 {
            let a=tri[k] as usize; let b=tri[(k+1)%3] as usize;
            if !adj[a].contains(&(b as u32)) { adj[a].push(b as u32); }
            if !adj[b].contains(&(a as u32)) { adj[b].push(a as u32); }
        }
    }
    adj
}

#[allow(dead_code)]
pub fn dist3(a:[f32;3],b:[f32;3]) -> f32 {
    let d=[b[0]-a[0],b[1]-a[1],b[2]-a[2]];
    (d[0]*d[0]+d[1]*d[1]+d[2]*d[2]).sqrt()
}

#[allow(dead_code)]
pub fn gaussian_smooth(positions:&[[f32;3]], indices:&[[u32;3]], config:&GaussianSmoothConfig) -> GaussianSmoothResult {
    let adj = build_adjacency(indices, positions.len());
    let mut pos = positions.to_vec();
    let mut max_disp = 0.0f32;
    for _ in 0..config.iterations {
        let old = pos.clone();
        for vi in 0..pos.len() {
            if adj[vi].is_empty() { continue; }
            let mut total_w = 1.0f32;
            let mut sum = old[vi];
            for &ni in &adj[vi] {
                let d = dist3(old[vi], old[ni as usize]);
                let w = gaussian_weight(d, config.sigma);
                sum[0]+=old[ni as usize][0]*w; sum[1]+=old[ni as usize][1]*w; sum[2]+=old[ni as usize][2]*w;
                total_w += w;
            }
            pos[vi] = [sum[0]/total_w, sum[1]/total_w, sum[2]/total_w];
            let disp = dist3(old[vi], pos[vi]);
            if disp > max_disp { max_disp = disp; }
        }
    }
    GaussianSmoothResult { positions: pos, max_displacement: max_disp }
}

#[allow(dead_code)]
pub fn gaussian_smooth_to_json(result:&GaussianSmoothResult) -> String {
    format!("{{\"vertices\":{},\"max_disp\":{:.6}}}", result.positions.len(), result.max_displacement)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn quad() -> (Vec<[f32;3]>,Vec<[u32;3]>) {
        (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[1.0,1.0,0.0],[0.0,1.0,0.0]], vec![[0,1,2],[0,2,3]])
    }
    #[test] fn test_default() { let c=default_gaussian_smooth_config(); assert!((c.sigma-1.0).abs()<1e-6); }
    #[test] fn test_gaussian_weight() { let w=gaussian_weight(0.0,1.0); assert!((w-1.0).abs()<1e-6); }
    #[test] fn test_dist3() { assert!((dist3([0.0,0.0,0.0],[3.0,4.0,0.0])-5.0).abs()<1e-5); }
    #[test] fn test_build_adjacency() { let(_,i)=quad(); let adj=build_adjacency(&i,4); assert!(!adj[0].is_empty()); }
    #[test] fn test_smooth() { let(p,i)=quad(); let r=gaussian_smooth(&p,&i,&default_gaussian_smooth_config()); assert_eq!(r.positions.len(),4); }
    #[test] fn test_max_disp() { let(p,i)=quad(); let r=gaussian_smooth(&p,&i,&default_gaussian_smooth_config()); assert!(r.max_displacement>=0.0); }
    #[test] fn test_to_json() { let(p,i)=quad(); let r=gaussian_smooth(&p,&i,&default_gaussian_smooth_config()); assert!(gaussian_smooth_to_json(&r).contains("max_disp")); }
    #[test] fn test_empty() { let r=gaussian_smooth(&[],&[],&default_gaussian_smooth_config()); assert!(r.positions.is_empty()); }
    #[test] fn test_weight_decreases() { let w1=gaussian_weight(0.0,1.0); let w2=gaussian_weight(1.0,1.0); assert!(w1>w2); }
    #[test] fn test_single_vertex() { let r=gaussian_smooth(&[[0.0,0.0,0.0]],&[],&default_gaussian_smooth_config()); assert_eq!(r.positions.len(),1); }
}
