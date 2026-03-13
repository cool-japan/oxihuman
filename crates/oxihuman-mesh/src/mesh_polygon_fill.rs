// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Polygon triangulation utilities for 3D polygons.

/// Triangulate a 3D polygon via fan triangulation from vertex 0.
/// Returns index triples into the polygon slice.
pub fn triangulate_polygon_3d(polygon: &[[f32; 3]]) -> Vec<[usize; 3]> {
    let n = polygon.len();
    if n < 3 {
        return vec![];
    }
    (1..n - 1).map(|i| [0, i, i + 1]).collect()
}

/// Signed area of a 3D polygon (magnitude = area, sign indicates winding via normal).
pub fn polygon_area_3d(polygon: &[[f32; 3]]) -> f32 {
    let tris = triangulate_polygon_3d(polygon);
    tris.iter()
        .map(|&[a, b, c]| {
            let pa = polygon[a];
            let pb = polygon[b];
            let pc = polygon[c];
            let e1 = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let e2 = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
            let cross = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt()
        })
        .sum()
}

/// Returns true if all polygon vertices are within `tolerance` of the best-fit plane.
pub fn polygon_is_planar(polygon: &[[f32; 3]], tolerance: f32) -> bool {
    let n = polygon.len();
    if n < 3 {
        return true;
    }
    let v0 = polygon[0];
    let e1 = [
        polygon[1][0] - v0[0],
        polygon[1][1] - v0[1],
        polygon[1][2] - v0[2],
    ];
    let e2 = [
        polygon[2][0] - v0[0],
        polygon[2][1] - v0[1],
        polygon[2][2] - v0[2],
    ];
    let normal = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if len < f32::EPSILON {
        return true;
    }
    let n_unit = [normal[0] / len, normal[1] / len, normal[2] / len];
    for &p in &polygon[3..] {
        let d =
            (p[0] - v0[0]) * n_unit[0] + (p[1] - v0[1]) * n_unit[1] + (p[2] - v0[2]) * n_unit[2];
        if d.abs() > tolerance {
            return false;
        }
    }
    true
}

/// Number of vertices in a polygon.
pub fn polygon_vertex_count(polygon: &[[f32; 3]]) -> usize {
    polygon.len()
}

/// Perimeter of a polygon (sum of edge lengths).
pub fn polygon_perimeter(polygon: &[[f32; 3]]) -> f32 {
    let n = polygon.len();
    if n < 2 {
        return 0.0;
    }
    (0..n)
        .map(|i| {
            let a = polygon[i];
            let b = polygon[(i + 1) % n];
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let dz = b[2] - a[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_square() -> Vec<[f32; 3]> {
        vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    #[test]
    fn test_triangulate_polygon_3d_square() {
        /* square → 2 triangles */
        let sq = unit_square();
        let tris = triangulate_polygon_3d(&sq);
        assert_eq!(tris.len(), 2);
    }

    #[test]
    fn test_polygon_area_3d_square() {
        let sq = unit_square();
        let a = polygon_area_3d(&sq);
        assert!((a - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_polygon_is_planar_flat() {
        let sq = unit_square();
        assert!(polygon_is_planar(&sq, 1e-4));
    }

    #[test]
    fn test_polygon_vertex_count() {
        let sq = unit_square();
        assert_eq!(polygon_vertex_count(&sq), 4);
    }

    #[test]
    fn test_polygon_perimeter_square() {
        let sq = unit_square();
        assert!((polygon_perimeter(&sq) - 4.0).abs() < 1e-5);
    }
}
