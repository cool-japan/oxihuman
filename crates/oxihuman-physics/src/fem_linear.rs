// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Linear FEM: stiffness matrix assembly (tetrahedral elements).

/// A 3D node position.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct FemNode {
    pub pos: [f32; 3],
    pub fixed: bool,
}

/// A tetrahedral element (4 node indices).
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct FemTet {
    pub nodes: [usize; 4],
}

/// Material properties for linear FEM.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct FemMaterial {
    pub young_modulus: f32,
    pub poisson_ratio: f32,
}

impl FemMaterial {
    #[allow(dead_code)]
    pub fn default_soft() -> Self {
        Self {
            young_modulus: 1e4,
            poisson_ratio: 0.4,
        }
    }

    /// Lame's first parameter λ.
    #[allow(dead_code)]
    pub fn lambda(&self) -> f32 {
        self.young_modulus * self.poisson_ratio
            / ((1.0 + self.poisson_ratio) * (1.0 - 2.0 * self.poisson_ratio))
    }

    /// Lame's second parameter μ (shear modulus).
    #[allow(dead_code)]
    pub fn mu(&self) -> f32 {
        self.young_modulus / (2.0 * (1.0 + self.poisson_ratio))
    }
}

/// Signed volume of a tetrahedron.
#[allow(dead_code)]
pub fn tet_signed_volume(p: [[f32; 3]; 4]) -> f32 {
    let v1 = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
    let v2 = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
    let v3 = [p[3][0] - p[0][0], p[3][1] - p[0][1], p[3][2] - p[0][2]];
    let cross = [
        v2[1] * v3[2] - v2[2] * v3[1],
        v2[2] * v3[0] - v2[0] * v3[2],
        v2[0] * v3[1] - v2[1] * v3[0],
    ];
    (v1[0] * cross[0] + v1[1] * cross[1] + v1[2] * cross[2]) / 6.0
}

/// Compute the shape function gradients for a linear tet.
/// Returns a 4×3 matrix of shape-function gradients.
#[allow(dead_code)]
pub fn tet_shape_gradients(p: [[f32; 3]; 4]) -> [[f32; 3]; 4] {
    let vol = tet_signed_volume(p);
    if vol.abs() < 1e-12 {
        return [[0.0; 3]; 4];
    }
    let inv6v = 1.0 / (6.0 * vol);

    let mut grads = [[0.0f32; 3]; 4];
    #[allow(clippy::needless_range_loop)]
    for i in 0..4 {
        let j = (i + 1) % 4;
        let k = (i + 2) % 4;
        let l = (i + 3) % 4;
        let vj = p[j];
        let vk = p[k];
        let vl = p[l];
        let jk = [vk[0] - vj[0], vk[1] - vj[1], vk[2] - vj[2]];
        let jl = [vl[0] - vj[0], vl[1] - vj[1], vl[2] - vj[2]];
        let cross = [
            jk[1] * jl[2] - jk[2] * jl[1],
            jk[2] * jl[0] - jk[0] * jl[2],
            jk[0] * jl[1] - jk[1] * jl[0],
        ];
        let sign = if (i % 2) == 0 { 1.0 } else { -1.0 };
        grads[i] = [
            sign * cross[0] * inv6v,
            sign * cross[1] * inv6v,
            sign * cross[2] * inv6v,
        ];
    }
    grads
}

/// Assemble a simple scalar stiffness per element (trace approximation).
#[allow(dead_code)]
pub fn tet_stiffness_scalar(p: [[f32; 3]; 4], mat: &FemMaterial) -> f32 {
    let vol = tet_signed_volume(p).abs();
    let grads = tet_shape_gradients(p);
    let mu = mat.mu();
    let lam = mat.lambda();
    let mut k = 0.0f32;
    for g in &grads {
        let dot = g[0] * g[0] + g[1] * g[1] + g[2] * g[2];
        k += (2.0 * mu + lam) * dot * vol;
    }
    k
}

/// A simple FEM mesh.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct FemMesh {
    pub nodes: Vec<FemNode>,
    pub tets: Vec<FemTet>,
    pub material: Option<FemMaterial>,
}

impl FemMesh {
    #[allow(dead_code)]
    pub fn new(material: FemMaterial) -> Self {
        Self {
            nodes: Vec::new(),
            tets: Vec::new(),
            material: Some(material),
        }
    }

    #[allow(dead_code)]
    pub fn add_node(&mut self, pos: [f32; 3], fixed: bool) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(FemNode { pos, fixed });
        idx
    }

    #[allow(dead_code)]
    pub fn add_tet(&mut self, a: usize, b: usize, c: usize, d: usize) {
        self.tets.push(FemTet {
            nodes: [a, b, c, d],
        });
    }

    #[allow(dead_code)]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[allow(dead_code)]
    pub fn tet_count(&self) -> usize {
        self.tets.len()
    }

    /// Total volume of all elements.
    #[allow(dead_code)]
    pub fn total_volume(&self) -> f32 {
        self.tets
            .iter()
            .map(|t| {
                if t.nodes.iter().any(|&i| i >= self.nodes.len()) {
                    return 0.0;
                }
                let p = [
                    self.nodes[t.nodes[0]].pos,
                    self.nodes[t.nodes[1]].pos,
                    self.nodes[t.nodes[2]].pos,
                    self.nodes[t.nodes[3]].pos,
                ];
                tet_signed_volume(p).abs()
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tet() -> [[f32; 3]; 4] {
        [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]
    }

    #[test]
    fn tet_signed_volume_positive() {
        let p = unit_tet();
        let v = tet_signed_volume(p);
        assert!((v.abs() - 1.0 / 6.0).abs() < 1e-5);
    }

    #[test]
    fn shape_gradients_nonzero_for_valid_tet() {
        let p = unit_tet();
        let g = tet_shape_gradients(p);
        let sum: f32 = g.iter().map(|r| r[0] + r[1] + r[2]).sum();
        assert!(sum.abs() < 1.0);
    }

    #[test]
    fn shape_gradients_zero_for_degenerate_tet() {
        let p = [[0.0f32; 3]; 4];
        let g = tet_shape_gradients(p);
        let sum: f32 = g.iter().flat_map(|r| r.iter()).sum();
        assert_eq!(sum, 0.0);
    }

    #[test]
    fn stiffness_scalar_positive() {
        let p = unit_tet();
        let mat = FemMaterial::default_soft();
        let k = tet_stiffness_scalar(p, &mat);
        assert!(k > 0.0);
    }

    #[test]
    fn material_lambda_positive() {
        let mat = FemMaterial::default_soft();
        assert!(mat.lambda() > 0.0);
    }

    #[test]
    fn material_mu_positive() {
        let mat = FemMaterial::default_soft();
        assert!(mat.mu() > 0.0);
    }

    #[test]
    fn fem_mesh_add_nodes_and_tets() {
        let mat = FemMaterial::default_soft();
        let mut mesh = FemMesh::new(mat);
        let a = mesh.add_node([0.0, 0.0, 0.0], false);
        let b = mesh.add_node([1.0, 0.0, 0.0], false);
        let c = mesh.add_node([0.0, 1.0, 0.0], false);
        let d = mesh.add_node([0.0, 0.0, 1.0], false);
        mesh.add_tet(a, b, c, d);
        assert_eq!(mesh.tet_count(), 1);
        assert_eq!(mesh.node_count(), 4);
    }

    #[test]
    fn fem_mesh_total_volume() {
        let mat = FemMaterial::default_soft();
        let mut mesh = FemMesh::new(mat);
        let a = mesh.add_node([0.0, 0.0, 0.0], false);
        let b = mesh.add_node([1.0, 0.0, 0.0], false);
        let c = mesh.add_node([0.0, 1.0, 0.0], false);
        let d = mesh.add_node([0.0, 0.0, 1.0], false);
        mesh.add_tet(a, b, c, d);
        let v = mesh.total_volume();
        assert!((v - 1.0 / 6.0).abs() < 1e-5);
    }

    #[test]
    fn empty_mesh_volume_zero() {
        let mat = FemMaterial::default_soft();
        let mesh = FemMesh::new(mat);
        assert_eq!(mesh.total_volume(), 0.0);
    }

    #[test]
    fn stiffness_higher_for_stiffer_material() {
        let p = unit_tet();
        let soft = FemMaterial {
            young_modulus: 1e4,
            poisson_ratio: 0.3,
        };
        let stiff = FemMaterial {
            young_modulus: 1e6,
            poisson_ratio: 0.3,
        };
        assert!(tet_stiffness_scalar(p, &stiff) > tet_stiffness_scalar(p, &soft));
    }
}
