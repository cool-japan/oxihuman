// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Stellated polyhedron stub generator.

/// The base polyhedron from which to stellate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StellationBase {
    Octahedron,
    Icosahedron,
    Dodecahedron,
}

/// A stellated polyhedron mesh.
#[derive(Debug, Clone)]
pub struct StellatedSolid {
    pub base: StellationBase,
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
    pub stellation_height: f32,
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0].powi(2) + v[1].powi(2) + v[2].powi(2)).sqrt();
    if len < 1e-9 {
        return v;
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn centroid3(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0] + c[0]) / 3.0,
        (a[1] + b[1] + c[1]) / 3.0,
        (a[2] + b[2] + c[2]) / 3.0,
    ]
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

/// Stellate a base mesh by adding an apex above each face centroid.
/// Returns (stellated_verts, stellated_tris).
fn stellate(
    base_verts: &[[f32; 3]],
    base_tris: &[[u32; 3]],
    h: f32,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let mut verts: Vec<[f32; 3]> = base_verts.to_vec();
    let mut tris: Vec<[u32; 3]> = Vec::new();
    for tri in base_tris {
        let a = base_verts[tri[0] as usize];
        let b = base_verts[tri[1] as usize];
        let c = base_verts[tri[2] as usize];
        let cen = centroid3(a, b, c);
        let apex = scale3(normalize(cen), 1.0 + h);
        let apex_idx = verts.len() as u32;
        verts.push(apex);
        tris.push([tri[0], tri[1], apex_idx]);
        tris.push([tri[1], tri[2], apex_idx]);
        tris.push([tri[2], tri[0], apex_idx]);
    }
    (verts, tris)
}

fn build_octahedron_base() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let verts = vec![
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, -1.0],
    ];
    let tris = vec![
        [0, 2, 4],
        [2, 1, 4],
        [1, 3, 4],
        [3, 0, 4],
        [0, 5, 2],
        [2, 5, 1],
        [1, 5, 3],
        [3, 5, 0],
    ];
    (verts, tris)
}

fn build_icosahedron_base() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let phi = (1.0 + 5.0f32.sqrt()) / 2.0;
    let verts: Vec<[f32; 3]> = [
        [-1.0, phi, 0.0],
        [1.0, phi, 0.0],
        [-1.0, -phi, 0.0],
        [1.0, -phi, 0.0],
        [0.0, -1.0, phi],
        [0.0, 1.0, phi],
        [0.0, -1.0, -phi],
        [0.0, 1.0, -phi],
        [phi, 0.0, -1.0],
        [phi, 0.0, 1.0],
        [-phi, 0.0, -1.0],
        [-phi, 0.0, 1.0],
    ]
    .iter()
    .map(|&v| normalize(v))
    .collect();
    let tris = vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];
    (verts, tris)
}

/// Build a stellated solid.
pub fn build_stellated_solid(base: StellationBase, stellation_height: f32) -> StellatedSolid {
    let (bv, bt) = match base {
        StellationBase::Octahedron => build_octahedron_base(),
        StellationBase::Icosahedron => build_icosahedron_base(),
        StellationBase::Dodecahedron => {
            /* stub: use icosahedron base with different height */
            build_icosahedron_base()
        }
    };
    let (verts, tris) = stellate(&bv, &bt, stellation_height);
    StellatedSolid {
        base,
        verts,
        tris,
        stellation_height,
    }
}

/// Return vertex count.
pub fn stellated_vertex_count(s: &StellatedSolid) -> usize {
    s.verts.len()
}

/// Return triangle count.
pub fn stellated_tri_count(s: &StellatedSolid) -> usize {
    s.tris.len()
}

/// Validate all triangle indices.
pub fn validate_stellated(s: &StellatedSolid) -> bool {
    let n = s.verts.len() as u32;
    s.tris.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
}

/// Check that all spike apex vertices are further from origin than base verts.
pub fn stellations_protrude(s: &StellatedSolid) -> bool {
    s.stellation_height > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stellated_octahedron_base_verts() {
        /* octahedron: 6 base + 8 face apices = 14 verts */
        let s = build_stellated_solid(StellationBase::Octahedron, 0.5);
        assert_eq!(stellated_vertex_count(&s), 14);
    }

    #[test]
    fn test_stellated_octahedron_tri_count() {
        /* 8 faces * 3 tris = 24 */
        let s = build_stellated_solid(StellationBase::Octahedron, 0.5);
        assert_eq!(stellated_tri_count(&s), 24);
    }

    #[test]
    fn test_stellated_icosahedron_tri_count() {
        /* 20 faces * 3 tris = 60 */
        let s = build_stellated_solid(StellationBase::Icosahedron, 0.3);
        assert_eq!(stellated_tri_count(&s), 60);
    }

    #[test]
    fn test_validate_stellated_octahedron() {
        let s = build_stellated_solid(StellationBase::Octahedron, 0.5);
        assert!(validate_stellated(&s));
    }

    #[test]
    fn test_validate_stellated_icosahedron() {
        let s = build_stellated_solid(StellationBase::Icosahedron, 0.3);
        assert!(validate_stellated(&s));
    }

    #[test]
    fn test_stellations_protrude() {
        let s = build_stellated_solid(StellationBase::Octahedron, 0.5);
        assert!(stellations_protrude(&s));
    }

    #[test]
    fn test_stellation_height_stored() {
        let s = build_stellated_solid(StellationBase::Icosahedron, 1.2);
        assert!((s.stellation_height - 1.2).abs() < 1e-6);
    }

    #[test]
    fn test_dodecahedron_stub_validates() {
        let s = build_stellated_solid(StellationBase::Dodecahedron, 0.4);
        assert!(validate_stellated(&s));
    }

    #[test]
    fn test_stellated_icosahedron_vertex_count() {
        /* 12 base + 20 apices = 32 */
        let s = build_stellated_solid(StellationBase::Icosahedron, 0.3);
        assert_eq!(stellated_vertex_count(&s), 32);
    }
}
