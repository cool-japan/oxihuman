// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::needless_range_loop)]

//! Tetrahedral mesh representation and volume computation.

use anyhow::{Context, Result};

use super::{mat3_det, mat3_inverse, sub3, Mat3, SoftBodyV2, Vec3};

/// A single tetrahedral element with precomputed reference data.
#[derive(Debug, Clone)]
pub struct TetrahedralElement {
    /// Node indices into the global position array.
    pub indices: [usize; 4],
    /// Inverse of the reference shape matrix (Dm^{-1}).
    pub inv_ref_shape: Mat3,
    /// Volume of the element in the rest configuration.
    pub rest_volume: f64,
}

impl TetrahedralElement {
    /// Build from node indices and rest positions.
    ///
    /// The reference shape matrix Dm is formed from the edge vectors
    /// (x1-x0, x2-x0, x3-x0) stored column-wise.
    pub fn new(indices: [usize; 4], rest_positions: &[[f64; 3]]) -> Result<Self> {
        let p0 = rest_positions
            .get(indices[0])
            .context("tet index 0 out of bounds")?;
        let p1 = rest_positions
            .get(indices[1])
            .context("tet index 1 out of bounds")?;
        let p2 = rest_positions
            .get(indices[2])
            .context("tet index 2 out of bounds")?;
        let p3 = rest_positions
            .get(indices[3])
            .context("tet index 3 out of bounds")?;

        let dm = reference_shape_matrix(p0, p1, p2, p3);
        let rest_volume = compute_tet_volume_from_dm(&dm);
        let inv_ref_shape =
            mat3_inverse(&dm).context("degenerate tetrahedron: singular reference shape matrix")?;

        Ok(Self {
            indices,
            inv_ref_shape,
            rest_volume,
        })
    }

    /// Compute the deformation gradient F = Ds * Dm^{-1}.
    ///
    /// `Ds` is the deformed shape matrix formed from current positions.
    pub fn deformation_gradient(&self, positions: &[[f64; 3]]) -> Result<Mat3> {
        let p0 = positions
            .get(self.indices[0])
            .context("deformation gradient: index 0 out of bounds")?;
        let p1 = positions
            .get(self.indices[1])
            .context("deformation gradient: index 1 out of bounds")?;
        let p2 = positions
            .get(self.indices[2])
            .context("deformation gradient: index 2 out of bounds")?;
        let p3 = positions
            .get(self.indices[3])
            .context("deformation gradient: index 3 out of bounds")?;

        let ds = reference_shape_matrix(p0, p1, p2, p3);
        Ok(super::mat3_mul(&ds, &self.inv_ref_shape))
    }

    /// Compute the current volume of this element.
    pub fn current_volume(&self, positions: &[[f64; 3]]) -> Result<f64> {
        let p0 = positions
            .get(self.indices[0])
            .context("current_volume: index 0 out of bounds")?;
        let p1 = positions
            .get(self.indices[1])
            .context("current_volume: index 1 out of bounds")?;
        let p2 = positions
            .get(self.indices[2])
            .context("current_volume: index 2 out of bounds")?;
        let p3 = positions
            .get(self.indices[3])
            .context("current_volume: index 3 out of bounds")?;

        let dm = reference_shape_matrix(p0, p1, p2, p3);
        Ok(compute_tet_volume_from_dm(&dm))
    }
}

/// The tetrahedral mesh containing all precomputed element data.
#[derive(Debug, Clone)]
pub struct TetMesh {
    /// Precomputed tetrahedral elements.
    pub elements: Vec<TetrahedralElement>,
    /// Total number of nodes.
    pub num_nodes: usize,
}

impl TetMesh {
    /// Build a `TetMesh` from the soft-body data.
    pub fn from_soft_body(body: &SoftBodyV2) -> Result<Self> {
        let mut elements = Vec::with_capacity(body.tetrahedra.len());
        for (idx, tet) in body.tetrahedra.iter().enumerate() {
            let elem = TetrahedralElement::new(*tet, &body.rest_positions)
                .with_context(|| format!("failed to build tetrahedron {idx}"))?;
            elements.push(elem);
        }
        Ok(Self {
            elements,
            num_nodes: body.positions.len(),
        })
    }

    /// Build from raw data.
    pub fn new(
        tetrahedra: &[[usize; 4]],
        rest_positions: &[[f64; 3]],
        num_nodes: usize,
    ) -> Result<Self> {
        let mut elements = Vec::with_capacity(tetrahedra.len());
        for (idx, tet) in tetrahedra.iter().enumerate() {
            let elem = TetrahedralElement::new(*tet, rest_positions)
                .with_context(|| format!("failed to build tetrahedron {idx}"))?;
            elements.push(elem);
        }
        Ok(Self {
            elements,
            num_nodes,
        })
    }

    /// Total rest volume of the mesh.
    pub fn total_rest_volume(&self) -> f64 {
        self.elements.iter().map(|e| e.rest_volume).sum()
    }

    /// Total current volume of the mesh.
    pub fn total_current_volume(&self, positions: &[[f64; 3]]) -> Result<f64> {
        let mut total = 0.0;
        for elem in &self.elements {
            total += elem.current_volume(positions)?;
        }
        Ok(total)
    }
}

/// Build the shape matrix from four vertices.
///
/// Columns are edge vectors: (p1-p0, p2-p0, p3-p0).
/// Stored row-major, so column j is `[m[0][j], m[1][j], m[2][j]]`.
fn reference_shape_matrix(p0: &Vec3, p1: &Vec3, p2: &Vec3, p3: &Vec3) -> Mat3 {
    let e1 = sub3(p1, p0);
    let e2 = sub3(p2, p0);
    let e3 = sub3(p3, p0);
    // Columns are e1, e2, e3
    [
        [e1[0], e2[0], e3[0]],
        [e1[1], e2[1], e3[1]],
        [e1[2], e2[2], e3[2]],
    ]
}

/// Compute tetrahedral volume from the shape matrix.
///
/// Volume = |det(Dm)| / 6.
fn compute_tet_volume_from_dm(dm: &Mat3) -> f64 {
    mat3_det(dm).abs() / 6.0
}

/// Compute the signed volume of a tetrahedron given its four vertices.
pub fn signed_tet_volume(p0: &Vec3, p1: &Vec3, p2: &Vec3, p3: &Vec3) -> f64 {
    let dm = reference_shape_matrix(p0, p1, p2, p3);
    mat3_det(&dm) / 6.0
}

/// Compute the unsigned volume of a tetrahedron given its four vertices.
pub fn tet_volume_from_points(p0: &Vec3, p1: &Vec3, p2: &Vec3, p3: &Vec3) -> f64 {
    signed_tet_volume(p0, p1, p2, p3).abs()
}

/// Compute the centroid of a tetrahedron.
pub fn tet_centroid(p0: &Vec3, p1: &Vec3, p2: &Vec3, p3: &Vec3) -> Vec3 {
    [
        (p0[0] + p1[0] + p2[0] + p3[0]) * 0.25,
        (p0[1] + p1[1] + p2[1] + p3[1]) * 0.25,
        (p0[2] + p1[2] + p2[2] + p3[2]) * 0.25,
    ]
}

/// Compute surface area of one triangular face given three vertices.
pub fn triangle_area(a: &Vec3, b: &Vec3, c: &Vec3) -> f64 {
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt()
}

/// Check if all tetrahedra in a mesh have positive volume (are not inverted).
pub fn validate_mesh(mesh: &TetMesh, positions: &[[f64; 3]]) -> Result<()> {
    for (idx, elem) in mesh.elements.iter().enumerate() {
        let vol = elem.current_volume(positions)?;
        anyhow::ensure!(vol > 0.0, "tetrahedron {idx} has non-positive volume {vol}");
    }
    Ok(())
}

/// Generate a simple cube tetrahedral mesh for testing.
///
/// Creates a unit cube \[0,1\]^3 divided into 5 tetrahedra.
pub fn make_unit_cube_tet_mesh() -> Result<(Vec<Vec3>, Vec<[usize; 4]>)> {
    let positions = vec![
        [0.0, 0.0, 0.0], // 0
        [1.0, 0.0, 0.0], // 1
        [1.0, 1.0, 0.0], // 2
        [0.0, 1.0, 0.0], // 3
        [0.0, 0.0, 1.0], // 4
        [1.0, 0.0, 1.0], // 5
        [1.0, 1.0, 1.0], // 6
        [0.0, 1.0, 1.0], // 7
    ];

    // Five tetrahedra that tile a cube
    let tetrahedra = vec![
        [0, 1, 3, 4],
        [1, 2, 3, 6],
        [1, 4, 5, 6],
        [3, 4, 6, 7],
        [1, 3, 4, 6],
    ];

    Ok((positions, tetrahedra))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standard_tet() -> (Vec<Vec3>, Vec<[usize; 4]>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let tets = vec![[0, 1, 2, 3]];
        (positions, tets)
    }

    #[test]
    fn test_tet_volume() {
        let (pos, _) = standard_tet();
        let vol = tet_volume_from_points(&pos[0], &pos[1], &pos[2], &pos[3]);
        assert!((vol - 1.0 / 6.0).abs() < 1e-12);
    }

    #[test]
    fn test_signed_volume() {
        let (pos, _) = standard_tet();
        let vol = signed_tet_volume(&pos[0], &pos[1], &pos[2], &pos[3]);
        // Should be positive for this orientation
        assert!(vol > 0.0);
        assert!((vol.abs() - 1.0 / 6.0).abs() < 1e-12);
    }

    #[test]
    fn test_tetrahedral_element() {
        let (pos, tets) = standard_tet();
        let elem = TetrahedralElement::new(tets[0], &pos).unwrap();
        assert!((elem.rest_volume - 1.0 / 6.0).abs() < 1e-12);

        // Deformation gradient at rest should be identity
        let f = elem.deformation_gradient(&pos).unwrap();
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (f[i][j] - expected).abs() < 1e-10,
                    "F[{i}][{j}] = {} expected {expected}",
                    f[i][j]
                );
            }
        }
    }

    #[test]
    fn test_tet_mesh_from_soft_body() {
        let (pos, tets) = standard_tet();
        let body = SoftBodyV2::new(pos, tets, 1.0).unwrap();
        let mesh = TetMesh::from_soft_body(&body).unwrap();
        assert_eq!(mesh.elements.len(), 1);
        assert!((mesh.total_rest_volume() - 1.0 / 6.0).abs() < 1e-12);
    }

    #[test]
    fn test_unit_cube_mesh() {
        let (pos, tets) = make_unit_cube_tet_mesh().unwrap();
        let mesh = TetMesh::new(&tets, &pos, pos.len()).unwrap();
        // A unit cube has volume 1.0
        assert!(
            (mesh.total_rest_volume() - 1.0).abs() < 1e-10,
            "Total volume = {}",
            mesh.total_rest_volume()
        );
    }

    #[test]
    fn test_scaled_deformation_gradient() {
        let (pos, tets) = standard_tet();
        let elem = TetrahedralElement::new(tets[0], &pos).unwrap();

        // Scale all positions by 2
        let scaled: Vec<Vec3> = pos
            .iter()
            .map(|p| [p[0] * 2.0, p[1] * 2.0, p[2] * 2.0])
            .collect();
        let f = elem.deformation_gradient(&scaled).unwrap();
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 2.0 } else { 0.0 };
                assert!(
                    (f[i][j] - expected).abs() < 1e-10,
                    "F[{i}][{j}] = {} expected {expected}",
                    f[i][j]
                );
            }
        }
    }

    #[test]
    fn test_triangle_area() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let area = triangle_area(&a, &b, &c);
        assert!((area - 0.5).abs() < 1e-12);
    }

    #[test]
    fn test_centroid() {
        let (pos, _) = standard_tet();
        let c = tet_centroid(&pos[0], &pos[1], &pos[2], &pos[3]);
        assert!((c[0] - 0.25).abs() < 1e-12);
        assert!((c[1] - 0.25).abs() < 1e-12);
        assert!((c[2] - 0.25).abs() < 1e-12);
    }

    #[test]
    fn test_validate_mesh_positive() {
        let (pos, tets) = standard_tet();
        let mesh = TetMesh::new(&tets, &pos, pos.len()).unwrap();
        validate_mesh(&mesh, &pos).unwrap();
    }
}
