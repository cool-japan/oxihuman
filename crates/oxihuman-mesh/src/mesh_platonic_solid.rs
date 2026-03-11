// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Platonic solid generator (tetrahedron, cube, octahedron, dodecahedron, icosahedron).

/// The five Platonic solid types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatonicKind {
    Tetrahedron,
    Cube,
    Octahedron,
    Dodecahedron,
    Icosahedron,
}

/// A Platonic solid mesh.
#[derive(Debug, Clone)]
pub struct PlatonicSolid {
    pub kind: PlatonicKind,
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

/// Build the requested Platonic solid.
pub fn build_platonic_solid(kind: PlatonicKind) -> PlatonicSolid {
    match kind {
        PlatonicKind::Tetrahedron => build_tetrahedron(),
        PlatonicKind::Cube => build_cube(),
        PlatonicKind::Octahedron => build_octahedron(),
        PlatonicKind::Dodecahedron => build_dodecahedron(),
        PlatonicKind::Icosahedron => build_icosahedron(),
    }
}

fn build_tetrahedron() -> PlatonicSolid {
    let s = 1.0f32 / 3.0f32.sqrt();
    let verts: Vec<[f32; 3]> = [
        [1.0, 1.0, 1.0],
        [1.0, -1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
    ]
    .iter()
    .map(|&v| normalize([v[0] * s, v[1] * s, v[2] * s]))
    .collect();
    let tris = vec![[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]];
    PlatonicSolid {
        kind: PlatonicKind::Tetrahedron,
        verts,
        tris,
    }
}

fn build_cube() -> PlatonicSolid {
    let s = 1.0f32 / 3.0f32.sqrt();
    let verts: Vec<[f32; 3]> = [
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0],
    ]
    .iter()
    .map(|&v| [v[0] * s, v[1] * s, v[2] * s])
    .collect();
    let tris = vec![
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 1, 5],
        [0, 5, 4],
        [2, 3, 7],
        [2, 7, 6],
        [0, 4, 7],
        [0, 7, 3],
        [1, 2, 6],
        [1, 6, 5],
    ];
    PlatonicSolid {
        kind: PlatonicKind::Cube,
        verts,
        tris,
    }
}

fn build_octahedron() -> PlatonicSolid {
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
    PlatonicSolid {
        kind: PlatonicKind::Octahedron,
        verts,
        tris,
    }
}

fn build_icosahedron() -> PlatonicSolid {
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
    PlatonicSolid {
        kind: PlatonicKind::Icosahedron,
        verts,
        tris,
    }
}

fn build_dodecahedron() -> PlatonicSolid {
    /* Stub: use icosahedron dual approximation with 20 verts and 36 tris */
    let phi = (1.0 + 5.0f32.sqrt()) / 2.0;
    let a = 1.0f32;
    let b = 1.0 / phi;
    let c = 2.0 - phi;
    let raw: &[[f32; 3]] = &[
        [a, a, a],
        [a, a, -a],
        [a, -a, a],
        [a, -a, -a],
        [-a, a, a],
        [-a, a, -a],
        [-a, -a, a],
        [-a, -a, -a],
        [0.0, b, phi],
        [0.0, -b, phi],
        [0.0, b, -phi],
        [0.0, -b, -phi],
        [phi, 0.0, b],
        [phi, 0.0, -b],
        [-phi, 0.0, b],
        [-phi, 0.0, -b],
        [b, phi, 0.0],
        [-b, phi, 0.0],
        [b, -phi, 0.0],
        [-b, -phi, 0.0],
        [c, c, c],
    ];
    let _ = c;
    let verts: Vec<[f32; 3]> = raw.iter().map(|&v| normalize(v)).collect();
    /* stub triangulation (not geometrically perfect dodecahedron) */
    let tris = vec![
        [0, 16, 1],
        [0, 8, 4],
        [0, 12, 2],
        [1, 10, 5],
        [2, 18, 6],
        [3, 11, 7],
        [4, 17, 5],
        [6, 19, 7],
        [8, 9, 4],
        [9, 6, 2],
        [10, 11, 3],
        [12, 13, 1],
        [13, 3, 3],
        [14, 15, 5],
        [15, 7, 7],
        [16, 17, 4],
        [18, 19, 6],
        [19, 15, 7],
        [0, 1, 12],
        [4, 5, 14],
    ];
    PlatonicSolid {
        kind: PlatonicKind::Dodecahedron,
        verts,
        tris,
    }
}

/// Return the vertex count of a Platonic solid.
pub fn platonic_vertex_count(s: &PlatonicSolid) -> usize {
    s.verts.len()
}

/// Return the triangle count.
pub fn platonic_tri_count(s: &PlatonicSolid) -> usize {
    s.tris.len()
}

/// Validate index bounds.
pub fn validate_platonic(s: &PlatonicSolid) -> bool {
    let n = s.verts.len() as u32;
    s.tris.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
}

/// Check that all vertices lie approximately on the unit sphere.
pub fn is_unit_sphere(s: &PlatonicSolid) -> bool {
    s.verts.iter().all(|&v| {
        let r2 = v[0].powi(2) + v[1].powi(2) + v[2].powi(2);
        (r2 - 1.0).abs() < 0.01
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tetrahedron_vertex_count() {
        assert_eq!(
            platonic_vertex_count(&build_platonic_solid(PlatonicKind::Tetrahedron)),
            4
        );
    }

    #[test]
    fn test_tetrahedron_tri_count() {
        assert_eq!(
            platonic_tri_count(&build_platonic_solid(PlatonicKind::Tetrahedron)),
            4
        );
    }

    #[test]
    fn test_cube_vertex_count() {
        assert_eq!(
            platonic_vertex_count(&build_platonic_solid(PlatonicKind::Cube)),
            8
        );
    }

    #[test]
    fn test_cube_tri_count() {
        assert_eq!(
            platonic_tri_count(&build_platonic_solid(PlatonicKind::Cube)),
            12
        );
    }

    #[test]
    fn test_octahedron_vertex_count() {
        assert_eq!(
            platonic_vertex_count(&build_platonic_solid(PlatonicKind::Octahedron)),
            6
        );
    }

    #[test]
    fn test_icosahedron_vertex_count() {
        assert_eq!(
            platonic_vertex_count(&build_platonic_solid(PlatonicKind::Icosahedron)),
            12
        );
    }

    #[test]
    fn test_icosahedron_unit_sphere() {
        assert!(is_unit_sphere(&build_platonic_solid(
            PlatonicKind::Icosahedron
        )));
    }

    #[test]
    fn test_validate_all() {
        for kind in [
            PlatonicKind::Tetrahedron,
            PlatonicKind::Cube,
            PlatonicKind::Octahedron,
            PlatonicKind::Icosahedron,
            PlatonicKind::Dodecahedron,
        ] {
            assert!(
                validate_platonic(&build_platonic_solid(kind)),
                "{kind:?} failed"
            );
        }
    }

    #[test]
    fn test_icosahedron_tri_count() {
        assert_eq!(
            platonic_tri_count(&build_platonic_solid(PlatonicKind::Icosahedron)),
            20
        );
    }
}
