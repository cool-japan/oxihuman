// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Extrude a cross-section profile along a path to generate a tube-like mesh.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExtrudedMesh {
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

#[allow(dead_code)]
pub fn circle_profile(radius: f32, segments: usize) -> Vec<[f32; 2]> {
    let seg = segments.max(3);
    (0..seg).map(|i| {
        let a = 2.0 * PI * i as f32 / seg as f32;
        [radius * a.cos(), radius * a.sin()]
    }).collect()
}

#[allow(dead_code)]
pub fn extrude_along_path(profile: &[[f32; 2]], path: &[[f32; 3]]) -> ExtrudedMesh {
    if path.len() < 2 || profile.is_empty() {
        return ExtrudedMesh { positions: Vec::new(), triangles: Vec::new() };
    }
    let n = profile.len();
    let mut positions = Vec::new();
    for (pi, &pt) in path.iter().enumerate() {
        let dir = if pi + 1 < path.len() {
            [path[pi+1][0]-pt[0], path[pi+1][1]-pt[1], path[pi+1][2]-pt[2]]
        } else {
            [pt[0]-path[pi-1][0], pt[1]-path[pi-1][1], pt[2]-path[pi-1][2]]
        };
        let dlen = (dir[0]*dir[0]+dir[1]*dir[1]+dir[2]*dir[2]).sqrt().max(1e-12);
        let fwd = [dir[0]/dlen, dir[1]/dlen, dir[2]/dlen];
        let up = if fwd[1].abs() < 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
        let right = [fwd[1]*up[2]-fwd[2]*up[1], fwd[2]*up[0]-fwd[0]*up[2], fwd[0]*up[1]-fwd[1]*up[0]];
        let rlen = (right[0]*right[0]+right[1]*right[1]+right[2]*right[2]).sqrt().max(1e-12);
        let r = [right[0]/rlen, right[1]/rlen, right[2]/rlen];
        let u = [fwd[1]*r[2]-fwd[2]*r[1], fwd[2]*r[0]-fwd[0]*r[2], fwd[0]*r[1]-fwd[1]*r[0]];
        for pf in profile {
            positions.push([pt[0]+r[0]*pf[0]+u[0]*pf[1], pt[1]+r[1]*pf[0]+u[1]*pf[1], pt[2]+r[2]*pf[0]+u[2]*pf[1]]);
        }
    }
    let mut triangles = Vec::new();
    for seg in 0..path.len()-1 {
        let base = (seg * n) as u32;
        let next = ((seg + 1) * n) as u32;
        for i in 0..n as u32 {
            let j = (i + 1) % n as u32;
            triangles.push([base+i, next+i, next+j]);
            triangles.push([base+i, next+j, base+j]);
        }
    }
    ExtrudedMesh { positions, triangles }
}

#[allow(dead_code)]
pub fn extrude_vertex_count(m: &ExtrudedMesh) -> usize { m.positions.len() }

#[allow(dead_code)]
pub fn extrude_tri_count(m: &ExtrudedMesh) -> usize { m.triangles.len() }

#[allow(dead_code)]
pub fn extrude_validate(m: &ExtrudedMesh) -> bool {
    m.triangles.iter().all(|t| t.iter().all(|&v| (v as usize) < m.positions.len()))
}

#[allow(dead_code)]
pub fn extrude_to_json(m: &ExtrudedMesh) -> String {
    format!("{{\"vertices\":{},\"triangles\":{}}}", m.positions.len(), m.triangles.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path3() -> Vec<[f32; 3]> { vec![[0.0,0.0,0.0],[0.0,1.0,0.0],[0.0,2.0,0.0]] }

    #[test] fn test_circle_profile() { let c = circle_profile(1.0, 8); assert_eq!(c.len(), 8); }
    #[test] fn test_extrude_basic() { let p = circle_profile(0.5, 4); let m = extrude_along_path(&p, &path3()); assert!(!m.positions.is_empty()); }
    #[test] fn test_vertex_count() { let p = circle_profile(0.5, 4); let m = extrude_along_path(&p, &path3()); assert_eq!(extrude_vertex_count(&m), 12); }
    #[test] fn test_tri_count() { let p = circle_profile(0.5, 4); let m = extrude_along_path(&p, &path3()); assert_eq!(extrude_tri_count(&m), 16); }
    #[test] fn test_validate() { let p = circle_profile(0.5, 4); let m = extrude_along_path(&p, &path3()); assert!(extrude_validate(&m)); }
    #[test] fn test_to_json() { let p = circle_profile(0.5, 4); let m = extrude_along_path(&p, &path3()); assert!(extrude_to_json(&m).contains("vertices")); }
    #[test] fn test_empty_path() { let m = extrude_along_path(&[[0.0,0.0]], &[]); assert!(m.positions.is_empty()); }
    #[test] fn test_single_point() { let m = extrude_along_path(&[[0.0,0.0]], &[[0.0,0.0,0.0]]); assert!(m.positions.is_empty()); }
    #[test] fn test_circle_min_seg() { let c = circle_profile(1.0, 1); assert_eq!(c.len(), 3); }
    #[test] fn test_two_point_path() { let p = circle_profile(0.5, 3); let m = extrude_along_path(&p, &[[0.0,0.0,0.0],[0.0,1.0,0.0]]); assert_eq!(m.positions.len(), 6); }
}
