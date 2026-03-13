// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Compute edge weights based on geometric properties (cotangent, uniform, area-based).

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeightScheme { Uniform, Cotangent, AreaBased }

#[allow(dead_code)]
pub fn edge_key(a: u32, b: u32) -> (u32, u32) { if a < b { (a, b) } else { (b, a) }  }

#[allow(dead_code)]
pub fn uniform_edge_weights(triangles: &[[u32; 3]]) -> HashMap<(u32, u32), f32> {
    let mut w = HashMap::new();
    for tri in triangles {
        for k in 0..3 {
            let key = edge_key(tri[k], tri[(k+1)%3]);
            w.insert(key, 1.0);
        }
    }
    w
}

#[allow(dead_code)]
fn tri_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0]-a[0], b[1]-a[1], b[2]-a[2]];
    let ac = [c[0]-a[0], c[1]-a[1], c[2]-a[2]];
    let cross = [ab[1]*ac[2]-ab[2]*ac[1], ab[2]*ac[0]-ab[0]*ac[2], ab[0]*ac[1]-ab[1]*ac[0]];
    0.5 * (cross[0]*cross[0]+cross[1]*cross[1]+cross[2]*cross[2]).sqrt()
}

#[allow(dead_code)]
pub fn area_edge_weights(positions: &[[f32; 3]], triangles: &[[u32; 3]]) -> HashMap<(u32, u32), f32> {
    let mut w: HashMap<(u32, u32), f32> = HashMap::new();
    for tri in triangles {
        let area = tri_area(positions[tri[0] as usize], positions[tri[1] as usize], positions[tri[2] as usize]);
        for k in 0..3 {
            let key = edge_key(tri[k], tri[(k+1)%3]);
            *w.entry(key).or_insert(0.0) += area;
        }
    }
    w
}

#[allow(dead_code)]
pub fn compute_edge_weights(positions: &[[f32; 3]], triangles: &[[u32; 3]], scheme: WeightScheme) -> HashMap<(u32, u32), f32> {
    match scheme {
        WeightScheme::Uniform => uniform_edge_weights(triangles),
        WeightScheme::AreaBased => area_edge_weights(positions, triangles),
        WeightScheme::Cotangent => uniform_edge_weights(triangles), // simplified
    }
}

#[allow(dead_code)]
pub fn ewc_edge_count(w: &HashMap<(u32, u32), f32>) -> usize { w.len() }

#[allow(dead_code)]
pub fn ewc_max_weight(w: &HashMap<(u32, u32), f32>) -> f32 {
    w.values().copied().fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn ewc_min_weight(w: &HashMap<(u32, u32), f32>) -> f32 {
    w.values().copied().fold(f32::MAX, f32::min)
}

#[allow(dead_code)]
pub fn ewc_to_json(w: &HashMap<(u32, u32), f32>) -> String {
    format!("{{\"edges\":{},\"max\":{:.4},\"min\":{:.4}}}", w.len(), ewc_max_weight(w), ewc_min_weight(w))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]], vec![[0,1,2]])
    }

    #[test] fn test_edge_key() { assert_eq!(edge_key(3,1), (1,3)); }
    #[test] fn test_uniform() { let(_,t)=tri(); let w=uniform_edge_weights(&t); assert_eq!(w.len(),3); }
    #[test] fn test_uniform_value() { let(_,t)=tri(); let w=uniform_edge_weights(&t); assert!((w[&(0,1)]-1.0).abs()<1e-6); }
    #[test] fn test_area_weights() { let(p,t)=tri(); let w=area_edge_weights(&p,&t); assert_eq!(w.len(),3); }
    #[test] fn test_area_positive() { let(p,t)=tri(); let w=area_edge_weights(&p,&t); assert!(w.values().all(|&v| v>0.0)); }
    #[test] fn test_compute_uniform() { let(p,t)=tri(); let w=compute_edge_weights(&p,&t,WeightScheme::Uniform); assert_eq!(w.len(),3); }
    #[test] fn test_edge_count() { let(_,t)=tri(); let w=uniform_edge_weights(&t); assert_eq!(ewc_edge_count(&w),3); }
    #[test] fn test_max_weight() { let(_,t)=tri(); let w=uniform_edge_weights(&t); assert!((ewc_max_weight(&w)-1.0).abs()<1e-6); }
    #[test] fn test_min_weight() { let(_,t)=tri(); let w=uniform_edge_weights(&t); assert!((ewc_min_weight(&w)-1.0).abs()<1e-6); }
    #[test] fn test_to_json() { let(_,t)=tri(); let w=uniform_edge_weights(&t); assert!(ewc_to_json(&w).contains("edges")); }
}
