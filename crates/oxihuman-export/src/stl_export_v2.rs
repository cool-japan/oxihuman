// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! STL export v2 with binary header.

#[allow(dead_code)]
pub struct StlTriangleV2 {
    pub normal: [f32; 3],
    pub vertices: [[f32; 3]; 3],
}

#[allow(dead_code)]
pub struct StlExportV2 {
    pub triangles: Vec<StlTriangleV2>,
    pub name: String,
}

#[allow(dead_code)]
pub fn new_stl_export_v2(name: &str) -> StlExportV2 {
    StlExportV2 { triangles: Vec::new(), name: name.to_string() }
}

#[allow(dead_code)]
pub fn stl2_add_triangle(
    e: &mut StlExportV2,
    normal: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) {
    e.triangles.push(StlTriangleV2 { normal, vertices: [v0, v1, v2] });
}

#[allow(dead_code)]
pub fn stl2_triangle_count(e: &StlExportV2) -> usize {
    e.triangles.len()
}

#[allow(dead_code)]
pub fn stl2_to_ascii(e: &StlExportV2) -> String {
    let mut out = format!("solid {}\n", e.name);
    for tri in &e.triangles {
        out.push_str(&format!(
            "  facet normal {} {} {}\n    outer loop\n",
            tri.normal[0], tri.normal[1], tri.normal[2]
        ));
        for v in &tri.vertices {
            out.push_str(&format!("      vertex {} {} {}\n", v[0], v[1], v[2]));
        }
        out.push_str("    endloop\n  endfacet\n");
    }
    out.push_str(&format!("endsolid {}\n", e.name));
    out
}

#[allow(dead_code)]
pub fn stl2_surface_area(e: &StlExportV2) -> f32 {
    e.triangles.iter().map(|tri| {
        let a = tri.vertices[0];
        let b = tri.vertices[1];
        let c = tri.vertices[2];
        let ab = [b[0]-a[0], b[1]-a[1], b[2]-a[2]];
        let ac = [c[0]-a[0], c[1]-a[1], c[2]-a[2]];
        let cross = [
            ab[1]*ac[2]-ab[2]*ac[1],
            ab[2]*ac[0]-ab[0]*ac[2],
            ab[0]*ac[1]-ab[1]*ac[0],
        ];
        (cross[0]*cross[0]+cross[1]*cross[1]+cross[2]*cross[2]).sqrt() * 0.5
    }).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let e = new_stl_export_v2("test");
        assert_eq!(stl2_triangle_count(&e), 0);
        assert_eq!(e.name, "test");
    }

    #[test]
    fn test_add_triangle() {
        let mut e = new_stl_export_v2("mesh");
        stl2_add_triangle(
            &mut e,
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        assert_eq!(stl2_triangle_count(&e), 1);
    }

    #[test]
    fn test_triangle_count() {
        let mut e = new_stl_export_v2("mesh");
        for _ in 0..5 {
            stl2_add_triangle(&mut e, [0.0, 0.0, 1.0], [0.0,0.0,0.0], [1.0,0.0,0.0], [0.0,1.0,0.0]);
        }
        assert_eq!(stl2_triangle_count(&e), 5);
    }

    #[test]
    fn test_to_ascii_contains_solid() {
        let e = new_stl_export_v2("part");
        let s = stl2_to_ascii(&e);
        assert!(s.contains("solid"));
    }

    #[test]
    fn test_to_ascii_contains_name() {
        let e = new_stl_export_v2("mypart");
        let s = stl2_to_ascii(&e);
        assert!(s.contains("mypart"));
    }

    #[test]
    fn test_surface_area_zero_empty() {
        let e = new_stl_export_v2("empty");
        assert_eq!(stl2_surface_area(&e), 0.0);
    }

    #[test]
    fn test_surface_area_unit_triangle() {
        let mut e = new_stl_export_v2("t");
        stl2_add_triangle(&mut e, [0.0,0.0,1.0], [0.0,0.0,0.0], [1.0,0.0,0.0], [0.0,1.0,0.0]);
        assert!((stl2_surface_area(&e) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_to_ascii_facet() {
        let mut e = new_stl_export_v2("m");
        stl2_add_triangle(&mut e, [0.0,0.0,1.0], [0.0,0.0,0.0], [1.0,0.0,0.0], [0.0,1.0,0.0]);
        let s = stl2_to_ascii(&e);
        assert!(s.contains("facet"));
    }
}
