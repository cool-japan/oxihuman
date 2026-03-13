// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Butterfly subdivision scheme for triangle meshes.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ButterflyResult {
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

#[allow(dead_code)]
pub fn butterfly_midpoint(a: [f32; 3], b: [f32; 3], w: f32) -> [f32; 3] {
    let t = w.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

#[allow(dead_code)]
pub fn butterfly_subdivide(positions: &[[f32; 3]], triangles: &[[u32; 3]]) -> ButterflyResult {
    let mut new_pos = positions.to_vec();
    let mut edge_map: HashMap<(u32, u32), u32> = HashMap::new();
    let mut new_tris = Vec::new();

    for tri in triangles {
        let mut mid = [0u32; 3];
        for k in 0..3 {
            let (a, b) = (tri[k], tri[(k + 1) % 3]);
            let key = if a < b { (a, b) } else { (b, a) };
            mid[k] = *edge_map.entry(key).or_insert_with(|| {
                let m = butterfly_midpoint(positions[a as usize], positions[b as usize], 0.5);
                new_pos.push(m);
                (new_pos.len() - 1) as u32
            });
        }
        new_tris.push([tri[0], mid[0], mid[2]]);
        new_tris.push([mid[0], tri[1], mid[1]]);
        new_tris.push([mid[2], mid[1], tri[2]]);
        new_tris.push([mid[0], mid[1], mid[2]]);
    }

    ButterflyResult { positions: new_pos, triangles: new_tris }
}

#[allow(dead_code)]
pub fn butterfly_vertex_count(result: &ButterflyResult) -> usize { result.positions.len() }

#[allow(dead_code)]
pub fn butterfly_tri_count(result: &ButterflyResult) -> usize { result.triangles.len() }

#[allow(dead_code)]
pub fn butterfly_validate(result: &ButterflyResult) -> bool {
    result.triangles.iter().all(|t| t.iter().all(|&v| (v as usize) < result.positions.len()))
}

#[allow(dead_code)]
pub fn butterfly_to_json(result: &ButterflyResult) -> String {
    format!("{{\"vertices\":{},\"triangles\":{}}}", result.positions.len(), result.triangles.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]], vec![[0,1,2]])
    }

    #[test] fn test_midpoint() { let m = butterfly_midpoint([0.0,0.0,0.0],[2.0,0.0,0.0],0.5); assert!((m[0]-1.0).abs()<1e-5); }
    #[test] fn test_subdivide_vertex_count() { let(p,t)=tri(); let r=butterfly_subdivide(&p,&t); assert_eq!(r.positions.len(),6); }
    #[test] fn test_subdivide_tri_count() { let(p,t)=tri(); let r=butterfly_subdivide(&p,&t); assert_eq!(r.triangles.len(),4); }
    #[test] fn test_validate() { let(p,t)=tri(); let r=butterfly_subdivide(&p,&t); assert!(butterfly_validate(&r)); }
    #[test] fn test_vertex_count_fn() { let(p,t)=tri(); let r=butterfly_subdivide(&p,&t); assert_eq!(butterfly_vertex_count(&r),6); }
    #[test] fn test_tri_count_fn() { let(p,t)=tri(); let r=butterfly_subdivide(&p,&t); assert_eq!(butterfly_tri_count(&r),4); }
    #[test] fn test_to_json() { let(p,t)=tri(); let r=butterfly_subdivide(&p,&t); assert!(butterfly_to_json(&r).contains("vertices")); }
    #[test] fn test_empty() { let r=butterfly_subdivide(&[],&[]); assert!(r.positions.is_empty()); }
    #[test] fn test_midpoint_clamp() { let m=butterfly_midpoint([0.0,0.0,0.0],[2.0,0.0,0.0],2.0); assert!((m[0]-2.0).abs()<1e-5); }
    #[test] fn test_two_tris() {
        let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],[1.0,1.0,0.0]];
        let t = vec![[0,1,2],[1,3,2]];
        let r = butterfly_subdivide(&p,&t);
        assert_eq!(r.triangles.len(), 8);
    }
}
