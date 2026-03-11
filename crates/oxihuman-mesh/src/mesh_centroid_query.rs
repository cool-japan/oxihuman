// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mesh centroid computation stub.

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Compute the surface centroid (area-weighted average of triangle centroids).
pub fn surface_centroid(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> Option<[f32; 3]> {
    if tris.is_empty() || verts.is_empty() {
        return None;
    }
    let mut weighted_sum = [0.0f32; 3];
    let mut total_area = 0.0f32;
    for tri in tris {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
            continue;
        }
        let (v0, v1, v2) = (verts[i0], verts[i1], verts[i2]);
        let e1 = sub3(v1, v0);
        let e2 = sub3(v2, v0);
        let area = len3(cross3(e1, e2)) * 0.5;
        let tc = [
            (v0[0] + v1[0] + v2[0]) / 3.0,
            (v0[1] + v1[1] + v2[1]) / 3.0,
            (v0[2] + v1[2] + v2[2]) / 3.0,
        ];
        for k in 0..3 {
            weighted_sum[k] += area * tc[k];
        }
        total_area += area;
    }
    if total_area < 1e-12 {
        return None;
    }
    Some([
        weighted_sum[0] / total_area,
        weighted_sum[1] / total_area,
        weighted_sum[2] / total_area,
    ])
}

/// Compute the vertex centroid (simple average of vertex positions).
pub fn vertex_centroid(verts: &[[f32; 3]]) -> Option<[f32; 3]> {
    if verts.is_empty() {
        return None;
    }
    let mut sum = [0.0f32; 3];
    for v in verts {
        for k in 0..3 {
            sum[k] += v[k];
        }
    }
    let n = verts.len() as f32;
    Some([sum[0] / n, sum[1] / n, sum[2] / n])
}

/// Compute the volume centroid (center of mass for a solid mesh).
pub fn volume_centroid(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> Option<[f32; 3]> {
    /* Uses divergence theorem: sum v0·(v1×v2) for tetrahedra from origin */
    if tris.is_empty() || verts.is_empty() {
        return None;
    }
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    let mut vol = 0.0f32;

    for tri in tris {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
            continue;
        }
        let (a, b, c) = (verts[i0], verts[i1], verts[i2]);
        let v_tet = dot3(a, cross3(b, c)) / 6.0;
        cx += v_tet * (a[0] + b[0] + c[0]) / 4.0;
        cy += v_tet * (a[1] + b[1] + c[1]) / 4.0;
        cz += v_tet * (a[2] + b[2] + c[2]) / 4.0;
        vol += v_tet;
    }
    if vol.abs() < 1e-12 {
        return None;
    }
    Some([cx / vol, cy / vol, cz / vol])
}

/// Distance from the surface centroid to the vertex centroid.
pub fn centroid_deviation(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> f32 {
    let sc = surface_centroid(verts, tris);
    let vc = vertex_centroid(verts);
    match (sc, vc) {
        (Some(s), Some(v)) => len3(sub3(s, v)),
        _ => 0.0,
    }
}

/// Translate all vertices so that the vertex centroid is at the origin.
pub fn center_mesh_at_centroid(verts: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let Some(c) = vertex_centroid(verts) else {
        return verts.to_vec();
    };
    verts
        .iter()
        .map(|v| [v[0] - c[0], v[1] - c[1], v[2] - c[2]])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_centroid_empty() {
        assert!(vertex_centroid(&[]).is_none() /* empty vertices */);
    }

    #[test]
    fn test_vertex_centroid_symmetric() {
        let verts = vec![[1.0f32, 0.0, 0.0], [-1.0, 0.0, 0.0]];
        let c = vertex_centroid(&verts).unwrap();
        assert!(c[0].abs() < 1e-5 /* symmetric → centroid at x=0 */);
    }

    #[test]
    fn test_surface_centroid_empty() {
        assert!(surface_centroid(&[], &[]).is_none() /* empty mesh */);
    }

    #[test]
    fn test_surface_centroid_single_triangle() {
        let verts = vec![[0.0f32, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let c = surface_centroid(&verts, &tris).unwrap();
        assert!((c[0] - 1.0).abs() < 1e-5 /* x centroid = 1 */);
        assert!((c[1] - 1.0).abs() < 1e-5 /* y centroid = 1 */);
    }

    #[test]
    fn test_volume_centroid_empty() {
        assert!(volume_centroid(&[], &[]).is_none() /* empty mesh */);
    }

    #[test]
    fn test_centroid_deviation_nonneg() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        assert!(centroid_deviation(&verts, &tris) >= 0.0 /* non-negative */);
    }

    #[test]
    fn test_center_mesh_at_centroid() {
        let verts = vec![[1.0f32, 0.0, 0.0], [-1.0, 0.0, 0.0]];
        let centered = center_mesh_at_centroid(&verts);
        let c = vertex_centroid(&centered).unwrap();
        assert!(c[0].abs() < 1e-5 /* centered at origin */);
    }

    #[test]
    fn test_center_mesh_empty() {
        let centered = center_mesh_at_centroid(&[]);
        assert!(centered.is_empty() /* empty stays empty */);
    }

    #[test]
    fn test_vertex_centroid_single() {
        let verts = vec![[3.0f32, 7.0, -2.0]];
        let c = vertex_centroid(&verts).unwrap();
        assert_eq!(
            c,
            [3.0, 7.0, -2.0] /* single vertex centroid equals itself */
        );
    }

    #[test]
    fn test_surface_centroid_degenerate_triangle() {
        /* Degenerate (zero-area) triangle should result in None */
        let verts = vec![[0.0f32; 3], [0.0; 3], [0.0; 3]];
        let tris = vec![[0u32, 1, 2]];
        assert!(surface_centroid(&verts, &tris).is_none() /* zero area */);
    }
}
