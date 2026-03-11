// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Extruded polygon (prism) mesh generator.

/// An extruded polygon prism mesh.
#[derive(Debug, Clone)]
pub struct ExtrudedPolygon {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
    pub base_n: usize,
    pub height: f32,
}

/// Build an extruded polygon from a flat base polygon and a height.
/// `base` is a list of 2-D points (x, z) in the XZ plane.
/// The prism extends along the Y-axis from 0 to `height`.
pub fn build_extruded_polygon(base: &[[f32; 2]], height: f32) -> ExtrudedPolygon {
    let n = base.len();
    if n < 3 {
        return ExtrudedPolygon {
            verts: vec![],
            tris: vec![],
            base_n: 0,
            height: 0.0,
        };
    }
    /* bottom ring + top ring + bottom cap center + top cap center */
    let mut verts = Vec::with_capacity(2 * n + 2);
    for &[x, z] in base {
        verts.push([x, 0.0, z]);
    }
    for &[x, z] in base {
        verts.push([x, height, z]);
    }
    let bot_center = (2 * n) as u32;
    let top_center = (2 * n + 1) as u32;
    /* compute centroids */
    let mut cx = 0.0f32;
    let mut cz = 0.0f32;
    for &[x, z] in base {
        cx += x;
        cz += z;
    }
    cx /= n as f32;
    cz /= n as f32;
    verts.push([cx, 0.0, cz]);
    verts.push([cx, height, cz]);

    let mut tris = Vec::new();
    /* side quads */
    for i in 0..n {
        let next = (i + 1) % n;
        let a = i as u32;
        let b = next as u32;
        let c = (n + i) as u32;
        let d = (n + next) as u32;
        tris.push([a, b, d]);
        tris.push([a, d, c]);
    }
    /* bottom cap fan */
    for i in 0..n {
        let next = (i + 1) % n;
        tris.push([bot_center, next as u32, i as u32]);
    }
    /* top cap fan */
    for i in 0..n {
        let next = (i + 1) % n;
        tris.push([top_center, (n + i) as u32, (n + next) as u32]);
    }
    ExtrudedPolygon {
        verts,
        tris,
        base_n: n,
        height,
    }
}

/// Return vertex count.
pub fn extruded_vertex_count(ep: &ExtrudedPolygon) -> usize {
    ep.verts.len()
}

/// Return triangle count.
pub fn extruded_tri_count(ep: &ExtrudedPolygon) -> usize {
    ep.tris.len()
}

/// Validate triangle index bounds.
pub fn validate_extruded_polygon(ep: &ExtrudedPolygon) -> bool {
    let n = ep.verts.len() as u32;
    ep.tris.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
}

/// Compute the lateral surface area (sum of side quad areas).
pub fn lateral_area(ep: &ExtrudedPolygon) -> f32 {
    let n = ep.base_n;
    (0..n)
        .map(|i| {
            let a = ep.verts[i];
            let b = ep.verts[(i + 1) % n];
            let dx = b[0] - a[0];
            let dz = b[2] - a[2];
            (dx * dx + dz * dz).sqrt() * ep.height.abs()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Vec<[f32; 2]> {
        vec![[1.0, 1.0], [-1.0, 1.0], [-1.0, -1.0], [1.0, -1.0]]
    }
    fn triangle() -> Vec<[f32; 2]> {
        vec![[0.0, 1.0], [-1.0, -1.0], [1.0, -1.0]]
    }

    #[test]
    fn test_extruded_vertex_count_square() {
        /* 4 base + 4 top + 2 centers = 10 */
        let ep = build_extruded_polygon(&square(), 1.0);
        assert_eq!(extruded_vertex_count(&ep), 10);
    }

    #[test]
    fn test_extruded_tri_count_square() {
        /* 4*2 sides + 4 bottom fan + 4 top fan = 16 */
        let ep = build_extruded_polygon(&square(), 1.0);
        assert_eq!(extruded_tri_count(&ep), 16);
    }

    #[test]
    fn test_extruded_tri_count_triangle() {
        /* 3*2 sides + 3 bottom + 3 top = 12 */
        let ep = build_extruded_polygon(&triangle(), 2.0);
        assert_eq!(extruded_tri_count(&ep), 12);
    }

    #[test]
    fn test_validate_extruded_polygon() {
        let ep = build_extruded_polygon(&square(), 1.0);
        assert!(validate_extruded_polygon(&ep));
    }

    #[test]
    fn test_empty_on_too_few_verts() {
        let ep = build_extruded_polygon(&[[0.0, 0.0], [1.0, 0.0]], 1.0);
        assert_eq!(extruded_vertex_count(&ep), 0);
    }

    #[test]
    fn test_lateral_area_square_unit() {
        /* square of side 2, height 1 → perimeter*h = 8*1 = 8 */
        let ep = build_extruded_polygon(&square(), 1.0);
        assert!((lateral_area(&ep) - 8.0).abs() < 1e-4);
    }

    #[test]
    fn test_height_stored() {
        let ep = build_extruded_polygon(&square(), 3.5);
        assert!((ep.height - 3.5).abs() < 1e-6);
    }

    #[test]
    fn test_base_n_stored() {
        let ep = build_extruded_polygon(&triangle(), 1.0);
        assert_eq!(ep.base_n, 3);
    }
}
