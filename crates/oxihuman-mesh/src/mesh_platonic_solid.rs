// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
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
    // Golden ratio φ = (1 + √5) / 2; its reciprocal 1/φ = φ - 1.
    let phi = (1.0f32 + 5.0f32.sqrt()) / 2.0;
    let inv_phi = 1.0 / phi; // = φ - 1

    // 20 canonical vertices of the dodecahedron (before normalisation):
    //   indices 0-7  : cube corners (±1, ±1, ±1)
    //   indices 8-11 : (0, ±1/φ, ±φ)
    //   indices 12-15: (±1/φ, ±φ, 0)  — note the task lists these as (±1/φ, ±φ, 0)
    //   indices 16-19: (±φ, 0, ±1/φ)
    //
    // Vertex layout matches the face topology given in the task spec exactly:
    //  0: ( 1,  1,  1)
    //  1: ( 1,  1, -1)
    //  2: ( 1, -1,  1)
    //  3: ( 1, -1, -1)
    //  4: (-1,  1,  1)
    //  5: (-1,  1, -1)
    //  6: (-1, -1,  1)
    //  7: (-1, -1, -1)
    //  8: (0,  1/φ,  φ)
    //  9: (0,  1/φ, -φ)
    // 10: (0, -1/φ,  φ)
    // 11: (0, -1/φ, -φ)
    // 12: ( 1/φ,  φ, 0)
    // 13: ( 1/φ, -φ, 0)
    // 14: (-1/φ,  φ, 0)
    // 15: (-1/φ, -φ, 0)
    // 16: ( φ, 0,  1/φ)
    // 17: ( φ, 0, -1/φ)
    // 18: (-φ, 0,  1/φ)
    // 19: (-φ, 0, -1/φ)
    let raw: [[f32; 3]; 20] = [
        [1.0, 1.0, 1.0],          //  0
        [1.0, 1.0, -1.0],         //  1
        [1.0, -1.0, 1.0],         //  2
        [1.0, -1.0, -1.0],        //  3
        [-1.0, 1.0, 1.0],         //  4
        [-1.0, 1.0, -1.0],        //  5
        [-1.0, -1.0, 1.0],        //  6
        [-1.0, -1.0, -1.0],       //  7
        [0.0, inv_phi, phi],       //  8
        [0.0, inv_phi, -phi],      //  9
        [0.0, -inv_phi, phi],      // 10
        [0.0, -inv_phi, -phi],     // 11
        [inv_phi, phi, 0.0],       // 12
        [inv_phi, -phi, 0.0],      // 13
        [-inv_phi, phi, 0.0],      // 14
        [-inv_phi, -phi, 0.0],     // 15
        [phi, 0.0, inv_phi],       // 16
        [phi, 0.0, -inv_phi],      // 17
        [-phi, 0.0, inv_phi],      // 18
        [-phi, 0.0, -inv_phi],     // 19
    ];

    // Normalize every vertex onto the unit sphere.
    let verts: Vec<[f32; 3]> = raw.iter().map(|&v| normalize(v)).collect();

    // 12 pentagonal faces — outward-facing CCW rings (verified by cross-product
    // winding: (v1−v0)×(v2−v0)·centroid > 0, where centroid ≈ origin for a
    // unit-sphere solid).  Fan-triangulated below: [v0,v1,v2], [v0,v2,v3], [v0,v3,v4].
    let pentagons: [[u32; 5]; 12] = [
        [0, 8, 10, 2, 16],
        [0, 16, 17, 1, 12],
        [0, 12, 14, 4, 8],
        [4, 14, 5, 19, 18],
        [4, 18, 6, 10, 8],
        [6, 18, 19, 7, 15],
        [6, 15, 13, 2, 10],
        [2, 13, 3, 17, 16],
        [1, 17, 3, 11, 9],
        [1, 9, 5, 14, 12],
        [5, 9, 11, 7, 19],
        [7, 11, 3, 13, 15],
    ];

    // Fan-triangulate each pentagon into 3 triangles.
    // For pentagon [v0, v1, v2, v3, v4]:
    //   tri 0: [v0, v1, v2]
    //   tri 1: [v0, v2, v3]
    //   tri 2: [v0, v3, v4]
    // Total: 12 × 3 = 36 triangles.
    let mut tris: Vec<[u32; 3]> = Vec::with_capacity(36);
    for pent in &pentagons {
        let v0 = pent[0];
        tris.push([v0, pent[1], pent[2]]);
        tris.push([v0, pent[2], pent[3]]);
        tris.push([v0, pent[3], pent[4]]);
    }

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

    // ── Dodecahedron tests ──────────────────────────────────────────────────

    #[test]
    fn test_dodecahedron_vertex_count() {
        let solid = build_platonic_solid(PlatonicKind::Dodecahedron);
        assert_eq!(solid.verts.len(), 20);
    }

    #[test]
    fn test_dodecahedron_tri_count() {
        let solid = build_platonic_solid(PlatonicKind::Dodecahedron);
        assert_eq!(solid.tris.len(), 36);
    }

    #[test]
    fn test_dodecahedron_unit_sphere() {
        let solid = build_platonic_solid(PlatonicKind::Dodecahedron);
        assert!(is_unit_sphere(&solid), "dodecahedron vertices not on unit sphere");
    }

    #[test]
    fn test_dodecahedron_no_degenerate_tris() {
        let solid = build_platonic_solid(PlatonicKind::Dodecahedron);
        for tri in &solid.tris {
            assert_ne!(tri[0], tri[1], "degenerate tri: indices 0 and 1 equal");
            assert_ne!(tri[1], tri[2], "degenerate tri: indices 1 and 2 equal");
            assert_ne!(tri[0], tri[2], "degenerate tri: indices 0 and 2 equal");
        }
    }

    #[test]
    fn test_dodecahedron_edge_equality() {
        // The 12 pentagon face definitions (pre-triangulation) — must match
        // build_dodecahedron exactly.
        let pentagons: [[u32; 5]; 12] = [
            [0, 8, 10, 2, 16],
            [0, 16, 17, 1, 12],
            [0, 12, 14, 4, 8],
            [4, 14, 5, 19, 18],
            [4, 18, 6, 10, 8],
            [6, 18, 19, 7, 15],
            [6, 15, 13, 2, 10],
            [2, 13, 3, 17, 16],
            [1, 17, 3, 11, 9],
            [1, 9, 5, 14, 12],
            [5, 9, 11, 7, 19],
            [7, 11, 3, 13, 15],
        ];

        // Collect 30 unique pentagon edges (each face contributes 5 edges;
        // each dodecahedron edge is shared by exactly 2 faces, yielding
        // 12×5/2 = 30 unique edges).
        let mut edge_set: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();
        for pent in &pentagons {
            for k in 0..5usize {
                let a = pent[k];
                let b = pent[(k + 1) % 5];
                let edge = if a < b { (a, b) } else { (b, a) };
                edge_set.insert(edge);
            }
        }
        assert_eq!(edge_set.len(), 30, "expected exactly 30 unique pentagon edges");

        let solid = build_platonic_solid(PlatonicKind::Dodecahedron);
        let v = &solid.verts;

        // Compute the length of each unique edge on the unit-sphere vertices.
        let edge_lengths: Vec<f32> = edge_set
            .iter()
            .map(|&(a, b)| {
                let va = v[a as usize];
                let vb = v[b as usize];
                let dx = va[0] - vb[0];
                let dy = va[1] - vb[1];
                let dz = va[2] - vb[2];
                (dx * dx + dy * dy + dz * dz).sqrt()
            })
            .collect();

        // All 30 edge lengths must be equal within 1e-3 (regular dodecahedron property).
        let first = edge_lengths[0];
        for (i, &len) in edge_lengths.iter().enumerate() {
            assert!(
                (len - first).abs() < 1e-3,
                "edge {i} length {len} differs from first edge length {first} by more than 1e-3"
            );
        }
    }
}
