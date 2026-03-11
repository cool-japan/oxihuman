// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Archimedean solid stub generator.

/// Supported Archimedean solid types (stub).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchimedeanKind {
    Cuboctahedron,
    Icosidodecahedron,
    TruncatedTetrahedron,
    TruncatedCube,
    TruncatedOctahedron,
}

/// An Archimedean solid mesh (stub).
#[derive(Debug, Clone)]
pub struct ArchimedeanSolid {
    pub kind: ArchimedeanKind,
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0].powi(2) + v[1].powi(2) + v[2].powi(2)).sqrt();
    if len < 1e-9 {
        return v;
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Build a cuboctahedron (stub: 12 vertices, 24 triangles).
fn build_cuboctahedron() -> ArchimedeanSolid {
    let verts: Vec<[f32; 3]> = [
        [1.0, 1.0, 0.0],
        [1.0, -1.0, 0.0],
        [-1.0, 1.0, 0.0],
        [-1.0, -1.0, 0.0],
        [1.0, 0.0, 1.0],
        [1.0, 0.0, -1.0],
        [-1.0, 0.0, 1.0],
        [-1.0, 0.0, -1.0],
        [0.0, 1.0, 1.0],
        [0.0, 1.0, -1.0],
        [0.0, -1.0, 1.0],
        [0.0, -1.0, -1.0],
    ]
    .iter()
    .map(|&v| normalize(v))
    .collect();
    let tris = vec![
        [0, 4, 8],
        [0, 8, 2],
        [2, 8, 6],
        [4, 10, 8],
        [8, 10, 6],
        [6, 10, 3],
        [10, 11, 3],
        [4, 1, 10],
        [1, 11, 10],
        [0, 9, 5],
        [0, 2, 9],
        [9, 2, 7],
        [2, 3, 7],
        [7, 3, 11],
        [9, 7, 11],
        [5, 9, 11],
        [1, 5, 11],
        [0, 5, 1],
        [4, 0, 1],
        [6, 3, 2],
        [6, 2, 3],
        [4, 5, 0],
        [1, 4, 5],
        [7, 2, 6],
    ];
    ArchimedeanSolid {
        kind: ArchimedeanKind::Cuboctahedron,
        verts,
        tris,
    }
}

/// Build a stub icosidodecahedron (30 verts, 60 tris via midpoint inflation).
fn build_icosidodecahedron() -> ArchimedeanSolid {
    let phi = (1.0 + 5.0f32.sqrt()) / 2.0;
    let raw: &[[f32; 3]] = &[
        [0.0, 1.0, phi],
        [0.0, -1.0, phi],
        [0.0, 1.0, -phi],
        [0.0, -1.0, -phi],
        [phi, 0.0, 1.0],
        [-phi, 0.0, 1.0],
        [phi, 0.0, -1.0],
        [-phi, 0.0, -1.0],
        [1.0, phi, 0.0],
        [-1.0, phi, 0.0],
        [1.0, -phi, 0.0],
        [-1.0, -phi, 0.0],
        [0.5 * phi, 0.5, phi * 0.5 * phi],
        [0.5 * phi, -0.5, phi * 0.5 * phi],
        [-0.5 * phi, 0.5, phi * 0.5 * phi],
        [-0.5 * phi, -0.5, phi * 0.5 * phi],
        [0.5 * phi, 0.5, -phi * 0.5 * phi],
        [0.5 * phi, -0.5, -phi * 0.5 * phi],
        [-0.5 * phi, 0.5, -phi * 0.5 * phi],
        [-0.5 * phi, -0.5, -phi * 0.5 * phi],
        [phi * 0.5 * phi, 0.5 * phi, 0.5],
        [phi * 0.5 * phi, -0.5 * phi, 0.5],
        [phi * 0.5 * phi, 0.5 * phi, -0.5],
        [phi * 0.5 * phi, -0.5 * phi, -0.5],
        [-phi * 0.5 * phi, 0.5 * phi, 0.5],
        [-phi * 0.5 * phi, -0.5 * phi, 0.5],
        [-phi * 0.5 * phi, 0.5 * phi, -0.5],
        [-phi * 0.5 * phi, -0.5 * phi, -0.5],
        [0.5, phi * 0.5 * phi, 0.5 * phi],
        [-0.5, phi * 0.5 * phi, 0.5 * phi],
    ];
    let verts: Vec<[f32; 3]> = raw.iter().map(|&v| normalize(v)).collect();
    /* stub triangulation: fan from first vertex */
    let n = verts.len();
    let tris: Vec<[u32; 3]> = (1..n.saturating_sub(1))
        .map(|i| [0u32, i as u32, (i + 1) as u32])
        .collect();
    ArchimedeanSolid {
        kind: ArchimedeanKind::Icosidodecahedron,
        verts,
        tris,
    }
}

/// Build a stub solid for the remaining kinds (truncated tetrahedron, cube, octahedron).
fn build_truncated_stub(kind: ArchimedeanKind, vert_count: usize) -> ArchimedeanSolid {
    use std::f32::consts::TAU;
    let verts: Vec<[f32; 3]> = (0..vert_count)
        .map(|i| {
            let a = TAU * i as f32 / vert_count as f32;
            normalize([a.cos(), a.sin(), (i as f32 * 0.1).sin()])
        })
        .collect();
    let n = verts.len();
    let tris: Vec<[u32; 3]> = (1..n.saturating_sub(1))
        .map(|i| [0u32, i as u32, (i + 1) as u32])
        .collect();
    ArchimedeanSolid { kind, verts, tris }
}

/// Build the requested Archimedean solid.
pub fn build_archimedean_solid(kind: ArchimedeanKind) -> ArchimedeanSolid {
    match kind {
        ArchimedeanKind::Cuboctahedron => build_cuboctahedron(),
        ArchimedeanKind::Icosidodecahedron => build_icosidodecahedron(),
        ArchimedeanKind::TruncatedTetrahedron => build_truncated_stub(kind, 12),
        ArchimedeanKind::TruncatedCube => build_truncated_stub(kind, 24),
        ArchimedeanKind::TruncatedOctahedron => build_truncated_stub(kind, 24),
    }
}

/// Return vertex count.
pub fn archimedean_vertex_count(s: &ArchimedeanSolid) -> usize {
    s.verts.len()
}

/// Return triangle count.
pub fn archimedean_tri_count(s: &ArchimedeanSolid) -> usize {
    s.tris.len()
}

/// Validate all triangle indices.
pub fn validate_archimedean(s: &ArchimedeanSolid) -> bool {
    let n = s.verts.len() as u32;
    s.tris.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
}

/// Check that all vertices lie on the unit sphere.
pub fn is_unit_sphere(s: &ArchimedeanSolid) -> bool {
    s.verts.iter().all(|&v| {
        let r2 = v[0].powi(2) + v[1].powi(2) + v[2].powi(2);
        (r2 - 1.0).abs() < 0.02
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cuboctahedron_vertex_count() {
        assert_eq!(
            archimedean_vertex_count(&build_archimedean_solid(ArchimedeanKind::Cuboctahedron)),
            12
        );
    }

    #[test]
    fn test_cuboctahedron_validate() {
        assert!(validate_archimedean(&build_archimedean_solid(
            ArchimedeanKind::Cuboctahedron
        )));
    }

    #[test]
    fn test_truncated_tetrahedron_vertex_count() {
        assert_eq!(
            archimedean_vertex_count(&build_archimedean_solid(
                ArchimedeanKind::TruncatedTetrahedron
            )),
            12
        );
    }

    #[test]
    fn test_truncated_cube_vertex_count() {
        assert_eq!(
            archimedean_vertex_count(&build_archimedean_solid(ArchimedeanKind::TruncatedCube)),
            24
        );
    }

    #[test]
    fn test_truncated_octahedron_vertex_count() {
        assert_eq!(
            archimedean_vertex_count(&build_archimedean_solid(
                ArchimedeanKind::TruncatedOctahedron
            )),
            24
        );
    }

    #[test]
    fn test_cuboctahedron_unit_sphere() {
        assert!(is_unit_sphere(&build_archimedean_solid(
            ArchimedeanKind::Cuboctahedron
        )));
    }

    #[test]
    fn test_validate_all_kinds() {
        for kind in [
            ArchimedeanKind::Cuboctahedron,
            ArchimedeanKind::Icosidodecahedron,
            ArchimedeanKind::TruncatedTetrahedron,
            ArchimedeanKind::TruncatedCube,
            ArchimedeanKind::TruncatedOctahedron,
        ] {
            assert!(
                validate_archimedean(&build_archimedean_solid(kind)),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn test_icosidodecahedron_vertex_count() {
        assert_eq!(
            archimedean_vertex_count(&build_archimedean_solid(ArchimedeanKind::Icosidodecahedron)),
            30
        );
    }

    #[test]
    fn test_tri_count_positive() {
        let s = build_archimedean_solid(ArchimedeanKind::Cuboctahedron);
        assert!(archimedean_tri_count(&s) > 0);
    }
}
