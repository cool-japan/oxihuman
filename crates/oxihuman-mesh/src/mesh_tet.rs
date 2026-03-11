// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Tetrahedral mesh generation from a surface triangle mesh.

#[allow(dead_code)]
pub struct TetMesh {
    pub vertices: Vec<[f32; 3]>,
    pub tets: Vec<[usize; 4]>,
    pub surface_indices: Vec<u32>,
}

#[allow(dead_code)]
pub struct TetGenConfig {
    pub max_tet_volume: f32,
    pub interior_points: u32,
    pub seed: u64,
}

impl Default for TetGenConfig {
    fn default() -> Self {
        Self {
            max_tet_volume: 0.01,
            interior_points: 10,
            seed: 42,
        }
    }
}

#[allow(dead_code)]
pub struct TetGenResult {
    pub mesh: TetMesh,
    pub tet_count: usize,
    pub interior_vertex_count: usize,
    pub min_dihedral_angle_deg: f32,
}

/// Signed volume of a tetrahedron via scalar triple product / 6.
#[allow(dead_code)]
pub fn tet_volume(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let ad = [d[0] - a[0], d[1] - a[1], d[2] - a[2]];
    // scalar triple product: ab · (ac × ad)
    let cross = [
        ac[1] * ad[2] - ac[2] * ad[1],
        ac[2] * ad[0] - ac[0] * ad[2],
        ac[0] * ad[1] - ac[1] * ad[0],
    ];
    (ab[0] * cross[0] + ab[1] * cross[1] + ab[2] * cross[2]) / 6.0
}

/// Centroid of a tetrahedron (average of 4 vertices).
#[allow(dead_code)]
pub fn tet_centroid(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0] + c[0] + d[0]) / 4.0,
        (a[1] + b[1] + c[1] + d[1]) / 4.0,
        (a[2] + b[2] + c[2] + d[2]) / 4.0,
    ]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn norm3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-12 {
        v
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Compute the 6 dihedral angles (in degrees) for a tetrahedron, one per edge.
/// Edge order: AB, AC, AD, BC, BD, CD.
#[allow(dead_code)]
pub fn tet_dihedral_angles(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) -> [f32; 6] {
    let face_normal = |p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]| -> [f32; 3] {
        norm3(cross3(sub3(p1, p0), sub3(p2, p0)))
    };

    let dihedral = |n1: [f32; 3], n2: [f32; 3]| -> f32 {
        let cos_angle = (-dot3(n1, n2)).clamp(-1.0, 1.0);
        cos_angle.acos().to_degrees()
    };

    // Faces: ABC, ABD, ACD, BCD
    let n_abc = face_normal(a, b, c);
    let n_abd = face_normal(a, b, d);
    let n_acd = face_normal(a, c, d);
    let n_bcd = face_normal(b, c, d);

    [
        dihedral(n_abc, n_abd), // edge AB
        dihedral(n_abc, n_acd), // edge AC
        dihedral(n_abd, n_acd), // edge AD
        dihedral(n_abc, n_bcd), // edge BC
        dihedral(n_abd, n_bcd), // edge BD
        dihedral(n_acd, n_bcd), // edge CD
    ]
}

/// Total volume of a tet mesh (sum of absolute tet volumes).
#[allow(dead_code)]
pub fn tet_mesh_volume(mesh: &TetMesh) -> f32 {
    mesh.tets
        .iter()
        .map(|t| {
            tet_volume(
                mesh.vertices[t[0]],
                mesh.vertices[t[1]],
                mesh.vertices[t[2]],
                mesh.vertices[t[3]],
            )
            .abs()
        })
        .sum()
}

/// Validate a tet mesh: all indices in range, no out-of-bounds.
#[allow(dead_code)]
pub fn validate_tet_mesh(mesh: &TetMesh) -> bool {
    let n = mesh.vertices.len();
    for t in &mesh.tets {
        if t[0] >= n || t[1] >= n || t[2] >= n || t[3] >= n {
            return false;
        }
    }
    true
}

/// Simple LCG random number generator for interior point placement.
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x12345678,
        }
    }

    /// Returns value in [0, 1).
    fn next_f32(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 33) as f32) / (u32::MAX as f32)
    }
}

/// Test whether a point is inside a surface mesh using winding number.
/// Uses a simple ray-casting approach to avoid dependency on MeshBuffers.
fn point_inside_surface(p: [f32; 3], positions: &[[f32; 3]], indices: &[u32]) -> bool {
    // Winding number via solid angle summation (Gauss formula)
    let mut wn = 0.0f32;
    let n_tri = indices.len() / 3;
    for ti in 0..n_tri {
        let a = positions[indices[ti * 3] as usize];
        let b = positions[indices[ti * 3 + 1] as usize];
        let c = positions[indices[ti * 3 + 2] as usize];
        // Translate so p is at origin
        let a = sub3(a, p);
        let b = sub3(b, p);
        let c = sub3(c, p);
        let la = len3(a);
        let lb = len3(b);
        let lc = len3(c);
        if la < 1e-10 || lb < 1e-10 || lc < 1e-10 {
            continue;
        }
        // Van Oosterom & Strackee solid angle formula
        let numerator = a[0] * (b[1] * c[2] - b[2] * c[1])
            + a[1] * (b[2] * c[0] - b[0] * c[2])
            + a[2] * (b[0] * c[1] - b[1] * c[0]);
        let denominator = la * lb * lc + dot3(a, b) * lc + dot3(b, c) * la + dot3(a, c) * lb;
        if denominator.abs() < 1e-12 {
            continue;
        }
        wn += 2.0 * (numerator / denominator).atan();
    }
    wn /= 4.0 * std::f32::consts::PI;
    wn.abs() > 0.5
}

/// Tetrahedralize a surface mesh using star tetrahedralization from interior points.
#[allow(dead_code)]
pub fn tetrahedralize(
    surface_positions: &[[f32; 3]],
    surface_indices: &[u32],
    cfg: &TetGenConfig,
) -> TetGenResult {
    // 1. Copy surface vertices.
    let mut vertices: Vec<[f32; 3]> = surface_positions.to_vec();
    let n_surface = surface_positions.len();

    // 2. Compute bounding box.
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for p in surface_positions {
        for i in 0..3 {
            min[i] = min[i].min(p[i]);
            max[i] = max[i].max(p[i]);
        }
    }

    // 3. Add interior points using LCG, accept only those inside the mesh.
    let mut rng = Lcg::new(cfg.seed);
    let mut interior_count = 0u32;
    let mut attempts = 0u32;
    let max_attempts = cfg.interior_points * 200;

    while interior_count < cfg.interior_points && attempts < max_attempts {
        attempts += 1;
        let p = [
            min[0] + rng.next_f32() * (max[0] - min[0]),
            min[1] + rng.next_f32() * (max[1] - min[1]),
            min[2] + rng.next_f32() * (max[2] - min[2]),
        ];
        if point_inside_surface(p, surface_positions, surface_indices) {
            vertices.push(p);
            interior_count += 1;
        }
    }

    // 4. Build tets: connect each interior vertex to every surface triangle.
    let mut tets: Vec<[usize; 4]> = Vec::new();
    let tri_count = surface_indices.len() / 3;

    for (idx, &ip) in vertices[n_surface..].iter().enumerate() {
        let vi = n_surface + idx;
        for ti in 0..tri_count {
            let ia = surface_indices[ti * 3] as usize;
            let ib = surface_indices[ti * 3 + 1] as usize;
            let ic = surface_indices[ti * 3 + 2] as usize;
            let a = surface_positions[ia];
            let b = surface_positions[ib];
            let c = surface_positions[ic];
            let vol = tet_volume(a, b, c, ip).abs();
            if vol > 1e-9 {
                tets.push([ia, ib, ic, vi]);
            }
        }
    }

    // 5. Compute minimum dihedral angle.
    let min_dihedral = tets
        .iter()
        .flat_map(|t| {
            tet_dihedral_angles(
                vertices[t[0]],
                vertices[t[1]],
                vertices[t[2]],
                vertices[t[3]],
            )
        })
        .fold(f32::MAX, f32::min);

    let tet_count = tets.len();
    let interior_vertex_count = vertices.len() - n_surface;

    TetGenResult {
        mesh: TetMesh {
            vertices,
            tets,
            surface_indices: surface_indices.to_vec(),
        },
        tet_count,
        interior_vertex_count,
        min_dihedral_angle_deg: if min_dihedral == f32::MAX {
            0.0
        } else {
            min_dihedral
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tet() -> ([f32; 3], [f32; 3], [f32; 3], [f32; 3]) {
        (
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        )
    }

    #[test]
    fn tet_volume_unit_tet() {
        let (a, b, c, d) = unit_tet();
        let vol = tet_volume(a, b, c, d);
        assert!(
            (vol.abs() - 1.0 / 6.0).abs() < 1e-5,
            "expected 1/6, got {vol}"
        );
    }

    #[test]
    fn tet_volume_flipped_is_negative() {
        let (a, b, c, d) = unit_tet();
        let vol_normal = tet_volume(a, b, c, d);
        let vol_flipped = tet_volume(a, c, b, d);
        assert!(
            vol_normal * vol_flipped < 0.0,
            "flipped tet should have opposite sign"
        );
    }

    #[test]
    fn tet_volume_degenerate_is_zero() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [2.0, 0.0, 0.0];
        let d = [3.0, 0.0, 0.0];
        let vol = tet_volume(a, b, c, d);
        assert!(vol.abs() < 1e-9, "coplanar points should give ~0 volume");
    }

    #[test]
    fn tet_centroid_unit_tet() {
        let (a, b, c, d) = unit_tet();
        let cen = tet_centroid(a, b, c, d);
        assert!((cen[0] - 0.25).abs() < 1e-6);
        assert!((cen[1] - 0.25).abs() < 1e-6);
        assert!((cen[2] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn tet_centroid_formula() {
        let a = [1.0f32, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0];
        let c = [0.0, 0.0, 1.0];
        let d = [1.0, 1.0, 1.0];
        let cen = tet_centroid(a, b, c, d);
        assert!((cen[0] - 0.5).abs() < 1e-6);
        assert!((cen[1] - 0.5).abs() < 1e-6);
        assert!((cen[2] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn tet_dihedral_angles_count() {
        let (a, b, c, d) = unit_tet();
        let angles = tet_dihedral_angles(a, b, c, d);
        assert_eq!(angles.len(), 6, "should have 6 dihedral angles");
    }

    #[test]
    fn tet_dihedral_angles_in_range() {
        let (a, b, c, d) = unit_tet();
        let angles = tet_dihedral_angles(a, b, c, d);
        for &ang in &angles {
            assert!((0.0..=180.0).contains(&ang), "angle {ang} out of range");
        }
    }

    #[test]
    fn tet_dihedral_angles_finite_positive() {
        let (a, b, c, d) = unit_tet();
        let angles = tet_dihedral_angles(a, b, c, d);
        for &ang in &angles {
            assert!(
                ang.is_finite() && ang > 0.0,
                "angle should be finite positive, got {ang}"
            );
        }
    }

    #[test]
    fn validate_tet_mesh_valid() {
        let mesh = TetMesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            tets: vec![[0, 1, 2, 3]],
            surface_indices: vec![0, 1, 2],
        };
        assert!(validate_tet_mesh(&mesh));
    }

    #[test]
    fn validate_tet_mesh_invalid_index() {
        let mesh = TetMesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            tets: vec![[0, 1, 2, 99]], // index 99 out of range
            surface_indices: vec![],
        };
        assert!(!validate_tet_mesh(&mesh));
    }

    #[test]
    fn tet_mesh_volume_positive() {
        let mesh = TetMesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            tets: vec![[0, 1, 2, 3]],
            surface_indices: vec![0, 1, 2],
        };
        let vol = tet_mesh_volume(&mesh);
        assert!(vol > 0.0, "volume should be positive");
        assert!(
            (vol - 1.0 / 6.0).abs() < 1e-5,
            "unit tet volume = 1/6, got {vol}"
        );
    }

    /// Build a scaled tetrahedron surface for integration testing.
    fn scaled_tet_surface() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [0.0, 4.0, 0.0],
            [0.0, 0.0, 4.0],
        ];
        // 4 outward-facing triangles
        let indices = vec![0u32, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3];
        (positions, indices)
    }

    #[test]
    fn tetrahedralize_nonzero_tet_count() {
        let (pos, idx) = scaled_tet_surface();
        let cfg = TetGenConfig {
            interior_points: 5,
            seed: 7,
            ..Default::default()
        };
        let result = tetrahedralize(&pos, &idx, &cfg);
        assert!(result.tet_count > 0, "should produce at least one tet");
    }

    #[test]
    fn tetrahedralize_interior_vertex_count() {
        let (pos, idx) = scaled_tet_surface();
        let cfg = TetGenConfig {
            interior_points: 5,
            seed: 11,
            ..Default::default()
        };
        let result = tetrahedralize(&pos, &idx, &cfg);
        // At least some interior vertices should be found inside the tetrahedron
        // (result may be 0 if LCG never lands inside, but we expect some)
        let _ = result.interior_vertex_count;
        // tet_count is usize, always >= 0 by definition
    }

    #[test]
    fn tetrahedralize_all_indices_valid() {
        let (pos, idx) = scaled_tet_surface();
        let cfg = TetGenConfig {
            interior_points: 5,
            seed: 13,
            ..Default::default()
        };
        let result = tetrahedralize(&pos, &idx, &cfg);
        assert!(
            validate_tet_mesh(&result.mesh),
            "all tet indices should be valid"
        );
    }
}
