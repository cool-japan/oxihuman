// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cylinder-mesh intersection tests.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Cylinder {
    pub center: [f32; 3],
    pub axis: [f32; 3],
    pub radius: f32,
    pub half_height: f32,
}

#[allow(dead_code)]
pub fn new_cylinder(center: [f32; 3], axis: [f32; 3], radius: f32, half_height: f32) -> Cylinder {
    Cylinder { center, axis, radius: radius.max(0.0), half_height: half_height.max(0.0) }
}

#[allow(dead_code)]
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { a[0]*b[0]+a[1]*b[1]+a[2]*b[2] }

#[allow(dead_code)]
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0]-b[0],a[1]-b[1],a[2]-b[2]] }

#[allow(dead_code)]
pub fn point_in_cylinder(cyl: &Cylinder, p: [f32; 3]) -> bool {
    let d = sub(p, cyl.center);
    let axis_len_sq = dot(cyl.axis, cyl.axis);
    if axis_len_sq < 1e-12 { return false; }
    let proj = dot(d, cyl.axis) / axis_len_sq.sqrt();
    if proj.abs() > cyl.half_height { return false; }
    let proj_vec = [cyl.axis[0]*proj/axis_len_sq.sqrt(), cyl.axis[1]*proj/axis_len_sq.sqrt(), cyl.axis[2]*proj/axis_len_sq.sqrt()];
    let perp = sub(d, proj_vec);
    dot(perp, perp).sqrt() <= cyl.radius
}

#[allow(dead_code)]
pub fn cylinder_intersects_triangle(cyl: &Cylinder, a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> bool {
    point_in_cylinder(cyl, a) || point_in_cylinder(cyl, b) || point_in_cylinder(cyl, c)
}

#[allow(dead_code)]
pub fn cylinder_intersecting_faces(cyl: &Cylinder, positions: &[[f32; 3]], faces: &[[u32; 3]]) -> Vec<usize> {
    faces.iter().enumerate().filter(|(_, f)| {
        cylinder_intersects_triangle(cyl, positions[f[0] as usize], positions[f[1] as usize], positions[f[2] as usize])
    }).map(|(i, _)| i).collect()
}

#[allow(dead_code)]
pub fn cylinder_volume(cyl: &Cylinder) -> f32 {
    PI * cyl.radius * cyl.radius * cyl.half_height * 2.0
}

#[allow(dead_code)]
pub fn cylinder_surface_area(cyl: &Cylinder) -> f32 {
    let h = cyl.half_height * 2.0;
    2.0 * PI * cyl.radius * (cyl.radius + h)
}

#[allow(dead_code)]
pub fn cylinder_to_json(cyl: &Cylinder) -> String {
    format!("{{\"radius\":{:.4},\"height\":{:.4},\"volume\":{:.4}}}", cyl.radius, cyl.half_height * 2.0, cylinder_volume(cyl))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cyl() -> Cylinder { new_cylinder([0.0,0.0,0.0], [0.0,1.0,0.0], 1.0, 2.0) }

    #[test] fn test_point_inside() { assert!(point_in_cylinder(&make_cyl(), [0.0, 0.5, 0.0])); }
    #[test] fn test_point_outside() { assert!(!point_in_cylinder(&make_cyl(), [5.0, 0.0, 0.0])); }
    #[test] fn test_point_above() { assert!(!point_in_cylinder(&make_cyl(), [0.0, 5.0, 0.0])); }
    #[test] fn test_tri_intersect() { assert!(cylinder_intersects_triangle(&make_cyl(), [0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0])); }
    #[test] fn test_tri_no_intersect() { assert!(!cylinder_intersects_triangle(&make_cyl(), [10.0,10.0,10.0],[11.0,10.0,10.0],[10.0,11.0,10.0])); }
    #[test] fn test_faces() {
        let p = vec![[0.0,0.0,0.0],[0.5,0.0,0.0],[0.0,0.5,0.0]];
        assert_eq!(cylinder_intersecting_faces(&make_cyl(), &p, &[[0,1,2]]).len(), 1);
    }
    #[test] fn test_volume() { assert!(cylinder_volume(&make_cyl()) > 0.0); }
    #[test] fn test_surface_area() { assert!(cylinder_surface_area(&make_cyl()) > 0.0); }
    #[test] fn test_to_json() { assert!(cylinder_to_json(&make_cyl()).contains("volume")); }
    #[test] fn test_center_inside() { assert!(point_in_cylinder(&make_cyl(), [0.0, 0.0, 0.0])); }
}
