// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cone-mesh intersection tests.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Cone {
    pub apex: [f32; 3],
    pub axis: [f32; 3],
    pub half_angle: f32,
    pub height: f32,
}

#[allow(dead_code)]
pub fn new_cone(apex: [f32; 3], axis: [f32; 3], half_angle: f32, height: f32) -> Cone {
    Cone { apex, axis, half_angle: half_angle.clamp(0.0, PI * 0.5), height: height.max(0.0) }
}

#[allow(dead_code)]
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { a[0]*b[0]+a[1]*b[1]+a[2]*b[2] }

#[allow(dead_code)]
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0]-b[0],a[1]-b[1],a[2]-b[2]] }

#[allow(dead_code)]
fn len(v: [f32; 3]) -> f32 { dot(v, v).sqrt() }

#[allow(dead_code)]
pub fn point_in_cone(cone: &Cone, point: [f32; 3]) -> bool {
    let d = sub(point, cone.apex);
    let axis_len = len(cone.axis);
    if axis_len < 1e-12 { return false; }
    let proj = dot(d, cone.axis) / axis_len;
    if proj < 0.0 || proj > cone.height { return false; }
    let dist = len(d);
    if dist < 1e-12 { return true; }
    let angle = (proj / dist).acos();
    angle <= cone.half_angle
}

#[allow(dead_code)]
pub fn cone_intersects_triangle(cone: &Cone, a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> bool {
    point_in_cone(cone, a) || point_in_cone(cone, b) || point_in_cone(cone, c)
}

#[allow(dead_code)]
pub fn cone_intersecting_faces(cone: &Cone, positions: &[[f32; 3]], faces: &[[u32; 3]]) -> Vec<usize> {
    faces.iter().enumerate().filter(|(_, f)| {
        cone_intersects_triangle(cone, positions[f[0] as usize], positions[f[1] as usize], positions[f[2] as usize])
    }).map(|(i, _)| i).collect()
}

#[allow(dead_code)]
pub fn cone_base_radius(cone: &Cone) -> f32 { cone.height * cone.half_angle.tan() }

#[allow(dead_code)]
pub fn cone_volume(cone: &Cone) -> f32 {
    let r = cone_base_radius(cone);
    PI * r * r * cone.height / 3.0
}

#[allow(dead_code)]
pub fn cone_to_json(cone: &Cone) -> String {
    format!("{{\"half_angle\":{:.4},\"height\":{:.4},\"volume\":{:.4}}}", cone.half_angle, cone.height, cone_volume(cone))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cone() -> Cone { new_cone([0.0,0.0,0.0], [0.0,1.0,0.0], PI/4.0, 2.0) }

    #[test] fn test_point_inside() { assert!(point_in_cone(&make_cone(), [0.0, 1.0, 0.0])); }
    #[test] fn test_point_outside() { assert!(!point_in_cone(&make_cone(), [10.0, 0.0, 0.0])); }
    #[test] fn test_apex_inside() { assert!(point_in_cone(&make_cone(), [0.0, 0.0, 0.0])); }
    #[test] fn test_tri_intersect() { assert!(cone_intersects_triangle(&make_cone(), [0.0,1.0,0.0],[1.0,0.0,0.0],[0.0,0.0,1.0])); }
    #[test] fn test_tri_no_intersect() { assert!(!cone_intersects_triangle(&make_cone(), [10.0,10.0,10.0],[11.0,10.0,10.0],[10.0,11.0,10.0])); }
    #[test] fn test_faces() {
        let p = vec![[0.0,1.0,0.0],[1.0,0.0,0.0],[0.0,0.0,1.0]];
        let f = vec![[0,1,2]];
        assert_eq!(cone_intersecting_faces(&make_cone(), &p, &f).len(), 1);
    }
    #[test] fn test_base_radius() { let c = make_cone(); assert!(cone_base_radius(&c) > 0.0); }
    #[test] fn test_volume() { let c = make_cone(); assert!(cone_volume(&c) > 0.0); }
    #[test] fn test_to_json() { assert!(cone_to_json(&make_cone()).contains("volume")); }
    #[test] fn test_behind_apex() { assert!(!point_in_cone(&make_cone(), [0.0, -1.0, 0.0])); }
}
